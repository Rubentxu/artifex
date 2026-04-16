//! Keychain-backed credential store.
//!
//! Uses the OS keychain (via the `keyring` crate) for persistent
//! credential storage. Provides graceful degradation: if the keychain
//! is unavailable, operations return errors that callers can handle
//! by falling back to in-memory storage.

use artifex_model_config::credential_store::{CredentialError, CredentialStore};

/// Service name used for all keychain entries.
const SERVICE_NAME: &str = "com.artifex.game-ai-studio";

/// Keychain-backed credential store.
///
/// Each credential is stored with:
/// - service: "com.artifex.game-ai-studio"
/// - username: the credential_id (e.g., "replicate::api_key")
/// - password: the secret value
pub struct KeychainCredentialStore {
    _priv: (),
}

impl KeychainCredentialStore {
    /// Creates a new KeychainCredentialStore.
    pub fn new() -> Self {
        Self { _priv: () }
    }
}

impl Default for KeychainCredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialStore for KeychainCredentialStore {
    fn get(&self, credential_id: &str) -> Result<String, CredentialError> {
        let entry = keyring::Entry::new(SERVICE_NAME, credential_id)
            .map_err(|e| CredentialError::Unavailable(e.to_string()))?;

        entry
            .get_password()
            .map_err(|_| CredentialError::NotFound(credential_id.to_string()))
    }

    fn set(&self, credential_id: &str, value: &str) -> Result<(), CredentialError> {
        let entry = keyring::Entry::new(SERVICE_NAME, credential_id)
            .map_err(|e| CredentialError::Unavailable(e.to_string()))?;

        entry
            .set_password(value)
            .map_err(|e| CredentialError::StoreError(e.to_string()))
    }

    fn delete(&self, credential_id: &str) -> Result<(), CredentialError> {
        let entry = keyring::Entry::new(SERVICE_NAME, credential_id)
            .map_err(|e| CredentialError::Unavailable(e.to_string()))?;

        entry
            .delete_credential()
            .map_err(|_| CredentialError::NotFound(credential_id.to_string()))
    }

    fn list(&self) -> Result<Vec<String>, CredentialError> {
        // Keyring doesn't support listing all entries for a service,
        // so we return an empty list. This is a limitation of the keyring API.
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keychain_credential_store_creation() {
        let store = KeychainCredentialStore::new();
        // Can't test actual keychain operations in unit tests without
        // a real keyring available, but we can verify construction works.
        let _ = store;
    }
}
