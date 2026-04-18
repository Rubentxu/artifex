//! Atlas pack worker internal types.
//!
//! These types are worker-internal pipeline payload/manifest types
//! and should not be exposed through the DTO layer.

use serde::{Deserialize, Serialize};

use crate::dto::atlas::PackAtlasOptions;

/// Operation payload for atlas pack worker.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackAtlasOperation {
    pub project_id: String,
    pub atlas_name: String,
    pub source_assets: Vec<PackAtlasSourceAsset>,
    pub options: PackAtlasOptions,
}

/// Source asset info embedded in the pack operation.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackAtlasSourceAsset {
    pub asset_id: String,
    pub name: String,
    pub file_path: String,
}

/// Region within an atlas manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasRegion {
    pub asset_id: String,
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub source_width: u32,
    pub source_height: u32,
    pub rotated: bool,
}

/// Atlas manifest JSON structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasManifest {
    pub version: u32,
    pub atlas_name: String,
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub regions: Vec<AtlasRegion>,
}
