//! Seamless texture generation IPC commands.

use tauri::State;

use crate::dto::SeamlessTextureRequest;
use crate::state::AppState;

/// Generates a seamless texture from an existing image asset.
/// Creates a job with job_type "seamless_texture" and returns the job ID.
#[tauri::command]
pub async fn generate_seamless_texture(
    state: State<'_, AppState>,
    request: SeamlessTextureRequest,
) -> Result<String, String> {
    // Validate request based on mode
    match request.mode {
        crate::dto::SeamlessMode::FromPrompt => {
            // Validate prompt
            let prompt = request.prompt.as_ref().ok_or_else(|| {
                "prompt is required for FromPrompt mode".to_string()
            })?;
            if prompt.trim().is_empty() {
                return Err("prompt must not be empty".to_string());
            }
        }
        crate::dto::SeamlessMode::FromAsset => {
            // Validate asset_id
            if request.asset_id.is_none() {
                return Err("asset_id is required for FromAsset mode".to_string());
            }
        }
    }

    // Get source asset if FromAsset mode
    let source_asset = if let Some(asset_id) = &request.asset_id {
        Some(
            state
                .asset_service
                .get_asset(asset_id)
                .await
                .map_err(|e| e.to_string())?,
        )
    } else {
        None
    };

    // Validate asset kind if provided
    if let Some(ref asset) = source_asset {
        let kind_str = asset.kind.as_str();
        if !matches!(kind_str, "Image" | "Sprite" | "Tileset" | "Material") {
            return Err(format!(
                "asset must be Image, Sprite, Tileset, or Material, got {}",
                kind_str
            ));
        }
    }

    // Get source file path
    let source_file_path = source_asset
        .as_ref()
        .and_then(|a| a.file_path.clone())
        .ok_or_else(|| "Source asset has no file path".to_string())?;

    // Validate seam threshold
    if let Some(threshold) = request.seam_threshold {
        if threshold <= 0.0 || threshold > 1.0 {
            return Err("seam_threshold must be in (0.0, 1.0]".to_string());
        }
    }

    // Build image generation params for FromPrompt mode
    let image_gen_params = if let Some(ref prompt) = request.prompt {
        Some(artifex_model_config::image_provider::ImageGenParams {
            prompt: prompt.clone(),
            negative_prompt: request.negative_prompt.clone(),
            width: request.width.unwrap_or(512),
            height: request.height.unwrap_or(512),
            steps: 20,
            seed: None,
            num_images: 1,
            guidance_scale: 7.5,
            model_id: None,
        })
    } else {
        None
    };

    // Build operation JSON
    let operation = serde_json::json!({
        "source_asset_id": request.asset_id.clone().unwrap_or_default(),
        "secondary_asset_id": request.secondary_asset_id,
        "source_file_path": source_file_path,
        "secondary_file_path": Option::<String>::None,
        "mode": request.mode,
        "image_gen_params": image_gen_params,
        "seam_threshold": request.seam_threshold.unwrap_or(0.05),
        "padding_pixels": request.padding_pixels.unwrap_or(16),
        "blend_fraction": request.blend_fraction.unwrap_or(0.5),
    });

    // Create job
    let job = state
        .job_service
        .create_job(&request.project_id, "seamless_texture", operation)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job.id.into_uuid().to_string())
}