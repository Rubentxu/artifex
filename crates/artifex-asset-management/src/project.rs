//! Project aggregate.

use serde::{Deserialize, Serialize};

use artifex_shared_kernel::{ProjectId, ProjectPath, Timestamp};

use super::ProjectName;

/// Project status enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    /// Project is active and in use.
    #[default]
    Active,
    /// Project has been archived.
    Archived,
}

/// A project aggregate root.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Unique project identifier.
    pub id: ProjectId,
    /// Project display name.
    pub name: ProjectName,
    /// Absolute path to the project directory.
    pub path: ProjectPath,
    /// Current project status.
    pub status: ProjectStatus,
    /// When the project was created.
    pub created_at: Timestamp,
    /// When the project was last updated.
    pub updated_at: Timestamp,
}

impl Project {
    /// Creates a new project with Active status.
    ///
    /// # Errors
    /// Returns `ArtifexError::ValidationError` if the name is invalid.
    pub fn new(
        name: ProjectName,
        path: ProjectPath,
    ) -> Result<Self, artifex_shared_kernel::ArtifexError> {
        let now = Timestamp::now();
        Ok(Self {
            id: ProjectId::new(),
            name,
            path,
            status: ProjectStatus::Active,
            created_at: now,
            updated_at: now,
        })
    }

    /// Creates a new project for testing without validation.
    ///
    /// # Safety
    /// Intended for test fixtures only. The name must be a valid project name
    /// (non-empty, ≤128 chars, no leading/trailing whitespace).
    pub fn test_new(name: &str, path: &str) -> Self {
        let name = ProjectName::unchecked(name);
        let path = ProjectPath::try_from(path).expect("Invalid test path");
        Self {
            id: ProjectId::new(),
            name,
            path,
            status: ProjectStatus::Active,
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        }
    }

    /// Archives this project.
    pub fn archive(&mut self) {
        self.status = ProjectStatus::Archived;
        self.updated_at = Timestamp::now();
    }

    /// Renames this project.
    ///
    /// # Errors
    /// Returns `ArtifexError::ValidationError` if the new name is invalid.
    pub fn rename(
        &mut self,
        new_name: ProjectName,
    ) -> Result<(), artifex_shared_kernel::ArtifexError> {
        self.name = new_name;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Returns true if the project is active.
    pub fn is_active(&self) -> bool {
        self.status == ProjectStatus::Active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let project = Project::test_new("TestGame", "/tmp/test");

        assert_eq!(project.name.as_str(), "TestGame");
        assert_eq!(project.status, ProjectStatus::Active);
        // created_at may differ from updated_at by nanoseconds due to construction time
        assert!(project.updated_at >= project.created_at);
    }

    #[test]
    fn test_project_archive() {
        let mut project = Project::test_new("TestGame", "/tmp/test");

        assert!(project.is_active());
        project.archive();
        assert!(!project.is_active());
        assert_eq!(project.status, ProjectStatus::Archived);
        assert!(project.updated_at > project.created_at);
    }

    #[test]
    fn test_project_rename() {
        let mut project = Project::test_new("OldName", "/tmp/test");

        let new_name = ProjectName::new("NewName").unwrap();
        project.rename(new_name).expect("Valid rename");
        assert_eq!(project.name.as_str(), "NewName");
        assert!(project.updated_at > project.created_at);
    }

    #[test]
    fn test_project_status_default_is_active() {
        assert_eq!(ProjectStatus::default(), ProjectStatus::Active);
    }

    #[test]
    fn test_project_name_validation_empty_rejected() {
        let _path = ProjectPath::try_from("/tmp/test").unwrap();
        let name = ProjectName::new("").unwrap_err();
        assert!(matches!(
            name,
            artifex_shared_kernel::ArtifexError::ValidationError(_)
        ));
    }

    #[test]
    fn test_project_name_validation_too_long_rejected() {
        let _path = ProjectPath::try_from("/tmp/test").unwrap();
        let long_name = ProjectName::new("a".repeat(129)).unwrap_err();
        assert!(matches!(
            long_name,
            artifex_shared_kernel::ArtifexError::ValidationError(_)
        ));
    }
}
