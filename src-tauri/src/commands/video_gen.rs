//! Video generation IPC commands.

use base64::Engine;
use std::io::Read;
use tauri::State;

use crate::dto::GenerateVideoRequest;
use crate::state::AppState;

/// Generates a video from an image using the configured provider.
/// Creates a job with job_type "video_generate" and returns the job ID.
#[tauri::command]
pub async fn generate_video(
    state: State<'_, AppState>,
    request: GenerateVideoRequest,
) -> Result<String, String> {
    // Check tier - Pro required for video generation
    let tier = state
        .identity_service
        .get_tier()
        .await
        .map_err(|e| e.to_string())?;
    if !tier.is_pro() {
        return Err("Pro tier required for video generation".to_string());
    }

    // Validate that the source asset exists
    let source_asset = state
        .asset_service
        .get_asset(&request.source_image_asset_id)
        .await
        .map_err(|e| e.to_string())?;

    // Validate asset is an image or sprite
    let valid_kinds = [
        artifex_asset_management::AssetKind::Image,
        artifex_asset_management::AssetKind::Sprite,
    ];
    if !valid_kinds.contains(&source_asset.kind) {
        return Err("Source asset must be an image or sprite".to_string());
    }

    // Get the file path - it's required for video generation
    let source_file_path = source_asset
        .file_path
        .ok_or_else(|| "Source asset has no file path".to_string())?;

    // Read the source image file and convert to base64 data URI
    let source_image_url = read_file_as_data_uri(&source_file_path)
        .map_err(|e| format!("Failed to read source image: {}", e))?;

    // Build operation JSON from request params
    let operation = serde_json::json!({
        "source_image_url": source_image_url,
        "prompt": request.prompt,
        "negative_prompt": request.negative_prompt,
        "duration_secs": request.duration_secs,
        "seed": request.seed,
    });

    let job = state
        .job_service
        .create_job(&request.project_id, "video_generate", operation)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job.id.into_uuid().to_string())
}

/// Reads a file and converts it to a base64 data URI.
fn read_file_as_data_uri(file_path: &str) -> Result<String, std::io::Error> {
    let mut file = std::fs::File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Detect MIME type from file extension
    let extension = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();

    let mime_type = match extension.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        _ => "image/png",
    };

    let base64_data = base64::engine::general_purpose::STANDARD.encode(&buffer);
    Ok(format!("data:{};base64,{}", mime_type, base64_data))
}
