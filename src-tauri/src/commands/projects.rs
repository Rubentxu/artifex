//! Project management IPC commands (thin adapters).

use tauri::State;

use artifex_asset_management::Project;

use crate::dto::{CreateProjectRequest, ProjectResponse};
use crate::state::AppState;

/// Opens a project, setting it as the current active project.
#[tauri::command]
pub async fn open_project(
    state: State<'_, AppState>,
    id: String,
) -> Result<ProjectResponse, String> {
    let project = state
        .service
        .open_project(&id)
        .await
        .map_err(|e| e.to_string())?;

    *state.current_project_id.lock().expect("current_project_id lock poisoned") = Some(id);

    Ok(project_to_response(project))
}

/// Lists all active projects.
#[tauri::command]
pub async fn list_projects(
    state: State<'_, AppState>,
) -> Result<Vec<ProjectResponse>, String> {
    let projects = state
        .service
        .list_projects()
        .await
        .map_err(|e| e.to_string())?;

    Ok(projects.into_iter().map(project_to_response).collect())
}

/// Creates a new project.
#[tauri::command]
pub async fn create_project(
    state: State<'_, AppState>,
    request: CreateProjectRequest,
) -> Result<ProjectResponse, String> {
    let project = state
        .service
        .create_project(&request.name, &request.path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(project_to_response(project))
}

/// Gets a single project by ID.
#[tauri::command]
pub async fn get_project(
    state: State<'_, AppState>,
    id: String,
) -> Result<ProjectResponse, String> {
    let project = state
        .service
        .get_project(&id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(project_to_response(project))
}

/// Renames a project.
#[tauri::command]
pub async fn rename_project(
    state: State<'_, AppState>,
    id: String,
    new_name: String,
) -> Result<ProjectResponse, String> {
    let project = state
        .service
        .rename_project(&id, &new_name)
        .await
        .map_err(|e| e.to_string())?;

    Ok(project_to_response(project))
}

/// Archives a project.
#[tauri::command]
pub async fn archive_project(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    state
        .service
        .archive_project(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Deletes a project (hard delete — not implemented).
#[tauri::command]
pub async fn delete_project(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    // Verify project exists first
    state
        .service
        .get_project(&id)
        .await
        .map_err(|e| e.to_string())?;

    // Hard delete not implemented in Phase 0
    Err("Hard delete is not implemented. Use archive_project instead.".to_string())
}

/// Converts a domain Project to a ProjectResponse DTO.
pub fn project_to_response(project: Project) -> ProjectResponse {
    ProjectResponse {
        id: project.id.into_uuid().to_string(),
        name: project.name.to_string(),
        path: project.path.to_string(),
        status: match project.status {
            artifex_asset_management::ProjectStatus::Active => "active".to_string(),
            artifex_asset_management::ProjectStatus::Archived => "archived".to_string(),
        },
        created_at: project.created_at.to_string(),
        updated_at: project.updated_at.to_string(),
    }
}
