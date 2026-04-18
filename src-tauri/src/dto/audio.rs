//! Audio generation DTOs.

use serde::Deserialize;

/// Request type for generating audio (SFX or Music).
#[derive(Debug, Clone, Deserialize)]
pub struct GenerateAudioRequest {
    pub project_id: String,
    pub params: AudioGenParamsDto,
}

/// DTO for audio generation parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct AudioGenParamsDto {
    pub prompt: String,
    /// Kind of audio: "sfx" or "music".
    #[serde(default)]
    pub kind: Option<String>,
    /// Duration in seconds.
    #[serde(default)]
    pub duration_secs: Option<f32>,
    /// Model ID to use.
    #[serde(default)]
    pub model_id: Option<String>,
    /// Seed for reproducibility.
    #[serde(default)]
    pub seed: Option<u64>,
    /// Output format: "mp3", "wav", etc.
    #[serde(default)]
    pub output_format: Option<String>,
}

/// Request type for synthesizing speech (TTS).
#[derive(Debug, Clone, Deserialize)]
pub struct GenerateTtsRequest {
    pub project_id: String,
    pub params: TtsParamsDto,
}

/// DTO for TTS parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct TtsParamsDto {
    pub text: String,
    /// Voice ID (provider-specific).
    #[serde(default)]
    pub voice_id: Option<String>,
    /// Language code (e.g., "en-US").
    #[serde(default = "default_dto_language")]
    pub language: Option<String>,
    /// Speech rate (0.5 to 2.0).
    #[serde(default = "default_dto_speed")]
    pub speed: Option<f32>,
    /// Model ID to use.
    #[serde(default)]
    pub model_id: Option<String>,
    /// Voice stability setting (ElevenLabs).
    #[serde(default)]
    pub stability: Option<f32>,
    /// Voice similarity boost setting (ElevenLabs).
    #[serde(default)]
    pub similarity_boost: Option<f32>,
    /// Output format: "mp3", "wav", etc.
    #[serde(default)]
    pub output_format: Option<String>,
}

fn default_dto_language() -> Option<String> {
    Some("en-US".to_string())
}

fn default_dto_speed() -> Option<f32> {
    Some(1.0)
}
