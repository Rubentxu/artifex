//! Strongly-typed ID newtypes wrapping UUIDs.

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

/// User identifier (single-user desktop app uses a single fixed ID).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(String);

impl UserId {
    /// The default user ID for single-user desktop app.
    pub fn default_user() -> Self {
        Self("default-user".to_string())
    }

    /// Creates a UserId from a string.
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Returns the inner value as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::default_user()
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for UserId {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_string(s))
    }
}

/// Subscription tier for the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Free,
    Pro,
}

impl Tier {
    /// Returns true if this tier is Pro.
    pub fn is_pro(&self) -> bool {
        matches!(self, Tier::Pro)
    }

    /// Returns the string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Tier::Free => "free",
            Tier::Pro => "pro",
        }
    }
}

impl Default for Tier {
    fn default() -> Self {
        Tier::Free
    }
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Tier {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "free" => Ok(Tier::Free),
            "pro" => Ok(Tier::Pro),
            _ => Err(format!("Invalid tier: {}. Expected 'free' or 'pro'", s)),
        }
    }
}

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

    // UserId tests

    #[test]
    fn test_user_id_default() {
        let id = UserId::default();
        assert_eq!(id.as_str(), "default-user");
    }

    #[test]
    fn test_user_id_from_string() {
        let id = UserId::from_string("test-user");
        assert_eq!(id.as_str(), "test-user");
    }

    #[test]
    fn test_user_id_display() {
        let id = UserId::default();
        assert_eq!(format!("{}", id), "default-user");
    }

    #[test]
    fn test_user_id_serde_roundtrip() {
        let id = UserId::from_string("test-user");
        let serialized = serde_json::to_string(&id).unwrap();
        let deserialized: UserId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }

    // Tier tests

    #[test]
    fn test_tier_default() {
        assert_eq!(Tier::default(), Tier::Free);
    }

    #[test]
    fn test_tier_is_pro() {
        assert!(!Tier::Free.is_pro());
        assert!(Tier::Pro.is_pro());
    }

    #[test]
    fn test_tier_as_str() {
        assert_eq!(Tier::Free.as_str(), "free");
        assert_eq!(Tier::Pro.as_str(), "pro");
    }

    #[test]
    fn test_tier_display() {
        assert_eq!(format!("{}", Tier::Free), "free");
        assert_eq!(format!("{}", Tier::Pro), "pro");
    }

    #[test]
    fn test_tier_from_str() {
        assert_eq!(Tier::from_str("free").unwrap(), Tier::Free);
        assert_eq!(Tier::from_str("Free").unwrap(), Tier::Free);
        assert_eq!(Tier::from_str("FREE").unwrap(), Tier::Free);
        assert_eq!(Tier::from_str("pro").unwrap(), Tier::Pro);
        assert_eq!(Tier::from_str("Pro").unwrap(), Tier::Pro);
        assert!(Tier::from_str("enterprise").is_err());
    }

    #[test]
    fn test_tier_serde_roundtrip() {
        for tier in [Tier::Free, Tier::Pro] {
            let serialized = serde_json::to_string(&tier).unwrap();
            let deserialized: Tier = serde_json::from_str(&serialized).unwrap();
            assert_eq!(tier, deserialized);
        }
    }
}
