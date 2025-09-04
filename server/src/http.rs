use actix_web::{web, HttpResponse, Responder};
use posemesh_domain_http::{domain_data::DownloadQuery, DomainClient};
use uuid::Uuid;

use crate::{models::{CreateJobRequest, ListJobsRequest, RetryJobRequest}, stream::ws_index};

async fn create_job(
    pool: web::Data<sqlx::PgPool>,
    domain_client: web::Data<DomainClient>,
    data_dir: web::Data<String>,
    job: web::Json<CreateJobRequest>,
) -> impl Responder {
    let id = Uuid::new_v4().to_string();
    let res = serde_json::from_value::<DownloadQuery>(job.query.clone());
    if let Err(e) = res {
        tracing::error!("Failed to parse query: {:?}", e);
        return HttpResponse::BadRequest().body("Failed to parse query");
    }
    let query = res.unwrap();
    let count = crate::domain::download_for_job(&domain_client, &id, &job.domain_id, &data_dir, &query).await;
    if let Err(e) = count {
        tracing::error!("Failed to download domain data: {:?}", e);
        // Attempt to delete the input folder for this job
        let input_dir = format!("{}/input/{}", &*data_dir, &id);
        if let Err(e) = tokio::fs::remove_dir_all(&input_dir).await {
            tracing::warn!("Failed to delete input folder {}: {:?}", input_dir, e);
        }
        return HttpResponse::InternalServerError().body("Failed to download domain data");
    }
    if count.unwrap() == 0 {
        return HttpResponse::BadRequest().body("No data found");
    }

    let res = crate::pg::create_job(&pool, &id, &job.domain_id, &job.query, &job.input, &job.job_type).await;
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
    match crate::pg::list_jobs(&pool, query.limit, query.offset.unwrap_or(0), query.query.clone()).await {
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

async fn retry_job(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<String>,
    body: web::Json<RetryJobRequest>,
) -> impl Responder {
    let job_id = path.into_inner();
    // Fetch the job to check its status
    let job = match crate::pg::get_job_by_id(&pool, &job_id).await {
        Ok(Some(job)) => job,
        Ok(None) => return HttpResponse::NotFound().body("Job not found"),
        Err(e) => {
            tracing::error!("Failed to get job: {:?}", e);
            return HttpResponse::InternalServerError().body("Failed to get job");
        }
    };
    if body.job_type != job.job_type {
        return HttpResponse::BadRequest().body("Job type mismatch");
    }

    // Only allow retry if job is Failed or Cancelled
    use crate::models::JobStatus;
    match job.common.status {
        JobStatus::Failed | JobStatus::Cancelled | JobStatus::Completed => {
            // Set job status to Pending, clear error and output
            let res = crate::pg::retry_job(&pool, &job_id, &JobStatus::Pending, &body.input, &job.common.updated_at).await;
            match res {
                Ok(_) => (),
                Err(e) => {
                    tracing::error!("Failed to update job status: {:?}", e);
                    return HttpResponse::InternalServerError().body("Failed to update job status");
                }
            }
        }
        _ => return HttpResponse::BadRequest().body("Only failed or cancelled jobs can be retried"),
    }

    match crate::pg::get_job_by_id(&pool, &job_id).await {
        Ok(Some(job)) => return HttpResponse::Ok().json(job),
        Ok(None) => return HttpResponse::NotFound().body("Job not found"),
        Err(e) => {
            tracing::error!("Failed to get job: {:?}", e);
            return HttpResponse::InternalServerError().body("Failed to get job");
        }
    }
}

pub fn app_config(cfg: &mut web::ServiceConfig) {
    use actix_web::middleware::Logger;

    cfg
        .service(
            web::resource("/api/v1/jobs")
                .wrap(Logger::default())
                .route(web::post().to(create_job))
                .route(web::get().to(list_jobs))
        )
        .service(
            web::resource("/api/v1/jobs/{id}")
                .wrap(Logger::default())
                .route(web::get().to(get_job))
                .route(web::put().to(retry_job))
        )
        .service(
            web::resource("/api/v1/ws")
                .wrap(Logger::default())
                .route(web::get().to(ws_index))
        );
}
