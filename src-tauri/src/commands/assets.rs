//! Asset management IPC commands (thin adapters).

use tauri::State;

use artifex_asset_management::Asset;

use crate::dto::{
    AddToCollectionRequest, AssetLineageResponse, AssetResponse, CollectionCreateRequest,
    CollectionResponse, ImportAssetRequest, RegisterAssetRequest, TagAssetRequest,
    UntagAssetRequest,
};
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
        tags: asset.tags.clone(),
        import_source: asset.import_source.clone(),
        collection_id: asset.collection_id.clone(),
        derived_from: asset.derived_from_asset_id.clone(),
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

/// Tags an asset with a new tag.
#[tauri::command]
pub async fn tag_asset(
    state: State<'_, AppState>,
    request: TagAssetRequest,
) -> Result<AssetResponse, String> {
    let asset = state
        .asset_service
        .tag_asset(&request.asset_id, &request.tag)
        .await
        .map_err(|e| e.to_string())?;

    Ok(asset_to_response(&asset))
}

/// Removes a tag from an asset.
#[tauri::command]
pub async fn untag_asset(
    state: State<'_, AppState>,
    request: UntagAssetRequest,
) -> Result<AssetResponse, String> {
    let asset = state
        .asset_service
        .untag_asset(&request.asset_id, &request.tag)
        .await
        .map_err(|e| e.to_string())?;

    Ok(asset_to_response(&asset))
}

/// Creates a new collection.
#[tauri::command]
pub async fn create_collection(
    state: State<'_, AppState>,
    request: CollectionCreateRequest,
) -> Result<CollectionResponse, String> {
    let collection = state
        .asset_service
        .create_collection(&request.project_id, &request.name)
        .await
        .map_err(|e| e.to_string())?;

    Ok(CollectionResponse {
        id: collection.id,
        project_id: collection.project_id.into_uuid().to_string(),
        name: collection.name,
        created_at: collection.created_at.to_string(),
    })
}

/// Lists all collections for a project.
#[tauri::command]
pub async fn list_collections(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<CollectionResponse>, String> {
    let collections = state
        .asset_service
        .list_collections(&project_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(collections
        .iter()
        .map(|c| CollectionResponse {
            id: c.id.clone(),
            project_id: c.project_id.into_uuid().to_string(),
            name: c.name.clone(),
            created_at: c.created_at.to_string(),
        })
        .collect())
}

/// Deletes a collection.
#[tauri::command]
pub async fn delete_collection(
    state: State<'_, AppState>,
    collection_id: String,
) -> Result<(), String> {
    state
        .asset_service
        .delete_collection(&collection_id)
        .await
        .map_err(|e| e.to_string())
}

/// Adds an asset to a collection.
#[tauri::command]
pub async fn add_to_collection(
    state: State<'_, AppState>,
    request: AddToCollectionRequest,
) -> Result<AssetResponse, String> {
    let asset = state
        .asset_service
        .add_to_collection(&request.asset_id, &request.collection_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(asset_to_response(&asset))
}

/// Removes an asset from its collection.
#[tauri::command]
pub async fn remove_from_collection(
    state: State<'_, AppState>,
    asset_id: String,
) -> Result<AssetResponse, String> {
    let asset = state
        .asset_service
        .remove_from_collection(&asset_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(asset_to_response(&asset))
}

/// Gets the lineage chain for an asset.
#[tauri::command]
pub async fn get_asset_lineage(
    state: State<'_, AppState>,
    asset_id: String,
) -> Result<AssetLineageResponse, String> {
    let chain = state
        .asset_service
        .get_asset_lineage(&asset_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(AssetLineageResponse {
        chain: chain.iter().map(asset_to_response).collect(),
    })
}
