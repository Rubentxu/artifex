//! Integration tests for image generation worker.
//!
//! These tests verify the image generation worker can process jobs
//! using the model configuration system.

mod test_helpers;

use std::sync::Arc;

use artifex_job_queue::{Job, JobRepository, JobStatus};
use artifex_shared_kernel::{JobId, ProjectId};
use artifex_model_config::credential_store::InMemoryCredentialStore;
use artifex_model_config::image_provider::{ImageGenParams, ImageGenResult, ImageProvider};
use artifex_model_config::provider::{AuthType, ModelCapability, ProviderKind, ProviderMetadata, ProviderError};
use async_trait::async_trait;
use serde_json::json;
use src_tauri::model_config::SqliteModelConfigRepository;
use src_tauri::workers::image_gen_worker::ImageGenWorker;
use src_tauri::workers::JobWorker;
use test_helpers::setup_test_db;
use artifex_model_config::ModelRouter;
use artifex_model_config::ModelProfile;
use uuid::Uuid;

/// Mock image provider that tracks call count and can fail on demand.
struct MockImageProvider {
    metadata: ProviderMetadata,
    call_count: std::sync::atomic::AtomicUsize,
    should_fail: bool,
}

impl MockImageProvider {
    fn new(id: &str, should_fail: bool) -> Self {
        Self {
            metadata: ProviderMetadata {
                id: id.to_string(),
                name: id.to_string(),
                kind: ProviderKind::Replicate,
                base_url: format!("https://api.{}.com", id),
                supported_capabilities: vec![ModelCapability::ImageGen],
                auth_type: AuthType::ApiKey,
            },
            call_count: std::sync::atomic::AtomicUsize::new(0),
            should_fail,
        }
    }

    fn call_count(&self) -> usize {
        self.call_count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[async_trait]
impl ImageProvider for MockImageProvider {
    async fn generate(
        &self,
        _params: &ImageGenParams,
        _api_key: &str,
    ) -> Result<ImageGenResult, ProviderError> {
        self.call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if self.should_fail {
            Err(ProviderError::AuthFailed {
                provider: self.metadata.id.clone(),
                message: "Mock auth failure".to_string(),
            })
        } else {
            Ok(ImageGenResult::new(
                vec![0, 1, 2, 3],
                512,
                512,
                "png",
            ))
        }
    }

    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }
}

/// Test that worker can be created with model router.
#[tokio::test]
async fn test_image_gen_worker_creation() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = Arc::new(SqliteModelConfigRepository::new(pool));
    let registry = Arc::new(artifex_model_config::ProviderRegistry::new());
    let cred_store: Arc<dyn artifex_model_config::credential_store::CredentialStore> =
        Arc::new(InMemoryCredentialStore::new());

    let router = Arc::new(ModelRouter::new(
        registry,
        repo,
        cred_store.clone(),
    ));

    let worker = ImageGenWorker::new(
        router,
        cred_store,
        "/tmp/test_assets".to_string(),
    );

    assert!(worker.can_handle("image_generate"));
    assert!(!worker.can_handle("other_job"));
}

/// Test that worker correctly identifies image generation jobs.
#[tokio::test]
async fn test_image_gen_worker_can_handle() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = Arc::new(SqliteModelConfigRepository::new(pool));
    let registry = Arc::new(artifex_model_config::ProviderRegistry::new());
    let cred_store: Arc<dyn artifex_model_config::credential_store::CredentialStore> =
        Arc::new(InMemoryCredentialStore::new());

    let router = Arc::new(ModelRouter::new(
        registry,
        repo,
        cred_store.clone(),
    ));

    let worker = ImageGenWorker::new(
        router,
        cred_store,
        "/tmp/test_assets".to_string(),
    );

    // Worker should handle "image_generate" job type
    assert!(worker.can_handle("image_generate"));

    // Worker should not handle other job types
    assert!(!worker.can_handle("text_generate"));
    assert!(!worker.can_handle("audio_generate"));
}

/// E-5: Test that ImageGenWorker follows the full fallback chain via ModelRouter.
/// This test verifies that when the first profile fails (no credential),
/// the router correctly falls back to the second profile.
#[tokio::test]
async fn test_worker_fallback_chain() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");

    // Create two profiles: first will have no credential (will be skipped),
    // second will have credential (will succeed)
    let profile1 = ModelProfile::new(
        "provider1".to_string(),
        "model-1".to_string(),
        "Provider 1 Model".to_string(),
        vec![ModelCapability::ImageGen],
    );

    let profile2 = ModelProfile::new(
        "provider2".to_string(),
        "model-2".to_string(),
        "Provider 2 Model".to_string(),
        vec![ModelCapability::ImageGen],
    );

    // Create routing rule with profile1 as default and profile2 as fallback
    let rule = artifex_model_config::RoutingRule::new(
        "imagegen.txt2img".to_string(),
        profile1.id,
        vec![profile2.id],
    );

    // Insert profiles and rule into DB
    src_tauri::model_config::create_profile(&pool, &profile1).await.expect("create profile1");
    src_tauri::model_config::create_profile(&pool, &profile2).await.expect("create profile2");
    src_tauri::model_config::create_rule(&pool, &rule).await.expect("create rule");

    // Create repository and registry
    let repo = Arc::new(SqliteModelConfigRepository::new(pool));
    let registry = Arc::new(artifex_model_config::ProviderRegistry::new());

    // Create mock providers
    let provider1 = Arc::new(MockImageProvider::new("provider1", false));
    let provider2 = Arc::new(MockImageProvider::new("provider2", false));

    registry.register_image("provider1", provider1.clone()).expect("register provider1");
    registry.register_image("provider2", provider2.clone()).expect("register provider2");

    // Credential store: only provider2 has a credential
    let cred_store: Arc<dyn artifex_model_config::credential_store::CredentialStore> =
        Arc::new(InMemoryCredentialStore::new());
    cred_store.set("provider2::api_key", "test-key").expect("set credential");

    let router = Arc::new(ModelRouter::new(
        registry.clone(),
        repo,
        cred_store.clone(),
    ));

    let worker = ImageGenWorker::new(
        router,
        cred_store,
        "/tmp/test_assets".to_string(),
    );

    // Create a test job
    let job = Job::new(
        ProjectId::new(),
        "image_generate".to_string(),
        json!({
            "prompt": "test prompt",
            "width": 512,
            "height": 512,
            "steps": 20,
        }),
    );

    // Process the job - should succeed by falling back to provider2
    let result = worker.process(&job).await;
    assert!(result.is_ok(), "Worker should succeed via fallback chain");

    // Verify provider1 was NOT called (no credential, skipped)
    assert_eq!(provider1.call_count(), 0, "Provider1 should not be called (no credential)");

    // Verify provider2 WAS called (fallback succeeded)
    assert_eq!(provider2.call_count(), 1, "Provider2 should be called via fallback");
}
