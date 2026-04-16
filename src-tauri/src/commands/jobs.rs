//! Job management IPC commands (thin adapters).

use tauri::State;

use artifex_job_queue::Job;

use crate::dto::{CreateJobRequest, JobResponse};
use crate::state::AppState;

/// Creates a new job.
#[tauri::command]
pub async fn create_job(
    state: State<'_, AppState>,
    request: CreateJobRequest,
) -> Result<JobResponse, String> {
    let job = state
        .job_service
        .create_job(&request.project_id, &request.job_type, request.operation)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job_to_response(job))
}

/// Lists all jobs for a project with optional filters.
#[tauri::command]
pub async fn list_jobs(
    state: State<'_, AppState>,
    project_id: String,
    job_type: Option<String>,
    status: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<JobResponse>, String> {
    let jobs = state
        .job_service
        .list_jobs(&project_id, job_type.as_deref(), status.as_deref(), limit, offset)
        .await
        .map_err(|e| e.to_string())?;

    Ok(jobs.into_iter().map(job_to_response).collect())
}

/// Gets a single job by ID.
#[tauri::command]
pub async fn get_job(state: State<'_, AppState>, id: String) -> Result<JobResponse, String> {
    let job = state
        .job_service
        .get_job(&id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job_to_response(job))
}

/// Cancels a job. Only Pending jobs can be cancelled.
#[tauri::command]
pub async fn cancel_job(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .job_service
        .cancel_job(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Converts a domain Job to a JobResponse DTO.
pub fn job_to_response(job: Job) -> JobResponse {
    JobResponse {
        id: job.id.into_uuid().to_string(),
        project_id: job.project_id.into_uuid().to_string(),
        job_type: job.job_type,
        status: format!("{:?}", job.status).to_lowercase(),
        operation: job.operation,
        progress_percent: job.progress_percent,
        progress_message: job.progress_message,
        error_message: job.error_message,
        started_at: job.started_at.map(|t| t.to_string()),
        completed_at: job.completed_at.map(|t| t.to_string()),
        created_at: job.created_at.to_string(),
        updated_at: job.updated_at.to_string(),
    }
}
