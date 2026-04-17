//! Animation asset IPC commands.

use tauri::State;

use artifex_asset_management::{Asset, AssetKind};
use artifex_asset_management::asset::AnimationMetadata;
use artifex_shared_kernel::ProjectId;

use crate::dto::{CreateAnimationRequest, UpdateAnimationRequest, ExportAnimationRequest};
use crate::state::AppState;

/// Creates a new animation asset from existing frame assets.
///
/// Validates that all frame_asset_ids exist and are image/sprite assets
/// and computes uniform frame durations based on the provided default_fps.
#[tauri::command]
pub async fn create_animation(
    state: State<'_, AppState>,
    request: CreateAnimationRequest,
) -> Result<String, String> {
    // Validate name is not empty
    if request.name.trim().is_empty() {
        return Err("Animation name cannot be empty".to_string());
    }

    // Validate frame_asset_ids is not empty
    if request.frame_asset_ids.is_empty() {
        return Err("EmptyFrameList: Animation must have at least one frame".to_string());
    }

    // Validate all frame assets exist and are image/sprite assets
    for frame_id in &request.frame_asset_ids {
        let asset = state
            .asset_service
            .get_asset(frame_id)
            .await
            .map_err(|e| format!("InvalidAssetId({}): {}", frame_id, e))?;

        // Accept Image or Sprite kinds as valid frame sources
        match asset.kind {
            AssetKind::Image | AssetKind::Sprite => {}
            _ => {
                return Err(format!(
                    "InvalidAssetId({}): Frame asset must be Image or Sprite, got {:?}",
                    frame_id, asset.kind
                ));
            }
        }
    }

    // Create animation metadata with uniform frame durations
    let metadata = AnimationMetadata::with_uniform_fps(
        request.name.clone(),
        request.frame_asset_ids.clone(),
        request.default_fps,
        true, // Default to looping
    )
    .map_err(|e| format!("Validation error: {}", e))?;

    // Create the animation asset
    let project_id = request.project_id;
    let mut asset = Asset::register(
        ProjectId::from_uuid(uuid::Uuid::parse_str(&project_id).map_err(|e| format!("Invalid project id: {}", e))?),
        request.name.clone(),
        AssetKind::Animation,
    )
    .map_err(|e| format!("Asset registration failed: {}", e))?;

    // Store animation metadata as JSON
    asset.metadata = Some(serde_json::to_value(&metadata).map_err(|e| format!("Failed to serialize metadata: {}", e))?);

    // Create the asset via the service
    let asset_id = state
        .asset_service
        .create_asset(asset)
        .await
        .map_err(|e| format!("Failed to create animation asset: {}", e))?;

    Ok(asset_id.into_uuid().to_string())
}

/// Updates an existing animation asset.
#[tauri::command]
pub async fn update_animation(
    state: State<'_, AppState>,
    request: UpdateAnimationRequest,
) -> Result<String, String> {
    // Get existing animation
    let mut asset = state
        .asset_service
        .get_asset(&request.id)
        .await
        .map_err(|e| format!("Animation not found: {}", e))?;

    // Validate it's an animation asset
    if asset.kind != AssetKind::Animation {
        return Err(format!("Asset {} is not an animation", request.id));
    }

    // Parse existing metadata
    let metadata_ref = asset.metadata.as_ref()
        .ok_or_else(|| "Animation asset has no metadata".to_string())?;
    let existing_meta: AnimationMetadata = serde_json::from_value(metadata_ref.clone())
        .map_err(|e| format!("Failed to parse animation metadata: {}", e))?;

    // Apply updates
    let name = request.name.unwrap_or(existing_meta.name);
    let loop_animation = request.loop_animation.unwrap_or(existing_meta.loop_animation);

    // If frame_asset_ids is provided, validate all frames exist
    let frame_asset_ids = if let Some(ref ids) = request.frame_asset_ids {
        if ids.is_empty() {
            return Err("EmptyFrameList: Animation must have at least one frame".to_string());
        }

        for frame_id in ids {
            let frame_asset = state
                .asset_service
                .get_asset(frame_id)
                .await
                .map_err(|e| format!("InvalidAssetId({}): {}", frame_id, e))?;

            match frame_asset.kind {
                AssetKind::Image | AssetKind::Sprite => {}
                _ => {
                    return Err(format!(
                        "InvalidAssetId({}): Frame asset must be Image or Sprite",
                        frame_id
                    ));
                }
            }
        }
        ids.clone()
    } else {
        existing_meta.frame_asset_ids.clone()
    };

    // If frame_durations_ms is provided, validate it matches frame count
    let frame_durations_ms = if let Some(ref durations) = request.frame_durations_ms {
        if durations.len() != frame_asset_ids.len() {
            return Err(format!(
                "Frame durations count ({}) must match frame count ({})",
                durations.len(),
                frame_asset_ids.len()
            ));
        }
        if durations.iter().any(|&d| d == 0) {
            return Err("All frame durations must be positive".to_string());
        }
        durations.clone()
    } else {
        existing_meta.frame_durations_ms.clone()
    };

    // Recompute total_duration_ms
    let total_duration_ms: u32 = frame_durations_ms.iter().sum();

    let updated_meta = AnimationMetadata {
        name,
        frame_asset_ids,
        frame_durations_ms,
        loop_animation,
        total_duration_ms,
        default_fps: existing_meta.default_fps,
    };

    // Update asset metadata
    asset.name = updated_meta.name.clone();
    asset.metadata = Some(serde_json::to_value(&updated_meta).map_err(|e| format!("Failed to serialize metadata: {}", e))?);

    // Save the updated asset
    state
        .asset_service
        .update_asset(asset)
        .await
        .map_err(|e| format!("Failed to update animation asset: {}", e))?;

    Ok(request.id)
}

