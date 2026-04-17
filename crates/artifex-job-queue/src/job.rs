//! Job model.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use artifex_shared_kernel::{JobId, ProjectId, Timestamp};

/// Job status enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    /// Job is waiting to be processed.
    #[default]
    Pending,
    /// Job is currently being processed.
    Running,
    /// Job completed successfully.
    Completed,
    /// Job failed.
    Failed,
    /// Job was cancelled.
    Cancelled,
}

/// A job in the queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique job identifier.
    pub id: JobId,
    /// The project this job belongs to.
    pub project_id: ProjectId,
    /// Type/category of the job.
    pub job_type: String,
    /// Current job status.
    pub status: JobStatus,
    /// Serialized operation parameters (JSON).
    pub operation: JsonValue,
    /// Progress percentage (0–100).
    pub progress_percent: u8,
    /// Optional human-readable progress message.
    pub progress_message: Option<String>,
    /// Error message if the job failed.
    pub error_message: Option<String>,
    /// When the job started processing.
    pub started_at: Option<Timestamp>,
    /// When the job completed (success or failure).
    pub completed_at: Option<Timestamp>,
    /// When the job was created.
    pub created_at: Timestamp,
    /// When the job was last updated.
    pub updated_at: Timestamp,
}

impl Job {
    /// Creates a new job with Pending status.
    ///
    /// The operation parameter is serialized as JSON.
    pub fn new(project_id: ProjectId, job_type: impl Into<String>, operation: JsonValue) -> Self {
        let now = Timestamp::now();
        Self {
            id: JobId::new(),
            project_id,
            job_type: job_type.into(),
            status: JobStatus::Pending,
            operation,
            progress_percent: 0,
            progress_message: None,
            error_message: None,
            started_at: None,
            completed_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Transitions the job to Running status and sets started_at.
    ///
    /// # Panics
    /// Panics if the job is not in Pending status.
    pub fn start(&mut self) {
        assert_eq!(
            self.status,
            JobStatus::Pending,
            "start() called on a job that is not Pending"
        );
        let now = Timestamp::now();
        self.status = JobStatus::Running;
        self.started_at = Some(now);
        self.updated_at = Timestamp::now();
    }

    /// Updates the job progress.
    ///
    /// # Panics
    /// Panics if the job is not in Running status.
    pub fn update_progress(&mut self, percent: u8, message: Option<&str>) {
        assert_eq!(
            self.status,
            JobStatus::Running,
            "update_progress() called on a job that is not Running"
        );
        self.progress_percent = percent.min(100);
        self.progress_message = message.map(String::from);
        self.updated_at = Timestamp::now();
    }

    /// Transitions the job to Completed status and sets completed_at.
    ///
    /// # Panics
    /// Panics if the job is not in Running status.
    pub fn complete(&mut self) {
        assert_eq!(
            self.status,
            JobStatus::Running,
            "complete() called on a job that is not Running"
        );
        self.status = JobStatus::Completed;
        self.progress_percent = 100;
        self.completed_at = Some(Timestamp::now());
        self.updated_at = Timestamp::now();
    }

    /// Transitions the job to Failed status with an error message.
    ///
    /// # Panics
    /// Panics if the job is not in Running status.
    pub fn fail(&mut self, error: &str) {
        assert_eq!(
            self.status,
            JobStatus::Running,
            "fail() called on a job that is not Running"
        );
        self.status = JobStatus::Failed;
        self.error_message = Some(error.to_string());
        self.completed_at = Some(Timestamp::now());
        self.updated_at = Timestamp::now();
    }

    /// Transitions the job to Cancelled status.
    /// Only Pending jobs can be cancelled.
    ///
    /// # Errors
    /// Returns an error string if the job is not Pending.
    pub fn cancel(&mut self) -> Result<(), &'static str> {
        if self.status != JobStatus::Pending {
            return Err("Only Pending jobs can be cancelled");
        }
        self.status = JobStatus::Cancelled;
        self.completed_at = Some(Timestamp::now());
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Returns true if the job is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_job_created_with_pending_status() {
        let project_id = ProjectId::new();
        let operation = json!({"prompt": "test"});
        let job = Job::new(project_id, "test_job", operation.clone());

        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.job_type, "test_job");
        assert_eq!(job.progress_percent, 0);
        assert!(job.started_at.is_none());
        assert!(job.completed_at.is_none());
        assert_eq!(job.created_at, job.updated_at);
    }

    #[test]
    fn test_job_start_transitions_to_running() {
        let project_id = ProjectId::new();
        let job = Job::new(project_id, "test", json!({}));
        assert_eq!(job.status, JobStatus::Pending);

        let mut running_job = job;
        running_job.start();
        assert_eq!(running_job.status, JobStatus::Running);
        assert!(running_job.started_at.is_some());
    }

    #[test]
    fn test_job_update_progress() {
        let project_id = ProjectId::new();
        let mut job = Job::new(project_id, "test", json!({}));
        job.start();

        job.update_progress(50, Some("Halfway done"));
        assert_eq!(job.progress_percent, 50);
        assert_eq!(job.progress_message, Some("Halfway done".to_string()));
    }

    #[test]
    fn test_job_complete_transitions_to_completed() {
        let project_id = ProjectId::new();
        let mut job = Job::new(project_id, "test", json!({}));
        job.start();

        job.complete();
        assert_eq!(job.status, JobStatus::Completed);
        assert_eq!(job.progress_percent, 100);
        assert!(job.completed_at.is_some());
    }

    #[test]
    fn test_job_fail_transitions_to_failed_with_error() {
        let project_id = ProjectId::new();
        let mut job = Job::new(project_id, "test", json!({}));
        job.start();

        job.fail("Something went wrong");
        assert_eq!(job.status, JobStatus::Failed);
        assert_eq!(job.error_message, Some("Something went wrong".to_string()));
        assert!(job.completed_at.is_some());
    }

    #[test]
    fn test_job_cancel_pending_job() {
        let project_id = ProjectId::new();
        let mut job = Job::new(project_id, "test", json!({}));

        let result = job.cancel();
        assert!(result.is_ok());
        assert_eq!(job.status, JobStatus::Cancelled);
        assert!(job.completed_at.is_some());
    }

    #[test]
    fn test_job_cannot_cancel_running_job() {
        let project_id = ProjectId::new();
        let mut job = Job::new(project_id, "test", json!({}));
        job.start();

        let result = job.cancel();
        assert!(result.is_err());
        assert_eq!(job.status, JobStatus::Running); // status unchanged
    }

    #[test]
    fn test_job_is_terminal_for_completed() {
        let project_id = ProjectId::new();
        let job = Job::new(project_id, "test", json!({}));
        assert!(!job.is_terminal());
    }

    #[test]
    fn test_job_is_terminal_for_cancelled() {
        let project_id = ProjectId::new();
        let mut job = Job::new(project_id, "test", json!({}));
        job.cancel().unwrap();
        assert!(job.is_terminal());
    }

    #[test]
    fn test_job_status_default_is_pending() {
        assert_eq!(JobStatus::default(), JobStatus::Pending);
    }

    #[test]
    #[should_panic(expected = "start() called on a job that is not Pending")]
    fn test_start_on_non_pending_panics() {
        let project_id = ProjectId::new();
        let mut job = Job::new(project_id, "test", json!({}));
        job.start();
        job.start(); // second start should panic
    }
}
