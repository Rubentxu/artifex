//! Material generation IPC command.

use tauri::State;

use crate::dto::GenerateMaterialRequest;
use crate::state::AppState;

/// Generates PBR material maps from an image using the configured provider.
/// Creates a job with job_type "material_generate" and returns the job ID.
#[tauri::command]
pub async fn generate_material(
    state: State<'_, AppState>,
    request: GenerateMaterialRequest,
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

    // Get the file path - it's required for material generation
    let source_file_path = source_asset
        .file_path
        .ok_or_else(|| "Source asset has no file path".to_string())?;

    // Build operation JSON from request params
    let operation = serde_json::json!({
        "source_asset_id": request.asset_id,
        "source_file_path": source_file_path,
        "provider_id": request.provider_id,
        "model_id": request.model_id,
    });

    let job = state
        .job_service
        .create_job(&request.project_id, "material_generate", operation)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job.id.into_uuid().to_string())
}
