//! Quick sprites generation DTOs.

use serde::Deserialize;

/// Generation mode for quick sprites.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuickSpritesMode {
    /// Generate from existing image asset.
    FromImage,
    /// Generate from prompt (image gen first).
    FromPrompt,
}

/// Output format for quick sprites manifests.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum QuickSpritesOutputFormat {
    Json,
    Aseprite,
    #[default]
    Both,
}

/// Options for quick sprites generation.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickSpritesOptions {
    /// FPS for extracted frames. Default 10.
    #[serde(default = "default_quick_sprites_fps")]
    pub fps: u8,
    /// Dedup threshold (0.0-1.0). Default 0.03.
    #[serde(default = "default_quick_sprites_dedup_threshold")]
    pub dedup_threshold: f32,
    /// Max atlas size. Default 4096.
    #[serde(default = "default_quick_sprites_atlas_max_size")]
    pub atlas_max_size: u32,
    /// Padding between frames in atlas. Default 1.
    #[serde(default = "default_quick_sprites_padding")]
    pub padding: u8,
    /// Animation name. Default "idle".
    #[serde(default = "default_quick_sprites_animation_name")]
    pub animation_name: String,
    /// Output format. Default Both.
    #[serde(default)]
    pub output_format: QuickSpritesOutputFormat,
    /// Video duration in seconds (2-8). Default 4.
    #[serde(default)]
    pub video_duration_secs: Option<u8>,
    /// Video generation seed.
    #[serde(default)]
    pub video_seed: Option<u64>,
}

fn default_quick_sprites_fps() -> u8 {
    10
}

fn default_quick_sprites_dedup_threshold() -> f32 {
    0.03
}

fn default_quick_sprites_atlas_max_size() -> u32 {
    4096
}

fn default_quick_sprites_padding() -> u8 {
    1
}

fn default_quick_sprites_animation_name() -> String {
    "idle".to_string()
}

/// Request type for quick sprites generation.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickSpritesRequest {
    pub project_id: String,
    /// Generation mode.
    pub mode: QuickSpritesMode,
    /// Source image asset ID (for FromImage mode).
    pub source_image_asset_id: Option<String>,
    /// Source file path (resolved by command from asset).
    pub source_file_path: Option<String>,
    /// Motion prompt for video generation.
    pub motion_prompt: String,
    /// Optional negative prompt.
    #[serde(default)]
    pub negative_prompt: Option<String>,
    /// Image generation params (for FromPrompt mode).
    #[serde(default)]
    pub image_gen_params: Option<artifex_model_config::image_provider::ImageGenParams>,
    /// Generation options.
    #[serde(default)]
    pub options: Option<QuickSpritesOptions>,
}
