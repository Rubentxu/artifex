//! SQLite implementation of the job repository.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::{FromRow, SqlitePool};

use artifex_job_queue::{Job, JobRepository, JobStatus};
use artifex_shared_kernel::{is_unique_violation, ArtifexError, JobId, ProjectId, Timestamp};

/// SQLite row representation of a job.
#[derive(FromRow)]
struct JobRow {
    id: String,
    project_id: String,
    job_type: String,
    status: String,
    operation: String,
    progress_percent: i32,
    progress_message: Option<String>,
    error_message: Option<String>,
    started_at: Option<String>,
    completed_at: Option<String>,
    created_at: String,
    updated_at: String,
}

/// SQLite-backed job repository.
pub struct SqliteJobRepository {
    pool: SqlitePool,
}

impl SqliteJobRepository {
    /// Creates a new SqliteJobRepository.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JobRepository for SqliteJobRepository {
    async fn create(&self, job: &Job) -> Result<(), ArtifexError> {
        let result = sqlx::query(
            r#"INSERT INTO jobs (id, project_id, job_type, status, operation, progress_percent,
                                  progress_message, error_message, started_at, completed_at,
                                  created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(job.id.into_uuid().to_string())
        .bind(job.project_id.into_uuid().to_string())
        .bind(&job.job_type)
        .bind(status_to_string(job.status))
        .bind(serde_json::to_string(&job.operation).unwrap_or_else(|_| "{}".to_string()))
        .bind(job.progress_percent as i32)
        .bind(&job.progress_message)
        .bind(&job.error_message)
        .bind(job.started_at.map(|t| t.to_string()))
        .bind(job.completed_at.map(|t| t.to_string()))
        .bind(job.created_at.to_string())
        .bind(job.updated_at.to_string())
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) if is_unique_violation(&e) => {
                // UUID collisions are astronomically unlikely; treat as internal error
                Err(ArtifexError::Internal(format!(
                    "Duplicate job id (UUID collision): {}",
                    e
                )))
            }
            Err(e) => Err(ArtifexError::IoError(e.to_string())),
        }
    }

    async fn find_by_id(&self, id: &JobId) -> Result<Option<Job>, ArtifexError> {
        let row: Option<JobRow> = sqlx::query_as(
            r#"SELECT id, project_id, job_type, status, operation, progress_percent,
                      progress_message, error_message, started_at, completed_at,
                      created_at, updated_at
               FROM jobs WHERE id = ?"#,
        )
        .bind(id.into_uuid().to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ArtifexError::IoError(e.to_string()))?;

        row.map(|r| row_to_job(&r)).transpose()
    }

    async fn list_by_project(&self, project_id: &ProjectId) -> Result<Vec<Job>, ArtifexError> {
        let rows: Vec<JobRow> = sqlx::query_as(
            r#"SELECT id, project_id, job_type, status, operation, progress_percent,
                      progress_message, error_message, started_at, completed_at,
                      created_at, updated_at
               FROM jobs WHERE project_id = ? ORDER BY created_at DESC"#,
        )
        .bind(project_id.into_uuid().to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ArtifexError::IoError(e.to_string()))?;

        let mut jobs = Vec::with_capacity(rows.len());
        for row in rows {
            jobs.push(row_to_job(&row)?);
        }
        Ok(jobs)
    }

    async fn list_by_status(
        &self,
        project_id: &ProjectId,
        status: JobStatus,
    ) -> Result<Vec<Job>, ArtifexError> {
        let rows: Vec<JobRow> = sqlx::query_as(
            r#"SELECT id, project_id, job_type, status, operation, progress_percent,
                      progress_message, error_message, started_at, completed_at,
                      created_at, updated_at
               FROM jobs WHERE project_id = ? AND status = ? ORDER BY created_at DESC"#,
        )
        .bind(project_id.into_uuid().to_string())
        .bind(status_to_string(status))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ArtifexError::IoError(e.to_string()))?;

        let mut jobs = Vec::with_capacity(rows.len());
        for row in rows {
            jobs.push(row_to_job(&row)?);
        }
        Ok(jobs)
    }

    async fn list_all_by_status(&self, status: JobStatus) -> Result<Vec<Job>, ArtifexError> {
        let rows: Vec<JobRow> = sqlx::query_as(
            r#"SELECT id, project_id, job_type, status, operation, progress_percent,
                      progress_message, error_message, started_at, completed_at,
                      created_at, updated_at
               FROM jobs WHERE status = ? ORDER BY created_at ASC"#,
        )
        .bind(status_to_string(status))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ArtifexError::IoError(e.to_string()))?;

        let mut jobs = Vec::with_capacity(rows.len());
        for row in rows {
            jobs.push(row_to_job(&row)?);
        }
        Ok(jobs)
    }

    async fn update_status(&self, id: &JobId, status: JobStatus) -> Result<(), ArtifexError> {
        let now = Timestamp::now().to_string();

        // When transitioning to Running, also set started_at
        if status == JobStatus::Running {
            let result = sqlx::query(
                "UPDATE jobs SET status = ?, started_at = ?, updated_at = ? WHERE id = ?",
            )
            .bind(status_to_string(status))
            .bind(&now)
            .bind(&now)
            .bind(id.into_uuid().to_string())
            .execute(&self.pool)
            .await;

            match result {
                Ok(affected) if affected.rows_affected() == 0 => {
                    Err(ArtifexError::NotFound(format!("Job {} not found", id.into_uuid())))
                }
                Ok(_) => Ok(()),
                Err(e) => Err(ArtifexError::IoError(e.to_string())),
            }
        } else {
            let result = sqlx::query(
                "UPDATE jobs SET status = ?, updated_at = ? WHERE id = ?",
            )
            .bind(status_to_string(status))
            .bind(&now)
            .bind(id.into_uuid().to_string())
            .execute(&self.pool)
            .await;

            match result {
                Ok(affected) if affected.rows_affected() == 0 => {
                    Err(ArtifexError::NotFound(format!("Job {} not found", id.into_uuid())))
                }
                Ok(_) => Ok(()),
                Err(e) => Err(ArtifexError::IoError(e.to_string())),
            }
        }
    }

    async fn update_progress(
        &self,
        id: &JobId,
        percent: u8,
        message: Option<&str>,
    ) -> Result<(), ArtifexError> {
        let result = sqlx::query(
            "UPDATE jobs SET progress_percent = ?, progress_message = ?, updated_at = ? WHERE id = ?",
        )
        .bind(percent as i32)
        .bind(message)
        .bind(Timestamp::now().to_string())
        .bind(id.into_uuid().to_string())
        .execute(&self.pool)
        .await;

        match result {
            Ok(affected) if affected.rows_affected() == 0 => {
                Err(ArtifexError::NotFound(format!("Job {} not found", id.into_uuid())))
            }
            Ok(_) => Ok(()),
            Err(e) => Err(ArtifexError::IoError(e.to_string())),
        }
    }

    async fn update_failure(&self, id: &JobId, error_message: &str) -> Result<(), ArtifexError> {
        let now = Timestamp::now().to_string();
        let result = sqlx::query(
            "UPDATE jobs SET status = ?, error_message = ?, completed_at = ?, updated_at = ? WHERE id = ?",
        )
        .bind("failed")
        .bind(error_message)
        .bind(&now)
        .bind(&now)
        .bind(id.into_uuid().to_string())
        .execute(&self.pool)
        .await;

        match result {
            Ok(affected) if affected.rows_affected() == 0 => {
                Err(ArtifexError::NotFound(format!("Job {} not found", id.into_uuid())))
            }
            Ok(_) => Ok(()),
            Err(e) => Err(ArtifexError::IoError(e.to_string())),
        }
    }

    async fn mark_completed(&self, id: &JobId) -> Result<(), ArtifexError> {
        let now = Timestamp::now().to_string();
        let result = sqlx::query(
            "UPDATE jobs SET status = ?, completed_at = ?, updated_at = ? WHERE id = ?",
        )
        .bind("completed")
        .bind(&now)
        .bind(&now)
        .bind(id.into_uuid().to_string())
        .execute(&self.pool)
        .await;

        match result {
            Ok(affected) if affected.rows_affected() == 0 => {
                Err(ArtifexError::NotFound(format!("Job {} not found", id.into_uuid())))
            }
            Ok(_) => Ok(()),
            Err(e) => Err(ArtifexError::IoError(e.to_string())),
        }
    }
}

