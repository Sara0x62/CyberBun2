use sqlx::{pool::PoolConnection, Sqlite};
use tracing::info;

use super::super::Error;

pub async fn build_colors(mut conn: PoolConnection<Sqlite>) -> Result<(), Error> {
    let result = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS colors (
            role_id BIG INT PRIMARY KEY NOT NULL,
            uid BIG INT NOT NULL,
            guid BIG INT NOT NULL,
            color INT NOT NULL,
            role_name TEXT NOT NULL
        );
        "#
    ).execute(&mut *conn)
    .await?;

    conn.close().await?;

    match result.rows_affected() {
        0 => info!("Colors Database already exists"),
        _ => info!("Colors Database created successfully."),
    }

    Ok(())
}

pub async fn build_guild_settings(mut conn: PoolConnection<Sqlite>) -> Result<(), Error> {
    // Table for all the guild related settings - eg Starboard enabled, Starboard channel, ...

    let result = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS guild_settings (
            guild_id BIG INT PRIMARY KEY NOT NULL,
            starboard_enabled BOOLEAN NOT NULL,
            starboard_channel BIG INT,
            starboard_min SMALL INT NOT NULL
        );
        "#)
        .execute(&mut *conn)
        .await?;

    conn.close().await?;
    
    match result.rows_affected() {
        0 => info!("Guild settings Database already exists"),
        _ => info!("Guild settings Database created successfully."),
    }


    Ok(())
}

pub async fn build_starred_messages(mut conn: PoolConnection<Sqlite>) -> Result<(), Error> {

    let result = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS starred_messages (
            msg_id BIG INT PRIMARY KEY NOT NULL
        );
        "#)
        .execute(&mut *conn)
        .await?;

    conn.close().await?;

    match result.rows_affected() {
        0 => info!("Starred messages Database already exists"),
        _ => info!("Starred messages Database created successfully."),
    }

    Ok(())
}
