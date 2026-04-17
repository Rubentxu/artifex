//! Atlas packing worker.
//!
//! Combines multiple Image/Sprite/Tileset assets into a single texture atlas PNG
//! with a JSON manifest sidecar.

use std::path::PathBuf;

use artifex_job_queue::Job;
use artifex_shared_kernel::AppError;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use texture_packer::exporter::ImageExporter;
use texture_packer::{TexturePacker, TexturePackerConfig};

use super::traits::{JobFuture, JobResult, JobWorker};

/// Valid max_size values for atlas packing.
const VALID_MAX_SIZES: &[u32] = &[512, 1024, 2048, 4096];

/// Payload for atlas pack jobs.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackAtlasOperation {
    pub project_id: String,
    pub atlas_name: String,
    pub source_assets: Vec<PackAtlasSourceAsset>,
    pub options: super::super::dto::PackAtlasOptions,
}

/// Source asset info embedded in the pack operation.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackAtlasSourceAsset {
    pub asset_id: String,
    pub name: String,
    pub file_path: String,
}

/// Atlas manifest JSON structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasManifest {
    pub version: u32,
    pub atlas_name: String,
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub regions: Vec<AtlasRegion>,
}

/// Region within an atlas manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasRegion {
    pub asset_id: String,
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub source_width: u32,
    pub source_height: u32,
    pub rotated: bool,
}

/// Worker for packing texture atlases.
pub struct AtlasPackWorker {
    /// Base directory for assets.
    assets_dir: String,
}

impl AtlasPackWorker {
    /// Creates a new AtlasPackWorker.
    pub fn new(assets_dir: String) -> Self {
        Self { assets_dir }
    }
}

impl JobWorker for AtlasPackWorker {
    fn can_handle(&self, job_type: &str) -> bool {
        job_type == "pack_atlas"
    }

    fn process(&self, job: &Job) -> JobFuture {
        let assets_dir = self.assets_dir.clone();
        let job_id = job.id;
        let operation = job.operation.clone();

        Box::pin(async move {
            let op: PackAtlasOperation = serde_json::from_value(operation)
                .map_err(|e| AppError::validation(format!("Invalid pack atlas operation: {}", e)))?;

            tracing::info!(
                "AtlasPackWorker processing job {} for project {}",
                job_id.into_uuid(),
                op.project_id
            );

            let worker = AtlasPackWorker::new(assets_dir);
            worker.process_pack_job(job_id, op).await
        })
    }
}

impl AtlasPackWorker {
    /// Main processing function for atlas packing.
    async fn process_pack_job(
        &self,
        _job_id: artifex_shared_kernel::JobId,
        op: PackAtlasOperation,
    ) -> Result<JobResult, AppError> {
        // Validate max_size
        if !VALID_MAX_SIZES.contains(&op.options.max_size) {
            return Err(AppError::validation(format!(
                "invalid max_size: {} (must be one of {:?})",
                op.options.max_size,
                VALID_MAX_SIZES
            )));
        }

        let output_dir = PathBuf::from(&self.assets_dir)
            .join(&op.project_id)
            .join("sprites");

        tokio::fs::create_dir_all(&output_dir)
            .await
            .map_err(|e| AppError::io_error(format!("Failed to create output directory: {}", e)))?;

        // Load all source images
        let mut frames: Vec<(String, String, DynamicImage)> = Vec::new();
        for source in &op.source_assets {
            let path = PathBuf::from(&source.file_path);
            if !path.exists() {
                return Err(AppError::validation(format!(
                    "Source image file not found: {}",
                    path.display()
                )));
            }
            let img = image::open(&path)
                .map_err(|e| AppError::internal(format!(
                    "Failed to load image {}: {}",
                    path.display(),
                    e
                )))?;
            frames.push((source.asset_id.clone(), source.name.clone(), img));
        }

        if frames.is_empty() {
            return Err(AppError::validation("No frames to pack".to_string()));
        }

        // Configure texture packer
        let config = TexturePackerConfig {
            max_width: op.options.max_size,
            max_height: op.options.max_size,
            allow_rotation: op.options.allow_rotation,
            border_padding: op.options.padding as u32,
            ..Default::default()
        };

        let mut packer = TexturePacker::new_skyline(config);

        // Pack each frame
        let mut unpacked_names: Vec<String> = Vec::new();
        for (_asset_id, name, img) in &frames {
            match packer.pack_own(name.clone(), img.clone()) {
                Ok(_) => {}
                Err(e) => {
                    let err_msg = format!("{:?}", e);
                    if err_msg.contains("Geometry") || err_msg.contains("overflow") || err_msg.contains("too large") {
                        unpacked_names.push(name.clone());
                    } else {
                        return Err(AppError::validation(format!(
                            "Failed to pack frame '{}': {:?}",
                            name, e
                        )));
                    }
                }
            }
        }

        if !unpacked_names.is_empty() {
            return Err(AppError::validation(format!(
                "Atlas overflow: these assets could not fit: {}",
                unpacked_names.join(", ")
            )));
        }

        // Export the atlas
        let atlas = ImageExporter::export(&packer, None)
            .map_err(|e| AppError::internal(format!("Failed to export atlas: {:?}", e)))?;

        // Save atlas PNG
        let atlas_filename = format!("{}.png", op.atlas_name);
        let atlas_path = output_dir.join(&atlas_filename);
        atlas
            .save(&atlas_path)
            .map_err(|e| AppError::internal(format!("Failed to save atlas: {}", e)))?;

        let atlas_width = atlas.width();
        let atlas_height = atlas.height();

        // Build manifest
        let frames_map = packer.get_frames();
        let mut regions: Vec<AtlasRegion> = Vec::new();

        for (asset_id, name, img) in &frames {
            if let Some(frame) = frames_map.get(name) {
                regions.push(AtlasRegion {
                    asset_id: asset_id.clone(),
                    name: name.clone(),
                    x: frame.frame.x,
                    y: frame.frame.y,
                    w: frame.frame.w,
                    h: frame.frame.h,
                    source_width: img.width(),
                    source_height: img.height(),
                    // Note: texture_packer doesn't expose rotation flag in an easily accessible way
                    // Rotation is handled internally by swapping w/h and storing rotated pixels
                    rotated: false,
                });
            }
        }

        let region_count = regions.len();
        let manifest = AtlasManifest {
            version: 1,
            atlas_name: op.atlas_name.clone(),
            atlas_width,
            atlas_height,
            regions,
        };

        // Write manifest JSON
        let manifest_filename = format!("{}.json", op.atlas_name);
        let manifest_path = output_dir.join(&manifest_filename);
        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| AppError::internal(format!("Failed to serialize manifest: {}", e)))?;
        std::fs::write(&manifest_path, manifest_json)
            .map_err(|e| AppError::io_error(format!("Failed to write manifest: {}", e)))?;

