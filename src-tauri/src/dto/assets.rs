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
