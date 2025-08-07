use std::path::Path;
use std::time::Duration;
use sqlx::postgres::PgPoolOptions;
use sqlx::migrate::Migrator;
use sqlx::PgPool;

use crate::models::{Job, JobStatus};

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
            postgres_url: std::env::var("POSTGRES_URL").unwrap_or("postgres://postgres:postgres@localhost:5432/postgres".to_string()),
            postgres_pool_size: std::env::var("POSTGRES_POOL_SIZE").unwrap_or("10".to_string()).parse::<u32>()?,
            postgres_pool_idle_timeout: std::env::var("POSTGRES_POOL_IDLE_TIMEOUT").unwrap_or("300".to_string()).parse::<u64>()?,
            postgres_pool_connection_timeout: std::env::var("POSTGRES_POOL_CONNECTION_TIMEOUT").unwrap_or("10".to_string()).parse::<u64>()?,
            migrations_path: std::env::var("MIGRATIONS_PATH").unwrap_or("migrations".to_string()),
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

pub async fn create_job(
    pool: &PgPool,
    id: &str,
    input: &serde_json::Value,
    job_type: &str,
) -> Result<Job, sqlx::Error> {
    let rec = sqlx::query_as::<_, Job>(
        "
        INSERT INTO jobs (id, input, job_type, job_status)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "
    )
    .bind(id)
    .bind(input)
    .bind(job_type)
    .bind(JobStatus::Pending)
    .fetch_one(pool)
    .await?;
    Ok(rec)
}

pub async fn list_jobs(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<Job>, sqlx::Error> {
    let jobs = sqlx::query_as::<_, Job>(
        r#"
        SELECT *
        FROM jobs
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(jobs)
}

pub async fn get_job_by_id(
    pool: &PgPool,
    id: &str,
) -> Result<Option<Job>, sqlx::Error> {
    let job = sqlx::query_as::<_, Job>(
        r#"
        SELECT *
        FROM jobs
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(job)
}

pub async fn retry_job(
    pool: &PgPool,
    id: &str,
    status: &JobStatus,
    updated_at: &chrono::DateTime<chrono::Utc>,
) -> Result<Option<Job>, sqlx::Error> {
    let job = sqlx::query_as::<_, Job>(
        r#"
        UPDATE jobs
        SET job_status = $1, updated_at = now(), error = null, output = null
        WHERE id = $2 AND updated_at = $3
        RETURNING *
        "#
    )
    .bind(status)
    .bind(id)
    .bind(updated_at)
    .fetch_optional(pool)
    .await?;
    Ok(job)
}


