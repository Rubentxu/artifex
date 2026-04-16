//! Asset management IPC commands (thin adapters).

use tauri::State;

use artifex_asset_management::Asset;

use crate::dto::{AssetResponse, ImportAssetRequest, RegisterAssetRequest};
use crate::state::AppState;

/// Converts a domain Asset to an AssetResponse DTO.
pub fn asset_to_response(asset: &Asset) -> AssetResponse {
    // Extract audio metadata from metadata JSON if present
    let duration_secs = asset
        .metadata
        .as_ref()
        .and_then(|m| m.get("duration_secs"))
        .and_then(|v| v.as_f64())
        .map(|f| f as f32);

    let sample_rate = asset
        .metadata
        .as_ref()
        .and_then(|m| m.get("sample_rate"))
        .and_then(|v| v.as_u64())
        .map(|u| u as u32);

    AssetResponse {
        id: asset.id.into_uuid().to_string(),
        project_id: asset.project_id.into_uuid().to_string(),
        name: asset.name.clone(),
        kind: asset.kind.as_str().to_string(),
        file_path: asset.file_path.clone(),
        metadata: asset.metadata.clone(),
        file_size: asset.file_size,
        width: asset.width,
        height: asset.height,
        duration_secs,
        sample_rate,
        created_at: asset.created_at.to_string(),
    }
}

/// Lists all assets for a project.
#[tauri::command]
pub async fn list_assets(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<AssetResponse>, String> {
    let assets = state
        .asset_service
        .list_assets(&project_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(assets.iter().map(asset_to_response).collect())
}

/// Gets a single asset by ID.
#[tauri::command]
pub async fn get_asset(state: State<'_, AppState>, id: String) -> Result<AssetResponse, String> {
    let asset = state
        .asset_service
        .get_asset(&id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(asset_to_response(&asset))
}

/// Deletes an asset by ID.
#[tauri::command]
pub async fn delete_asset(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .asset_service
        .delete_asset(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Imports a file as an asset.
#[tauri::command]
pub async fn import_asset(
    state: State<'_, AppState>,
    request: ImportAssetRequest,
) -> Result<AssetResponse, String> {
    let asset = state
        .asset_service
        .import_file(&request.project_id, &request.source_path, &request.name, &request.kind)
        .await
        .map_err(|e| e.to_string())?;

    Ok(asset_to_response(&asset))
}

/// Registers an existing file as an asset.
#[tauri::command]
pub async fn register_asset(
    state: State<'_, AppState>,
    request: RegisterAssetRequest,
) -> Result<AssetResponse, String> {
    let asset = state
        .asset_service
        .register_asset(
            &request.project_id,
            &request.name,
            &request.kind,
            &request.file_path,
            request.metadata,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(asset_to_response(&asset))
}
