use sqlx::{pool::PoolConnection, Sqlite};

use super::super::Error;


#[derive(sqlx::FromRow)]
pub struct GuildSettings {
    pub guild_id: u64,
    pub starboard_enabled: bool,
    pub starboard_channel: Option<u64>,
    pub starboard_min: u8,
}

#[derive(sqlx::FromRow)]
struct TmpGuildSettings {
    guild_id: i64,
    starboard_enabled: bool,
    starboard_channel: Option<i64>,
    starboard_min: u8,
}

impl TmpGuildSettings {
    fn swap(other: &GuildSettings) -> Self {
        TmpGuildSettings {
            guild_id: other.guild_id as i64,
            starboard_enabled: other.starboard_enabled,
            starboard_channel: match other.starboard_channel { Some(channel) => Some(channel as i64), None => None},
            starboard_min: other.starboard_min,
        }
    }
}

pub async fn update_guild_settings(mut conn: PoolConnection<Sqlite>, new: GuildSettings) -> Result<(), Error> {
    let tmp = TmpGuildSettings::swap(&new);

    let _result = sqlx::query(
        r#"
        REPLACE INTO guild_settings(guild_id, starboard_enabled, starboard_channel, starboard_min)
        VALUES(?, ?, ?, ?);
        "#
    )
    .bind(tmp.guild_id)
    .bind(tmp.starboard_enabled)
    .bind(tmp.starboard_channel)
    .bind(tmp.starboard_min)
    .execute(&mut *conn)
    .await?;

    conn.close().await?;

    Ok(())
}

pub async fn toggle_starboard(mut conn: PoolConnection<Sqlite>, guid: u64, enabled: bool) -> Result<(), Error> {

    let _result = sqlx::query(r#"
        UPDATE guild_settings
        set starboard_enabled = ?
        WHERE guild_id = ?;
    "#).bind(enabled).bind(guid as i64).execute(&mut *conn).await?;

    conn.close().await?;
    Ok(())
}

pub async fn set_required_stars(mut conn: PoolConnection<Sqlite>, guid: u64, new_stars: u8) -> Result<bool, Error> {
    let result = sqlx::query(
        r#"
        UPDATE guild_settings
        SET starboard_min = ?
        WHERE guild_id = ?;
        "#
    )
    .bind(new_stars)
    .bind(guid as i64)
    .execute(&mut *conn).await?;

    conn.close().await?;
    
    // Returns false if NO rows where updated. - otherwise returns true
    Ok(result.rows_affected() != 0)
}

pub async fn get_guild_settings(mut conn: PoolConnection<Sqlite>, guid: u64) -> Result<Option<GuildSettings>, Error> {
    
    let result = sqlx::query_as::<_, TmpGuildSettings>(
        r#"
        SELECT *
        FROM guild_settings
        WHERE guild_id = ?
        "#,
    )
    .bind(guid as i64)
    .fetch_optional(&mut *conn)
    .await?
    .map(|r| GuildSettings {
        guild_id: r.guild_id as u64,
        starboard_enabled: r.starboard_enabled,
        starboard_channel: match r.starboard_channel { Some(channel) => Some(channel as u64), None => None},
        starboard_min: r.starboard_min,
    });

    conn.close().await?;

    Ok(result)
}

pub async fn message_exists(mut conn: PoolConnection<Sqlite>, msg_id: u64) -> Result<bool, Error> {
    let result = sqlx::query(
        r#"
        SELECT * FROM starred_messages WHERE msg_id = ?;
        "#
    )
    .bind(msg_id as i64)
    .fetch_optional(&mut *conn)
    .await?;

    conn.close().await?;

    if result.is_some() { return Ok(true); } else { return Ok(false); }
}

pub async fn insert_message(mut conn: PoolConnection<Sqlite>, msg_id: u64)  -> Result<(), Error> {

    let _result = sqlx::query(
        r#"INSERT INTO starred_messages (msg_id)
            VALUES (?);"#
        )
        .bind(msg_id as i64)
        .execute(&mut *conn)
        .await?;

    conn.close().await?;

    Ok(())
}