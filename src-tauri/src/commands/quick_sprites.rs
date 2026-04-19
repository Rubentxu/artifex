//! Quick sprites generation IPC commands.

use tauri::State;

use crate::dto::{QuickSpritesMode, QuickSpritesRequest};
use crate::state::AppState;

/// Generates sprites from an image or prompt using the quick sprites pipeline.
/// Creates a job with job_type "quick_sprites" and returns the job ID.
#[tauri::command]
pub async fn generate_quick_sprites(
    state: State<'_, AppState>,
    request: QuickSpritesRequest,
) -> Result<String, String> {
    // Check tier - Pro required for quick sprites
    let tier = state
        .identity_service
        .get_tier()
        .await
        .map_err(|e| e.to_string())?;
    if !tier.is_pro() {
        return Err("Pro tier required for quick sprites generation".to_string());
    }

    // Validate based on mode and extract source file path
    let source_file_path = match request.mode {
        QuickSpritesMode::FromImage => {
            // Validate that source asset exists and is an image/sprite
            let asset_id = request.source_image_asset_id.as_ref()
                .ok_or_else(|| "FromImage mode requires source_image_asset_id".to_string())?;

            let source_asset = state
                .asset_service
                .get_asset(asset_id)
                .await
                .map_err(|e| e.to_string())?;

            let valid_kinds = [
                artifex_asset_management::AssetKind::Image,
                artifex_asset_management::AssetKind::Sprite,
            ];
            if !valid_kinds.contains(&source_asset.kind) {
                return Err("Source asset must be an image or sprite".to_string());
            }

            source_asset.file_path.clone()
        }
        QuickSpritesMode::FromPrompt => {
            // FromPrompt mode requires image_gen_params
            if request.image_gen_params.is_none() {
                return Err("FromPrompt mode requires image_gen_params".to_string());
            }
            None
        }
    };

    // Build operation JSON from request
    let operation = serde_json::json!({
        "mode": request.mode,
        "source_asset_id": request.source_image_asset_id,
        "source_file_path": source_file_path,
        "motion_prompt": request.motion_prompt,
        "negative_prompt": request.negative_prompt,
        "image_gen_params": request.image_gen_params,
        "video_duration_secs": request.options.as_ref().and_then(|o| o.video_duration_secs),
        "video_seed": request.options.as_ref().and_then(|o| o.video_seed),
        "fps": request.options.as_ref().map(|o| o.fps).unwrap_or(10),
        "dedup_threshold": request.options.as_ref().map(|o| o.dedup_threshold).unwrap_or(0.03),
        "atlas_max_size": request.options.as_ref().map(|o| o.atlas_max_size).unwrap_or(4096),
        "padding": request.options.as_ref().map(|o| o.padding).unwrap_or(1),
        "animation_name": request.options.as_ref().map(|o| o.animation_name.clone()).unwrap_or_else(|| "idle".to_string()),
        "output_format": request.options.as_ref().map(|o| o.output_format.clone()).unwrap_or(crate::dto::QuickSpritesOutputFormat::Both),
        "project_id": request.project_id,
    });

    let job = state
        .job_service
        .create_job(&request.project_id, "quick_sprites", operation)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job.id.into_uuid().to_string())
}