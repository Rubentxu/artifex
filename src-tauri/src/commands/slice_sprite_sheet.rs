//! Sprite sheet slicing IPC commands.

use tauri::State;

use crate::dto::SliceSpriteSheetRequest;
use crate::state::AppState;

/// Supported image formats for sprite sheet slicing.
const SUPPORTED_IMAGE_FORMATS: [&str; 5] = ["png", "jpg", "jpeg", "gif", "webp"];

/// Slices a sprite sheet image into individual frames.
/// Creates a job with job_type "sprite_slice" and returns the job ID.
#[tauri::command]
pub async fn slice_sprite_sheet(
    state: State<'_, AppState>,
    request: SliceSpriteSheetRequest,
) -> Result<String, String> {
    // Validate that the source asset exists
    let source_asset = state
        .asset_service
        .get_asset(&request.asset_id)
        .await
        .map_err(|e| e.to_string())?;

    // Validate asset is an image or sprite
    use artifex_asset_management::AssetKind;
    match source_asset.kind {
        AssetKind::Image | AssetKind::Sprite => {}
        _ => {
            return Err("Source asset must be an Image or Sprite".to_string());
        }
    }

    // Validate file extension is supported
    let source_file_path = source_asset
        .file_path
        .ok_or_else(|| "Source asset has no file path".to_string())?;

    let path = std::path::Path::new(&source_file_path);
    if let Some(ext) = path.extension() {
        let ext_lower = ext.to_string_lossy().to_lowercase();
        if !SUPPORTED_IMAGE_FORMATS.contains(&ext_lower.as_str()) {
            return Err(format!(
                "Unsupported image format: '.{}'. Supported formats: {}",
                ext_lower,
                SUPPORTED_IMAGE_FORMATS.iter().map(|s| format!(".{}", s)).collect::<Vec<_>>().join(", ")
            ));
        }
    } else {
        return Err("Source file has no extension".to_string());
    }

    // Build operation JSON from request params
    let operation = serde_json::json!({
        "source_file_path": source_file_path,
        "mode": request.mode,
        "grid_params": request.grid_params,
        "auto_detect_params": request.auto_detect_params,
        "source_asset_id": request.asset_id,
        "project_id": request.project_id,
    });

    let job = state
        .job_service
        .create_job(&request.project_id, "sprite_slice", operation)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job.id.into_uuid().to_string())
}
