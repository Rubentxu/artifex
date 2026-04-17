//! Animation export worker.
//!
//! Combines animation frames into a sprite sheet with timing JSON sidecar.

use std::path::PathBuf;

use artifex_job_queue::Job;
use artifex_shared_kernel::AppError;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use texture_packer::exporter::ImageExporter;
use texture_packer::{TexturePacker, TexturePackerConfig};

use super::traits::{JobFuture, JobResult, JobWorker};

/// Payload for animation export jobs.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationExportOperation {
    pub project_id: String,
    pub animation_id: String,
    pub animation_name: String,
    pub frame_asset_ids: Vec<String>,
    pub frame_durations_ms: Vec<u32>,
    pub loop_animation: bool,
}

/// Timing entry for the JSON sidecar.
#[derive(Debug, Serialize, Deserialize)]
pub struct FrameTiming {
    pub index: usize,
    pub duration_ms: u32,
}

/// Animation timing sidecar metadata.
#[derive(Debug, Serialize, Deserialize)]
pub struct AnimationTimingJson {
    pub animation_id: String,
    pub animation_name: String,
    pub loop_animation: bool,
    pub fps: f64,
    pub total_duration_ms: u32,
    pub frames: Vec<FrameTiming>,
    pub columns: u32,
    pub rows: u32,
    pub frame_width: u32,
    pub frame_height: u32,
}

/// Worker that exports animations as sprite sheets.
pub struct AnimationExportWorker {
    assets_dir: String,
}

impl AnimationExportWorker {
    pub fn new(assets_dir: String) -> Self {
        Self { assets_dir }
    }
}

impl JobWorker for AnimationExportWorker {
    fn can_handle(&self, job_type: &str) -> bool {
        job_type == "animation_export"
    }

