//! Project application service.
//!
//! Orchestrates business operations for projects, enforcing domain rules
//! and coordinating with the repository layer. This is the application
//! layer in clean architecture — it knows about use cases but not about
//! IPC adapters or UI concerns.

use std::sync::Arc;

use artifex_asset_management::{Project, ProjectName, ProjectRepository};
use artifex_shared_kernel::{ArtifexError, ProjectId, ProjectPath};

/// Application service for project operations.
///
/// This service encapsulates all business logic for project management,
/// acting as the single entry point for project operations from the
/// IPC command layer. It enforces domain invariants and validation rules.
#[derive(Clone)]
pub struct ProjectApplicationService {
    repo: Arc<dyn ProjectRepository>,
}

impl ProjectApplicationService {
    /// Creates a new ProjectApplicationService.
    pub fn new(repo: Arc<dyn ProjectRepository>) -> Self {
        Self { repo }
    }

    /// Creates a new project.
    ///
    /// # Errors
    /// - `ValidationError` if the name is empty, too long, or has whitespace
    /// - `DuplicateName` if a project with the same name already exists
    pub async fn create_project(
        &self,
        name: &str,
        path: &str,
    ) -> Result<Project, ArtifexError> {
        // Validate name at domain level
        let project_name = ProjectName::new(name)?;

        // Check for duplicate name
        if self.repo.exists_by_name(name).await? {
            return Err(ArtifexError::duplicate_name(name));
        }

        // Build path
        let project_path = ProjectPath::try_from(path)
            .map_err(|e| ArtifexError::validation(format!("Invalid path: {}", e)))?;

        // Construct domain entity (validates internally)
        let project = Project::new(project_name, project_path)?;

        // Persist
        self.repo.create(&project).await?;

        Ok(project)
    }

    /// Lists all active projects.
    pub async fn list_projects(&self) -> Result<Vec<Project>, ArtifexError> {
        self.repo.list_active().await
    }

    /// Gets a single project by ID.
    ///
    /// # Errors
    /// - `NotFound` if the project does not exist
    pub async fn get_project(&self, id: &str) -> Result<Project, ArtifexError> {
        let project_id = parse_project_id(id)?;
        self.repo
            .find_by_id(&project_id)
            .await?
            .ok_or_else(|| ArtifexError::not_found("Project", id))
    }

    /// Renames a project.
    ///
    /// # Errors
    /// - `ValidationError` if the new name is invalid
    /// - `DuplicateName` if another project already uses the new name
    /// - `NotFound` if the project does not exist
    pub async fn rename_project(
        &self,
        id: &str,
        new_name: &str,
    ) -> Result<Project, ArtifexError> {
        let project_id = parse_project_id(id)?;

        // Validate name at domain level
        let _ = ProjectName::new(new_name)?;

        // Check for duplicate name (excluding current project)
        if let Some(existing) = self.repo.find_by_name(new_name).await? {
            if existing.id != project_id {
                return Err(ArtifexError::duplicate_name(new_name));
            }
        }

        // Get current project
        let mut project = self
            .repo
            .find_by_id(&project_id)
            .await?
            .ok_or_else(|| ArtifexError::not_found("Project", id))?;

        // No-op if same name
        if project.name.as_str() == new_name {
            return Ok(project);
        }

        // Rename and persist
        let new_project_name = ProjectName::new(new_name)?;
        project.rename(new_project_name)?;
        self.repo.update(&project).await?;

        Ok(project)
    }

    /// Archives a project.
    ///
    /// # Errors
    /// - `NotFound` if the project does not exist
    pub async fn archive_project(&self, id: &str) -> Result<(), ArtifexError> {
        let project_id = parse_project_id(id)?;
        self.repo.archive(&project_id).await
    }

    /// Opens a project, validating it exists and is active.
    ///
    /// # Errors
    /// - `NotFound` if the project does not exist
    pub async fn open_project(&self, id: &str) -> Result<Project, ArtifexError> {
        let project_id = parse_project_id(id)?;
        let project = self
            .repo
            .find_by_id(&project_id)
            .await?
            .ok_or_else(|| ArtifexError::not_found("Project", id))?;

        if !project.is_active() {
            return Err(ArtifexError::validation(
                "Cannot open archived project",
            ));
        }

        Ok(project)
    }
}

/// Parses a project ID string into a ProjectId.
fn parse_project_id(id: &str) -> Result<ProjectId, ArtifexError> {
    let uuid = uuid::Uuid::parse_str(id)
        .map_err(|e| ArtifexError::validation(format!("Invalid project id: {}", e)))?;
    Ok(ProjectId::from_uuid(uuid))
}
