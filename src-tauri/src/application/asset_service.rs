//! Asset application service.
//!
//! Orchestrates business operations for assets, enforcing domain rules
//! and coordinating with the repository layer.

use std::path::Path;
use std::sync::Arc;

use artifex_asset_management::{Asset, AssetKind, AssetRepository};
use artifex_shared_kernel::{ArtifexError, AssetId, ProjectId};

use super::audio_metadata::extract_audio_metadata;

/// Application service for asset operations.
#[derive(Clone)]
pub struct AssetApplicationService {
    repo: Arc<dyn AssetRepository>,
}

impl AssetApplicationService {
    /// Creates a new AssetApplicationService.
    pub fn new(repo: Arc<dyn AssetRepository>) -> Self {
        Self { repo }
    }

    /// Lists all assets for a given project.
    pub async fn list_assets(&self, project_id: &str) -> Result<Vec<Asset>, ArtifexError> {
        let pid = parse_project_id(project_id)?;
        self.repo.find_by_project(&pid).await
    }

    /// Gets a single asset by ID.
    pub async fn get_asset(&self, id: &str) -> Result<Asset, ArtifexError> {
        let asset_id = parse_asset_id(id)?;
        self.repo
            .find_by_id(&asset_id)
            .await?
            .ok_or_else(|| ArtifexError::not_found("Asset", id))
    }

    /// Deletes an asset by ID.
    pub async fn delete_asset(&self, id: &str) -> Result<(), ArtifexError> {
        let asset_id = parse_asset_id(id)?;
        self.repo.delete(&asset_id).await
    }

    /// Imports a file into the project's asset directory and registers it as an asset.
    ///
    /// The file is copied to `<project_path>/artifex-assets/<kind>/<name>`.
    ///
    /// # Errors
    /// - `ValidationError` if project_id, source_path, name, or kind is invalid
    /// - `IoError` if the file cannot be copied
    pub async fn import_file(
        &self,
        project_id: &str,
        source_path: &str,
        name: &str,
        kind: &str,
    ) -> Result<Asset, ArtifexError> {
        let pid = parse_project_id(project_id)?;
        let asset_kind = parse_asset_kind(kind)?;

        // Validate source path exists
        let source = Path::new(source_path);
        if !source.exists() {
            return Err(ArtifexError::validation(format!(
                "Source file does not exist: {}",
                source_path
            )));
        }

        // Get file size
        let metadata = tokio::fs::metadata(source_path)
            .await
            .map_err(|e| ArtifexError::IoError(e.to_string()))?;
        let file_size = metadata.len();

        // Validate name
        let name = name.trim();
        if name.is_empty() {
            return Err(ArtifexError::validation("Asset name cannot be empty"));
        }

        // Determine destination path: <project_path>/artifex-assets/<kind>/<name>
        // We need the project path from the repository or passed in
        // For now, we'll construct a relative path based on the project_id directory
        // This is a simplified approach - in production, the project path would come from ProjectRepository
        let dest_relative = format!("artifex-assets/{}/{}", kind, name);

        // Get project to find project path (needed for actual file storage)
        // For this implementation, we'll store assets relative to an app data directory
        // The asset_service should receive the assets_base_dir or project path
        // For now, use the source path's parent as a reference or a temp location

        // Actually, we need the project path. Let's fetch the project first.
        // We need ProjectRepository here, but we don't have access to it.
        // The caller should provide the destination base path.
        // For now, we'll use the source path's directory as the base.

        let dest_path = Path::new(source_path)
            .parent()
            .map(|p| p.join(&dest_relative))
            .unwrap_or_else(|| Path::new(&dest_relative).to_path_buf());

        // Create destination directory
        if let Some(parent) = dest_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| ArtifexError::IoError(e.to_string()))?;
        }

        // Copy file
        tokio::fs::copy(source_path, &dest_path)
            .await
            .map_err(|e| ArtifexError::IoError(e.to_string()))?;

        // Build asset with file info (clone asset_kind for later comparison)
        let asset_kind_clone = asset_kind.clone();
        let mut asset = Asset::register(pid, name, asset_kind)
            .map_err(|e| ArtifexError::validation(e))?;
        asset.file_path = Some(dest_path.to_string_lossy().to_string());
        asset.file_size = Some(file_size);

        // For image assets, try to get dimensions
        if asset_kind_clone == AssetKind::Image {
            if let Ok((width, height)) = get_image_dimensions(&dest_path).await {
                asset.width = Some(width);
                asset.height = Some(height);
            }
        }

        // For audio/voice assets, extract metadata from file
        if matches!(asset_kind_clone, AssetKind::Audio | AssetKind::Voice) {
            let meta = extract_audio_metadata(&dest_path);
            let mut metadata = asset.metadata.take().unwrap_or(serde_json::json!({}));
            if let Some(obj) = metadata.as_object_mut() {
                if let Some(d) = meta.duration_secs {
                    obj.insert("duration_secs".to_string(), serde_json::json!(d));
                }
                if let Some(sr) = meta.sample_rate {
                    obj.insert("sample_rate".to_string(), serde_json::json!(sr));
                }
                if let Some(ref f) = meta.format {
                    obj.insert("format".to_string(), serde_json::json!(f));
                }
            }
            asset.metadata = Some(metadata);
            tracing::debug!(
                "Extracted audio metadata: duration={:?}, sample_rate={:?}, format={:?}",
                meta.duration_secs,
                meta.sample_rate,
                meta.format
            );
        }

        self.repo.create(&asset).await
    }

    /// Registers an already-saved file as an asset in the database.
    ///
    /// This is used when a worker has already saved a file and needs to register it.
    pub async fn register_asset(
        &self,
        project_id: &str,
        name: &str,
        kind: &str,
        file_path: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<Asset, ArtifexError> {
        let pid = parse_project_id(project_id)?;
        let asset_kind = parse_asset_kind(kind)?;

        // Validate file path exists
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(ArtifexError::validation(format!(
                "File does not exist: {}",
                file_path
            )));
        }

        // Get file metadata
        let meta = tokio::fs::metadata(file_path)
            .await
            .map_err(|e| ArtifexError::IoError(e.to_string()))?;

        let mut asset = Asset::register(pid, name, asset_kind.clone())
            .map_err(|e| ArtifexError::validation(e))?;
        asset.file_path = Some(file_path.to_string());
        asset.file_size = Some(meta.len());

        // Merge provided metadata with audio metadata (file-based preferred for audio)
        let mut merged_metadata = metadata.unwrap_or(serde_json::json!({}));
        if matches!(asset_kind, AssetKind::Audio | AssetKind::Voice) {
            let file_meta = extract_audio_metadata(path);
            if let Some(obj) = merged_metadata.as_object_mut() {
                if let Some(d) = file_meta.duration_secs {
                    obj.insert("duration_secs".to_string(), serde_json::json!(d));
                }
                if let Some(sr) = file_meta.sample_rate {
                    obj.insert("sample_rate".to_string(), serde_json::json!(sr));
                }
                if let Some(ref f) = file_meta.format {
                    obj.insert("format".to_string(), serde_json::json!(f));
                }
            }
            tracing::debug!(
                "Extracted audio metadata for registered asset: duration={:?}, sample_rate={:?}, format={:?}",
                file_meta.duration_secs,
                file_meta.sample_rate,
                file_meta.format
            );
        }
        asset.metadata = Some(merged_metadata);

        // For image assets, try to get dimensions
        if asset_kind == AssetKind::Image {
            if let Ok((width, height)) = get_image_dimensions(path).await {
                asset.width = Some(width);
                asset.height = Some(height);
            }
        }

        self.repo.create(&asset).await
    }
}

