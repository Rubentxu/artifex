//! SQLite implementation of the project repository.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};

use artifex_asset_management::{Project, ProjectName, ProjectRepository, ProjectStatus};
use artifex_shared_kernel::{is_unique_violation, ArtifexError, ProjectId, ProjectPath, Timestamp};

/// SQLite row representation of a project.
#[derive(FromRow)]
struct ProjectRow {
    id: String,
    name: String,
    path: String,
    status: String,
    created_at: String,
    updated_at: String,
}

/// SQLite-backed project repository.
pub struct SqliteProjectRepository {
    pool: SqlitePool,
}

impl SqliteProjectRepository {
    /// Creates a new SqliteProjectRepository.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectRepository for SqliteProjectRepository {
    async fn create(&self, project: &Project) -> Result<(), ArtifexError> {
        let result = sqlx::query(
            r#"INSERT INTO projects (id, name, path, status, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?)"#,
        )
        .bind(project.id.into_uuid().to_string())
        .bind(project.name.as_str())
        .bind(project.path.to_string())
        .bind(status_to_string(project.status))
        .bind(project.created_at.to_string())
        .bind(project.updated_at.to_string())
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) if is_unique_violation(&e) => {
                Err(ArtifexError::duplicate_name(project.name.as_str()))
            }
            Err(e) => Err(ArtifexError::IoError(e.to_string())),
        }
    }

    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, ArtifexError> {
        let row: Option<ProjectRow> = sqlx::query_as(
            "SELECT id, name, path, status, created_at, updated_at FROM projects WHERE id = ?",
        )
        .bind(id.into_uuid().to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ArtifexError::IoError(e.to_string()))?;

        match row {
            Some(r) => row_to_project(&r).map(Some),
            None => Ok(None),
        }
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Project>, ArtifexError> {
        let row: Option<ProjectRow> = sqlx::query_as(
            "SELECT id, name, path, status, created_at, updated_at FROM projects WHERE name = ? AND status != 'archived'",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ArtifexError::IoError(e.to_string()))?;

        row.map(|r| row_to_project(&r)).transpose()
    }

    async fn exists_by_name(&self, name: &str) -> Result<bool, ArtifexError> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) as cnt FROM projects WHERE name = ? AND status != 'archived'",
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ArtifexError::IoError(e.to_string()))?;
        Ok(row.0 > 0)
    }

    async fn list_active(&self) -> Result<Vec<Project>, ArtifexError> {
        let rows: Vec<ProjectRow> = sqlx::query_as(
            "SELECT id, name, path, status, created_at, updated_at FROM projects WHERE status = 'active' ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ArtifexError::IoError(e.to_string()))?;

        let mut projects = Vec::with_capacity(rows.len());
        for row in rows {
            projects.push(row_to_project(&row)?);
        }
        Ok(projects)
    }

    async fn list_all(&self) -> Result<Vec<Project>, ArtifexError> {
        let rows: Vec<ProjectRow> = sqlx::query_as(
            "SELECT id, name, path, status, created_at, updated_at FROM projects ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ArtifexError::IoError(e.to_string()))?;

        let mut projects = Vec::with_capacity(rows.len());
        for row in rows {
            projects.push(row_to_project(&row)?);
        }
        Ok(projects)
    }

    async fn update(&self, project: &Project) -> Result<(), ArtifexError> {
        let result = sqlx::query(
            r#"UPDATE projects
               SET name = ?, path = ?, status = ?, updated_at = ?
               WHERE id = ?"#,
        )
        .bind(project.name.as_str())
        .bind(project.path.to_string())
        .bind(status_to_string(project.status))
        .bind(project.updated_at.to_string())
        .bind(project.id.into_uuid().to_string())
        .execute(&self.pool)
        .await;

        match result {
            Ok(affected) if affected.rows_affected() == 0 => {
                Err(ArtifexError::NotFound(format!(
                    "Project {} not found",
                    project.id.into_uuid()
                )))
            }
            Ok(_) => Ok(()),
            Err(e) if is_unique_violation(&e) => {
                Err(ArtifexError::duplicate_name(project.name.as_str()))
            }
            Err(e) => Err(ArtifexError::IoError(e.to_string())),
        }
    }

    async fn archive(&self, id: &ProjectId) -> Result<(), ArtifexError> {
        let affected = sqlx::query(
            "UPDATE projects SET status = 'archived', updated_at = ? WHERE id = ?",
        )
        .bind(Timestamp::now().to_string())
        .bind(id.into_uuid().to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| ArtifexError::IoError(e.to_string()))?;

        if affected.rows_affected() == 0 {
            return Err(ArtifexError::NotFound(format!(
                "Project {} not found",
                id.into_uuid()
            )));
        }

        Ok(())
    }
}

/// Converts a database row to a Project domain object.
fn row_to_project(row: &ProjectRow) -> Result<Project, ArtifexError> {
    let id_uuid = uuid::Uuid::parse_str(&row.id)
        .map_err(|e| ArtifexError::ValidationError(format!("Invalid project id: {}", e)))?;
    let id = ProjectId::from_uuid(id_uuid);

    let path = ProjectPath::try_from(row.path.as_str())
        .map_err(|e| ArtifexError::ValidationError(format!("Invalid path: {}", e)))?;

    let status = string_to_status(&row.status)?;

    // Parse RFC3339 timestamps
    let created_at = parse_rfc3339(&row.created_at)?;
    let updated_at = parse_rfc3339(&row.updated_at)?;

    // Validate DB data; will fail loudly on corruption
    let name = ProjectName::new(&row.name)
        .map_err(|e| ArtifexError::ValidationError(format!("Invalid project name in DB: {}", e)))?;

    Ok(Project {
        id,
        name,
        path,
        status,
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

/// Converts ProjectStatus to a string for database storage.
fn status_to_string(status: ProjectStatus) -> &'static str {
    match status {
        ProjectStatus::Active => "active",
        ProjectStatus::Archived => "archived",
    }
}

/// Parses a status string to ProjectStatus.
fn string_to_status(s: &str) -> Result<ProjectStatus, ArtifexError> {
    match s {
        "active" => Ok(ProjectStatus::Active),
        "archived" => Ok(ProjectStatus::Archived),
        _ => Err(ArtifexError::ValidationError(format!(
            "Unknown status: {}",
            s
        ))),
    }
}
