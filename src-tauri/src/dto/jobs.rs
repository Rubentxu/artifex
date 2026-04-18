//! Job DTOs.

use serde::{Deserialize, Serialize};

/// Response type for job data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResponse {
    pub id: String,
    pub project_id: String,
    pub job_type: String,
    pub status: String,
    pub operation: serde_json::Value,
    pub progress_percent: u8,
    pub progress_message: Option<String>,
    pub error_message: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Request type for creating a job.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateJobRequest {
    pub project_id: String,
    pub job_type: String,
    pub operation: serde_json::Value,
}
