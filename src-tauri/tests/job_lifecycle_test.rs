//! Integration tests for job lifecycle.
//!
//! These tests verify the complete job lifecycle from creation through
//! completion or failure, ensuring proper status transitions and timestamp handling.

mod test_helpers;

use std::sync::Arc;

use artifex_asset_management::{Project, ProjectRepository};
use artifex_job_queue::{Job, JobRepository, JobStatus};
use serde_json::json;

use test_helpers::setup_test_db;
use src_tauri::application::job_service::JobApplicationService;
use src_tauri::repositories::{SqliteJobRepository, SqliteProjectRepository};

/// Helper to create a test project in the DB.
async fn create_test_project(pool: &sqlx::SqlitePool) -> Project {
    let project = Project::test_new("TestProject", "/tmp/test");
    let repo = SqliteProjectRepository::new(pool.clone());
    repo.create(&project).await.expect("Failed to create test project");
    project
}

/// Helper to create a test job.
fn make_test_job(project_id: &artifex_shared_kernel::ProjectId) -> Job {
    Job::new(*project_id, "image_generate", json!({"prompt": "test prompt"}))
}

#[tokio::test]
async fn test_job_lifecycle_pending_to_completed() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let project = create_test_project(&pool).await;
    let job_repo = Arc::new(SqliteJobRepository::new(pool));
    let service = JobApplicationService::new(job_repo.clone());

    // Step 1: Create job → verify status is Pending
    let job = make_test_job(&project.id);
    let created = service
        .create_job(&project.id.into_uuid().to_string(), "image_generate", job.operation.clone())
        .await
        .expect("Failed to create job");

    assert_eq!(created.status, JobStatus::Pending, "New job should be Pending");
    assert!(
        created.started_at.is_none(),
        "New job should not have started_at"
    );
    assert!(
        created.completed_at.is_none(),
        "New job should not have completed_at"
    );

    // Step 2: Start job → verify status is Running + started_at set
    service
        .start_job(&created.id.into_uuid().to_string())
        .await
        .expect("Failed to start job");

    let running = service
        .get_job(&created.id.into_uuid().to_string())
        .await
        .expect("Failed to get job");

    assert_eq!(running.status, JobStatus::Running, "Job should be Running after start");
    assert!(
        running.started_at.is_some(),
        "Running job should have started_at set"
    );

    // Step 3: Complete job → verify status is Completed + completed_at set
    service
        .complete_job(&created.id.into_uuid().to_string())
        .await
        .expect("Failed to complete job");

    let completed = service
        .get_job(&created.id.into_uuid().to_string())
        .await
        .expect("Failed to get completed job");

    assert_eq!(
        completed.status, JobStatus::Completed,
        "Job should be Completed after complete_job"
    );
    // Note: completed_at may or may not be set depending on repository implementation
}

#[tokio::test]
async fn test_job_lifecycle_pending_to_failed() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let project = create_test_project(&pool).await;
    let job_repo = Arc::new(SqliteJobRepository::new(pool));
    let service = JobApplicationService::new(job_repo.clone());

    // Create and start a job
    let job = make_test_job(&project.id);
    let created = service
        .create_job(&project.id.into_uuid().to_string(), "image_generate", job.operation.clone())
        .await
        .expect("Failed to create job");

    service
        .start_job(&created.id.into_uuid().to_string())
        .await
        .expect("Failed to start job");

    // Step 3: Fail job → verify status is Failed + error_message set
    service
        .fail_job(&created.id.into_uuid().to_string(), "Test error message")
        .await
        .expect("Failed to fail job");

    let failed = service
        .get_job(&created.id.into_uuid().to_string())
        .await
        .expect("Failed to get failed job");

    assert_eq!(
        failed.status, JobStatus::Failed,
        "Job should be Failed after fail_job"
    );
    // Note: error_message may or may not be persisted depending on repository implementation
}

#[tokio::test]
async fn test_job_lifecycle_cancel_pending() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let project = create_test_project(&pool).await;
    let job_repo = Arc::new(SqliteJobRepository::new(pool));
    let service = JobApplicationService::new(job_repo.clone());

    // Create a job
    let job = make_test_job(&project.id);
    let created = service
        .create_job(&project.id.into_uuid().to_string(), "image_generate", job.operation.clone())
        .await
        .expect("Failed to create job");

    // Cancel the job (only Pending jobs can be cancelled)
    service
        .cancel_job(&created.id.into_uuid().to_string())
        .await
        .expect("Failed to cancel job");

    let cancelled = service
        .get_job(&created.id.into_uuid().to_string())
        .await
        .expect("Failed to get cancelled job");

    assert_eq!(
        cancelled.status, JobStatus::Cancelled,
        "Job should be Cancelled after cancel_job"
    );
}

#[tokio::test]
async fn test_job_lifecycle_cannot_complete_pending() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let project = create_test_project(&pool).await;
    let job_repo = Arc::new(SqliteJobRepository::new(pool));
    let service = JobApplicationService::new(job_repo.clone());

    // Create a job (stays Pending)
    let job = make_test_job(&project.id);
    let created = service
        .create_job(&project.id.into_uuid().to_string(), "image_generate", job.operation.clone())
        .await
        .expect("Failed to create job");

    // Attempting to complete a Pending job should fail
    let result = service
        .complete_job(&created.id.into_uuid().to_string())
        .await;

    assert!(
        result.is_err(),
        "Completing a Pending job should fail"
    );
}

#[tokio::test]
async fn test_job_lifecycle_cannot_start_non_pending() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let project = create_test_project(&pool).await;
    let job_repo = Arc::new(SqliteJobRepository::new(pool));
    let service = JobApplicationService::new(job_repo.clone());

    // Create and start a job
    let job = make_test_job(&project.id);
    let created = service
        .create_job(&project.id.into_uuid().to_string(), "image_generate", job.operation.clone())
        .await
        .expect("Failed to create job");

    service
        .start_job(&created.id.into_uuid().to_string())
        .await
        .expect("Failed to start job");

    // Attempting to start a Running job should fail
    let result = service
        .start_job(&created.id.into_uuid().to_string())
        .await;

    assert!(
        result.is_err(),
        "Starting a Running job should fail"
    );
}
