//! Atlas packing DTOs.

use serde::{Deserialize, Serialize};

// Re-export atlas pack types from worker module for use by command handlers
pub use crate::workers::atlas_pack_types::{AtlasManifest, AtlasRegion, PackAtlasOperation, PackAtlasSourceAsset};

/// Sort mode for texture atlas packing.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AtlasSortMode {
    Area,
    MaxSide,
    Width,
    Height,
    #[default]
    None,
}

/// Options for packing a texture atlas.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackAtlasOptions {
    #[serde(default = "default_pack_atlas_max_size")]
    pub max_size: u32,
    #[serde(default = "default_pack_atlas_padding")]
    pub padding: u8,
    #[serde(default)]
    pub allow_rotation: bool,
    #[serde(default)]
    pub sort_mode: AtlasSortMode,
}

fn default_pack_atlas_max_size() -> u32 {
    2048
}

fn default_pack_atlas_padding() -> u8 {
    1
}

impl Default for PackAtlasOptions {
    fn default() -> Self {
        Self {
            max_size: default_pack_atlas_max_size(),
            padding: default_pack_atlas_padding(),
            allow_rotation: false,
            sort_mode: AtlasSortMode::None,
        }
    }
}

/// Request to pack multiple assets into a texture atlas.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackAtlasRequest {
    pub project_id: String,
    pub atlas_name: String,
    pub source_asset_ids: Vec<String>,
    #[serde(default)]
    pub options: PackAtlasOptions,
}
