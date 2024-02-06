use crate::db_handlers::starboard_handlers::*;

use super::{Context, Error};
use poise::serenity_prelude::{Channel, Mentionable};


#[poise::command(slash_command, subcommands("setup", "enabled", "stars"))]
pub async fn starboard(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Set a channel for starboard quotes - to enable set the optional parameter to true
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR", guild_only, ephemeral)]
pub async fn setup(
    ctx: Context<'_>,
    #[description = "The channel to post the starboard messages in"] channel: Channel,
    #[description = "Enable / Disable the starboard (True = Enabled / False = Disabled)"] enabled: Option<bool>,
    #[description = "Minimum star reactions to pin (Default = 3)"]#[min = 1]  min_stars: Option<u8>,
) -> Result<(), Error> {
    // See if guild already has settings in DB and update or insert new settings
    // Set the channel to use for the starboard

    let enabled = match enabled { Some(opt) => opt, None => false };
    let min_stars = match min_stars { Some(opt) => opt, None => 3 };

    // Try to get existing settings
    let conn = ctx.data().pool.acquire().await?;
    let res = get_guild_settings(conn, ctx.guild_id().unwrap().get()).await?;

    let new_settings: GuildSettings = match res {
        Some(mut settings) => {
            // Existing settings found
            settings.starboard_channel = Some(channel.id().get());
            settings.starboard_min = min_stars;

            settings
        },
        None => {
            // Settings dont exist in DB
            GuildSettings {
                guild_id: ctx.guild_id().unwrap().get(),
                starboard_enabled: enabled,
                starboard_channel: Some(channel.id().get()),
                starboard_min: min_stars,
            }
        },
    };
    
    let conn = ctx.data().pool.acquire().await?;
    update_guild_settings(conn, new_settings).await?;

    ctx.reply(
        format!("New settings - channel set to {} | Starboard is currently: {}",
        channel.mention(), 
        if enabled { "Enabled" } else { "Disabled" }
    )).await?;


    Ok(())
}


/// Toggles the status on the starboard to enabled or disabled
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR", guild_only)]
pub async fn enabled(
    ctx: Context<'_>,
    #[description = "Enable / Disable the starboard (True = Enabled / False = Disabled)"] switch: bool,
) -> Result<(), Error> {

    // Enables or disables the starboard
    // - When trying to enable - check if there is a channel set already!
    
    // Try to get existing settings
    let conn = ctx.data().pool.acquire().await?;
    let res = get_guild_settings(conn, ctx.guild_id().unwrap().get()).await?;

    match res {
        Some(_) => {
            let conn = ctx.data().pool.acquire().await?;
            toggle_starboard(conn, ctx.guild_id().unwrap().get(), switch).await?;

            ctx.reply(format!("Starboard is now {}", if switch { "Enabled" } else { "Disabled" } )).await?;
        },
        None => {
            ctx.reply(format!("Please configure a channel to use first with `/starboard setup`")).await?;
        },
    };

    Ok(())
}


/// Set the required stars for the starboard
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR", guild_only)]
pub async fn stars(
    ctx: Context<'_>,
    #[description = "Minimum star reactions to pin (Default = 3)"]#[min = 1] min_stars: u8,
) -> Result<(), Error> {

    let guid = ctx.guild_id().unwrap().get();

    let conn = ctx.data().pool.acquire().await?;
    let res = set_required_stars(conn, guid, min_stars).await?;

    match res {
        true => { ctx.reply(format!("Setting edited, messages now require at least {} stars", min_stars)).await?; }
        false => { ctx.reply("No existing settings found, did you run the setup command atleast once?").await?; }
    }

    Ok(())
}