use super::super::Error;
use sqlx::{pool::PoolConnection, Row, Sqlite};
use tracing::{info, warn};


pub async fn create_color_role(
    mut conn: PoolConnection<Sqlite>, 
    role_id: u64, uid: u64, guid: u64, color: u32, 
    name: String
) -> Result<(), Error> {

    let result = sqlx::query(
        r#"
        INSERT INTO colors (
            role_id, uid, guid, color, role_name
        ) VALUES (?, ?, ?, ?, ?);
        "#
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

pub async fn color_role_exists(
    mut conn: PoolConnection<Sqlite>,
    uid: u64, guid: u64
) -> Result<(bool, Option<u64>), Error> {
    
    let result = sqlx::query(
        r#"
        SELECT role_id
        FROM colors
        WHERE uid = ? AND guid = ?;
        "#
    )
    .bind(uid as i64)
    .bind(guid as i64)
    .fetch_one(&mut *conn)
    .await;

    conn.close().await?;

    match result {
        Ok(row) => {
            return Ok( (true, Some(row.get::<i64, _>("role_id") as u64)) );
        },
        Err(err) => {
            match err {
                sqlx::Error::RowNotFound => return Ok ( (false, None) ),
                _ => warn!("Unexpected error checking if color role exists: {}", err),
            }
        }
    }
    

    Ok( (false, None) )
}

pub async fn update_color_role(
    mut conn: PoolConnection<Sqlite>,
    role_id: u64, color: u32
) -> Result<(), Error> {

    let result = sqlx::query(
        r#"
        UPDATE colors
        set color = ?
        WHERE role_id = ?;
        "#
    )
    .bind(color)
    .bind(role_id as i64)
    .execute(&mut *conn)
    .await?;

    conn.close().await?;

    Ok(())
}

pub async fn get_color_code(
    mut conn: PoolConnection<Sqlite>,
    uid: u64, guid: u64
) -> Result<u32, Error> {

    let result = sqlx::query(
        r#"
        SELECT color
        FROM colors
        WHERE uid = ? AND guid = ?
        "#)
        .bind(uid as i64)
        .bind(guid as i64)
        .fetch_one(&mut *conn)
        .await?;

    conn.close().await?;

    return Ok(result.get::<u32, _>("color"));
}