/// Converts a database row to a Job domain object.
fn row_to_job(row: &JobRow) -> Result<Job, ArtifexError> {
    let id_uuid = uuid::Uuid::parse_str(&row.id)
        .map_err(|e| ArtifexError::ValidationError(format!("Invalid job id: {}", e)))?;
    let id = JobId::from_uuid(id_uuid);

    let project_uuid = uuid::Uuid::parse_str(&row.project_id)
        .map_err(|e| ArtifexError::ValidationError(format!("Invalid project id: {}", e)))?;
    let project_id = ProjectId::from_uuid(project_uuid);

    let status = string_to_status(&row.status)?;

    let operation: JsonValue =
        serde_json::from_str(&row.operation).unwrap_or(JsonValue::Object(Default::default()));

    let started_at = row.started_at.as_ref().map(|s| parse_rfc3339(s)).transpose()?;
    let completed_at = row.completed_at.as_ref().map(|s| parse_rfc3339(s)).transpose()?;
    let created_at = parse_rfc3339(&row.created_at)?;
    let updated_at = parse_rfc3339(&row.updated_at)?;

    Ok(Job {
        id,
        project_id,
        job_type: row.job_type.clone(),
        status,
        operation,
        progress_percent: row.progress_percent as u8,
        progress_message: row.progress_message.clone(),
        error_message: row.error_message.clone(),
        started_at,
        completed_at,
        created_at,
        updated_at,
    })
}

