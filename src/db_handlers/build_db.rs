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

    info!("building 'colors' table - Result: {:?}", result);

    Ok(())
}

pub async fn build_moderation() {
    ()
}

pub async fn build_bugs() {
    ()
}