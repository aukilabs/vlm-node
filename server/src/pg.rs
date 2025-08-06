use std::path::Path;
use std::time::Duration;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::migrate::Migrator;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct Config {
    pub postgres_url: String,
    pub postgres_pool_size: u32,
    pub postgres_pool_idle_timeout: u64,
    pub postgres_pool_connection_timeout: u64,
    pub migrations_path: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Config {
            postgres_url: std::env::var("POSTGRES_URL")?,
            postgres_pool_size: std::env::var("POSTGRES_POOL_SIZE").unwrap_or("10".to_string()).parse::<u32>()?,
            postgres_pool_idle_timeout: std::env::var("POSTGRES_POOL_IDLE_TIMEOUT").unwrap_or("300".to_string()).parse::<u64>()?,
            postgres_pool_connection_timeout: std::env::var("POSTGRES_POOL_CONNECTION_TIMEOUT").unwrap_or("10".to_string()).parse::<u64>()?,
            migrations_path: std::env::var("MIGRATIONS_PATH").unwrap_or("../migrations".to_string()),
        })
    }
}

pub async fn init_pg(config: &Config) -> Result<PgPool, Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(config.postgres_pool_size)
        .idle_timeout(Duration::from_secs(config.postgres_pool_idle_timeout))
        .acquire_timeout(Duration::from_secs(config.postgres_pool_connection_timeout))
        .connect(&config.postgres_url)
        .await?;

    let migrator = Migrator::new(Path::new(&config.migrations_path))
        .await?;
    migrator.run(&pool)
        .await?;

    Ok(pool)
}


