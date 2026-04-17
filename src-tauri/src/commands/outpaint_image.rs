//! Outpaint image IPC command.

use tauri::State;

use crate::dto::OutpaintRequest;
use crate::state::AppState;

/// Outpaints an image by extending the canvas in a given direction.
/// Creates a job with job_type "image_outpaint" and returns the job ID.
#[tauri::command]
pub async fn outpaint_image(
    state: State<'_, AppState>,
    request: OutpaintRequest,
) -> Result<String, String> {
    // Validate prompt
    if request.prompt.trim().is_empty() {
        return Err("Prompt cannot be empty".to_string());
    }

    // Validate extend_pixels
    if request.extend_pixels < 64 {
        return Err("Extension must be at least 64 pixels".to_string());
    }
    if request.extend_pixels > 1024 {
        return Err("Extension cannot exceed 1024 pixels".to_string());
    }

    // Validate that the source asset exists
    let source_asset = state
        .asset_service
        .get_asset(&request.asset_id)
        .await
        .map_err(|e| e.to_string())?;

    // Validate asset is an image
    if source_asset.kind != artifex_asset_management::AssetKind::Image {
        return Err("Asset must be an image".to_string());
    }

    // Get the file path - it's required for outpainting
    let source_file_path = source_asset
        .file_path
        .ok_or_else(|| "Source asset has no file path".to_string())?;

    // Build operation JSON from request params
    let operation = serde_json::json!({
        "source_asset_id": request.asset_id,
        "source_file_path": source_file_path,
        "direction": request.direction,
        "extend_pixels": request.extend_pixels,
        "prompt": request.prompt,
        "negative_prompt": request.negative_prompt,
        "strength": request.strength,
        "guidance_scale": request.guidance_scale,
        "steps": request.steps,
        "provider_mode": request.provider_mode,
    });

    let job = state
        .job_service
        .create_job(&request.project_id, "image_outpaint", operation)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job.id.into_uuid().to_string())
}
