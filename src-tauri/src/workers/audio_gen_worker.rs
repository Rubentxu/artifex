//! Audio generation worker.

use std::path::PathBuf;
use std::sync::Arc;

use artifex_job_queue::Job;
use artifex_model_config::audio_provider::AudioGenParams;
use artifex_model_config::credential_store::CredentialStore;
use artifex_model_config::ModelRouter;
use artifex_model_config::tts_provider::TtsParams;
use artifex_shared_kernel::AppError;

use super::traits::{JobFuture, JobResult, JobWorker};

/// Worker for audio generation and TTS synthesis jobs.
pub struct AudioGenWorker {
    /// Model router for resolving providers and fallback chain.
    router: Arc<ModelRouter>,
    /// Credential store for API keys.
    credential_store: Arc<dyn CredentialStore>,
    /// Base directory for saving output assets.
    assets_dir: String,
}

impl AudioGenWorker {
    /// Creates a new AudioGenWorker.
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

    /// Determines the operation type based on audio kind.
    fn determine_audio_operation(kind: &Option<String>) -> &'static str {
        match kind.as_deref() {
            Some("music") => "audiogen.music",
            _ => "audiogen.sfx",
        }
    }
}

impl JobWorker for AudioGenWorker {
    fn can_handle(&self, job_type: &str) -> bool {
        job_type == "audio_generate" || job_type == "tts_synthesize"
    }

    fn process(&self, job: &Job) -> JobFuture {
        let router = self.router.clone();
        let credential_store = self.credential_store.clone();
        let assets_dir = self.assets_dir.clone();
        let operation = job.operation.clone();
        let job_id = job.id;
        let project_id = job.project_id;
        let job_type = job.job_type.clone();

        Box::pin(async move {
            match job_type.as_str() {
                "audio_generate" => {
                    Self::handle_audio_generate(
                        router,
                        credential_store,
                        assets_dir,
                        operation,
                        job_id,
                        project_id,
                    )
                    .await
                }
                "tts_synthesize" => {
                    Self::handle_tts_synthesize(
                        router,
                        credential_store,
                        assets_dir,
                        operation,
                        job_id,
                        project_id,
                    )
                    .await
                }
                _ => Err(AppError::validation(format!(
                    "Unsupported job type: {}",
                    job_type
                ))),
            }
        })
    }
}

impl AudioGenWorker {
    /// Handles audio generation (SFX/Music) jobs.
    async fn handle_audio_generate(
        router: Arc<ModelRouter>,
        credential_store: Arc<dyn CredentialStore>,
        assets_dir: String,
        operation: serde_json::Value,
        job_id: artifex_shared_kernel::JobId,
        project_id: artifex_shared_kernel::ProjectId,
    ) -> Result<JobResult, AppError> {
        // Deserialize operation JSON into AudioGenParams
        let mut params: AudioGenParams = serde_json::from_value(operation)
            .map_err(|e| AppError::validation(format!("Invalid audio generation params: {}", e)))?;

        // Validate params before calling provider
        params.validate().map_err(|e| AppError::validation(e))?;

        // Determine operation type based on kind
        let operation_type = Self::determine_audio_operation(&params.kind);

        // Resolve the model profile using the router (includes fallback chain)
        let resolved = router
            .resolve_audio(operation_type)
            .await
            .map_err(|e| AppError::internal(format!("Failed to resolve audio model: {}", e)))?;

        // Inject the model_id from the resolved profile into params
        params.model_id = Some(resolved.profile.model_id.clone());

        // Get credential from store
        let credential_id = format!("{}::api_key", resolved.profile.provider_name);
        let api_key = credential_store
            .get(&credential_id)
            .map_err(|_| AppError::internal(format!("Credential not found for {}", resolved.profile.provider_name)))?;

        // Call provider to generate audio
        let result = resolved
            .provider
            .generate(&params, &api_key)
            .await
            .map_err(|e| AppError::internal(format!("Provider error: {}", e)))?;

        // Determine output directory and file extension based on kind
        let (output_subdir, extension) = match params.kind.as_deref() {
            Some("music") => ("music", "mp3"),
            _ => ("audio", "mp3"),
        };

        // Build output path: {assets_dir}/{project_id}/{audio|music}/{job_id}.{ext}
        let output_dir = PathBuf::from(&assets_dir)
            .join(project_id.into_uuid().to_string())
            .join(output_subdir);

        // Create directory structure
        tokio::fs::create_dir_all(&output_dir)
            .await
            .map_err(|e| AppError::io_error(format!("Failed to create output directory: {}", e)))?;

        let output_file = output_dir.join(format!("{}.{}", job_id.into_uuid(), extension));

        // Save audio to file
        tokio::fs::write(&output_file, &result.audio_data)
            .await
            .map_err(|e| AppError::io_error(format!("Failed to write audio file: {}", e)))?;

        // Return JobResult with full generation metadata for asset persistence
        let job_result = JobResult::with_metadata(
            vec![output_file.clone()],
            serde_json::json!({
                "prompt": params.prompt,
                "kind": params.kind,
                "duration_secs": result.duration_secs,
                "format": result.format,
                "model": resolved.profile.model_id,
                "provider": resolved.profile.provider_name,
                "project_id": project_id.into_uuid().to_string(),
            }),
        );

        Ok(job_result)
    }

