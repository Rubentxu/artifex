//! Job repository trait.

use async_trait::async_trait;

use artifex_shared_kernel::{ArtifexError, JobId, ProjectId};
use super::JobStatus;

use super::Job;

/// Repository trait for job persistence.
#[async_trait]
pub trait JobRepository: Send + Sync {
    /// Creates a new job.
    async fn create(&self, job: &Job) -> Result<(), ArtifexError>;

    /// Finds a job by its ID.
    async fn find_by_id(&self, id: &JobId) -> Result<Option<Job>, ArtifexError>;

    /// Lists all jobs for a given project.
    async fn list_by_project(&self, project_id: &ProjectId) -> Result<Vec<Job>, ArtifexError>;

    /// Lists all jobs for a given project with a specific status.
    async fn list_by_status(
        &self,
        project_id: &ProjectId,
        status: JobStatus,
    ) -> Result<Vec<Job>, ArtifexError>;

    /// Lists all jobs with a specific status regardless of project.
    /// Used by the worker runner to poll for pending jobs across all projects.
    async fn list_all_by_status(&self, status: JobStatus) -> Result<Vec<Job>, ArtifexError>;

    /// Updates the status of a job.
    async fn update_status(&self, id: &JobId, status: JobStatus) -> Result<(), ArtifexError>;

    /// Updates the progress of a running job atomically.
    async fn update_progress(
        &self,
        id: &JobId,
        percent: u8,
        message: Option<&str>,
    ) -> Result<(), ArtifexError>;

    /// Updates a job as failed with an error message.
    /// Sets status to Failed, persists the error_message, and sets completed_at.
    async fn update_failure(&self, id: &JobId, error_message: &str) -> Result<(), ArtifexError>;

    /// Marks a job as completed successfully.
    /// Sets status to Completed and sets completed_at timestamp.
    async fn mark_completed(&self, id: &JobId) -> Result<(), ArtifexError>;
}
