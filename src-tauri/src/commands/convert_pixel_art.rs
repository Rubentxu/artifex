//! Pixel art conversion IPC commands.

use tauri::State;

use crate::dto::ConvertPixelArtRequest;
use crate::state::AppState;

/// Converts an image to pixel art using the configured palette and dithering.
/// Creates a job with job_type "pixel_art_convert" and returns the job ID.
#[tauri::command]
pub async fn convert_pixel_art(
    state: State<'_, AppState>,
    request: ConvertPixelArtRequest,
) -> Result<String, String> {
    // Check tier - Pro required for pixel art conversion
    let tier = state
        .identity_service
        .get_tier()
        .await
        .map_err(|e| e.to_string())?;
    if !tier.is_pro() {
        return Err("Pro tier required for pixel art conversion".to_string());
    }

    // Validate that the source asset exists
    let source_asset = state
        .asset_service
        .get_asset(&request.asset_id)
        .await
        .map_err(|e| e.to_string())?;

    // Validate asset is an image
    if source_asset.kind != artifex_asset_management::AssetKind::Image {
        return Err("Source asset must be an image".to_string());
    }

    // Get the file path for local processing
    let source_file_path = source_asset
        .file_path
        .ok_or_else(|| "Source asset has no file path".to_string())?;

    // Build operation JSON from request params
    let operation = serde_json::json!({
        "source_asset_id": request.asset_id,
        "source_file_path": source_file_path,
        "target_width": request.target_width,
        "target_height": request.target_height,
        "palette": request.palette,
        "dithering": request.dithering,
        "outline": request.outline,
        "outline_threshold": request.outline_threshold,
    });

    let job = state
        .job_service
        .create_job(&request.project_id, "pixel_art_convert", operation)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job.id.into_uuid().to_string())
}