//! Collection model.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use artifex_shared_kernel::{ProjectId, Timestamp};

/// A named group of assets within a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    /// Unique collection identifier.
    pub id: String,
    /// The project this collection belongs to.
    pub project_id: ProjectId,
    /// Collection display name.
    pub name: String,
    /// When the collection was created.
    pub created_at: Timestamp,
}

impl Collection {
    /// Creates a new collection with a generated UUID.
    pub fn new(project_id: ProjectId, name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            project_id,
            name: name.into(),
            created_at: Timestamp::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_creation() {
        let project_id = ProjectId::new();
        let collection = Collection::new(project_id, "Environment");

        assert_eq!(collection.name, "Environment");
        assert_eq!(collection.project_id, project_id);
        assert!(!collection.id.is_empty());
    }

    #[test]
    fn test_collection_name() {
        let project_id = ProjectId::new();
        let collection = Collection::new(project_id, "Characters");

        assert_eq!(collection.name, "Characters");
    }
}
