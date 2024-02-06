use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::super::Error;
use poise::serenity_prelude::futures::TryStreamExt;
use sqlx::{pool::PoolConnection, Row, Sqlite};


pub struct Reminder {
    pub id: u64,
    pub timestamp: u64,
    pub message: String,
    pub user_id: u64,
    pub channel_id: u64,
    pub private: bool,
}
pub struct NewReminder {
    pub timestamp: u64,
    pub message: String,
    pub user_id: u64,
    pub channel_id: u64,
    pub private: bool,
}

pub async fn get_expired_reminders(
    mut conn: PoolConnection<Sqlite>
) -> Result<Vec<Reminder>, Error> {
    
    let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Error, time went backwards").as_secs();

    let mut rows = sqlx::query(
        r#"
        SELECT id, timestamp, message, user_id, channel_id, private FROM reminders
        WHERE completed = 0 AND timestamp <= ?;
        "#,
    )
    .bind(now as i64)
    .fetch_all(&mut *conn).await?;
    conn.close().await?;

    let mut results: Vec<Reminder> = Vec::new();

    for row in rows {
        let id: u64 = row.get::<i64, _>("id") as u64;
        let timestamp: u64 = row.get::<i64, _>("timestamp") as u64;
        let message: String = row.get("message");
        let user_id: u64 = row.get::<i64, _>("user_id") as u64;
        let channel_id: u64 = row.get::<i64, _>("channel_id") as u64;
        let private: bool = row.get("private");

        let r = Reminder {
            id, timestamp, message, user_id, channel_id, private
        };

        results.push(r);
    }

    Ok(results)
}

pub async fn set_completed (
    mut conn: PoolConnection<Sqlite>,
    id: u64
) -> Result<(), Error> {

    let mut rows = sqlx::query(
        r#"
        UPDATE reminders
        SET completed = true
        WHERE id = ?;
        "#,
    )
    .bind(id as i64)
    .execute(&mut *conn)
    .await?;

    conn.close().await?;


    Ok(())
}

pub async fn new_reminder(
    mut conn: PoolConnection<Sqlite>,
    new: NewReminder,
) -> Result<(), Error> {

    let _ = sqlx::query(r#"
        INSERT INTO reminders (timestamp, message, user_id, channel_id, private, completed)
        VALUES (?, ?, ?, ?, ?, FALSE);
    "#)
    .bind(new.timestamp as i64)
    .bind(new.message)
    .bind(new.user_id as i64)
    .bind(new.channel_id as i64)
    .bind(new.private)
    .execute(&mut *conn)
    .await?;

    conn.close().await?;

    Ok(())
}