//! Credential store trait and implementations.

use std::collections::HashMap;
use std::sync::RwLock;

use thiserror::Error;

/// Errors that can occur when accessing credentials.
#[derive(Debug, Clone, Error)]
pub enum CredentialError {
    #[error("Credential not found: {0}")]
    NotFound(String),

    #[error("Failed to store credential: {0}")]
    StoreError(String),

    /// Credential store is unavailable (e.g., keyring not accessible).
    #[error("Credential store unavailable: {0}")]
    Unavailable(String),
}

/// Trait for credential storage.
///
/// Implementors provide secure storage for API keys and other secrets.
pub trait CredentialStore: Send + Sync {
    /// Gets a credential by its ID.
    ///
    /// # Errors
    /// Returns `CredentialError::NotFound` if the credential doesn't exist.
    fn get(&self, credential_id: &str) -> Result<String, CredentialError>;

    /// Sets a credential.
    ///
    /// # Errors
    /// Returns `CredentialError::StoreError` if storage fails.
    fn set(&self, credential_id: &str, value: &str) -> Result<(), CredentialError>;

    /// Deletes a credential.
    ///
    /// # Errors
    /// Returns `CredentialError::NotFound` if the credential doesn't exist.
    fn delete(&self, credential_id: &str) -> Result<(), CredentialError>;

    /// Lists all credential IDs.
    fn list(&self) -> Result<Vec<String>, CredentialError>;
}

/// In-memory credential store for testing.
pub struct InMemoryCredentialStore {
    credentials: RwLock<HashMap<String, String>>,
}

impl InMemoryCredentialStore {
    /// Creates a new empty in-memory credential store.
    pub fn new() -> Self {
        Self {
            credentials: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryCredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialStore for InMemoryCredentialStore {
    fn get(&self, credential_id: &str) -> Result<String, CredentialError> {
        let guard = self
            .credentials
            .read()
            .map_err(|e| CredentialError::StoreError(e.to_string()))?;
        guard
            .get(credential_id)
            .cloned()
            .ok_or_else(|| CredentialError::NotFound(credential_id.to_string()))
    }

    fn set(&self, credential_id: &str, value: &str) -> Result<(), CredentialError> {
        let mut guard = self
            .credentials
            .write()
            .map_err(|e| CredentialError::StoreError(e.to_string()))?;
        guard.insert(credential_id.to_string(), value.to_string());
        Ok(())
    }

    fn delete(&self, credential_id: &str) -> Result<(), CredentialError> {
        let mut guard = self
            .credentials
            .write()
            .map_err(|e| CredentialError::StoreError(e.to_string()))?;
        guard
            .remove(credential_id)
            .ok_or_else(|| CredentialError::NotFound(credential_id.to_string()))?;
        Ok(())
    }

    fn list(&self) -> Result<Vec<String>, CredentialError> {
        let guard = self
            .credentials
            .read()
            .map_err(|e| CredentialError::StoreError(e.to_string()))?;
        Ok(guard.keys().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_credential_store_set_get() {
        let store = InMemoryCredentialStore::new();

        store.set("replicate::api_key", "test-key-123").unwrap();

        let value = store.get("replicate::api_key").unwrap();
        assert_eq!(value, "test-key-123");
    }

    #[test]
    fn test_in_memory_credential_store_get_not_found() {
        let store = InMemoryCredentialStore::new();

        let result = store.get("nonexistent");
        assert!(matches!(result, Err(CredentialError::NotFound(_))));
    }

    #[test]
    fn test_in_memory_credential_store_delete() {
        let store = InMemoryCredentialStore::new();

        store.set("test::key", "value").unwrap();
        assert!(store.get("test::key").is_ok());

        store.delete("test::key").unwrap();

        let result = store.get("test::key");
        assert!(matches!(result, Err(CredentialError::NotFound(_))));
    }

    #[test]
    fn test_in_memory_credential_store_delete_not_found() {
        let store = InMemoryCredentialStore::new();

        let result = store.delete("nonexistent");
        assert!(matches!(result, Err(CredentialError::NotFound(_))));
    }

    #[test]
    fn test_in_memory_credential_store_list() {
        let store = InMemoryCredentialStore::new();

        store.set("key1", "value1").unwrap();
        store.set("key2", "value2").unwrap();
        store.set("key3", "value3").unwrap();

        let keys = store.list().unwrap();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"key3".to_string()));
    }

    #[test]
    fn test_in_memory_credential_store_update() {
        let store = InMemoryCredentialStore::new();

        store.set("test::key", "old_value").unwrap();
        assert_eq!(store.get("test::key").unwrap(), "old_value");

        store.set("test::key", "new_value").unwrap();
        assert_eq!(store.get("test::key").unwrap(), "new_value");
    }

    #[test]
    fn test_credential_error_display() {
        let err = CredentialError::NotFound("test-key".to_string());
        assert!(err.to_string().contains("not found"));
        assert!(err.to_string().contains("test-key"));

        let err2 = CredentialError::StoreError("disk full".to_string());
        assert!(err2.to_string().contains("Failed to store"));
    }
}
