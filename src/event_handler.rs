use poise::{serenity_prelude::{ActivityData, Context, FullEvent}, FrameworkContext};
use std::sync::atomic::Ordering::SeqCst;
use tracing::info;
use super::{Data, Error};


pub async fn event_handler (
    ctx: &Context,
    event: &FullEvent,
    _framework: FrameworkContext<'_, Data, Error>,
    data: &Data) -> Result<(), Error>
{

    match event {

        FullEvent::Ready { data_about_bot } => {
            info!("Bot ready! - serving {} server(s)...", data_about_bot.guilds.len());
            ctx.set_activity( 
                Some(
                    ActivityData::watching(format!("{} server(s) - OwO", data_about_bot.guilds.len()))
                )
            );
        },

        FullEvent::GuildCreate { guild: _, is_new } => {
            match is_new {
                Some(true) => {
                    info!("Joined 1 new server!");

                    let new_count = data.server_count.load(SeqCst) + 1;
                    data.server_count.store(new_count, SeqCst);

                    ctx.set_activity( 
                        Some(
                            ActivityData::watching(format!("{} server(s) - OwO", new_count) )
                        )
                    );
                },
                Some(false) => (),
                _ => ()
            }
        },

        FullEvent::GuildDelete { incomplete, full: _ } => {
            let new_count = data.server_count.load(SeqCst) - 1;
            data.server_count.store(new_count, SeqCst);

            match incomplete.unavailable {
                false => info!("I Got removed from 1 server"),
                true => info!("A server I was serving became unavailable"),
            }

            ctx.set_activity( 
                Some(
                    ActivityData::watching(format!("{} server(s) - OwO", new_count) )
                )
            );

        },


        _ => (),
    }

    Ok(())
}