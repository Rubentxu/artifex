//! Data transfer objects for IPC communication.

use serde::{Deserialize, Serialize};

/// Response type for project data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    pub path: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Request type for creating a project.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub path: String,
}

/// Response type for job data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResponse {
    pub id: String,
    pub project_id: String,
    pub job_type: String,
    pub status: String,
    pub operation: serde_json::Value,
    pub progress_percent: u8,
    pub progress_message: Option<String>,
    pub error_message: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Request type for creating a job.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateJobRequest {
    pub project_id: String,
    pub job_type: String,
    pub operation: serde_json::Value,
}

/// Response type for asset data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetResponse {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub kind: String,
    pub file_path: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub file_size: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_secs: Option<f32>,
    pub sample_rate: Option<u32>,
    pub created_at: String,
}

/// Request type for importing an asset file.
#[derive(Debug, Clone, Deserialize)]
pub struct ImportAssetRequest {
    pub project_id: String,
    pub source_path: String,
    pub name: String,
    pub kind: String,
}

/// Request type for registering an existing asset.
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterAssetRequest {
    pub project_id: String,
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub metadata: Option<serde_json::Value>,
}

/// Request type for generating an image.
#[derive(Debug, Clone, Deserialize)]
pub struct GenerateImageRequest {
    pub project_id: String,
    pub prompt: String,
    #[serde(default)]
    pub negative_prompt: Option<String>,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default = "default_steps")]
    pub steps: u32,
    pub seed: Option<u64>,
    // Note: model_id is no longer sent by the frontend - it's resolved
    // by the ModelRouter at job execution time based on routing rules.
}

fn default_width() -> u32 {
    512
}

fn default_height() -> u32 {
    512
}

fn default_steps() -> u32 {
    20
}

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

/// Request type for removing background from an image.
#[derive(Debug, Clone, Deserialize)]
pub struct RemoveBackgroundRequest {
    pub project_id: String,
    pub asset_id: String,
    #[serde(default)]
    pub provider_mode: Option<String>,
}

/// Request type for generating a tile.
#[derive(Debug, Clone, Deserialize)]
pub struct GenerateTileRequest {
    pub project_id: String,
    pub prompt: String,
    #[serde(default = "default_tile_size")]
    pub width: u32,
    #[serde(default = "default_tile_size")]
    pub height: u32,
    #[serde(default)]
    pub biome: Option<String>,
    #[serde(default = "default_seamless")]
    pub seamless: bool,
}

fn default_tile_size() -> u32 {
    256
}

fn default_seamless() -> bool {
    true
}

/// Palette mode for pixel art conversion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "colors")]
pub enum PaletteMode {
    /// Pico-8 palette (16 colors)
    Pico8,
    /// GameBoy palette (4 colors)
    GameBoy,
    /// NES palette (54 colors)
    Nes,
    /// Custom palette with specified colors
    Custom(Vec<[u8; 3]>),
}

/// Dithering mode for pixel art conversion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DitheringMode {
    #[default]
    None,
    FloydSteinberg,
    Bayer,
    Atkinson,
}

/// Request type for converting an image to pixel art.
#[derive(Debug, Clone, Deserialize)]
pub struct ConvertPixelArtRequest {
    pub project_id: String,
    pub asset_id: String,
    #[serde(default = "default_pixel_art_size")]
    pub target_width: u32,
    #[serde(default = "default_pixel_art_size")]
    pub target_height: u32,
    pub palette: PaletteMode,
    #[serde(default)]
    pub dithering: DitheringMode,
    #[serde(default)]
    pub outline: bool,
    #[serde(default = "default_outline_threshold")]
    pub outline_threshold: u8,
}

fn default_pixel_art_size() -> u32 {
    64
}

fn default_outline_threshold() -> u8 {
    128
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_response_serialization_snake_case() {
        let response = ProjectResponse {
            id: "test-id".to_string(),
            name: "TestProject".to_string(),
            path: "/tmp/test".to_string(),
            status: "active".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        // Verify snake_case field names in JSON output
        assert!(json.contains("\"created_at\""));
        assert!(json.contains("\"updated_at\""));
        assert!(!json.contains("\"createdAt\""));
        assert!(!json.contains("\"updatedAt\""));
    }
}
