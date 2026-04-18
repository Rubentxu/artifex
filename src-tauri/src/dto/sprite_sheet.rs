//! Sprite sheet generation DTOs.

use serde::{Deserialize, Serialize};

/// Output format for sprite sheet manifests.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum OutputFormat {
    Json,
    Aseprite,
    #[default]
    Both,
}

/// Request type for generating a sprite sheet from a video asset.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateSpriteSheetRequest {
    pub asset_id: String,
    pub project_id: String,
    #[serde(default = "default_sprite_fps")]
    pub fps: u8,
    #[serde(default = "default_dedup_threshold")]
    pub dedup_threshold: f32,
    #[serde(default = "default_atlas_max_size")]
    pub atlas_max_size: u32,
    #[serde(default = "default_padding")]
    pub padding: u8,
    #[serde(default = "default_animation_name")]
    pub animation_name: String,
    #[serde(default)]
    pub output_format: OutputFormat,
}

fn default_sprite_fps() -> u8 {
    10
}

fn default_dedup_threshold() -> f32 {
    0.03
}

fn default_atlas_max_size() -> u32 {
    4096
}

fn default_padding() -> u8 {
    1
}

fn default_animation_name() -> String {
    "idle".to_string()
}

/// Sort order for auto-detect slicing.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum SortOrder {
    LeftToRight,
    #[default]
    TopToBottom,
}

/// Slice mode for sprite sheet slicing.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum SliceMode {
    #[default]
    Grid,
    AutoDetect,
}

/// Grid parameters for grid-based sprite sheet slicing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridSliceParams {
    #[serde(default = "default_rows")]
    pub rows: u32,
    #[serde(default = "default_cols")]
    pub cols: u32,
    #[serde(default)]
    pub margin: u32,
}

impl Default for GridSliceParams {
    fn default() -> Self {
        Self {
            rows: default_rows(),
            cols: default_cols(),
            margin: 0,
        }
    }
}

fn default_rows() -> u32 {
    4
}

fn default_cols() -> u32 {
    4
}

/// Auto-detect parameters for content-aware sprite sheet slicing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDetectSliceParams {
    #[serde(default = "default_min_area")]
    pub min_area: u32,
    #[serde(default)]
    pub sort_order: SortOrder,
}

impl Default for AutoDetectSliceParams {
    fn default() -> Self {
        Self {
            min_area: default_min_area(),
            sort_order: SortOrder::default(),
        }
    }
}

fn default_min_area() -> u32 {
    100
}

/// Request type for slicing a sprite sheet (grid or auto-detect).
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SliceSpriteSheetRequest {
    pub asset_id: String,
    pub project_id: String,
    #[serde(default)]
    pub mode: SliceMode,
    #[serde(default)]
    pub grid_params: GridSliceParams,
    #[serde(default)]
    pub auto_detect_params: AutoDetectSliceParams,
}
