//! Publishing / Export DTOs.

use serde::{Deserialize, Serialize};

/// Request type for exporting a project as ZIP.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportProjectRequest {
    pub project_id: String,
    #[serde(default = "default_true")]
    pub include_html_gallery: bool,
    pub output_path: Option<String>,
}

fn default_true() -> bool {
    true
}

/// Response type for export result.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportProjectResponse {
    pub output_path: String,
    pub file_size_bytes: u64,
    pub asset_count: usize,
    pub manifest_path: String,
}
