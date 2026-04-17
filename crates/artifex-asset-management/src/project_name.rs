//! Project name value object.

use serde::{Deserialize, Serialize};

use artifex_shared_kernel::ArtifexError;

/// Maximum allowed project name length.
const MAX_NAME_LENGTH: usize = 128;

/// A validated project name.
///
/// Wraps a `String` with validation rules:
/// - Must not be empty (after trimming)
/// - Must not exceed 128 characters
/// - Must not have leading or trailing whitespace
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectName(String);

impl ProjectName {
    /// Creates a new `ProjectName` after validation.
    ///
    /// # Errors
    /// Returns `ArtifexError::ValidationError` if:
    /// - The name is empty or whitespace-only
    /// - The name exceeds 128 characters
    /// - The name has leading or trailing whitespace
    pub fn new(name: impl Into<String>) -> Result<Self, ArtifexError> {
        let name = name.into();
        let trimmed = name.trim();

        if trimmed.is_empty() {
            return Err(ArtifexError::validation("Project name cannot be empty"));
        }

        if trimmed.len() > MAX_NAME_LENGTH {
            return Err(ArtifexError::validation(format!(
                "Project name cannot exceed {} characters",
                MAX_NAME_LENGTH
            )));
        }

        if trimmed != name {
            return Err(ArtifexError::validation(
                "Project name cannot have leading or trailing whitespace",
            ));
        }

        Ok(Self(trimmed.to_string()))
    }

    /// Creates a `ProjectName` without validation.
    ///
    /// # Safety
    /// Caller must ensure the name is valid (non-empty, ≤128 chars, no leading/trailing whitespace).
    /// This is intended for test fixtures and repository layer only.
    pub fn unchecked(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Returns the name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProjectName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for ProjectName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<ProjectName> for String {
    fn from(val: ProjectName) -> Self {
        val.0
    }
}

impl std::ops::Deref for ProjectName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_name() {
        let name = ProjectName::new("MyGame").unwrap();
        assert_eq!(name.as_str(), "MyGame");
    }

    #[test]
    fn test_name_with_leading_whitespace_rejected() {
        let result = ProjectName::new("  MyGame");
        assert!(result.is_err());
    }

    #[test]
    fn test_name_with_trailing_whitespace_rejected() {
        let result = ProjectName::new("MyGame  ");
        assert!(result.is_err());
    }

    #[test]
    fn test_name_valid_with_internal_spaces() {
        // Internal spaces are fine
        let name = ProjectName::new("My Game Studio").unwrap();
        assert_eq!(name.as_str(), "My Game Studio");
    }

    #[test]
    fn test_empty_name_rejected() {
        let result = ProjectName::new("");
        assert!(result.is_err());
    }

    #[test]
    fn test_whitespace_only_name_rejected() {
        let result = ProjectName::new("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_name_too_long_rejected() {
        let long_name = "a".repeat(129);
        let result = ProjectName::new(&long_name);
        assert!(result.is_err());
    }

    #[test]
    fn test_name_exactly_128_chars_accepted() {
        let name_128 = "a".repeat(128);
        let result = ProjectName::new(&name_128);
        assert!(result.is_ok());
    }

    #[test]
    fn test_leading_whitespace_rejected() {
        let result = ProjectName::new(" MyGame");
        assert!(result.is_err());
    }

    #[test]
    fn test_trailing_whitespace_rejected() {
        let result = ProjectName::new("MyGame ");
        assert!(result.is_err());
    }

    #[test]
    fn test_display() {
        let name = ProjectName::new("MyGame").unwrap();
        assert_eq!(format!("{}", name), "MyGame");
    }

    #[test]
    fn test_eq() {
        let name1 = ProjectName::new("MyGame").unwrap();
        let name2 = ProjectName::new("MyGame").unwrap();
        assert_eq!(name1, name2);
    }

    #[test]
    fn test_deref() {
        let name = ProjectName::new("MyGame").unwrap();
        let s: &str = &name;
        assert_eq!(s, "MyGame");
    }

    #[test]
    fn test_into_string() {
        let name = ProjectName::new("MyGame").unwrap();
        let s: String = name.into();
        assert_eq!(s, "MyGame");
    }

    #[test]
    fn test_serde_roundtrip() {
        let name = ProjectName::new("MyGame").unwrap();
        let serialized = serde_json::to_string(&name).unwrap();
        let deserialized: ProjectName = serde_json::from_str(&serialized).unwrap();
        assert_eq!(name, deserialized);
    }
}
