use super::{Context, Error};
use poise::serenity_prelude::CreateEmbed;
use poise::CreateReply;
use tracing::info;

pub mod colors;
pub mod reminders;
pub mod starboard;

/// Show the HELP menu
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> Result<(), Error> {
    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "\
            Type ~help command for more info on a command.
            You can edit your message to the bot and the bot will edit its response.",
        ..Default::default()
    };

    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

/// Register new commands
#[poise::command(prefix_command, owners_only, hide_in_help)]
pub async fn register_commands(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    info!("New commands registered / updated!");
    Ok(())
}

#[poise::command(prefix_command, owners_only, hide_in_help, ephemeral)]
pub async fn sysinfo(ctx: Context<'_>) -> Result<(), Error> {
    // Todo: Find a good crate to get current process info - cpu usage / ram usage / etc.

    let embed = CreateEmbed::new()
        .title("System Resource Load");


    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}
