use serde::{Deserialize, Serialize};
use sqlx::types::chrono;

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone)]
#[serde(rename_all="snake_case")]
#[sqlx(rename_all="lowercase", type_name="text")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelling,
    Cancelled,
    Completing,
    Uploading,
}

#[derive(Serialize, Deserialize)]
pub struct JobError {
    pub code: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct JobCommon {
    pub id: String,
    #[sqlx(rename = "job_status")]
    pub status: JobStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub domain_id: String,
    pub query: serde_json::Value,
    #[sqlx(skip)]
    pub hash: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Job {
    #[serde(flatten)]
    #[sqlx(flatten)]
    pub common: JobCommon,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
    pub job_type: String,
}

#[derive(Deserialize, Debug)]
pub struct CreateJobRequest {
    pub job_type: String,
    pub domain_id: String,
    pub query: serde_json::Value,
    pub input: serde_json::Value,
}

#[derive(Deserialize, Debug)]
pub struct RetryJobRequest {
    pub job_type: String,
    pub input: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueryJob {
    pub status: Option<JobStatus>,
    pub job_type: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ListJobsRequest {
    pub limit: i64,
    pub offset: Option<i64>,
    #[serde(flatten)]
    pub query: Option<QueryJob>,
}
