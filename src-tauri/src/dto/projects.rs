//! Project DTOs.

use serde::{Deserialize, Serialize};

/// Response type for project data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    pub path: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Request type for creating a project.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub path: String,
}
