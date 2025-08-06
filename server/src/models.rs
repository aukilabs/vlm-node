use serde::{Deserialize, Serialize};
use sqlx::types::chrono;

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[serde(rename_all="snake_case")]
#[sqlx(rename_all="lowercase", type_name="text")]
pub enum JobStatus {
    Pending,
    Running,
    Postprocessing,
    Completed,
    Failed,
    Cancelling,
    Cancelled,
    Completing,
}

#[derive(Serialize, Deserialize)]
pub struct JobError {
    pub code: String,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct JobResult {
    pub job_id: String,
    pub data: Option<JobOutput>,
    pub error: Option<JobError>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "job_type", content = "data", rename_all = "snake_case")]
pub enum JobOutput {
    TaskTimingV1(TaskTimingV1Output),
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct JobCommon {
    pub id: String,
    #[sqlx(rename = "job_status")]
    pub status: JobStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[sqlx(skip)]
    pub hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct Job {
    #[serde(flatten)]
    pub common: JobCommon,
    pub error: Option<serde_json::Value>,
    pub job_type: JobDescription,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct JobSchema {
    #[serde(flatten)]
    #[sqlx(flatten)]
    pub common: JobCommon,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
    pub job_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskTimingV1Input {
    pub prompt: String,
    pub webhook_url: String,
    pub domain_data_ids: Vec<String>,
    pub domain_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskTimingV1Output {
    pub start_image_id: String,
    pub end_image_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "job_type", rename_all = "snake_case")]
pub enum JobDescription {
    TaskTimingV1 {
        input: TaskTimingV1Input,
        output: Option<TaskTimingV1Output>,
    },
}

#[derive(Deserialize, Debug)]
pub struct CreateJobRequest {
    pub job_type: String,
    pub input: serde_json::Value,
}

#[derive(Deserialize, Debug)]
pub struct ListJobsRequest {
    pub limit: i64,
    pub offset: Option<i64>,
}

pub const TASK_TIMING_V1: &str = "task_timing_v1";
