//! Integration tests for SqliteProjectRepository.
//!
//! These tests verify the database round-trip behavior of the repository.

mod test_helpers;

use artifex_asset_management::{Project, ProjectName, ProjectRepository, ProjectStatus};
use std::sync::Arc;

use test_helpers::setup_test_db;
use src_tauri::repositories::SqliteProjectRepository;

/// Helper to create a test project.
fn make_test_project(name: &str) -> Project {
    Project::test_new(name, "/tmp/test")
}



#[tokio::test]
async fn test_create_project_and_find_by_id() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = SqliteProjectRepository::new(pool);
    let repo = Arc::new(repo);

    let project = make_test_project("TestGame");

    // Create
    repo.create(&project).await.expect("Failed to create project");

    // Find by ID
    let found = repo
        .find_by_id(&project.id)
        .await
        .expect("Failed to find project")
        .expect("Project not found");

    assert_eq!(found.id, project.id);
    assert_eq!(found.name.as_str(), "TestGame");
    assert_eq!(found.status, ProjectStatus::Active);
    assert_eq!(found.path, project.path);
}

#[tokio::test]
async fn test_list_projects() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = SqliteProjectRepository::new(pool);
    let repo = Arc::new(repo);

    // Create 3 projects with slightly different timestamps
    let p1 = make_test_project("Alpha");
    let p2 = make_test_project("Beta");
    let p3 = make_test_project("Gamma");

    repo.create(&p1).await.expect("Failed to create p1");
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    repo.create(&p2).await.expect("Failed to create p2");
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    repo.create(&p3).await.expect("Failed to create p3");

    // List all
    let all = repo.list_all().await.expect("Failed to list all");
    assert_eq!(all.len(), 3);

    // Should be ordered by updated_at DESC (most recent first)
    // p3 was created last, so it should be first
    assert_eq!(all[0].name.as_str(), "Gamma");
    assert_eq!(all[1].name.as_str(), "Beta");
    assert_eq!(all[2].name.as_str(), "Alpha");
}

#[tokio::test]
async fn test_list_active_projects() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = SqliteProjectRepository::new(pool);
    let repo = Arc::new(repo);

    // Create 2 active projects
    let active1 = make_test_project("Active1");
    let active2 = make_test_project("Active2");

    // Create and archive a project
    let mut archived = make_test_project("Archived");
    archived.status = ProjectStatus::Archived;
    archived.updated_at = artifex_shared_kernel::Timestamp::now();

    repo.create(&active1).await.expect("Failed to create active1");
    repo.create(&active2).await.expect("Failed to create active2");
    repo.create(&archived).await.expect("Failed to create archived");

    // List active only
    let active = repo.list_active().await.expect("Failed to list active");
    assert_eq!(active.len(), 2);

    // Verify all returned are active
    for project in &active {
        assert_eq!(project.status, ProjectStatus::Active);
    }

    // Verify names
    let names: Vec<_> = active.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"Active1"));
    assert!(names.contains(&"Active2"));
}

#[tokio::test]
async fn test_update_project() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = SqliteProjectRepository::new(pool);
    let repo = Arc::new(repo);

    let mut project = make_test_project("Original");
    repo.create(&project).await.expect("Failed to create project");

    // Update the project name using rename
    let new_name = ProjectName::new("Renamed").unwrap();
    project.rename(new_name).expect("rename should succeed");
    repo.update(&project).await.expect("Failed to update project");

    // Verify update persisted
    let found = repo
        .find_by_id(&project.id)
        .await
        .expect("Failed to find project")
        .expect("Project not found");

    assert_eq!(found.name.as_str(), "Renamed");
}

#[tokio::test]
async fn test_archive_project() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = SqliteProjectRepository::new(pool);
    let repo = Arc::new(repo);

    let project = make_test_project("ToArchive");
    repo.create(&project).await.expect("Failed to create project");

    // Archive
    repo.archive(&project.id).await.expect("Failed to archive project");

    // Verify it's no longer in active list
    let active = repo.list_active().await.expect("Failed to list active");
    assert!(active.is_empty());

    // But it should be in the all list
    let all = repo.list_all().await.expect("Failed to list all");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].status, ProjectStatus::Archived);
}

#[tokio::test]
async fn test_find_nonexistent_project() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = SqliteProjectRepository::new(pool);

    // Try to find a random UUID
    let random_id = artifex_shared_kernel::ProjectId::new();
    let found = repo
        .find_by_id(&random_id)
        .await
        .expect("Failed to find project");

    assert!(found.is_none());
}

#[tokio::test]
async fn test_find_by_name() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = SqliteProjectRepository::new(pool);
    let repo = Arc::new(repo);

    let project = make_test_project("FindMe");
    repo.create(&project).await.expect("Failed to create project");

    // Find by name
    let found = repo.find_by_name("FindMe").await.expect("Failed to find by name");
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, project.id);

    // Find by name that doesn't exist
    let not_found = repo.find_by_name("DoesNotExist").await.expect("Failed to find by name");
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_find_by_name_excludes_archived() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = SqliteProjectRepository::new(pool);
    let repo = Arc::new(repo);

    let project = make_test_project("ShouldNotFind");
    repo.create(&project).await.expect("Failed to create project");
    repo.archive(&project.id).await.expect("Failed to archive project");

    // Archived project should not be found by name
    let found = repo.find_by_name("ShouldNotFind").await.expect("Failed to find by name");
    assert!(found.is_none());
}

#[tokio::test]
async fn test_exists_by_name() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = SqliteProjectRepository::new(pool);
    let repo = Arc::new(repo);

    let project = make_test_project("ExistsCheck");
    repo.create(&project).await.expect("Failed to create project");

    // Exists
    let exists = repo.exists_by_name("ExistsCheck").await.expect("Failed to check existence");
    assert!(exists);

    // Does not exist
    let not_exists = repo.exists_by_name("DoesNotExist").await.expect("Failed to check existence");
    assert!(!not_exists);
}

#[tokio::test]
async fn test_exists_by_name_excludes_archived() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = SqliteProjectRepository::new(pool);
    let repo = Arc::new(repo);

    let project = make_test_project("ArchivedExists");
    repo.create(&project).await.expect("Failed to create project");
    repo.archive(&project.id).await.expect("Failed to archive project");

    // Archived project should not count as existing
    let exists = repo.exists_by_name("ArchivedExists").await.expect("Failed to check existence");
    assert!(!exists);
}

#[tokio::test]
async fn test_update_nonexistent_project_returns_not_found() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = SqliteProjectRepository::new(pool);

    let project = make_test_project("Ghost");
    // Note: we don't create the project, so it doesn't exist

    let result = repo.update(&project).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, artifex_shared_kernel::ArtifexError::NotFound(_)));
}

#[tokio::test]
async fn test_archive_nonexistent_project_returns_not_found() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = SqliteProjectRepository::new(pool);

    let random_id = artifex_shared_kernel::ProjectId::new();
    let result = repo.archive(&random_id).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, artifex_shared_kernel::ArtifexError::NotFound(_)));
}
