//! Model configuration infrastructure setup.

use std::sync::Arc;

use artifex_model_config::credential_store::CredentialStore;
use artifex_model_config::ModelRouter;

use crate::model_config::{
    register_builtin_providers, ModelConfigService, SqliteModelConfigRepository,
};

/// Keychain credential store probe result.
pub enum KeychainProbeResult {
    Available,
    Unavailable(String),
}

/// Attempts to create a keychain credential store.
///
/// Returns Ok if the keychain is accessible, Err otherwise.
/// The error contains a message describing why keychain is unavailable.
pub fn try_create_keychain_store() -> Result<crate::model_config::KeychainCredentialStore, String> {
    let store = crate::model_config::KeychainCredentialStore::new();
    // Probe the keychain by attempting to list credentials.
    // This will fail if the keyring service is not available.
    match store.list() {
        Ok(_) => Ok(store),
        Err(e) => Err(format!("keychain probe failed: {}", e)),
    }
}

/// Creates the credential store with graceful degradation.
/// Tries KeychainCredentialStore first; if keyring is unavailable, falls back to in-memory.
pub fn create_credential_store() -> Arc<dyn CredentialStore> {
    match try_create_keychain_store() {
        Ok(store) => {
            tracing::info!("Using keychain credential store");
            Arc::new(store)
        }
        Err(e) => {
            tracing::warn!("Keychain unavailable ({:?}), falling back to in-memory credential store", e);
            Arc::new(artifex_model_config::credential_store::InMemoryCredentialStore::new())
        }
    }
}

/// Creates model config infrastructure (registry, router, service, credential_store).
pub fn create_model_config(
    pool: sqlx::SqlitePool,
) -> (
    Arc<ModelConfigService>,
    Arc<ModelRouter>,
    Arc<dyn CredentialStore>,
) {
    let model_config_repo = Arc::new(SqliteModelConfigRepository::new(pool));

    // Create provider registry and register built-in providers
    let provider_registry = Arc::new(artifex_model_config::ProviderRegistry::new());
    register_builtin_providers(&provider_registry);

    // Create credential store with graceful degradation.
    let credential_store = create_credential_store();

    // Create model router (uses async ModelConfigRepository trait impl on SqliteModelConfigRepository)
    let model_router = Arc::new(ModelRouter::new(
        provider_registry.clone(),
        model_config_repo.clone(),
        credential_store.clone(),
    ));

    // Create model config service
    let model_config_service = Arc::new(ModelConfigService::new(
        model_config_repo.clone(),
        provider_registry.clone(),
        credential_store.clone(),
    ));

    (model_config_service, model_router, credential_store)
}

/// Seeds default model profiles and routing rules.
pub fn seed_model_config(model_config_service: &Arc<ModelConfigService>) {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            if let Err(e) = model_config_service.seed_defaults().await {
                tracing::warn!("Failed to seed default model config: {}", e);
            }
        })
    });
}
