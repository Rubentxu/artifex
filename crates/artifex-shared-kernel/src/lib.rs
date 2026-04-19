//! Artifex Shared Kernel
//!
//! Common types, errors, and traits used across all Artifex crates.

pub mod errors;
pub mod events;
pub mod ids;
pub mod paths;
pub mod time;

// Re-export all public types for convenient access
pub use errors::{is_unique_violation, AppError, ArtifexError, DomainError};
pub use events::DomainEvent;
pub use ids::{AssetId, AssetVersionId, JobId, ProjectId, Tier, UserId};
pub use paths::ProjectPath;
pub use time::Timestamp;
