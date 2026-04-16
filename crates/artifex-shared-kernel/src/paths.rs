//! Validated path types.

use std::path::PathBuf;

use crate::errors::ArtifexError;

/// A validated absolute path to a project directory.
///
/// This value object ensures paths are absolute and valid UTF-8.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ProjectPath(PathBuf);

impl ProjectPath {
    /// Creates a ProjectPath from a string, validating it's an absolute path.
    pub fn try_from(s: &str) -> Result<Self, ArtifexError> {
        let path = PathBuf::from(s);

        // Check if path is absolute
        if !path.is_absolute() {
            return Err(ArtifexError::ValidationError(format!(
                "Path must be absolute, got: {}",
                s
            )));
        }

        // Check if path is valid UTF-8
        match path.to_str() {
            Some(_) => Ok(Self(path)),
            None => Err(ArtifexError::ValidationError(
                "Path must be valid UTF-8".to_string(),
            )),
        }
    }

    /// Returns the underlying PathBuf.
    pub fn into_path_buf(self) -> PathBuf {
        self.0
    }

    /// Returns the path as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.to_str().unwrap()
    }

    /// Returns a reference to the underlying PathBuf.
    pub fn as_path_buf(&self) -> &PathBuf {
        &self.0
    }
}

impl std::fmt::Display for ProjectPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

impl AsRef<PathBuf> for ProjectPath {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_path_accepts_valid_absolute_path() {
        let result = ProjectPath::try_from("/home/user/projects/mygame");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.as_str(), "/home/user/projects/mygame");
    }

    #[test]
    fn test_project_path_rejects_relative_path() {
        let result = ProjectPath::try_from("./foo");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("must be absolute"));
    }

    #[test]
    fn test_project_path_rejects_relative_path_with_dots() {
        let result = ProjectPath::try_from("foo/bar");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be absolute"));
    }

    #[test]
    fn test_project_path_rejects_empty_string() {
        let result = ProjectPath::try_from("");
        assert!(result.is_err());
    }

    #[test]
    fn test_project_path_serde_roundtrip() {
        let path = ProjectPath::try_from("/tmp/test").unwrap();
        let serialized = serde_json::to_string(&path).unwrap();
        let deserialized: ProjectPath = serde_json::from_str(&serialized).unwrap();
        assert_eq!(path, deserialized);
    }
}
