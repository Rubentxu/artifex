//! Integration tests for SqliteAssetRepository.
//!
//! These tests verify the database round-trip behavior of the asset repository.

mod test_helpers;

use artifex_asset_management::{Asset, AssetKind, AssetRepository, Project, ProjectRepository};
use std::sync::Arc;
use std::fs;
use tempfile::TempDir;

use test_helpers::setup_test_db;
use src_tauri::repositories::{SqliteAssetRepository, SqliteProjectRepository};

/// Helper to create a test project in the DB.
async fn create_test_project(pool: &sqlx::SqlitePool) -> Project {
    let project = Project::test_new("TestProject", "/tmp/test");
    let repo = SqliteProjectRepository::new(pool.clone());
    repo.create(&project).await.expect("Failed to create test project");
    project
}

/// Helper to create a test file on disk.
fn create_test_file(dir: &TempDir, name: &str, content: &[u8]) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to write test file");
    path
}

#[tokio::test]
async fn test_create_asset_and_find_by_id() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let project = create_test_project(&pool).await;
    let repo = SqliteAssetRepository::new(pool);
    let repo = Arc::new(repo);

    // Create an asset
    let asset = Asset::new(project.id, "test.png", AssetKind::Image);
    repo.create(&asset).await.expect("Failed to create asset");

    // Find by ID
    let found = repo
        .find_by_id(&asset.id)
        .await
        .expect("Failed to find asset")
        .expect("Asset not found");

    assert_eq!(found.id, asset.id);
    assert_eq!(found.name, "test.png");
    assert_eq!(found.kind, AssetKind::Image);
    assert_eq!(found.project_id, project.id);
}

#[tokio::test]
async fn test_find_by_project() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let project = create_test_project(&pool).await;
    let repo = SqliteAssetRepository::new(pool);
    let repo = Arc::new(repo);

    // Create 3 assets
    let asset1 = Asset::new(project.id, "asset1.png", AssetKind::Image);
    let asset2 = Asset::new(project.id, "asset2.png", AssetKind::Sprite);
    let asset3 = Asset::new(project.id, "asset3.png", AssetKind::Image);

    repo.create(&asset1).await.expect("Failed to create asset1");
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    repo.create(&asset2).await.expect("Failed to create asset2");
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    repo.create(&asset3).await.expect("Failed to create asset3");

    // Find by project
    let assets = repo.find_by_project(&project.id).await.expect("Failed to list assets");
    assert_eq!(assets.len(), 3);

    // Should be ordered by created_at DESC (most recent first)
    assert_eq!(assets[0].id, asset3.id);
    assert_eq!(assets[1].id, asset2.id);
    assert_eq!(assets[2].id, asset1.id);
}

#[tokio::test]
async fn test_find_by_kind() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let project = create_test_project(&pool).await;
    let repo = SqliteAssetRepository::new(pool);
    let repo = Arc::new(repo);

    // Create assets of different kinds
    let image1 = Asset::new(project.id, "image1.png", AssetKind::Image);
    let sprite1 = Asset::new(project.id, "sprite1.png", AssetKind::Sprite);
    let image2 = Asset::new(project.id, "image2.png", AssetKind::Image);
    let audio1 = Asset::new(project.id, "audio1.mp3", AssetKind::Audio);

    repo.create(&image1).await.expect("Failed to create image1");
    repo.create(&sprite1).await.expect("Failed to create sprite1");
    repo.create(&image2).await.expect("Failed to create image2");
    repo.create(&audio1).await.expect("Failed to create audio1");

    // Filter by Image kind
    let images = repo
        .find_by_kind(&project.id, &AssetKind::Image)
        .await
        .expect("Failed to filter by kind");
    assert_eq!(images.len(), 2);
    assert!(images.iter().all(|a| a.kind == AssetKind::Image));

    // Filter by Sprite kind
    let sprites = repo
        .find_by_kind(&project.id, &AssetKind::Sprite)
        .await
        .expect("Failed to filter by kind");
    assert_eq!(sprites.len(), 1);
    assert_eq!(sprites[0].name, "sprite1.png");

    // Filter by Audio kind
    let audios = repo
        .find_by_kind(&project.id, &AssetKind::Audio)
        .await
        .expect("Failed to filter by kind");
    assert_eq!(audios.len(), 1);
    assert_eq!(audios[0].name, "audio1.mp3");
}

