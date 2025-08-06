use actix_web::{web, HttpResponse, Responder};
use posemesh_domain_http::{domain_data::DownloadQuery, DomainClient};
use uuid::Uuid;

use crate::models::{CreateJobRequest, ListJobsRequest, TaskTimingV1Input, TASK_TIMING_V1};

async fn create_job(
    pool: web::Data<sqlx::PgPool>,
    domain_client: web::Data<DomainClient>,
    data_dir: web::Data<String>,
    job: web::Json<CreateJobRequest>,
) -> impl Responder {
    let id = Uuid::new_v4().to_string();
    let job_type = job.job_type.as_str();
    match job_type {
        TASK_TIMING_V1 => {
            let res = serde_json::from_value::<TaskTimingV1Input>(job.input.clone());
            if let Err(e) = res {
                tracing::error!("Failed to parse input: {:?}", e);
                return HttpResponse::BadRequest().body("Failed to parse input");
            }
            let input = res.unwrap();
            if let Err(e) = crate::domain::download_for_job(&domain_client, &id, &input.domain_id, &data_dir, &DownloadQuery {
                ids: input.domain_data_ids,
                name: None,
                data_type: None,
            }).await {
                tracing::error!("Failed to download domain data: {:?}", e);
                return HttpResponse::InternalServerError().body("Failed to download domain data");
            }
        }
        _ => {
            return HttpResponse::BadRequest().body("Invalid job type");
        }
    }

    let res = crate::pg::create_job(&pool, &id, &job.input, &job.job_type).await;
    if let Err(e) = res {
        tracing::error!("Failed to create job: {:?}", e);
        return HttpResponse::InternalServerError().body("Failed to create job");
    }
    let job_schema = res.unwrap();
    HttpResponse::Ok().json(job_schema)
}

async fn list_jobs(
    pool: web::Data<sqlx::PgPool>,
    query: web::Query<ListJobsRequest>,
) -> impl Responder {
    match crate::pg::list_jobs(&pool, query.limit, query.offset.unwrap_or(0)).await {
        Ok(jobs) => HttpResponse::Ok().json(jobs),
        Err(e) => {
            tracing::error!("Failed to list jobs: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to list jobs")
        }
    }
}

async fn get_job(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<String>,
) -> impl Responder {
    let job_id = path.into_inner();
    match crate::pg::get_job_by_id(&pool, &job_id).await {
        Ok(Some(job)) => HttpResponse::Ok().json(job),
        Ok(None) => HttpResponse::NotFound().body("Job not found"),
        Err(e) => {
            tracing::error!("Failed to get job: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to get job")
        }
    }
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
