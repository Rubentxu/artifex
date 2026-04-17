//! Sprite sheet generation IPC commands.

use std::path::Path;
use tauri::State;

use crate::dto::GenerateSpriteSheetRequest;
use crate::state::AppState;

/// Supported video formats for sprite sheet generation.
const SUPPORTED_VIDEO_FORMATS: [&str; 3] = ["mp4", "gif", "webm"];

/// Generates a sprite sheet from a video asset.
/// Creates a job with job_type "sprite_generate" and returns the job ID.
#[tauri::command]
pub async fn generate_sprite_sheet(
    state: State<'_, AppState>,
    request: GenerateSpriteSheetRequest,
) -> Result<String, String> {
    // Validate that the source asset exists
    let source_asset = state
        .asset_service
        .get_asset(&request.asset_id)
        .await
        .map_err(|e| e.to_string())?;

    // Validate asset is a video
    if source_asset.kind != artifex_asset_management::AssetKind::Video {
        return Err("Source asset must be a video".to_string());
    }

    // Validate file extension is supported
    let source_file_path = source_asset
        .file_path
        .ok_or_else(|| "Source asset has no file path".to_string())?;

    let path = Path::new(&source_file_path);
    if let Some(ext) = path.extension() {
        let ext_lower = ext.to_string_lossy().to_lowercase();
        if !SUPPORTED_VIDEO_FORMATS.contains(&ext_lower.as_str()) {
            return Err(format!(
                "Unsupported video format: '.{}'. Supported formats: {}",
                ext_lower,
                SUPPORTED_VIDEO_FORMATS.iter().map(|s| format!(".{}", s)).collect::<Vec<_>>().join(", ")
            ));
        }
    } else {
        return Err("Source file has no extension".to_string());
    }

    // Build operation JSON from request params
    // output_dir will be resolved by the worker using its assets_dir
    let operation = serde_json::json!({
        "source_video_path": source_file_path,
        "fps": request.fps,
        "dedup_threshold": request.dedup_threshold,
        "atlas_max_size": request.atlas_max_size,
        "padding": request.padding,
        "animation_name": request.animation_name,
        "output_format": request.output_format,
        "source_asset_id": request.asset_id,
        "project_id": request.project_id,
    });

    let job = state
        .job_service
        .create_job(&request.project_id, "sprite_generate", operation)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job.id.into_uuid().to_string())
}
