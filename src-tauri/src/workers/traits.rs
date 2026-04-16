//! Worker traits and types.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::pin::Pin;

use artifex_job_queue::Job;
use artifex_shared_kernel::AppError;

/// Result of a job processed by a worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    /// Paths to output files produced by the job.
    pub output_files: Vec<PathBuf>,
    /// Arbitrary metadata about the job result.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl JobResult {
    /// Creates a new JobResult with the given output files and empty metadata.
    pub fn new(output_files: Vec<PathBuf>) -> Self {
        Self {
            output_files,
            metadata: serde_json::Value::Object(Default::default()),
        }
    }

    /// Creates a new JobResult with the given output files and metadata.
    pub fn with_metadata(output_files: Vec<PathBuf>, metadata: serde_json::Value) -> Self {
        Self {
            output_files,
            metadata,
        }
    }
}

/// Future type for job processing.
pub type JobFuture = Pin<Box<dyn std::future::Future<Output = Result<JobResult, AppError>> + Send>>;

/// Worker trait for processing jobs.
///
/// Implementors are responsible for handling specific job types
/// and producing results.
pub trait JobWorker: Send + Sync {
    /// Returns true if this worker can handle jobs of the given type.
    fn can_handle(&self, job_type: &str) -> bool;

    /// Processes a job and returns the result.
    ///
    /// # Errors
    /// Returns an error if processing fails.
    fn process(&self, job: &Job) -> JobFuture;
}
