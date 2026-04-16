//! Audio generation and TTS synthesis IPC commands.

use tauri::State;

use crate::dto::{AudioGenParamsDto, GenerateAudioRequest, GenerateTtsRequest, TtsParamsDto};
use crate::state::AppState;

/// Generates audio (SFX or Music) using the configured provider.
/// Creates a job with job_type "audio_generate" and returns the job ID.
#[tauri::command]
pub async fn generate_audio(
    state: State<'_, AppState>,
    request: GenerateAudioRequest,
) -> Result<String, String> {
    // Build operation JSON from request params
    let operation = serde_json::json!({
        "prompt": request.params.prompt,
        "kind": request.params.kind,
        "duration_secs": request.params.duration_secs,
        "model_id": request.params.model_id,
        "seed": request.params.seed,
        "output_format": request.params.output_format,
    });

    let job = state
        .job_service
        .create_job(&request.project_id, "audio_generate", operation)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job.id.into_uuid().to_string())
}

/// Synthesizes speech (TTS) using the configured provider.
/// Creates a job with job_type "tts_synthesize" and returns the job ID.
#[tauri::command]
pub async fn synthesize_speech(
    state: State<'_, AppState>,
    request: GenerateTtsRequest,
) -> Result<String, String> {
    // Build operation JSON from request params
    let operation = serde_json::json!({
        "text": request.params.text,
        "voice_id": request.params.voice_id,
        "language": request.params.language,
        "speed": request.params.speed,
        "model_id": request.params.model_id,
        "stability": request.params.stability,
        "similarity_boost": request.params.similarity_boost,
        "output_format": request.params.output_format,
    });

    let job = state
        .job_service
        .create_job(&request.project_id, "tts_synthesize", operation)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job.id.into_uuid().to_string())
}
