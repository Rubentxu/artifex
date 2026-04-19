//! Code generation IPC command.

use tauri::State;

use crate::dto::{CodeAgentRequest, GenerateCodeRequest};
use crate::state::AppState;
use artifex_shared_kernel::ArtifexError;

/// Supported code engines.
const SUPPORTED_ENGINES: [&str; 2] = ["godot", "unity"];

/// Generates game code/scripts using AI.
/// Creates a job with job_type "code_generate" and returns the job ID.
#[tauri::command]
pub async fn generate_code(
    state: State<'_, AppState>,
    request: GenerateCodeRequest,
) -> Result<String, String> {
    // Check tier - Pro required for code generation
    let tier = state
        .identity_service
        .get_tier()
        .await
        .map_err(|e| e.to_string())?;
    if !tier.is_pro() {
        return Err("Pro tier required for code generation".to_string());
    }

    // Validate that the project exists
    let _project = state
        .project_service
        .get_project(&request.project_id)
        .await
        .map_err(|e: ArtifexError| e.to_string())?;

    // Validate engine is supported
    let engine_lower = request.engine.to_lowercase();
    if !SUPPORTED_ENGINES.contains(&engine_lower.as_str()) {
        return Err(format!(
            "Unsupported engine: {}. Supported engines are: {}",
            request.engine,
            SUPPORTED_ENGINES.join(", ")
        ));
    }

    // Validate prompt is not empty
    if request.prompt.trim().is_empty() {
        return Err("Prompt cannot be empty".to_string());
    }

    // Build operation JSON from request params
    let operation = serde_json::json!({
        "project_id": request.project_id,
        "engine": engine_lower,
        "prompt": request.prompt.trim(),
        "template_id": request.template_id,
        "model_id": request.model_id,
        "temperature": request.temperature,
        "max_tokens": request.max_tokens,
    });

    let job = state
        .job_service
        .create_job(&request.project_id, "code_generate", operation)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job.id.into_uuid().to_string())
}

/// Starts a multi-step code agent job.
/// Creates a job with job_type "code_agent" and returns the job ID.
#[tauri::command]
pub async fn start_code_agent(
    state: State<'_, AppState>,
    request: CodeAgentRequest,
) -> Result<String, String> {
    // Check tier - Pro required for code agent
    let tier = state
        .identity_service
        .get_tier()
        .await
        .map_err(|e| e.to_string())?;
    if !tier.is_pro() {
        return Err("Pro tier required for code agent".to_string());
    }

    // Validate that the project exists
    let _project = state
        .project_service
        .get_project(&request.project_id)
        .await
        .map_err(|e: ArtifexError| e.to_string())?;

    // Validate engine is supported
    let engine_lower = request.engine.to_lowercase();
    if !SUPPORTED_ENGINES.contains(&engine_lower.as_str()) {
        return Err(format!(
            "Unsupported engine: {}. Supported engines are: {}",
            request.engine,
            SUPPORTED_ENGINES.join(", ")
        ));
    }

    // Validate prompt is not empty
    if request.prompt.trim().is_empty() {
        return Err("Prompt cannot be empty".to_string());
    }

    // Build operation JSON from request params
    let operation = serde_json::json!({
        "project_id": request.project_id,
        "engine": engine_lower,
        "prompt": request.prompt.trim(),
        "model_id": request.model_id,
        "max_duration_secs": request.max_duration_secs,
    });

    let job = state
        .job_service
        .create_job(&request.project_id, "code_agent", operation)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job.id.into_uuid().to_string())
}
