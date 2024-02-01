use std::{env, sync::atomic::AtomicUsize};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, SqlitePool};
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter, prelude::*};
use poise::serenity_prelude::{ClientBuilder, GatewayIntents};

// =================================================================
mod commands;
mod db_handlers;
mod event_handler;
use event_handler::event_handler;

use crate::db_handlers::build_db;


// == GLOBAL DATA ==
struct Data { server_count: AtomicUsize, pool: SqlitePool } // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;


#[tokio::main]
async fn main() {

    match dotenv::dotenv() {
        Ok(env) => env,
        Err(err) => panic!("Error launching environment: {}", err),
    };

    logger_setup().await;

    let pool = match db_setup().await {
        Ok(pool) => pool,
        Err(err) => panic!("Error setting up Database pool: {}", err),
    };

    let bot_token = env::var("CYBERBUN_TOKEN").expect("ERROR: CYBERBUN_TOKEN NOT FOUND");   
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let commands = vec![
        commands::help(),
        commands::register_commands(),

        commands::colors::color(),

        commands::starboard::starboard(),
    ];

    let options = poise::FrameworkOptions {
        commands,

        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("~".into()),
            ..Default::default()
        },

        event_handler: |ctx, event, framework, data| {
            Box::pin(event_handler(ctx, event, framework, data))
        },

        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .options(options)
        .setup(|_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    server_count: AtomicUsize::new(0),
                    pool,
                })
            })
        })
        .build();

    let client = ClientBuilder::new(bot_token, intents)
        .framework(framework)
        .await;

    info!("Starting CyberBunny...");

    client.unwrap().start().await.unwrap();
}


async fn logger_setup() {
    let filter = EnvFilter::from_env("CYBERBUN_FILTER");

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    info!("Logger ready - Filter {:?}", env::var("CYBERBUN_FILTER").unwrap());
}

async fn db_setup() -> Result<SqlitePool, Error> {

    let db_url = env::var("CYBERBUN_DB_URL").expect("NO DATABASE URL FOUND");

    if !sqlx::Sqlite::database_exists(&db_url).await? {
        sqlx::Sqlite::create_database(&db_url).await?;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    info!("DB Pool ready ~ making sure tables exist");

    let conn = pool.acquire().await?;
    build_db::build_colors(conn).await?;
    
    let conn = pool.acquire().await?;
    build_db::build_guild_settings(conn).await?;

    let conn = pool.acquire().await?;
    build_db::build_starred_messages(conn).await?;

    info!("Database pool ready");

    Ok(pool)
}