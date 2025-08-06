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
