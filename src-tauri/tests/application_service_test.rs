//! Integration tests for ProjectApplicationService.
//!
//! These tests verify the service layer business logic by testing through
//! the repository layer with an in-memory SQLite database.

mod test_helpers;

use std::sync::Arc;

use src_tauri::application::project_service::ProjectApplicationService;
use src_tauri::repositories::SqliteProjectRepository;
use test_helpers::setup_test_db;

/// Test that create_project succeeds with valid inputs
#[tokio::test]
async fn test_create_project_success() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = Arc::new(SqliteProjectRepository::new(pool));
    let service = ProjectApplicationService::new(repo);

    let result = service.create_project("MyGame", "/tmp/my-game").await;
    assert!(result.is_ok(), "create_project should succeed: {:?}", result);

    let project = result.unwrap();
    assert_eq!(project.name.as_str(), "MyGame");
    assert_eq!(project.path.to_string(), "/tmp/my-game");
    assert!(project.is_active());
}

/// Test that create_project with empty name returns ValidationError
#[tokio::test]
async fn test_create_project_empty_name_validation_error() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = Arc::new(SqliteProjectRepository::new(pool));
    let service = ProjectApplicationService::new(repo);

    let result = service.create_project("", "/tmp/test").await;
    assert!(result.is_err(), "create_project with empty name should fail");

    let err = result.unwrap_err();
    assert!(matches!(err, artifex_shared_kernel::ArtifexError::ValidationError(_)));
}

/// Test that create_project with duplicate name returns DuplicateName
#[tokio::test]
async fn test_create_project_duplicate_name_error() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = Arc::new(SqliteProjectRepository::new(pool));
    let service = ProjectApplicationService::new(repo);

    // Create first project
    service
        .create_project("DuplicateTest", "/tmp/first")
        .await
        .expect("First create should succeed");

    // Try to create second project with same name
    let result = service
        .create_project("DuplicateTest", "/tmp/second")
        .await;

    assert!(result.is_err(), "Duplicate name should fail");
    let err = result.unwrap_err();
    assert!(matches!(err, artifex_shared_kernel::ArtifexError::DuplicateName(_)));
}

/// Test that list_projects returns all active projects
#[tokio::test]
async fn test_list_projects_returns_active_only() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = Arc::new(SqliteProjectRepository::new(pool));
    let service = ProjectApplicationService::new(repo);

    // Create two projects
    service
        .create_project("ProjectA", "/tmp/proj-a")
        .await
        .expect("Failed to create ProjectA");
    service
        .create_project("ProjectB", "/tmp/proj-b")
        .await
        .expect("Failed to create ProjectB");

    let projects = service.list_projects().await.expect("list_projects should succeed");
    assert_eq!(projects.len(), 2);
}

/// Test that list_projects does not include archived projects
#[tokio::test]
async fn test_list_projects_excludes_archived() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = Arc::new(SqliteProjectRepository::new(pool));
    let service = ProjectApplicationService::new(repo);

    // Create and archive a project
    let created = service
        .create_project("ToArchive", "/tmp/archive")
        .await
        .expect("Failed to create project");
    service
        .archive_project(&created.id.into_uuid().to_string())
        .await
        .expect("Failed to archive project");

    let projects = service.list_projects().await.expect("list_projects should succeed");
    assert!(projects.is_empty(), "Archived project should not appear in list");
}

/// Test that get_project returns NotFound for nonexistent ID
#[tokio::test]
async fn test_get_project_not_found() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = Arc::new(SqliteProjectRepository::new(pool));
    let service = ProjectApplicationService::new(repo);

    let result = service.get_project("00000000-0000-0000-0000-000000000000").await;
    assert!(result.is_err(), "get_project for nonexistent should fail");

    let err = result.unwrap_err();
    assert!(matches!(err, artifex_shared_kernel::ArtifexError::NotFound(_)));
}

/// Test that rename_project succeeds with valid new name
#[tokio::test]
async fn test_rename_project_success() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = Arc::new(SqliteProjectRepository::new(pool));
    let service = ProjectApplicationService::new(repo);

    let created = service
        .create_project("Original", "/tmp/original")
        .await
        .expect("Failed to create project");

    let renamed = service
        .rename_project(&created.id.into_uuid().to_string(), "NewName")
        .await
        .expect("rename_project should succeed");

    assert_eq!(renamed.name.as_str(), "NewName");
}

/// Test that rename_project with duplicate name returns DuplicateName
#[tokio::test]
async fn test_rename_project_duplicate_name_error() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = Arc::new(SqliteProjectRepository::new(pool));
    let service = ProjectApplicationService::new(repo);

    // Create two projects
    let proj1 = service
        .create_project("NameOne", "/tmp/one")
        .await
        .expect("Failed to create proj1");
    service
        .create_project("NameTwo", "/tmp/two")
        .await
        .expect("Failed to create proj2");

    // Try to rename proj1 to proj2's name
    let result = service
        .rename_project(&proj1.id.into_uuid().to_string(), "NameTwo")
        .await;

    assert!(result.is_err(), "rename to duplicate name should fail");
    let err = result.unwrap_err();
    assert!(matches!(err, artifex_shared_kernel::ArtifexError::DuplicateName(_)));
}

/// Test that rename_project to same name is a no-op
#[tokio::test]
async fn test_rename_project_same_name_noop() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = Arc::new(SqliteProjectRepository::new(pool));
    let service = ProjectApplicationService::new(repo);

    let created = service
        .create_project("SameName", "/tmp/test")
        .await
        .expect("Failed to create project");

    let result = service
        .rename_project(&created.id.into_uuid().to_string(), "SameName")
        .await;

    assert!(result.is_ok(), "Rename to same name should succeed (no-op)");
    assert_eq!(result.unwrap().name.as_str(), "SameName");
}

/// Test that archive_project removes project from active list
#[tokio::test]
async fn test_archive_project_removes_from_active_list() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = Arc::new(SqliteProjectRepository::new(pool));
    let service = ProjectApplicationService::new(repo);

    let created = service
        .create_project("ToArchive", "/tmp/archive")
        .await
        .expect("Failed to create project");

    service
        .archive_project(&created.id.into_uuid().to_string())
        .await
        .expect("archive_project should succeed");

    let projects = service.list_projects().await.expect("list_projects should succeed");
    assert!(projects.is_empty(), "Archived project should not appear in active list");
}

/// Test that open_project validates project is active
#[tokio::test]
async fn test_open_project_validates_active_status() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = Arc::new(SqliteProjectRepository::new(pool));
    let service = ProjectApplicationService::new(repo);

    let created = service
        .create_project("ToArchive", "/tmp/archive")
        .await
        .expect("Failed to create project");

    service
        .archive_project(&created.id.into_uuid().to_string())
        .await
        .expect("archive_project should succeed");

    let result = service.open_project(&created.id.into_uuid().to_string()).await;
    assert!(result.is_err(), "open_project on archived project should fail");
    let err = result.unwrap_err();
    assert!(matches!(err, artifex_shared_kernel::ArtifexError::ValidationError(_)));
}

/// Test that open_project succeeds for active project
#[tokio::test]
async fn test_open_project_success() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = Arc::new(SqliteProjectRepository::new(pool));
    let service = ProjectApplicationService::new(repo);

    let created = service
        .create_project("ActiveProject", "/tmp/active")
        .await
        .expect("Failed to create project");

    let result = service
        .open_project(&created.id.into_uuid().to_string())
        .await;

    assert!(result.is_ok(), "open_project should succeed for active project");
    assert_eq!(result.unwrap().name.as_str(), "ActiveProject");
}
