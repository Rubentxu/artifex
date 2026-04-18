//! Tile and texture generation DTOs.

use serde::Deserialize;

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

/// Generation mode for seamless texture.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeamlessMode {
    /// Generate from prompt, then apply mirror-padding.
    FromPrompt,
    /// Process existing asset with mirror-padding.
    FromAsset,
}

/// Request type for seamless texture generation.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeamlessTextureRequest {
    pub project_id: String,
    /// Generation mode.
    pub mode: SeamlessMode,
    /// Prompt for generation (required for FromPrompt mode).
    #[serde(default)]
    pub prompt: Option<String>,
    /// Optional negative prompt.
    #[serde(default)]
    pub negative_prompt: Option<String>,
    /// Image width (for FromPrompt mode). Default 512.
    #[serde(default)]
    pub width: Option<u32>,
    /// Image height (for FromPrompt mode). Default 512.
    #[serde(default)]
    pub height: Option<u32>,
    /// Source asset ID (required for FromAsset mode).
    #[serde(default)]
    pub asset_id: Option<String>,
    /// Secondary asset ID for blending (optional).
    #[serde(default)]
    pub secondary_asset_id: Option<String>,
    /// Seam threshold (0.0-1.0). Default 0.05.
    #[serde(default)]
    pub seam_threshold: Option<f32>,
    /// Padding pixels for mirror-padding. Default 16.
    #[serde(default)]
    pub padding_pixels: Option<u32>,
    /// Blend fraction for overlap zones (0.0-1.0). Default 0.5.
    #[serde(default)]
    pub blend_fraction: Option<f32>,
}