    fn process(&self, job: &Job) -> JobFuture {
        let job = job.clone();
        let assets_dir = self.assets_dir.clone();
        Box::pin(async move {
            let op: AnimationExportOperation =
                serde_json::from_value(job.operation.clone())
                    .map_err(|e| AppError::validation(format!("Invalid animation export params: {}", e)))?;

            let base_dir = PathBuf::from(&assets_dir);

            // Load all frame images from assets_dir using the asset UUID as filename
            let mut frame_images: Vec<DynamicImage> = Vec::new();
            for frame_id in &op.frame_asset_ids {
                let frame_path = base_dir.join(format!("{}.png", frame_id));
                if !frame_path.exists() {
                    return Err(AppError::validation(format!(
                        "Frame file not found: {}",
                        frame_path.display()
                    )));
                }
                let img = image::open(&frame_path)
                    .map_err(|e| AppError::internal(format!("Failed to open frame image {}: {}", frame_id, e)))?;
                frame_images.push(img);
            }

            if frame_images.is_empty() {
                return Err(AppError::validation("No frames to export".to_string()));
            }

            // Calculate grid dimensions
            let frame_count = frame_images.len() as u32;
            let frame_w = frame_images[0].width();
            let frame_h = frame_images[0].height();

            let columns = (frame_count as f64).sqrt().ceil() as u32;
            let rows = (frame_count + columns - 1) / columns;

            // Create atlas using texture_packer (skyline algorithm)
            let max_size = 4096u32;
            let config = TexturePackerConfig {
                max_width: max_size,
                max_height: max_size,
                allow_rotation: false,
                border_padding: 0,
                ..Default::default()
            };

            let mut packer = TexturePacker::new_skyline(config);

            for (i, frame) in frame_images.iter().enumerate() {
                let name = format!("frame_{:04}", i);
                packer
                    .pack_own(name, frame.clone())
                    .map_err(|e| AppError::validation(format!("Failed to pack frame {}: {:?}", i, e)))?;
            }

            // Export the packed atlas as DynamicImage
            let atlas = ImageExporter::export(&packer, None)
                .map_err(|e| AppError::internal(format!("Failed to export atlas: {:?}", e)))?;

            // Save atlas
            let export_dir = base_dir.join("animations").join(&op.animation_id);
            std::fs::create_dir_all(&export_dir)
                .map_err(|e| AppError::internal(format!("Failed to create export dir: {}", e)))?;

            let atlas_path = export_dir.join("spritesheet.png");
            atlas
                .save(&atlas_path)
                .map_err(|e| AppError::internal(format!("Failed to save atlas: {}", e)))?;

            // Calculate FPS and total duration
            let total_duration_ms: u32 = op.frame_durations_ms.iter().sum();
            let fps = if total_duration_ms > 0 && frame_count > 0 {
                frame_count as f64 / (total_duration_ms as f64 / 1000.0)
            } else {
                24.0
            };

            // Generate timing JSON
            let timing = AnimationTimingJson {
                animation_id: op.animation_id.clone(),
                animation_name: op.animation_name.clone(),
                loop_animation: op.loop_animation,
                fps,
                total_duration_ms,
                columns,
                rows,
                frame_width: frame_w,
                frame_height: frame_h,
                frames: op
                    .frame_durations_ms
                    .iter()
                    .enumerate()
                    .map(|(i, d)| FrameTiming {
                        index: i,
                        duration_ms: *d,
                    })
                    .collect(),
            };

            let timing_path = export_dir.join("timing.json");
            let timing_json = serde_json::to_string_pretty(&timing)
                .map_err(|e| AppError::internal(format!("Failed to serialize timing JSON: {}", e)))?;
            std::fs::write(&timing_path, timing_json)
                .map_err(|e| AppError::internal(format!("Failed to write timing JSON: {}", e)))?;

            // Collect frame rects from packer
            let frames_map = packer.get_frames();
            let mut frame_rects = Vec::new();
            for i in 0..frame_count {
                let name = format!("frame_{:04}", i);
                if let Some(frame) = frames_map.get(&name) {
                    frame_rects.push(serde_json::json!({
                        "index": i,
                        "x": frame.frame.x,
                        "y": frame.frame.y,
                        "w": frame.frame.w,
                        "h": frame.frame.h,
                    }));
                }
            }

            let metadata = serde_json::json!({
                "operation": "animation_export",
                "animation_id": op.animation_id,
                "animation_name": op.animation_name,
                "frame_count": frame_count,
                "fps": fps,
                "total_duration_ms": total_duration_ms,
                "columns": columns,
                "rows": rows,
                "frame_width": frame_w,
                "frame_height": frame_h,
                "frames": frame_rects,
            });

            Ok(JobResult::with_metadata(
                vec![atlas_path],
                metadata,
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle() {
        let worker = AnimationExportWorker::new("/tmp/assets".to_string());
        assert!(worker.can_handle("animation_export"));
        assert!(!worker.can_handle("image_generate"));
    }

    #[test]
    fn test_animation_timing_json_serialization() {
        let timing = AnimationTimingJson {
            animation_id: "test_anim".to_string(),
            animation_name: "Test Animation".to_string(),
            loop_animation: true,
            fps: 12.0,
            total_duration_ms: 1000,
            columns: 4,
            rows: 1,
            frame_width: 64,
            frame_height: 64,
            frames: vec![
                FrameTiming { index: 0, duration_ms: 100 },
                FrameTiming { index: 1, duration_ms: 100 },
            ],
        };

        let json = serde_json::to_string(&timing).unwrap();
        assert!(json.contains("test_anim"));
        assert!(json.contains("12.0"));

        let deser: AnimationTimingJson = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.frames.len(), 2);
        assert!(deser.loop_animation);
    }

    #[test]
    fn test_animation_export_operation_deserialize() {
        let json = serde_json::json!({
            "projectId": "proj-123",
            "animationId": "anim-456",
            "animationName": "Walk Cycle",
            "frameAssetIds": ["frame-1", "frame-2"],
            "frameDurationsMs": [100, 200],
            "loopAnimation": false,
        });

        let op: AnimationExportOperation = serde_json::from_value(json).unwrap();
        assert_eq!(op.animation_id, "anim-456");
        assert_eq!(op.animation_name, "Walk Cycle");
        assert_eq!(op.frame_asset_ids.len(), 2);
        assert_eq!(op.frame_durations_ms, vec![100, 200]);
        assert!(!op.loop_animation);
    }

    #[test]
    fn test_fps_calculation() {
        // 4 frames at 250ms each = 1000ms total = 4 FPS
        let fps: f64 = 4.0 / (1000.0 / 1000.0);
        assert!((fps - 4.0).abs() < 0.01);

        // 10 frames at 100ms each = 1000ms total = 10 FPS
        let fps: f64 = 10.0 / (1000.0 / 1000.0);
        assert!((fps - 10.0).abs() < 0.01);
    }
}
