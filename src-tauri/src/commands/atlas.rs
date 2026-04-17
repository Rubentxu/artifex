//! Atlas asset IPC commands.

use tauri::State;

use artifex_asset_management::AssetKind;

use crate::dto::PackAtlasRequest;
use crate::state::AppState;

/// Packs multiple Image/Sprite/Tileset assets into a single texture atlas.
///
/// Validates that at least 2 source assets are provided, all asset IDs exist,
/// and all assets are Image/Sprite/Tileset kind. Creates a pack_atlas job
/// and returns the job ID.
#[tauri::command]
pub async fn pack_atlas(
    state: State<'_, AppState>,
    request: PackAtlasRequest,
) -> Result<String, String> {
    // Validate at least 2 source assets
    if request.source_asset_ids.len() < 2 {
        return Err("at least 2 assets required".to_string());
    }

    // Validate atlas name is not empty
    if request.atlas_name.trim().is_empty() {
        return Err("atlas name cannot be empty".to_string());
    }

    // Collect source assets with file paths
    let mut source_assets = Vec::new();
    for asset_id in &request.source_asset_ids {
        let asset = state
            .asset_service
            .get_asset(asset_id)
            .await
            .map_err(|e| format!("asset {}: {}", asset_id, e))?;

        // Accept Image, Sprite, or Tileset kinds
        match asset.kind {
            AssetKind::Image | AssetKind::Sprite | AssetKind::Tileset => {}
            _ => {
                return Err(format!(
                    "asset {} is {:?}, must be Image/Sprite/Tileset",
                    asset_id, asset.kind
                ));
            }
        }

        // Get file_path - it's stored as metadata or we need to construct it
        let file_path = asset
            .file_path
            .ok_or_else(|| format!("asset {} has no file_path", asset_id))?;

        source_assets.push(serde_json::json!({
            "asset_id": asset.id.into_uuid().to_string(),
            "name": asset.name,
            "file_path": file_path,
        }));
    }

    // Build operation JSON with all data the worker needs
    let operation = serde_json::json!({
        "project_id": request.project_id,
        "atlas_name": request.atlas_name.trim(),
        "source_assets": source_assets,
        "options": request.options,
    });

    // Create pack_atlas job
    let job = state
        .job_service
        .create_job(&request.project_id, "pack_atlas", operation)
        .await
        .map_err(|e| format!("Failed to create pack atlas job: {}", e))?;

    Ok(job.id.into_uuid().to_string())
}