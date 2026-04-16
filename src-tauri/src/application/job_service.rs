//! Job application service.
//!
//! Orchestrates business operations for jobs, enforcing domain rules
//! and coordinating with the repository layer.

use std::sync::Arc;

use artifex_job_queue::{Job, JobRepository, JobStatus};
use artifex_shared_kernel::{ArtifexError, JobId, ProjectId, Timestamp};
use serde_json::Value as JsonValue;

/// Application service for job operations.
#[derive(Clone)]
pub struct JobApplicationService {
    repo: Arc<dyn JobRepository>,
}

impl JobApplicationService {
    /// Creates a new JobApplicationService.
    pub fn new(repo: Arc<dyn JobRepository>) -> Self {
        Self { repo }
    }

    /// Creates a new job with Pending status.
    pub async fn create_job(
        &self,
        project_id: &str,
        job_type: &str,
        operation: JsonValue,
    ) -> Result<Job, ArtifexError> {
        let project_uuid = uuid::Uuid::parse_str(project_id)
            .map_err(|e| ArtifexError::validation(format!("Invalid project id: {}", e)))?;
        let pid = ProjectId::from_uuid(project_uuid);

        let job = Job::new(pid, job_type, operation);
        self.repo.create(&job).await?;
        Ok(job)
    }

    /// Gets a single job by ID.
    pub async fn get_job(&self, id: &str) -> Result<Job, ArtifexError> {
        let job_uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| ArtifexError::validation(format!("Invalid job id: {}", e)))?;
        let jid = JobId::from_uuid(job_uuid);

        self.repo
            .find_by_id(&jid)
            .await?
            .ok_or_else(|| ArtifexError::not_found("Job", id))
    }

    /// Lists all jobs for a given project with optional filters.
    pub async fn list_jobs(
        &self,
        project_id: &str,
        job_type: Option<&str>,
        status: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Job>, ArtifexError> {
        let project_uuid = uuid::Uuid::parse_str(project_id)
            .map_err(|e| ArtifexError::validation(format!("Invalid project id: {}", e)))?;
        let pid = ProjectId::from_uuid(project_uuid);

        let mut jobs = self.repo.list_by_project(&pid).await?;

        // Filter by job_type if provided
        if let Some(jt) = job_type {
            jobs.retain(|j| j.job_type == jt);
        }

        // Filter by status if provided
        if let Some(s) = status {
            let status_lower = s.to_lowercase();
            jobs.retain(|j| format!("{:?}", j.status).to_lowercase() == status_lower);
        }

        // Apply offset and limit for pagination
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(100) as usize;

        jobs = jobs.into_iter().skip(offset).take(limit).collect();

        Ok(jobs)
    }

    /// Cancels a job. Only Pending jobs can be cancelled.
    ///
    /// # Errors
    /// - `NotFound` if the job does not exist
    /// - `ValidationError` if the job is not in Pending status
    pub async fn cancel_job(&self, id: &str) -> Result<(), ArtifexError> {
        let job_uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| ArtifexError::validation(format!("Invalid job id: {}", e)))?;
        let jid = JobId::from_uuid(job_uuid);

        let mut job = self
            .repo
            .find_by_id(&jid)
            .await?
            .ok_or_else(|| ArtifexError::not_found("Job", id))?;

        if job.status != JobStatus::Pending {
            return Err(ArtifexError::validation(
                "Only Pending jobs can be cancelled",
            ));
        }

        job.status = JobStatus::Cancelled;
        job.completed_at = Some(Timestamp::now());
        job.updated_at = Timestamp::now();

        self.repo.update_status(&jid, JobStatus::Cancelled).await
    }

    /// Updates the progress of a running job.
    pub async fn update_job_progress(
        &self,
        id: &str,
        percent: u8,
        message: Option<&str>,
    ) -> Result<(), ArtifexError> {
        let job_uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| ArtifexError::validation(format!("Invalid job id: {}", e)))?;
        let jid = JobId::from_uuid(job_uuid);

        self.repo.update_progress(&jid, percent, message).await
    }

    /// Transitions a job to Running status.
    pub async fn start_job(&self, id: &str) -> Result<(), ArtifexError> {
        let job_uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| ArtifexError::validation(format!("Invalid job id: {}", e)))?;
        let jid = JobId::from_uuid(job_uuid);

        let job = self
            .repo
            .find_by_id(&jid)
            .await?
            .ok_or_else(|| ArtifexError::not_found("Job", id))?;

        if job.status != JobStatus::Pending {
            return Err(ArtifexError::validation(format!(
                "Cannot start job in status {:?}",
                job.status
            )));
        }

        self.repo.update_status(&jid, JobStatus::Running).await
    }

    /// Transitions a job to Completed status.
    pub async fn complete_job(&self, id: &str) -> Result<(), ArtifexError> {
        let job_uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| ArtifexError::validation(format!("Invalid job id: {}", e)))?;
        let jid = JobId::from_uuid(job_uuid);

        let job = self
            .repo
            .find_by_id(&jid)
            .await?
            .ok_or_else(|| ArtifexError::not_found("Job", id))?;

        if job.status != JobStatus::Running {
            return Err(ArtifexError::validation(format!(
                "Cannot complete job in status {:?}",
                job.status
            )));
        }

        self.repo.update_status(&jid, JobStatus::Completed).await
    }

    /// Transitions a job to Failed status with an error message.
    pub async fn fail_job(&self, id: &str, _error: &str) -> Result<(), ArtifexError> {
        let job_uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| ArtifexError::validation(format!("Invalid job id: {}", e)))?;
        let jid = JobId::from_uuid(job_uuid);

        let job = self
            .repo
            .find_by_id(&jid)
            .await?
            .ok_or_else(|| ArtifexError::not_found("Job", id))?;

        if job.status != JobStatus::Running {
            return Err(ArtifexError::validation(format!(
                "Cannot fail job in status {:?}",
                job.status
            )));
        }

        self.repo.update_status(&jid, JobStatus::Failed).await
    }

    /// Lists jobs by status for a project.
    pub async fn list_jobs_by_status(
        &self,
        project_id: &str,
        status: JobStatus,
    ) -> Result<Vec<Job>, ArtifexError> {
        let project_uuid = uuid::Uuid::parse_str(project_id)
            .map_err(|e| ArtifexError::validation(format!("Invalid project id: {}", e)))?;
        let pid = ProjectId::from_uuid(project_uuid);

        self.repo.list_by_status(&pid, status).await
    }
}
