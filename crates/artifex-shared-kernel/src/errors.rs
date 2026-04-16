//! Domain error types.

use serde::{Deserialize, Serialize};
use sqlx::Error as SqlxError;
use thiserror::Error;

/// Trait for domain-level errors.
///
/// Implementors must provide a human-readable message and a unique error code.
pub trait DomainError: std::fmt::Debug + Send + Sync {
    /// Returns a unique error code for this error type.
    fn error_code(&self) -> &'static str;
}

/// Main error type for Artifex operations.
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum ArtifexError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Duplicate name: {0}")]
    DuplicateName(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl ArtifexError {
    /// Creates a NotFound error.
    pub fn not_found(entity: &str, id: &str) -> Self {
        Self::NotFound(format!("{} with id {} not found", entity, id))
    }

    /// Creates a ValidationError.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::ValidationError(message.into())
    }

    /// Creates a DuplicateName error.
    pub fn duplicate_name(name: &str) -> Self {
        Self::DuplicateName(format!("Name '{}' is already in use", name))
    }

    /// Creates an IoError from an std::io::Error.
    pub fn from_io(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

impl DomainError for ArtifexError {
    fn error_code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NOT_FOUND",
            Self::ValidationError(_) => "VALIDATION_ERROR",
            Self::DuplicateName(_) => "DUPLICATE_NAME",
            Self::IoError(_) => "IO_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }
}

/// Alias for application-level errors.
pub type AppError = ArtifexError;

impl ArtifexError {
    /// Creates an internal error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    /// Creates an IO error from a message string.
    pub fn io_error(message: impl Into<String>) -> Self {
        Self::IoError(message.into())
    }
}

/// Detects SQLite UNIQUE constraint violations.
///
/// Checks for error code 2067 or message containing "UNIQUE constraint failed".
pub fn is_unique_violation(err: &SqlxError) -> bool {
    match err {
        SqlxError::Database(db_err) => {
            let msg = db_err.message();
            // SQLite UNIQUE constraint error code is 2067
            // Also check message as fallback
            msg.contains("UNIQUE constraint failed")
                || msg.contains("unique")
                || db_err.code().map_or(false, |c| c == "2067")
        }
        _ => false,
    }
}
