//! 3D renderer DTOs.

use serde::{Deserialize, Serialize};

/// Camera angle specification.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraAngle {
    pub yaw_degrees: f32,
    pub pitch_degrees: f32,
}

/// Request type for rendering a 3D model to sprites.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Render3dRequest {
    pub project_id: String,
    pub model_file_path: String,
    /// Camera preset: "isometric" | "topdown" | "custom"
    pub camera_preset: String,
    /// Custom angles for "custom" preset.
    #[serde(default)]
    pub custom_angles: Option<Vec<CameraAngle>>,
    /// Output width in pixels. Default 256.
    #[serde(default = "default_render_output_size")]
    pub output_width: u32,
    /// Output height in pixels. Default 256.
    #[serde(default = "default_render_output_size")]
    pub output_height: u32,
    /// Optional: animation name to extract frames from (GLTF only).
    #[serde(default)]
    pub animation_name: Option<String>,
    /// FPS for animation extraction. Default 12.
    #[serde(default = "default_animation_fps")]
    pub animation_fps: u16,
}

fn default_render_output_size() -> u32 {
    256
}

fn default_animation_fps() -> u16 {
    12
}