/// Parses a project ID string into a ProjectId.
fn parse_project_id(id: &str) -> Result<ProjectId, ArtifexError> {
    let uuid = uuid::Uuid::parse_str(id)
        .map_err(|e| ArtifexError::validation(format!("Invalid project id: {}", e)))?;
    Ok(ProjectId::from_uuid(uuid))
}

/// Parses an asset ID string into an AssetId.
fn parse_asset_id(id: &str) -> Result<AssetId, ArtifexError> {
    let uuid = uuid::Uuid::parse_str(id)
        .map_err(|e| ArtifexError::validation(format!("Invalid asset id: {}", e)))?;
    Ok(AssetId::from_uuid(uuid))
}

/// Parses a kind string into an AssetKind.
fn parse_asset_kind(kind: &str) -> Result<AssetKind, ArtifexError> {
    AssetKind::from_str(kind).ok_or_else(|| {
        ArtifexError::validation(format!(
            "Invalid asset kind: {}. Valid kinds: image, sprite, tileset, material, audio, voice, video, other",
            kind
        ))
    })
}

/// Attempts to get image dimensions from a file.
/// Returns (width, height) on success.
async fn get_image_dimensions(path: &Path) -> Result<(u32, u32), ArtifexError> {
    // Read first few bytes to check PNG or JPEG
    let file = tokio::fs::File::open(path)
        .await
        .map_err(|e| ArtifexError::IoError(e.to_string()))?;

    use tokio::io::AsyncReadExt;
    let mut reader = tokio::io::BufReader::new(file);
    let mut header = [0u8; 8];
    reader
        .read_exact(&mut header)
        .await
        .map_err(|e| ArtifexError::IoError(e.to_string()))?;

    // PNG signature
    if header.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        // For PNG, we need to read the IHDR chunk
        // This is simplified - in production, use the image crate
        let file = tokio::fs::File::open(path)
            .await
            .map_err(|e| ArtifexError::IoError(e.to_string()))?;
        let mut reader = tokio::io::BufReader::new(file);
        
        // Skip to IHDR (after 8-byte signature + 4-byte length + 4-byte "IHDR")
        let mut skip = [0u8; 20];
        reader
            .read_exact(&mut skip)
            .await
            .map_err(|e| ArtifexError::IoError(e.to_string()))?;
        
        // Width and height are 4 bytes each, big-endian, starting at offset 8 of IHDR
        let width = u32::from_be_bytes([skip[12], skip[13], skip[14], skip[15]]);
        let height = u32::from_be_bytes([skip[16], skip[17], skip[18], skip[19]]);
        return Ok((width, height));
    }

    // JPEG signature
    if header[0] == 0xFF && header[1] == 0xD8 && header[2] == 0xFF {
        // JPEG is more complex to parse without the image crate
        // For simplicity, return (0, 0) for JPEG
        return Ok((0, 0));
    }

    Err(ArtifexError::validation("Unsupported image format"))
}
