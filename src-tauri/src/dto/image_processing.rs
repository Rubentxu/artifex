//! Image processing DTOs.

use serde::{Deserialize, Serialize};

/// Request type for removing background from an image.
#[derive(Debug, Clone, Deserialize)]
pub struct RemoveBackgroundRequest {
    pub project_id: String,
    pub asset_id: String,
    #[serde(default)]
    pub provider_mode: Option<String>,
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

/// Request type for inpainting an image.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InpaintRequest {
    pub project_id: String,
    pub asset_id: String,
    /// Path to the mask file (written by frontend).
    pub mask_path: String,
    /// Text prompt describing the desired edit.
    pub prompt: String,
    /// Optional negative prompt.
    #[serde(default)]
    pub negative_prompt: Option<String>,
    /// How strongly to follow the mask (0.0-1.0).
    #[serde(default = "default_inpaint_strength")]
    pub strength: f32,
    /// Guidance scale (1.0-20.0).
    #[serde(default = "default_inpaint_guidance")]
    pub guidance_scale: f32,
    /// Inference steps.
    #[serde(default = "default_inpaint_steps")]
    pub steps: u32,
    /// Optional provider mode override.
    #[serde(default)]
    pub provider_mode: Option<String>,
}

fn default_inpaint_strength() -> f32 {
    0.8
}

fn default_inpaint_guidance() -> f32 {
    7.5
}

fn default_inpaint_steps() -> u32 {
    30
}

/// Direction for outpainting.
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutpaintDirection {
    #[default]
    Right,
    Left,
    Top,
    Bottom,
}

/// Request type for outpainting an image.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutpaintRequest {
    pub project_id: String,
    pub asset_id: String,
    /// Direction to extend the canvas.
    pub direction: OutpaintDirection,
    /// Number of pixels to extend.
    #[serde(default = "default_outpaint_extension")]
    pub extend_pixels: u32,
    /// Text prompt describing the desired fill.
    pub prompt: String,
    /// Optional negative prompt.
    #[serde(default)]
    pub negative_prompt: Option<String>,
    /// How strongly to follow the prompt (0.0-1.0).
    #[serde(default = "default_inpaint_strength")]
    pub strength: f32,
    /// Guidance scale (1.0-20.0).
    #[serde(default = "default_inpaint_guidance")]
    pub guidance_scale: f32,
    /// Inference steps.
    #[serde(default = "default_inpaint_steps")]
    pub steps: u32,
    /// Optional provider mode override.
    #[serde(default)]
    pub provider_mode: Option<String>,
}

fn default_outpaint_extension() -> u32 {
    256
}
