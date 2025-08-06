use actix_web::{web, App, HttpServer};
use posemesh_domain_http::{config::Config, DomainClient};

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
    let domain_client = DomainClient::new_with_app_credential(&domain_config.api_url, &domain_config.dds_url, &domain_config.client_id, &domain_config.app_key, &domain_config.app_secret).await.expect("Failed to initialize domain client");
    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "../data".to_string());
    
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(domain_client.clone()))
            .app_data(web::Data::new(data_dir.clone()))
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
