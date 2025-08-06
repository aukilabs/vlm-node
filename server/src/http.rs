use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Job {
    id: String,
    status: String,
    job_type: String,
    created_at: String,
    updated_at: String,
    input: serde_json::Value,
    output: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct CreateJobRequest {
    job_type: String,
    input: serde_json::Value,
}

async fn create_job(job: web::Json<CreateJobRequest>) -> impl Responder {
    // TODO: Insert job into database and return created job
    HttpResponse::Ok().json(Job {
        id: "mock_id".to_string(),
        status: "created".to_string(),
        job_type: job.job_type.clone(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
        input: job.input.clone(),
        output: None,
        error: None,
    })
}

async fn list_jobs() -> impl Responder {
    // TODO: Query jobs from database
    let jobs: Vec<Job> = vec![]; // Placeholder
    HttpResponse::Ok().json(jobs)
}

async fn get_job(path: web::Path<String>) -> impl Responder {
    let job_id = path.into_inner();
    // TODO: Query job by id from database
    let job = Job {
        id: job_id,
        status: "created".to_string(),
        job_type: "mock_type".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
        input: serde_json::json!({}),
        output: None,
        error: None,
    };
    HttpResponse::Ok().json(job)
}

pub fn app_config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(
            web::resource("/jobs")
                .route(web::post().to(create_job))
                .route(web::get().to(list_jobs))
        )
        .service(
            web::resource("/jobs/{id}")
                .route(web::get().to(get_job))
        );
}