/// Gets an animation asset by ID.
#[tauri::command]
pub async fn get_animation(
    state: State<'_, AppState>,
    id: String,
) -> Result<serde_json::Value, String> {
    let asset = state
        .asset_service
        .get_asset(&id)
        .await
        .map_err(|e| format!("Animation not found: {}", e))?;

    if asset.kind != AssetKind::Animation {
        return Err(format!("Asset {} is not an animation", id));
    }

    Ok(serde_json::json!({
        "id": asset.id.into_uuid().to_string(),
        "project_id": asset.project_id.into_uuid().to_string(),
        "name": asset.name,
        "kind": "animation",
        "metadata": asset.metadata,
        "created_at": asset.created_at.to_string(),
    }))
}

/// Lists all animation assets for a project.
#[tauri::command]
pub async fn list_animations(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let assets = state
        .asset_service
        .list_assets(&project_id)
        .await
        .map_err(|e| format!("Failed to list assets: {}", e))?;

    let animations: Vec<serde_json::Value> = assets
        .into_iter()
        .filter(|a| a.kind == AssetKind::Animation)
        .map(|asset| {
            serde_json::json!({
                "id": asset.id.into_uuid().to_string(),
                "project_id": asset.project_id.into_uuid().to_string(),
                "name": asset.name,
                "kind": "animation",
                "metadata": asset.metadata,
                "created_at": asset.created_at.to_string(),
            })
        })
        .collect();

    Ok(animations)
}

/// Deletes an animation asset (does NOT delete the frame assets).
#[tauri::command]
pub async fn delete_animation(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    // Verify the asset exists and is an animation
    let asset = state
        .asset_service
        .get_asset(&id)
        .await
        .map_err(|e| format!("Animation not found: {}", e))?;

    if asset.kind != AssetKind::Animation {
        return Err(format!("Asset {} is not an animation", id));
    }

    // Delete the animation asset (frame assets remain untouched)
    state
        .asset_service
        .delete_asset(&id)
        .await
        .map_err(|e| format!("Failed to delete animation asset: {}", e))?;

    Ok(())
}

/// Exports an animation as a sprite sheet PNG + timing JSON.
///
/// Enqueues an async export job and returns the job_id immediately.
#[tauri::command]
pub async fn export_animation(
    state: State<'_, AppState>,
    request: ExportAnimationRequest,
) -> Result<String, String> {
    // Verify the animation exists
    let asset = state
        .asset_service
        .get_asset(&request.animation_id)
        .await
        .map_err(|e| format!("Animation not found: {}", e))?;

    if asset.kind != AssetKind::Animation {
        return Err(format!("Asset {} is not an animation", request.animation_id));
    }

    // Extract animation metadata to embed in the operation
    let metadata_val = asset.metadata.as_ref()
        .ok_or_else(|| "Animation has no metadata".to_string())?;
    let meta: AnimationMetadata = serde_json::from_value(metadata_val.clone())
        .map_err(|e| format!("Failed to parse animation metadata: {}", e))?;

    // Build operation JSON with all data the worker needs
    let operation = serde_json::json!({
        "animation_id": request.animation_id,
        "animation_name": asset.name,
        "project_id": request.project_id,
        "format": "spritesheet_json",
        "frame_asset_ids": meta.frame_asset_ids,
        "frame_durations_ms": meta.frame_durations_ms,
        "loop_animation": meta.loop_animation,
    });

    // Create export job
    let job = state
        .job_service
        .create_job(&request.project_id, "animation_export", operation)
        .await
        .map_err(|e| format!("Failed to create export job: {}", e))?;

    Ok(job.id.into_uuid().to_string())
}
