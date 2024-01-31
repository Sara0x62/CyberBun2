use super::super::Error;
use sqlx::{pool::PoolConnection, Sqlite};
use tracing::info;

#[derive(sqlx::FromRow)]
pub struct ColorRow {
    pub role_id: u64,
    pub uid: u64,
    pub guid: u64,
    pub color: u32,
    pub role_name: String,
}

pub async fn create_color_role(
    mut conn: PoolConnection<Sqlite>,
    role_id: u64,
    uid: u64,
    guid: u64,
    color: u32,
    name: String,
) -> Result<(), Error> {

    let result = sqlx::query(
        r#"
        INSERT INTO colors (
            role_id, uid, guid, color, role_name
        ) VALUES (?, ?, ?, ?, ?);
        "#,
    )
    .bind(role_id as i64)
    .bind(uid as i64)
    .bind(guid as i64)
    .bind(color)
    .bind(name)
    .execute(&mut *conn)
    .await?;

    conn.close().await?;

    match result.rows_affected() {
        0 => info!("Color did not insert?"),
        _ => info!("Inserted new color into Database"),
    }
    Ok(())
}

pub async fn get_color(
    mut conn: PoolConnection<Sqlite>,
    uid: u64,
    guid: u64,
) -> Result<Option<ColorRow>, Error> {

    #[derive(sqlx::FromRow)]
    struct TempColor {
        role_id: i64,
        uid: i64,
        guid: i64,
        color: u32,
        role_name: String,
    }

    let result = sqlx::query_as::<_, TempColor>(
        r#"
        SELECT *
        FROM colors
        WHERE uid = ? AND guid = ?;
        "#,
    )
    .bind(uid as i64)
    .bind(guid as i64)
    .fetch_optional(&mut *conn)
    .await?
    .map(|r| ColorRow {
        role_id: r.role_id as u64,
        uid: r.uid as u64,
        guid: r.guid as u64,
        color: r.color,
        role_name: r.role_name,
    });

    Ok(result)
}

pub async fn update_color_role(
    mut conn: PoolConnection<Sqlite>,
    role_id: u64,
    color: u32,
) -> Result<(), Error> {
    
    let _result = sqlx::query(
        r#"
        UPDATE colors
        set color = ?
        WHERE role_id = ?;
        "#,
    )
    .bind(color)
    .bind(role_id as i64)
    .execute(&mut *conn)
    .await?;

    conn.close().await?;

    Ok(())
}
