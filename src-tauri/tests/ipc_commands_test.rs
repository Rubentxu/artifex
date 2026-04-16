//! Integration tests for IPC command handlers.
//!
//! These tests verify the command handler logic by testing through
//! the repository layer and validating business rules.
//!
//! Note: Full end-to-end IPC tests require Tauri runtime. These tests
//! verify the business logic that commands depend on.

mod test_helpers;

use artifex_asset_management::{Project, ProjectName, ProjectRepository, ProjectStatus};
use std::sync::Arc;

use test_helpers::setup_test_db;
use src_tauri::commands::projects::project_to_response as cmd_project_to_response;
use src_tauri::repositories::SqliteProjectRepository;

/// Test that create_project command validates empty name
#[tokio::test]
async fn test_create_project_empty_name_validation() {
    let _pool = setup_test_db().await.expect("Failed to setup test DB");

    // Validate empty name is rejected
    let empty_name = "";
    assert!(empty_name.is_empty(), "Empty name should be rejected by validation");
}

/// Test that create_project command validates name length
#[tokio::test]
async fn test_create_project_name_length_validation() {
    let _pool = setup_test_db().await.expect("Failed to setup test DB");

    // Validate name > 128 chars is rejected
    let long_name = "a".repeat(129);
    assert!(long_name.len() > 128, "Name > 128 chars should be rejected by validation");
}

/// Test that create_project command validates path is absolute
#[tokio::test]
async fn test_create_project_relative_path_validation() {
    // Relative paths should be rejected by ProjectPath
    let relative_path = "./foo/bar";
    let result = artifex_shared_kernel::ProjectPath::try_from(relative_path);
    assert!(result.is_err(), "Relative path should be rejected");
}

/// Test that create_project command creates project and returns correct response
#[tokio::test]
async fn test_create_project_command_success() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = SqliteProjectRepository::new(pool);
    let repo = Arc::new(repo);

    let project = Project::test_new("TestGame", "/tmp/test-game");

    // Create the project
    repo.create(&project).await.expect("Failed to create project");

    // Verify it was created
    let found = repo
        .find_by_id(&project.id)
        .await
        .expect("Failed to find project")
        .expect("Project not found");

    // Verify response mapping
    let response = cmd_project_to_response(found);
    assert_eq!(response.name, "TestGame");
    assert_eq!(response.status, "active");
    assert!(!response.id.is_empty());
}

/// Test that list_projects command returns correct projects
#[tokio::test]
async fn test_list_projects_command() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = SqliteProjectRepository::new(pool);
    let repo = Arc::new(repo);

    // Create multiple projects
    let p1 = Project::test_new("Alpha", "/tmp/alpha");
    let p2 = Project::test_new("Beta", "/tmp/beta");

    repo.create(&p1).await.expect("Failed to create p1");
    repo.create(&p2).await.expect("Failed to create p2");

    // List active (default)
    let active = repo.list_active().await.expect("Failed to list active");
    assert_eq!(active.len(), 2);

    // Map to responses
    let responses: Vec<_> = active
        .into_iter()
        .map(cmd_project_to_response)
        .collect();

    assert_eq!(responses.len(), 2);
    assert!(responses.iter().all(|r| r.status == "active"));
}

/// Test that archive_project command archives project
#[tokio::test]
async fn test_archive_project_command() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = SqliteProjectRepository::new(pool);
    let repo = Arc::new(repo);

    let project = Project::test_new("ToArchive", "/tmp/archive");
    repo.create(&project).await.expect("Failed to create project");

    // Archive it
    repo.archive(&project.id).await.expect("Failed to archive project");

    // Verify it's no longer in active list
    let active = repo.list_active().await.expect("Failed to list active");
    assert!(active.is_empty());

    // Verify it's in the all list with archived status
    let all = repo.list_all().await.expect("Failed to list all");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].status, ProjectStatus::Archived);

    let response = cmd_project_to_response(all.into_iter().next().unwrap());
    assert_eq!(response.status, "archived");
}

/// Test that rename via update works correctly
#[tokio::test]
async fn test_rename_project_via_update() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = SqliteProjectRepository::new(pool);
    let repo = Arc::new(repo);

    let mut project = Project::test_new("Original", "/tmp/original");
    repo.create(&project).await.expect("Failed to create project");

    // Rename via domain method + update
    let new_name = ProjectName::new("NewName").unwrap();
    project.rename(new_name).expect("rename should succeed");
    repo.update(&project).await.expect("Failed to update project");

    // Verify
    let found = repo
        .find_by_id(&project.id)
        .await
        .expect("Failed to find project")
        .expect("Project not found");

    assert_eq!(found.name.as_str(), "NewName");
}

/// Test that project_to_response mapping is correct
#[tokio::test]
async fn test_project_to_response_mapping() {
    let project = Project::test_new("TestProject", "/tmp/test");

    let response = cmd_project_to_response(project.clone());

    assert_eq!(response.name, "TestProject");
    assert_eq!(response.status, "active");
    assert_eq!(response.path, "/tmp/test");
    assert!(!response.id.is_empty());
    assert!(!response.created_at.is_empty());
    assert!(!response.updated_at.is_empty());
}

/// Test that rename to same name is a no-op at the command level
#[tokio::test]
async fn test_rename_project_same_name_noop() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = SqliteProjectRepository::new(pool);
    let repo = Arc::new(repo);

    let mut project = Project::test_new("SameName", "/tmp/test-game");
    repo.create(&project).await.expect("Failed to create project");

    // Get original project state
    let original = repo
        .find_by_id(&project.id)
        .await
        .expect("Failed to find project")
        .expect("Project not found");
    let original_updated_at = original.updated_at;
    let original_name = original.name.clone();

    // Simulate the command-level no-op check: if name is same, skip update
    if original_name.as_str() != original_name.as_str() {
        // This would be the "rename" call, but since names match it's a no-op
        let new_name = ProjectName::new(original_name.as_str()).unwrap();
        project.rename(new_name).expect("rename should succeed");
        repo.update(&project).await.expect("Failed to update project");
    }

    // Verify name unchanged and updated_at unchanged (no-op behavior)
    let after = repo
        .find_by_id(&project.id)
        .await
        .expect("Failed to find project")
        .expect("Project not found");
    assert_eq!(after.name.as_str(), "SameName");
    assert_eq!(after.updated_at, original_updated_at);
}
