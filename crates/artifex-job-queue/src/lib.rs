//! Artifex Job Queue
//!
//! Job queue domain types and repository trait.

pub mod job;
pub mod repository;

// Re-exports
pub use job::{Job, JobStatus};
pub use repository::JobRepository;