/// Parses an RFC3339 timestamp string into a Timestamp.
fn parse_rfc3339(s: &str) -> Result<Timestamp, ArtifexError> {
    let dt = DateTime::parse_from_rfc3339(s)
        .map_err(|e| ArtifexError::ValidationError(format!("Invalid timestamp: {}", e)))?;
    Ok(Timestamp::from_datetime(dt.with_timezone(&Utc)))
}

/// Converts JobStatus to a string for database storage.
fn status_to_string(status: JobStatus) -> &'static str {
    match status {
        JobStatus::Pending => "pending",
        JobStatus::Running => "running",
        JobStatus::Completed => "completed",
        JobStatus::Failed => "failed",
        JobStatus::Cancelled => "cancelled",
    }
}

/// Parses a status string to JobStatus.
fn string_to_status(s: &str) -> Result<JobStatus, ArtifexError> {
    match s {
        "pending" => Ok(JobStatus::Pending),
        "running" => Ok(JobStatus::Running),
        "completed" => Ok(JobStatus::Completed),
        "failed" => Ok(JobStatus::Failed),
        "cancelled" => Ok(JobStatus::Cancelled),
        _ => Err(ArtifexError::ValidationError(format!("Unknown job status: {}", s))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_status_to_string() {
        assert_eq!(status_to_string(JobStatus::Pending), "pending");
        assert_eq!(status_to_string(JobStatus::Running), "running");
        assert_eq!(status_to_string(JobStatus::Completed), "completed");
        assert_eq!(status_to_string(JobStatus::Failed), "failed");
        assert_eq!(status_to_string(JobStatus::Cancelled), "cancelled");
    }

    #[test]
    fn test_string_to_status() {
        assert_eq!(string_to_status("pending").unwrap(), JobStatus::Pending);
        assert_eq!(string_to_status("running").unwrap(), JobStatus::Running);
        assert_eq!(string_to_status("completed").unwrap(), JobStatus::Completed);
        assert_eq!(string_to_status("failed").unwrap(), JobStatus::Failed);
        assert_eq!(string_to_status("cancelled").unwrap(), JobStatus::Cancelled);
        assert!(string_to_status("unknown").is_err());
    }

    #[test]
    fn test_row_to_job_parses_correctly() {
        let row = JobRow {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: uuid::Uuid::new_v4().to_string(),
            job_type: "image_generate".to_string(),
            status: "pending".to_string(),
            operation: r#"{"prompt": "test"}"#.to_string(),
            progress_percent: 0,
            progress_message: None,
            error_message: None,
            started_at: None,
            completed_at: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let job = row_to_job(&row).unwrap();
        assert_eq!(job.job_type, "image_generate");
        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.progress_percent, 0);
        assert_eq!(job.operation, json!({"prompt": "test"}));
    }
}
