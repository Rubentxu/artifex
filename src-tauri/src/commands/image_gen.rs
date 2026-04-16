//! Image generation IPC commands.

use tauri::State;

use crate::dto::GenerateImageRequest;
use crate::state::AppState;

/// Generates an image using the configured provider.
/// Creates a job with job_type "image_generate" and returns the job ID.
#[tauri::command]
pub async fn generate_image(
    state: State<'_, AppState>,
    request: GenerateImageRequest,
) -> Result<String, String> {
    // Build operation JSON from request params
    let operation = serde_json::json!({
        "prompt": request.prompt,
        "negative_prompt": request.negative_prompt,
        "width": request.width,
        "height": request.height,
        "steps": request.steps,
        "seed": request.seed,
    });

    let job = state
        .job_service
        .create_job(&request.project_id, "image_generate", operation)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job.id.into_uuid().to_string())
}