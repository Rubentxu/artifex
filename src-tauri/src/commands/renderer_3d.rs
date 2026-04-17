//! 3D Renderer IPC commands.

use std::path::Path;

use tauri::State;

use crate::dto::Render3dRequest;
use crate::state::AppState;

/// Renders a 3D model (GLTF/GLB/OBJ) to sprite sheets from configurable camera angles.
///
/// Validates that the model file exists and the camera preset is valid.
/// Creates a render_3d job and returns the job ID.
#[tauri::command]
pub async fn render_3d_to_sprites(
    state: State<'_, AppState>,
    request: Render3dRequest,
) -> Result<String, String> {
    // Validate model file exists
    if !Path::new(&request.model_file_path).exists() {
        return Err(format!("Model file not found: {}", request.model_file_path));
    }

    // Validate camera preset
    match request.camera_preset.as_str() {
        "isometric" | "topdown" | "custom" => {}
        _ => {
            return Err(format!(
                "Invalid camera preset: {} (valid: isometric, topdown, custom)",
                request.camera_preset
            ));
        }
    }

    // For custom preset, validate that angles are provided
    if request.camera_preset == "custom" && request.custom_angles.as_ref().map_or(true, |a| a.is_empty()) {
        return Err("Custom camera preset requires at least one angle".to_string());
    }

    // Build operation JSON
    let operation = serde_json::json!({
        "project_id": request.project_id,
        "model_file_path": request.model_file_path,
        "camera_preset": request.camera_preset,
        "custom_angles": request.custom_angles,
        "output_width": request.output_width,
        "output_height": request.output_height,
        "animation_name": request.animation_name,
        "animation_fps": request.animation_fps,
    });

    // Create render_3d job
    let job = state
        .job_service
        .create_job(&request.project_id, "render_3d", operation)
        .await
        .map_err(|e| format!("Failed to create render_3d job: {}", e))?;

    Ok(job.id.into_uuid().to_string())
}