    /// Handles TTS synthesis jobs.
    async fn handle_tts_synthesize(
        router: Arc<ModelRouter>,
        credential_store: Arc<dyn CredentialStore>,
        assets_dir: String,
        operation: serde_json::Value,
        job_id: artifex_shared_kernel::JobId,
        project_id: artifex_shared_kernel::ProjectId,
    ) -> Result<JobResult, AppError> {
        // Deserialize operation JSON into TtsParams
        let mut params: TtsParams = serde_json::from_value(operation)
            .map_err(|e| AppError::validation(format!("Invalid TTS params: {}", e)))?;

        // Validate params before calling provider
        params.validate().map_err(|e| AppError::validation(e))?;

        // Resolve the model profile using the router (includes fallback chain)
        let resolved = router
            .resolve_tts("tts.npc_line")
            .await
            .map_err(|e| AppError::internal(format!("Failed to resolve TTS model: {}", e)))?;

        // Inject the model_id from the resolved profile into params
        params.model_id = Some(resolved.profile.model_id.clone());

        // Get credential from store
        let credential_id = format!("{}::api_key", resolved.profile.provider_name);
        let api_key = credential_store
            .get(&credential_id)
            .map_err(|_| AppError::internal(format!("Credential not found for {}", resolved.profile.provider_name)))?;

        // Call provider to synthesize speech
        let result = resolved
            .provider
            .synthesize(&params, &api_key)
            .await
            .map_err(|e| AppError::internal(format!("Provider error: {}", e)))?;

        // Build output path: {assets_dir}/{project_id}/voice/{job_id}.mp3
        let output_dir = PathBuf::from(&assets_dir)
            .join(project_id.into_uuid().to_string())
            .join("voice");

        // Create directory structure
        tokio::fs::create_dir_all(&output_dir)
            .await
            .map_err(|e| AppError::io_error(format!("Failed to create output directory: {}", e)))?;

        let output_file = output_dir.join(format!("{}.mp3", job_id.into_uuid()));

        // Save audio to file
        tokio::fs::write(&output_file, &result.audio_data)
            .await
            .map_err(|e| AppError::io_error(format!("Failed to write audio file: {}", e)))?;

        // Return JobResult with full synthesis metadata for asset persistence
        let job_result = JobResult::with_metadata(
            vec![output_file.clone()],
            serde_json::json!({
                "text": params.text,
                "voice_id": params.voice_id,
                "language": params.language,
                "speed": params.speed,
                "duration_secs": result.duration_secs,
                "format": result.format,
                "model": resolved.profile.model_id,
                "provider": resolved.profile.provider_name,
                "project_id": project_id.into_uuid().to_string(),
            }),
        );

        Ok(job_result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full unit tests for AudioGenWorker are limited because the worker
    // requires async dependencies (ModelRouter, CredentialStore).
    // Full integration tests are in src-tauri/tests/ directory.

    #[test]
    fn test_determine_audio_operation_sfx() {
        let kind = Some("sfx".to_string());
        assert_eq!(AudioGenWorker::determine_audio_operation(&kind), "audiogen.sfx");
    }

    #[test]
    fn test_determine_audio_operation_music() {
        let kind = Some("music".to_string());
        assert_eq!(AudioGenWorker::determine_audio_operation(&kind), "audiogen.music");
    }

    #[test]
    fn test_determine_audio_operation_default() {
        let kind = None;
        assert_eq!(AudioGenWorker::determine_audio_operation(&kind), "audiogen.sfx");
    }
}