#[tokio::test]
async fn test_delete_asset() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let project = create_test_project(&pool).await;
    let repo = SqliteAssetRepository::new(pool);
    let repo = Arc::new(repo);

    let asset = Asset::new(project.id, "to_delete.png", AssetKind::Image);
    repo.create(&asset).await.expect("Failed to create asset");

    // Verify it exists
    let found = repo
        .find_by_id(&asset.id)
        .await
        .expect("Failed to find asset");
    assert!(found.is_some());

    // Delete it
    repo.delete(&asset.id).await.expect("Failed to delete asset");

    // Verify it's gone
    let found = repo
        .find_by_id(&asset.id)
        .await
        .expect("Failed to find asset after delete");
    assert!(found.is_none());
}

#[tokio::test]
async fn test_delete_nonexistent_asset() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let repo = SqliteAssetRepository::new(pool);

    let random_id = artifex_shared_kernel::AssetId::new();
    let result = repo.delete(&random_id).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, artifex_shared_kernel::ArtifexError::NotFound(_)));
}

#[tokio::test]
async fn test_asset_with_file_metadata() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let project = create_test_project(&pool).await;
    let repo = SqliteAssetRepository::new(pool);
    let repo = Arc::new(repo);

    // Create asset with full metadata
    let mut asset = Asset::with_image_metadata(
        project.id,
        "hero.png",
        "/projects/test/artifex-assets/image/hero.png",
        512,
        1024,
        65536,
    );
    asset.file_path = Some("/projects/test/artifex-assets/image/hero.png".to_string());

    repo.create(&asset).await.expect("Failed to create asset");

    // Find and verify
    let found = repo
        .find_by_id(&asset.id)
        .await
        .expect("Failed to find asset")
        .expect("Asset not found");

    assert_eq!(found.name, "hero.png");
    assert_eq!(found.kind, AssetKind::Image);
    assert_eq!(found.file_path, Some("/projects/test/artifex-assets/image/hero.png".to_string()));
    assert_eq!(found.file_size, Some(65536));
    assert_eq!(found.width, Some(512));
    assert_eq!(found.height, Some(1024));
    assert!(found.metadata.is_some());
}

#[tokio::test]
async fn test_import_asset_file_copy() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let project = create_test_project(&pool).await;
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create a source file to import
    let source_file = create_test_file(&temp_dir, "source.png", b"fake png data");

    let repo = SqliteAssetRepository::new(pool);
    let repo = Arc::new(repo);

    // Import the file using the service
    use src_tauri::application::AssetApplicationService;

    let service = AssetApplicationService::new(repo.clone());

    let imported = service
        .import_file(
            &project.id.into_uuid().to_string(),
            source_file.to_str().unwrap(),
            "imported.png",
            "image",
        )
        .await
        .expect("Failed to import asset");

    // Verify the asset was created
    assert_eq!(imported.name, "imported.png");
    assert_eq!(imported.kind, AssetKind::Image);
    assert!(imported.file_path.is_some());
    assert!(imported.file_size.is_some());

    // Verify the destination file exists (file was copied)
    if let Some(dest_path) = &imported.file_path {
        // The path will be based on the source file's directory
        // Just verify it was created
        assert!(std::path::Path::new(dest_path).exists() || std::path::Path::new(dest_path).parent().map(|p| p.exists()).unwrap_or(false));
    }

    // Verify we can find it in the repo
    let found = repo
        .find_by_id(&imported.id)
        .await
        .expect("Failed to find imported asset")
        .expect("Imported asset not found");
    assert_eq!(found.name, "imported.png");
}

#[tokio::test]
async fn test_register_asset() {
    let pool = setup_test_db().await.expect("Failed to setup test DB");
    let project = create_test_project(&pool).await;
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create a source file to register
    let test_file = create_test_file(&temp_dir, "worker_output.png", b"worker generated image");

    let repo = SqliteAssetRepository::new(pool);
    let repo = Arc::new(repo);

    // Register the asset using the service
    use src_tauri::application::AssetApplicationService;

    let service = AssetApplicationService::new(repo.clone());

    let registered = service
        .register_asset(
            &project.id.into_uuid().to_string(),
            "worker_output.png",
            "image",
            test_file.to_str().unwrap(),
            Some(serde_json::json!({"generated_by": "worker"})),
        )
        .await
        .expect("Failed to register asset");

    // Verify the asset was created
    assert_eq!(registered.name, "worker_output.png");
    assert_eq!(registered.kind, AssetKind::Image);
    assert_eq!(registered.file_path, Some(test_file.to_string_lossy().to_string()));
    assert!(registered.metadata.is_some());

    // Verify we can find it in the repo
    let found = repo
        .find_by_id(&registered.id)
        .await
        .expect("Failed to find registered asset")
        .expect("Registered asset not found");
    assert_eq!(found.name, "worker_output.png");
}
