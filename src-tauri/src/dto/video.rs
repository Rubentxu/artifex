//! Video generation DTOs.

use serde::Deserialize;

/// Request type for generating a video from an image.
#[derive(Debug, Clone, Deserialize)]
pub struct GenerateVideoRequest {
    pub project_id: String,
    /// Source image asset ID.
    pub source_image_asset_id: String,
    /// Prompt describing the desired video motion.
    pub prompt: String,
    /// Duration in seconds (2-8).
    #[serde(default = "default_video_duration")]
    pub duration_secs: u8,
    /// Optional negative prompt.
    #[serde(default)]
    pub negative_prompt: Option<String>,
    /// Optional seed for reproducibility.
    #[serde(default)]
    pub seed: Option<u64>,
}

fn default_video_duration() -> u8 {
    4
}