        let output_files = vec![atlas_path, manifest_path];

        let metadata = serde_json::json!({
            "operation": "pack_atlas",
            "atlas_name": op.atlas_name,
            "project_id": op.project_id,
            "atlas_width": atlas_width,
            "atlas_height": atlas_height,
            "region_count": region_count,
            "atlas_manifest": manifest,
        });

        Ok(JobResult::with_metadata(output_files, metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle() {
        let worker = AtlasPackWorker::new("/tmp/assets".to_string());
        assert!(worker.can_handle("pack_atlas"));
        assert!(!worker.can_handle("sprite_generate"));
        assert!(!worker.can_handle("animation_export"));
    }

    #[test]
    fn test_atlas_manifest_serialization() {
        let manifest = AtlasManifest {
            version: 1,
            atlas_name: "test_atlas".to_string(),
            atlas_width: 1024,
            atlas_height: 512,
            regions: vec![
                AtlasRegion {
                    asset_id: "asset-1".to_string(),
                    name: "hero".to_string(),
                    x: 0,
                    y: 0,
                    w: 64,
                    h: 128,
                    source_width: 64,
                    source_height: 128,
                    rotated: false,
                },
                AtlasRegion {
                    asset_id: "asset-2".to_string(),
                    name: "sword".to_string(),
                    x: 64,
                    y: 0,
                    w: 32,
                    h: 64,
                    source_width: 32,
                    source_height: 64,
                    rotated: false,
                },
            ],
        };

        let json = serde_json::to_string_pretty(&manifest).unwrap();
        assert!(json.contains("\"version\": 1"));
        assert!(json.contains("\"atlasName\": \"test_atlas\""));
        assert!(json.contains("\"atlasWidth\": 1024"));
        assert!(json.contains("\"atlasHeight\": 512"));
        assert!(json.contains("\"regions\":"));
        assert!(json.contains("\"name\": \"hero\""));
        assert!(json.contains("\"name\": \"sword\""));

        // Verify deserialization
        let deser: AtlasManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.version, 1);
        assert_eq!(deser.regions.len(), 2);
        assert_eq!(deser.regions[0].name, "hero");
        assert_eq!(deser.regions[1].name, "sword");
    }

    #[test]
    fn test_pack_atlas_operation_deserialize() {
        let json = serde_json::json!({
            "projectId": "proj-123",
            "atlasName": "my_atlas",
            "sourceAssets": [
                {
                    "assetId": "asset-1",
                    "name": "hero",
                    "filePath": "/assets/proj-123/hero.png"
                },
                {
                    "assetId": "asset-2",
                    "name": "enemy",
                    "filePath": "/assets/proj-123/enemy.png"
                }
            ],
            "options": {
                "maxSize": 2048,
                "padding": 2,
                "allowRotation": false,
                "sortMode": "area"
            }
        });

        let op: PackAtlasOperation = serde_json::from_value(json).unwrap();
        assert_eq!(op.project_id, "proj-123");
        assert_eq!(op.atlas_name, "my_atlas");
        assert_eq!(op.source_assets.len(), 2);
        assert_eq!(op.source_assets[0].name, "hero");
        assert_eq!(op.source_assets[1].name, "enemy");
        assert_eq!(op.options.max_size, 2048);
        assert_eq!(op.options.padding, 2);
        assert!(!op.options.allow_rotation);
    }

    #[test]
    fn test_valid_max_sizes() {
        assert!(VALID_MAX_SIZES.contains(&512u32));
        assert!(VALID_MAX_SIZES.contains(&1024u32));
        assert!(VALID_MAX_SIZES.contains(&2048u32));
        assert!(VALID_MAX_SIZES.contains(&4096u32));
        assert!(!VALID_MAX_SIZES.contains(&256u32));
        assert!(!VALID_MAX_SIZES.contains(&8192u32));
    }
}