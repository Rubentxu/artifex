//! Image generation DTOs.

use serde::Deserialize;

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
