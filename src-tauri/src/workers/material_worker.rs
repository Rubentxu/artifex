//! Material generation worker.
//!
//! Handles PBR material map generation from source images.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use artifex_job_queue::Job;
use artifex_model_config::credential_store::CredentialStore;
use artifex_model_config::image_provider::MaterialGenParams;
use artifex_model_config::ModelRouter;
use artifex_shared_kernel::AppError;
use serde::Deserialize;

use super::traits::{JobFuture, JobResult, JobWorker};

/// Payload for material generation jobs.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MaterialGeneratePayload {
    source_asset_id: String,
    source_file_path: String,
    provider_id: Option<String>,
    model_id: Option<String>,
}

/// Worker for PBR material generation jobs.
pub struct MaterialWorker {
    /// Model router for resolving providers.
    router: Arc<ModelRouter>,
    /// Credential store for API keys.
    credential_store: Arc<dyn CredentialStore>,
    /// Base directory for saving output assets.
    assets_dir: String,
}

impl MaterialWorker {
    /// Creates a new MaterialWorker.
    pub fn new(
        router: Arc<ModelRouter>,
        credential_store: Arc<dyn CredentialStore>,
        assets_dir: String,
    ) -> Self {
        Self {
            router,
            credential_store,
            assets_dir,
        }
    }
}

impl JobWorker for MaterialWorker {
    fn can_handle(&self, job_type: &str) -> bool {
        job_type == "material_generate"
    }

    fn process(&self, job: &Job) -> JobFuture {
        let router = self.router.clone();
        let credential_store = self.credential_store.clone();
        let assets_dir = self.assets_dir.clone();
        let job_id = job.id;
        let project_id = job.project_id;
        let operation = job.operation.clone();

        Box::pin(async move {
            // Deserialize operation JSON
            let payload: MaterialGeneratePayload = serde_json::from_value(operation)
                .map_err(|e| AppError::validation(format!("Invalid job payload: {}", e)))?;

            // Read source image
            let source_bytes = tokio::fs::read(&payload.source_file_path)
                .await
                .map_err(|e| AppError::io_error(format!("Failed to read source image: {}", e)))?;

            // Resolve provider using routing key
            let resolved = router
                .resolve_image("materialgen.from_image")
                .await
                .map_err(|e| AppError::internal(format!("Failed to resolve provider: {}", e)))?;

            // Get credential
            let credential_id = format!("{}::api_key", resolved.profile.provider_name);
            let api_key = credential_store
                .get(&credential_id)
                .map_err(|_| AppError::internal(format!("Credential not found for {}", resolved.profile.provider_name)))?;

            // Build material generation params
            let params = MaterialGenParams {
                resolution: None,
            };

            // Call provider to generate material maps
            let result = resolved
                .provider
                .generate_material(&source_bytes, &params, &api_key)
                .await
                .map_err(|e| AppError::internal(format!("Provider error: {}", e)))?;

            // Build output directory structure: project_id/materials/job_id/
            let output_dir = PathBuf::from(&assets_dir)
                .join(project_id.into_uuid().to_string())
                .join("materials")
                .join(job_id.into_uuid().to_string());

            tokio::fs::create_dir_all(&output_dir)
                .await
                .map_err(|e| AppError::io_error(format!("Failed to create output directory: {}", e)))?;

            // Save each map to disk
            let mut map_paths = HashMap::new();
            for (map_kind, map_data) in &result.maps {
                let filename = format!("{}.png", map_kind.as_str());
                let map_path = output_dir.join(&filename);

                tokio::fs::write(&map_path, map_data)
                    .await
                    .map_err(|e| AppError::io_error(format!("Failed to write {} map: {}", map_kind.as_str(), e)))?;

                map_paths.insert(map_kind.as_str().to_string(), map_path.to_string_lossy().to_string());
            }

            // Determine basecolor path for the primary file_path (for preview)
            let basecolor_path = map_paths
                .get("basecolor")
                .cloned()
                .unwrap_or_else(|| {
                    // If no basecolor, use the first available map
                    map_paths.values().next().cloned().unwrap_or_default()
                });

            // Build metadata with all map paths
            let metadata = serde_json::json!({
                "operation": "material_generate",
                "source_asset_id": payload.source_asset_id,
                "provider": resolved.profile.provider_name,
                "model": resolved.profile.model_id,
                "project_id": project_id.into_uuid().to_string(),
                "maps": map_paths,
            });

            // Return result with the basecolor as the primary output file
            // and metadata containing all map paths
            Ok(JobResult::with_metadata(
                vec![PathBuf::from(&basecolor_path)],
                metadata,
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle() {
        let worker = MaterialWorker::new(
            Arc::new(ModelRouter::new(
                Arc::new(artifex_model_config::ProviderRegistry::new()),
                Arc::new(TestRepo),
                Arc::new(artifex_model_config::credential_store::InMemoryCredentialStore::new()),
            )),
            Arc::new(artifex_model_config::credential_store::InMemoryCredentialStore::new()),
            "/tmp".to_string(),
        );

        assert!(worker.can_handle("material_generate"));
        assert!(!worker.can_handle("image_generate"));
        assert!(!worker.can_handle("remove_background"));
    }

    // Mock repository for testing
    struct TestRepo;

    #[async_trait::async_trait]
    impl artifex_model_config::router::ModelConfigRepository for TestRepo {
        async fn find_profile(
            &self,
            _id: &uuid::Uuid,
        ) -> Result<Option<artifex_model_config::ModelProfile>, String> {
            Ok(None)
        }

        async fn find_rule(
            &self,
            _operation_type: &str,
        ) -> Result<Option<artifex_model_config::RoutingRule>, String> {
            Ok(None)
        }

        async fn list_enabled_profiles(
            &self,
            _capability: artifex_model_config::ModelCapability,
        ) -> Result<Vec<artifex_model_config::ModelProfile>, String> {
            Ok(vec![])
        }
    }
}
