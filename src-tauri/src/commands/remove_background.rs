//! Background removal IPC commands.

use tauri::State;

use crate::dto::RemoveBackgroundRequest;
use crate::state::AppState;

/// Removes background from an image using the configured provider.
/// Creates a job with job_type "image_remove_background" and returns the job ID.
#[tauri::command]
pub async fn remove_background(
    state: State<'_, AppState>,
    request: RemoveBackgroundRequest,
) -> Result<String, String> {
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

    // Get the file path - it's required for background removal
    let source_file_path = source_asset
        .file_path
        .ok_or_else(|| "Source asset has no file path".to_string())?;

    // Build operation JSON from request params
    let operation = serde_json::json!({
        "source_asset_id": request.asset_id,
        "source_file_path": source_file_path,
        "provider_mode": request.provider_mode,
    });

    let job = state
        .job_service
        .create_job(&request.project_id, "image_remove_background", operation)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job.id.into_uuid().to_string())
}