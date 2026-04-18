//! Material generation DTOs.

use serde::Deserialize;

/// Request type for generating PBR materials from an image.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateMaterialRequest {
    pub project_id: String,
    pub asset_id: String,
    #[serde(default)]
    pub provider_id: Option<String>,
    #[serde(default)]
    pub model_id: Option<String>,
}
