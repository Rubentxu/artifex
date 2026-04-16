//! Project repository trait.

use async_trait::async_trait;

use artifex_shared_kernel::{ArtifexError, ProjectId};

use super::Project;

/// Repository trait for project persistence.
///
/// This is a trait-only implementation in Phase 0.
/// Concrete implementations will be added in Phase B.
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// Creates a new project.
    async fn create(&self, project: &Project) -> Result<(), ArtifexError>;

    /// Finds a project by its ID.
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, ArtifexError>;

    /// Finds a project by its name.
    ///
    /// Returns the project if found and not archived.
    async fn find_by_name(&self, name: &str) -> Result<Option<Project>, ArtifexError>;

    /// Checks if a project with the given name exists (and is not archived).
    async fn exists_by_name(&self, name: &str) -> Result<bool, ArtifexError>;

    /// Lists all active projects, ordered by updated_at descending.
    async fn list_active(&self) -> Result<Vec<Project>, ArtifexError>;

    /// Lists all projects (including archived), ordered by updated_at descending.
    async fn list_all(&self) -> Result<Vec<Project>, ArtifexError>;

    /// Updates an existing project.
    async fn update(&self, project: &Project) -> Result<(), ArtifexError>;

    /// Archives a project.
    async fn archive(&self, id: &ProjectId) -> Result<(), ArtifexError>;
}
