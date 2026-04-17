//! Integration tests for credential lifecycle.
//!
//! These tests verify the credential status flow:
//! - Set credential → status returns has_credential: true
//! - Delete credential → status returns has_credential: false
//! - Get status for unknown provider → returns has_credential: false

mod test_helpers;

use std::sync::Arc;

use artifex_model_config::credential_store::InMemoryCredentialStore;
use artifex_model_config::provider::{AuthType, ModelCapability, ProviderKind, ProviderMetadata, ProviderError};
use artifex_model_config::image_provider::{ImageGenParams, ImageGenResult, ImageProvider};
use async_trait::async_trait;

use test_helpers::setup_test_db;
use src_tauri::model_config::service::ModelConfigService;
use src_tauri::model_config::SqliteModelConfigRepository;

/// Mock image provider for testing.
struct MockImageProvider {
    metadata: ProviderMetadata,
}

impl MockImageProvider {
    fn new() -> Self {
        Self {
            metadata: ProviderMetadata {
                id: "testprovider".to_string(),
                name: "TestProvider".to_string(),
                kind: ProviderKind::Replicate,
                base_url: "https://api.test.com".to_string(),
                supported_capabilities: vec![ModelCapability::ImageGen],
                auth_type: AuthType::ApiKey,
            },
        }
    }
}

#[async_trait]
impl ImageProvider for MockImageProvider {
    async fn generate(
        &self,
        _params: &ImageGenParams,
        _api_key: &str,
    ) -> Result<ImageGenResult, ProviderError> {
        Ok(ImageGenResult::new(vec![0, 1, 2, 3], 512, 512, "png"))
    }

    async fn remove_background(
        &self,
        _image_data: &[u8],
        _api_key: &str,
    ) -> Result<ImageGenResult, ProviderError> {
        Ok(ImageGenResult::new(vec![0, 1, 2, 3], 512, 512, "png"))
    }

    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }
}

#[tokio::test]
async fn test_credential_set_and_status() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let model_repo = Arc::new(SqliteModelConfigRepository::new(pool.clone()));
    let cred_store: Arc<dyn artifex_model_config::credential_store::CredentialStore> =
        Arc::new(InMemoryCredentialStore::new());

    // Register provider
    let registry = artifex_model_config::ProviderRegistry::new();
    let _ = registry.register_image("testprovider", Arc::new(MockImageProvider::new()));

    let service = ModelConfigService::new(
        model_repo,
        Arc::new(registry),
        cred_store.clone(),
    );

    // Set credential via service
    service
        .set_credential("testprovider", "test-api-key-123")
        .await
        .expect("set_credential should succeed");

    // Now status should show has_credential: true
    let status = service
        .get_credential_status("testprovider")
        .await
        .expect("get_credential_status should succeed");

    // Provider is registered and has credential
    assert!(
        status.has_credential,
        "Provider with credential should have has_credential: true"
    );
}

#[tokio::test]
async fn test_credential_delete_and_status() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let model_repo = Arc::new(SqliteModelConfigRepository::new(pool.clone()));
    let cred_store: Arc<dyn artifex_model_config::credential_store::CredentialStore> =
        Arc::new(InMemoryCredentialStore::new());

    // Register provider
    let registry = artifex_model_config::ProviderRegistry::new();
    let _ = registry.register_image("testprovider", Arc::new(MockImageProvider::new()));

    let service = ModelConfigService::new(
        model_repo,
        Arc::new(registry),
        cred_store.clone(),
    );

    // Set credential
    service
        .set_credential("testprovider", "test-api-key-123")
        .await
        .expect("set_credential should succeed");

    // Delete credential
    service
        .delete_credential("testprovider")
        .await
        .expect("delete_credential should succeed");

    // Now status should show has_credential: false
    let status = service
        .get_credential_status("testprovider")
        .await
        .expect("get_credential_status should succeed");

    assert!(
        !status.has_credential,
        "Provider after deleting credential should have has_credential: false"
    );
}

#[tokio::test]
async fn test_credential_status_unknown_provider() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let model_repo = Arc::new(SqliteModelConfigRepository::new(pool));
    let cred_store: Arc<dyn artifex_model_config::credential_store::CredentialStore> =
        Arc::new(InMemoryCredentialStore::new());

    // Registry WITHOUT registering any providers
    let registry = artifex_model_config::ProviderRegistry::new();

    let service = ModelConfigService::new(
        model_repo,
        Arc::new(registry),
        cred_store,
    );

    // Get status for unknown provider
    let status = service
        .get_credential_status("unknown_provider")
        .await
        .expect("get_credential_status should succeed");

    assert!(
        !status.has_credential,
        "Unknown provider should have has_credential: false"
    );
}
