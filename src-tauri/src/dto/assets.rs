//! Asset DTOs.

use serde::{Deserialize, Serialize};

/// Response type for asset data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetResponse {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub kind: String,
    pub file_path: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub file_size: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_secs: Option<f32>,
    pub sample_rate: Option<u32>,
    pub created_at: String,
    /// Tags for organization.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Source of import (e.g., "uploaded", "generated").
    pub import_source: String,
    /// Collection this asset belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<String>,
    /// Parent asset ID if this asset was derived from another.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub derived_from: Option<String>,
}

/// Request type for importing an asset file.
#[derive(Debug, Clone, Deserialize)]
pub struct ImportAssetRequest {
    pub project_id: String,
    pub source_path: String,
    pub name: String,
    pub kind: String,
}

/// Request type for registering an existing asset.
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterAssetRequest {
    pub project_id: String,
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub metadata: Option<serde_json::Value>,
}

/// Request type for tagging an asset.
#[derive(Debug, Clone, Deserialize)]
pub struct TagAssetRequest {
    pub asset_id: String,
    pub tag: String,
}

/// Request type for untagging an asset.
#[derive(Debug, Clone, Deserialize)]
pub struct UntagAssetRequest {
    pub asset_id: String,
    pub tag: String,
}

/// Request type for creating a collection.
#[derive(Debug, Clone, Deserialize)]
pub struct CollectionCreateRequest {
    pub project_id: String,
    pub name: String,
}

/// Response type for collection data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionResponse {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub created_at: String,
}

/// Request type for adding an asset to a collection.
#[derive(Debug, Clone, Deserialize)]
pub struct AddToCollectionRequest {
    pub asset_id: String,
    pub collection_id: String,
}

/// Response type for asset lineage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetLineageResponse {
    pub chain: Vec<AssetResponse>,
}
