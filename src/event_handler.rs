use crate::db_handlers::color_handlers::get_color;
use crate::db_handlers::reminder_handlers::{get_expired_reminders, set_completed, Reminder};
use crate::db_handlers::starboard_handlers::{get_guild_settings, insert_message, message_exists};
use poise::serenity_prelude::{CreateEmbedFooter, CreateMessage, Mentionable, UserId};
use poise::{
    serenity_prelude::{
        ActivityData, ChannelId, Context, CreateEmbed, FullEvent, ReactionType, Timestamp,
    },
    FrameworkContext,
};
use sqlx::{Error as sqlError, Pool};
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;
use tracing::info;
use tokio::time::Duration;

use sqlx::{pool::PoolConnection, Sqlite};

use super::{Data, Error};

pub async fn event_handler(
    ctx: &Context,
    event: &FullEvent,
    _framework: FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {

        FullEvent::Ready { data_about_bot } => {
            info!(
                "Bot ready! - serving {} server(s)...",
                data_about_bot.guilds.len()
            );
            data.server_count.store(data_about_bot.guilds.len(), SeqCst);

            ctx.set_activity(Some(ActivityData::watching(format!(
                "{} server(s) - OwO",
                data_about_bot.guilds.len()
            ))));
        }

        FullEvent::CacheReady { guilds: _ } => {
            let reminder_ctx = Arc::new(ctx.clone());
            let pool = Arc::from(data.pool.clone());
            
            tokio::spawn(async move {
                loop {
                    // Reminders event -
                    // Check Database for first upcoming reminder
                    let status = reminder_handler(&reminder_ctx, &pool).await;
                    
                    match status {
                        Ok(_) => continue,
                        Err(err) => info!("Error occured in reminder loop - {}", err)
                    }
                    tokio::time::sleep(Duration::from_secs(30)).await;
                }
            });
        } 

        FullEvent::GuildCreate { guild: _, is_new } => match is_new {
            Some(true) => {
                info!("Joined 1 new server!");

                let new_count = data.server_count.load(SeqCst) + 1;
                data.server_count.store(new_count, SeqCst);

                ctx.set_activity(Some(ActivityData::watching(format!(
                    "{} server(s) - OwO",
                    new_count
                ))));
            }
            Some(false) => (),
            _ => (),
        },

        FullEvent::GuildDelete { incomplete, full: _, } => {
            let new_count = data.server_count.load(SeqCst) - 1;
            data.server_count.store(new_count, SeqCst);

            match incomplete.unavailable {
                false => info!("I Got removed from 1 server"),
                true => info!("A server I was serving became unavailable"),
            }

            ctx.set_activity(Some(ActivityData::watching(format!(
                "{} server(s) - OwO",
                new_count
            ))));
        }

        FullEvent::GuildMemberAddition { new_member } => {
            // New user joined the guild, check if they were a member before / are in the Database
            let conn = data.pool.acquire().await?;
            let role =
                get_color(conn, new_member.user.id.into(), new_member.guild_id.into()).await?;

            match role {
                Some(role) => {
                    // Found matching role
                    new_member.add_role(&ctx.http, role.role_id).await?;
                }
                None => {}
            }
        }

        FullEvent::ReactionAdd { add_reaction } => {
            // Check if reaction was inside guild
            // And if reaction was a star emoji,
            // Count how many star emojis are on said message
            // if > 3 then continue;
            // check if guild id in DB settings for starboard
            // if all set and enabled and 3 stars post to starboard
            // Also ignore if star is inside the starboard channel

            if let Some(guild) = add_reaction.guild_id {
                let star = ReactionType::from('⭐');

                if add_reaction.emoji == star {
                    let message = &ctx
                        .http
                        .get_message(add_reaction.channel_id, add_reaction.message_id)
                        .await?;

                    let conn = data.pool.acquire().await?;
                    let exists = message_exists(conn, message.id.get()).await?;

                    if !exists {
                        // Try to find guild, starboard related settings
                        let conn = data.pool.acquire().await?;
                        let settings = get_guild_settings(conn, guild.get()).await?;

                        if let Some(settings) = settings {
                            if settings.starboard_enabled && settings.starboard_channel.is_some() {
                                let star_count = message
                                    .reactions
                                    .iter()
                                    .find(|f| {
                                        f.reaction_type == star
                                            && f.count == settings.starboard_min as u64
                                    })
                                    .is_some();

                                let star_channel = settings.starboard_channel.unwrap();

                                if star_channel != add_reaction.channel_id.get() && star_count {
                                    let starboard = ChannelId::from(star_channel);

                                    let user = &message.author;
                                    let nick = message
                                        .author_nick(&ctx.http)
                                        .await
                                        .unwrap_or(user.name.clone());
                                    let footer = CreateEmbedFooter::new(format!("CyberBun - ⭐"));

                                    let msg = CreateEmbed::default()
                                        .title(nick)
                                        .url(message.link())
                                        .thumbnail(user.avatar_url().unwrap_or("".to_string()))
                                        .description(&message.content)
                                        .footer(footer)
                                        .timestamp(Timestamp::now());
                                    let reply = CreateMessage::default().embed(msg);

                                    let conn = data.pool.acquire().await?;
                                    insert_message(conn, message.id.get()).await?;

                                    starboard.send_message(&ctx.http, reply).await?;
                                }
                            }
                        }
                    }
                }
            }
        }

        _ => (),
    }

    Ok(())
}

async fn reminder_handler(ctx: &Context, pool: &Arc<Pool<Sqlite>>) -> Result<(), Error> {
    // First load all reminders from database that;
    // - have not been completed yet
    // - Have timestamps equal to 'now' or already in the 'past'
    let conn = pool.acquire().await;
    let reminders: Vec<Reminder> = get_expired_reminders(conn.unwrap()).await?;

    for r in reminders.iter() {
        let user = UserId::from(r.user_id);
        let message = CreateMessage::new().content(
            format!("{} - Reminder; {}", user.mention(), r.message));

        let chan = ChannelId::from(r.channel_id);
        chan.send_message(&ctx.http, message).await?;

        let conn = pool.acquire().await;
        let _ = set_completed(conn.unwrap(), r.id).await;
    }
    
    Ok(())
}