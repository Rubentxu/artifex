//! Video generation worker.

use std::path::PathBuf;
use std::sync::Arc;

use artifex_job_queue::Job;
use artifex_model_config::video_provider::VideoGenParams;
use artifex_model_config::credential_store::CredentialStore;
use artifex_model_config::ModelRouter;
use artifex_shared_kernel::AppError;

use super::traits::{JobFuture, JobResult, JobWorker};

/// Worker for video generation jobs.
pub struct VideoGenWorker {
    /// Model router for resolving providers and fallback chain.
    router: Arc<ModelRouter>,
    /// Credential store for API keys.
    credential_store: Arc<dyn CredentialStore>,
    /// Base directory for saving output assets.
    assets_dir: String,
}

impl VideoGenWorker {
    /// Creates a new VideoGenWorker.
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

impl JobWorker for VideoGenWorker {
    fn can_handle(&self, job_type: &str) -> bool {
        job_type == "video_generate"
    }

    fn process(&self, job: &Job) -> JobFuture {
        let router = self.router.clone();
        let credential_store = self.credential_store.clone();
        let assets_dir = self.assets_dir.clone();
        let operation = job.operation.clone();
        let job_id = job.id;
        let project_id = job.project_id;

        Box::pin(async move {
            // Deserialize operation JSON into VideoGenParams
            let mut params: VideoGenParams = serde_json::from_value(operation)
                .map_err(|e| AppError::validation(format!("Invalid video generation params: {}", e)))?;

            // Validate params before calling provider
            params.validate().map_err(AppError::validation)?;

            // Resolve the model profile using the router (includes fallback chain)
            let resolved = router
                .resolve_video("videogen.img2video")
                .await
                .map_err(|e| AppError::internal(format!("Failed to resolve video model: {}", e)))?;

            // Inject the model_id from the resolved profile into params
            params.model_id = Some(resolved.profile.model_id.clone());

            // Get credential from store
            let credential_id = format!("{}::api_key", resolved.profile.provider_name);
            let api_key = credential_store
                .get(&credential_id)
                .map_err(|_| AppError::internal(format!("Credential not found for {}", resolved.profile.provider_name)))?;

            // Call provider to generate video
            let result = resolved
                .provider
                .generate_video(&params, &api_key)
                .await
                .map_err(|e| AppError::internal(format!("Provider error: {}", e)))?;

            // Build output path: {assets_dir}/{project_id}/videos/{job_id}.mp4
            let output_dir = PathBuf::from(&assets_dir)
                .join(project_id.into_uuid().to_string())
                .join("videos");

            // Create directory structure
            tokio::fs::create_dir_all(&output_dir)
                .await
                .map_err(|e| AppError::io_error(format!("Failed to create output directory: {}", e)))?;

            let output_file = output_dir.join(format!("{}.mp4", job_id.into_uuid()));

            // Save video to file
            tokio::fs::write(&output_file, &result.video_data)
                .await
                .map_err(|e| AppError::io_error(format!("Failed to write video file: {}", e)))?;

            // Return JobResult with full generation metadata for asset persistence
            let job_result = JobResult::with_metadata(
                vec![output_file.clone()],
                serde_json::json!({
                    "source_image_url": params.source_image_url,
                    "prompt": params.prompt,
                    "negative_prompt": params.negative_prompt,
                    "duration_secs": result.duration_secs,
                    "format": result.format,
                    "seed": params.seed,
                    "model": resolved.profile.model_id,
                    "provider": resolved.profile.provider_name,
                    "project_id": project_id.into_uuid().to_string(),
                }),
            );

            Ok(job_result)
        })
    }
}

#[cfg(test)]
mod tests {

    // Note: Unit tests for VideoGenWorker are limited because the worker
    // requires async dependencies (ModelRouter, CredentialStore).
    // Full integration tests are in src-tauri/tests/ directory.
    //
    // The can_handle method is tested implicitly through the worker's
    // ability to be instantiated and handle the correct job type.
}
