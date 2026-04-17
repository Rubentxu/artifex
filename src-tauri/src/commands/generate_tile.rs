//! Tile generation IPC commands.

use tauri::State;

use crate::dto::GenerateTileRequest;
use crate::state::AppState;

/// Generates a tile using the configured provider.
/// Creates a job with job_type "tile_generate" and returns the job ID.
#[tauri::command]
pub async fn generate_tile(
    state: State<'_, AppState>,
    request: GenerateTileRequest,
) -> Result<String, String> {
    // Build operation JSON from request params
    let operation = serde_json::json!({
        "prompt": request.prompt,
        "width": request.width,
        "height": request.height,
        "biome": request.biome,
        "seamless": request.seamless,
    });

    let job = state
        .job_service
        .create_job(&request.project_id, "tile_generate", operation)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job.id.into_uuid().to_string())
}