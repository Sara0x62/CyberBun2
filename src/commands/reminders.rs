use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::db_handlers::reminder_handlers::{new_reminder, NewReminder};

use super::super::{Data, Error};
use poise::Modal;
use tracing::info;

type ApplicationContext<'a> = poise::ApplicationContext<'a, Data, Error>;

/// A simple reminder command
#[poise::command(slash_command, ephemeral)]
pub async fn remindme(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let modal = ReminderModal::execute(ctx).await?;
    info!("Got modal : {:?}", modal);

    // Unwrap inputs or default to 0 if none given
    let modal_vals = modal.unwrap();
    let days = modal_vals.days.unwrap_or("0".to_string());
    let hours = modal_vals.hours.unwrap_or("0".to_string());
    let minutes = modal_vals.minutes.unwrap_or("0".to_string());

    // Parse inputs to u32's
    let days = days.parse::<u32>();
    let hours = hours.parse::<u32>();
    let minutes = minutes.parse::<u32>();

    if days.is_ok() && hours.is_ok() && minutes.is_ok() {
        let days = days.unwrap();
        let hours = hours.unwrap();
        let minutes = minutes.unwrap();
        // All where valid numbers for u32
        ctx.reply(format!(
            "Setting reminder for you in {} days {} hours {} and minutes",
            &days,
            &hours,
            &minutes
        ))
        .await?;

        let total_time: u64 = (
            days as u64 * 24 * 60 * 60) +
            ( hours as u64 * 60 * 60 ) +
            ( minutes as u64 * 60);

        let mut timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("Error with the time");
        timestamp += Duration::new(total_time, 0);

        let new = NewReminder { 
            timestamp: timestamp.as_secs(), 
            message: modal_vals.message, 
            user_id: ctx.author().id.get(),
            channel_id: ctx.channel_id().get(), 
            private: false, 
        };

        let conn = ctx.data.pool.acquire().await?;
        new_reminder(conn, new).await?;
    } else {
        // Invalid number detected - tell user
        ctx.reply("One of the inputs you gave was not a valid number, please only use numbers in the fields").await?;
    }
    Ok(())
}

#[derive(Modal, Debug)]
#[name = "CyberBun - Reminder"]
struct ReminderModal {
    #[name = "days"]
    #[placeholder = "0"]
    days: Option<String>,

    #[name = "hours"]
    #[placeholder = "0"]
    hours: Option<String>,

    #[name = "minutes"]
    #[placeholder = "0"]
    minutes: Option<String>,

    #[name = "Message"]
    #[placeholder = "Reminder message - required"]
    message: String,
}