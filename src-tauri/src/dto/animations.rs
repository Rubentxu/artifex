//! Animation DTOs.

use serde::{Deserialize, Serialize};

/// Request type for creating an animation asset.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAnimationRequest {
    pub project_id: String,
    pub name: String,
    pub frame_asset_ids: Vec<String>,
    #[serde(default = "default_animation_fps")]
    pub default_fps: u16,
}

/// Request type for updating an animation asset.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAnimationRequest {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub frame_asset_ids: Option<Vec<String>>,
    #[serde(default)]
    pub frame_durations_ms: Option<Vec<u32>>,
    #[serde(default)]
    pub loop_animation: Option<bool>,
}

fn default_animation_fps() -> u16 {
    12
}

/// Request type for exporting an animation as a sprite sheet.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportAnimationRequest {
    pub animation_id: String,
    pub project_id: String,
    #[serde(default = "default_export_format")]
    pub format: ExportAnimationFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportAnimationFormat {
    #[default]
    SpritesheetJson,
}

fn default_export_format() -> ExportAnimationFormat {
    ExportAnimationFormat::SpritesheetJson
}
