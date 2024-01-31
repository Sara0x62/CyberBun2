use crate::db_handlers::starboard_handlers::{insert_message, message_exists};
use crate::db_handlers::{color_handlers::get_color, starboard_handlers};
use poise::serenity_prelude::{CreateEmbedFooter, CreateMessage};
use poise::{
    serenity_prelude::{
        ActivityData, ChannelId, Context, CreateEmbed, FullEvent, ReactionType, Timestamp,
    },
    FrameworkContext,
};
use std::sync::atomic::Ordering::SeqCst;
use tracing::info;

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
            ctx.set_activity(Some(ActivityData::watching(format!(
                "{} server(s) - OwO",
                data_about_bot.guilds.len()
            ))));
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

        FullEvent::GuildDelete {
            incomplete,
            full: _,
        } => {
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

            match add_reaction.guild_id {
                Some(guild) => {
                    let star = ReactionType::from('⭐');

                    if add_reaction.emoji == star {
                        let message = &ctx
                            .http
                            .get_message(add_reaction.channel_id, add_reaction.message_id)
                            .await?;

                        let conn = data.pool.acquire().await?;

                        if message
                            .reactions
                            .iter()
                            .find(|f| f.reaction_type == star && f.count == 1)
                            .is_some()
                            && !message_exists(conn, message.id.get()).await?
                        {
                            // Try to find guild, starboard related settings
                            let conn = data.pool.acquire().await?;
                            let settings =
                                starboard_handlers::get_guild_settings(conn, guild.get()).await?;
                            match settings {
                                Some(settings) => {
                                    if settings.starboard_enabled
                                        && settings.starboard_channel.is_some()
                                    {
                                        if settings.starboard_channel.unwrap()
                                            != add_reaction.channel_id.get()
                                        {
                                            let starboard = ChannelId::from(
                                                settings.starboard_channel.unwrap(),
                                            );
                                            let user = &message.author;
                                            let nick = message
                                                .author_nick(&ctx.http)
                                                .await
                                                .unwrap_or(user.name.clone());
                                            let footer = CreateEmbedFooter::new(format!("CyberBun - ⭐"));

                                            let msg = CreateEmbed::default()
                                                .title(nick)
                                                .url(message.link())
                                                .thumbnail(
                                                    user.avatar_url().unwrap_or("".to_string()),
                                                )
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
                                None => {}
                            };
                        }
                    }
                }
                None => {}
            }
        }

        _ => (),
    }

    Ok(())
}
