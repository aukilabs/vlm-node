use actix_cors::Cors;
use actix_web::{http::header::{AUTHORIZATION, CONTENT_TYPE}, web, App, HttpServer};
use posemesh_domain_http::{config::Config, DomainClient};

use crate::{domain::upload_for_job, models::{JobStatus, QueryJob}};

mod pg;
mod http;
mod models;
mod domain;

pub fn init_tracing() -> tracing::span::Span {
    let machine_id = match machine_uid::get() {
        Ok(id) => id,
        Err(_) => hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string()),
    };
    
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with_span_list(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_current_span(true)
        .flatten_event(true)
        .init();

    tracing::span!(tracing::Level::INFO, "machine_id", machine_id = %machine_id)
}

#[tokio::main]
async fn main() {
    let span = init_tracing();
    let _guard = span.enter();
    
    let pool = pg::init_pg(&pg::Config::from_env().unwrap()).await.expect("Failed to initialize database");
    let domain_config = Config::from_env().expect("Failed to initialize domain config");
    let domain_client = DomainClient::new_with_user_credential(&domain_config.api_url, &domain_config.dds_url, &domain_config.client_id, &domain_config.email.as_ref().unwrap(), &domain_config.password.as_ref().unwrap()).await.expect("Failed to initialize domain client");
    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "../data".to_string());

    let domain_client_clone = domain_client.clone();
    let data_dir_clone = data_dir.clone();
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        loop {
            interval.tick().await;
            let jobs = pg::list_jobs(&pool_clone, 1, 0, Some(QueryJob {
                status: Some(JobStatus::Uploading),
                job_type: None,
            })).await;
            if let Ok(jobs) = jobs {
                if jobs.is_empty() {
                    continue;
                }
                let job = &jobs[0];
                let job_id = &job.common.id;
                let data_dir = format!("{}/output/{}", &data_dir_clone, job_id);
                if !std::path::Path::new(&data_dir).exists() {
                    continue;
                }
                let res = upload_for_job(&domain_client_clone, &job.common.domain_id, &data_dir).await;
                if let Err(e) = res {
                    if let Err(e) = pg::fail_job(&pool_clone, job_id, &e.to_string(), &job.common.updated_at).await {
                        tracing::error!("Failed to fail job: {:?}", e);
                    }
                }
            } else {
                tracing::error!("Failed to list jobs: {:?}", jobs.err());
            }
        }
    });
    
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "PUT"])
            .allowed_headers(vec![AUTHORIZATION, CONTENT_TYPE])
            .supports_credentials()
            .max_age(3600);
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(domain_client.clone()))
            .app_data(web::Data::new(data_dir.clone()))
            .wrap(cors)
            .configure(http::app_config)
    })
        .bind(format!(
            "0.0.0.0:{}",
            std::env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string())
        ))
        .unwrap()
        .run();

    let admin_server = HttpServer::new(move || {
        App::new()
        .route("/health", web::get().to(|| async { "OK" }))
    })
        .bind(format!(
            "0.0.0.0:{}",
            std::env::var("SERVER_ADMIN_PORT").unwrap_or_else(|_| "18190".to_string())
        ))
        .unwrap()
        .run();

    tracing::info!("Starting servers...");

    tokio::try_join!(server, admin_server).expect("Failed to start servers");
}
