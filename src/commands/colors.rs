use super::{Context, Error};
use super::super::db_handlers::color_handlers::*;
use poise::serenity_prelude::EditRole;

#[poise::command(slash_command, subcommands("set", "info", "steal"))]
pub async fn color(_ctx: Context<'_>) -> Result<(), Error>{
    Ok(())
}


#[poise::command(slash_command, ephemeral)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "Color code - hexadecimal (eg. 0xffaa99 | #11ffaa)"] color: String
) -> Result<(), Error> {
    // Convert hex string to int
    let color = match u32::from_str_radix(&color.replace("0x", "").replace("#", ""), 16) {
        Ok(val) => val,
        Err(err) => {
            ctx.reply(format!("Invalid color code? err : {}", err)).await?;
            return Ok(());
        }
    };

    let name = ctx.author().name.to_string();
    let uid = ctx.author().id;

    let guid = match ctx.guild_id() {
        Some(g) => g,
        None => {
            ctx.reply("Can only be used inside a server").await?;
            return Ok(());
        }
    };

    // see if color exists
    let conn = ctx.data().pool.acquire().await?;
    let (exists, role_id) = color_role_exists(conn, uid.into(), guid.into()).await?;
    
    let guild = ctx.guild_id().unwrap();
    let conn = ctx.data().pool.acquire().await?;

    let new_role = EditRole::new()
        .name(name.clone())
        .hoist(false)
        .mentionable(false)
        .colour(color);

    match exists {
        true => {
            // Role exists, edit color
            let role_id = role_id.unwrap();

            update_color_role(conn, role_id, color).await?;
            guild.edit_role(&ctx.http(), role_id, new_role).await?;

            
            // Add role incase user doesnt have it yet/anymore
            let mem = guild.member(&ctx.http(), uid).await?;
            mem.add_role(&ctx.http(), role_id).await?;

            ctx.reply("Color updated").await?;
        },
        false => {
            // Create new role if it doesn't exist
            let result = guild.create_role(&ctx.http(), new_role).await;

            match result {
                Ok(role) => {
                    create_color_role(
                        conn, role.id.into(), uid.into(), guid.into(), 
                        color, name).await?;

                            
                    // Add role incase user doesnt have it yet/anymore
                    let mem = guild.member(&ctx.http(), uid).await?;
                    mem.add_role(&ctx.http(), role.id).await?;
                    
                    ctx.reply("Color role created").await?;
                },
                Err(err) => {
                    ctx.reply(format!("Error! : {}", err)).await?;
                }
            }
        }
    };

    Ok(())
}

#[poise::command(slash_command)]
pub async fn info(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn steal(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}