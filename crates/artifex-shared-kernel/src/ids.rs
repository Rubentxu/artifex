//! Strongly-typed ID newtypes wrapping UUIDs.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Project identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(Uuid);

impl ProjectId {
    /// Creates a new random ProjectId.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a ProjectId from a UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the underlying UUID.
    pub fn into_uuid(self) -> Uuid {
        self.0
    }

    /// Returns the UUID as a string.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for ProjectId {
    fn default() -> Self {
        Self::new()
    }
}

/// Asset identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(Uuid);

impl AssetId {
    /// Creates a new random AssetId.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates an AssetId from a UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the underlying UUID.
    pub fn into_uuid(self) -> Uuid {
        self.0
    }

    /// Returns the UUID as a string.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for AssetId {
    fn default() -> Self {
        Self::new()
    }
}

/// Job identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(Uuid);

impl JobId {
    /// Creates a new random JobId.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a JobId from a UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the underlying UUID.
    pub fn into_uuid(self) -> Uuid {
        self.0
    }

    /// Returns the UUID as a string.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

/// Asset version identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetVersionId(Uuid);

impl AssetVersionId {
    /// Creates a new random AssetVersionId.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates an AssetVersionId from a UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the underlying UUID.
    pub fn into_uuid(self) -> Uuid {
        self.0
    }

    /// Returns the UUID as a string.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for AssetVersionId {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_id_creation() {
        let id = ProjectId::new();
        assert_ne!(id.into_uuid(), Uuid::nil());
    }

    #[test]
    fn test_project_id_serde_roundtrip() {
        let id = ProjectId::new();
        let serialized = serde_json::to_string(&id).unwrap();
        let deserialized: ProjectId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_asset_id_serde_roundtrip() {
        let id = AssetId::new();
        let serialized = serde_json::to_string(&id).unwrap();
        let deserialized: AssetId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_job_id_serde_roundtrip() {
        let id = JobId::new();
        let serialized = serde_json::to_string(&id).unwrap();
        let deserialized: JobId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_asset_version_id_serde_roundtrip() {
        let id = AssetVersionId::new();
        let serialized = serde_json::to_string(&id).unwrap();
        let deserialized: AssetVersionId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_ids_are_unique() {
        let id1 = ProjectId::new();
        let id2 = ProjectId::new();
        assert_ne!(id1, id2);
    }
}
