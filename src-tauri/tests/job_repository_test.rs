//! Integration tests for SqliteJobRepository.
//!
//! These tests verify the database round-trip behavior of the job repository.

mod test_helpers;

use artifex_asset_management::{Project, ProjectRepository};
use artifex_job_queue::{Job, JobRepository, JobStatus};
use std::sync::Arc;
use serde_json::json;

use test_helpers::setup_test_db;
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
async fn test_create_job_and_find_by_id() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let project = create_test_project(&pool).await;
    let repo = SqliteJobRepository::new(pool);
    let repo = Arc::new(repo);

    let job = make_test_job(&project.id);

    // Create
    repo.create(&job).await.expect("Failed to create job");

    // Find by ID
    let found = repo
        .find_by_id(&job.id)
        .await
        .expect("Failed to find job")
        .expect("Job not found");

    assert_eq!(found.id, job.id);
    assert_eq!(found.job_type, "image_generate");
    assert_eq!(found.status, JobStatus::Pending);
    assert_eq!(found.progress_percent, 0);
}

#[tokio::test]
async fn test_list_jobs_by_project() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let project = create_test_project(&pool).await;
    let repo = SqliteJobRepository::new(pool);
    let repo = Arc::new(repo);

    // Create 3 jobs
    let job1 = make_test_job(&project.id);
    let job2 = make_test_job(&project.id);
    let job3 = make_test_job(&project.id);

    repo.create(&job1).await.expect("Failed to create job1");
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    repo.create(&job2).await.expect("Failed to create job2");
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    repo.create(&job3).await.expect("Failed to create job3");

    // List by project
    let jobs = repo.list_by_project(&project.id).await.expect("Failed to list jobs");
    assert_eq!(jobs.len(), 3);

    // Should be ordered by created_at DESC (most recent first)
    assert_eq!(jobs[0].id, job3.id);
    assert_eq!(jobs[1].id, job2.id);
    assert_eq!(jobs[2].id, job1.id);
}

#[tokio::test]
async fn test_list_jobs_by_status() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let project = create_test_project(&pool).await;
    let repo = SqliteJobRepository::new(pool);
    let repo = Arc::new(repo);

    let job1 = make_test_job(&project.id);
    let job2 = make_test_job(&project.id);
    let job3 = make_test_job(&project.id);

    repo.create(&job1).await.expect("Failed to create job1");
    repo.create(&job2).await.expect("Failed to create job2");
    repo.create(&job3).await.expect("Failed to create job3");

    // Update job2 to Running
    repo.update_status(&job2.id, JobStatus::Running)
        .await
        .expect("Failed to update status");

    // List Pending jobs
    let pending = repo
        .list_by_status(&project.id, JobStatus::Pending)
        .await
        .expect("Failed to list pending jobs");
    assert_eq!(pending.len(), 2);

    // List Running jobs
    let running = repo
        .list_by_status(&project.id, JobStatus::Running)
        .await
        .expect("Failed to list running jobs");
    assert_eq!(running.len(), 1);
    assert_eq!(running[0].id, job2.id);

    // List Completed jobs (should be none)
    let completed = repo
        .list_by_status(&project.id, JobStatus::Completed)
        .await
        .expect("Failed to list completed jobs");
    assert!(completed.is_empty());
}

#[tokio::test]
async fn test_update_progress() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let project = create_test_project(&pool).await;
    let repo = SqliteJobRepository::new(pool);
    let repo = Arc::new(repo);

    let job = make_test_job(&project.id);
    repo.create(&job).await.expect("Failed to create job");

    // Start the job
    repo.update_status(&job.id, JobStatus::Running)
        .await
        .expect("Failed to start job");

    // Update progress
    repo.update_progress(&job.id, 50, Some("Halfway done"))
        .await
        .expect("Failed to update progress");

    // Verify
    let found = repo
        .find_by_id(&job.id)
        .await
        .expect("Failed to find job")
        .expect("Job not found");

    assert_eq!(found.status, JobStatus::Running);
    assert_eq!(found.progress_percent, 50);
    assert_eq!(found.progress_message, Some("Halfway done".to_string()));
}

#[tokio::test]
async fn test_cancel_pending_job() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let project = create_test_project(&pool).await;
    let repo = SqliteJobRepository::new(pool);
    let repo = Arc::new(repo);

    let job = make_test_job(&project.id);
    repo.create(&job).await.expect("Failed to create job");

    // Cancel the job
    repo.update_status(&job.id, JobStatus::Cancelled)
        .await
        .expect("Failed to cancel job");

    // Verify
    let found = repo
        .find_by_id(&job.id)
        .await
        .expect("Failed to find job")
        .expect("Job not found");

    assert_eq!(found.status, JobStatus::Cancelled);
}

#[tokio::test]
async fn test_find_nonexistent_job() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = SqliteJobRepository::new(pool);

    // Try to find a random UUID
    let random_id = artifex_shared_kernel::JobId::new();
    let found = repo.find_by_id(&random_id).await.expect("Failed to find job");

    assert!(found.is_none());
}

#[tokio::test]
async fn test_update_status_not_found() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = SqliteJobRepository::new(pool);

    let random_id = artifex_shared_kernel::JobId::new();
    let result = repo.update_status(&random_id, JobStatus::Running).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, artifex_shared_kernel::ArtifexError::NotFound(_)));
}

#[tokio::test]
async fn test_update_progress_not_found() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = SqliteJobRepository::new(pool);

    let random_id = artifex_shared_kernel::JobId::new();
    let result = repo.update_progress(&random_id, 50, None).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, artifex_shared_kernel::ArtifexError::NotFound(_)));
}
