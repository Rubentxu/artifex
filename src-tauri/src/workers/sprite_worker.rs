//! Sprite sheet generation worker.
//!
//! Handles video-to-sprite-sheet conversion jobs.

use std::path::{Path, PathBuf};

use artifex_job_queue::Job;
use artifex_shared_kernel::AppError;
use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::version::ffmpeg_version;
use image::DynamicImage;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;
use texture_packer::exporter::ImageExporter;
use texture_packer::{TexturePacker, TexturePackerConfig};

use super::traits::{JobFuture, JobResult, JobWorker};

/// Payload for sprite sheet generation jobs.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteOperation {
    pub source_video_path: String,
    pub fps: u8,
    pub dedup_threshold: f32,
    pub atlas_max_size: u32,
    pub padding: u8,
    pub animation_name: String,
    pub output_format: OutputFormat,
    pub source_asset_id: String,
    pub project_id: String,
}

/// Output format for sprite sheet manifests.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum OutputFormat {
    Json,
    Aseprite,
    #[default]
    Both,
}

/// Worker for sprite sheet generation jobs.
pub struct SpriteWorker {
    /// Base directory for saving output assets.
    assets_dir: String,
}

impl SpriteWorker {
    /// Creates a new SpriteWorker.
    pub fn new(assets_dir: String) -> Self {
        Self { assets_dir }
    }

    /// Main processing function for sprite sheet generation.
    async fn process_sprite_job(
        &self,
        job_id: artifex_shared_kernel::JobId,
        project_id: artifex_shared_kernel::ProjectId,
        payload: SpriteOperation,
    ) -> Result<JobResult, AppError> {
        let output_dir = PathBuf::from(&self.assets_dir)
            .join(project_id.into_uuid().to_string())
            .join("sprites");

        tokio::fs::create_dir_all(&output_dir)
            .await
            .map_err(|e| AppError::io_error(format!("Failed to create output directory: {}", e)))?;

        // Create temp directory for frames
        let temp_dir = TempDir::new()
            .map_err(|e| AppError::io_error(format!("Failed to create temp directory: {}", e)))?;

        // Step 1: Extract frames from video
        let frame_paths = extract_frames(
            Path::new(&payload.source_video_path),
            payload.fps,
            temp_dir.path(),
        )
        .await?;

        // Step 2: Load frames as DynamicImages
        let mut frames: Vec<DynamicImage> = Vec::new();
        for frame_path in &frame_paths {
            let img = image::open(frame_path)
                .map_err(|e| AppError::internal(format!("Failed to load frame {}: {}", frame_path.display(), e)))?;
            frames.push(img);
        }

        // Step 3: Dedup frames
        let (deduped_frames, dedup_removed) = dedup_frames(&frames, payload.dedup_threshold);
        tracing::info!(
            "Dedup: {} frames removed (threshold={})",
            dedup_removed,
            payload.dedup_threshold
        );

        // Step 4: Pack atlas
        let (atlas_img, frame_rects) = pack_atlas(
            &deduped_frames,
            payload.atlas_max_size,
            payload.padding,
        )
        .await?;

        // Step 5: Save atlas PNG
        let atlas_filename = format!("{}_{}.png", payload.animation_name, job_id.into_uuid());
        let atlas_path = output_dir.join(&atlas_filename);
        atlas_img
            .save(&atlas_path)
            .map_err(|e| AppError::internal(format!("Failed to save atlas: {}", e)))?;

        // Step 6: Extract source FPS from video
        let source_fps = extract_source_fps(Path::new(&payload.source_video_path));
        tracing::info!("Source video FPS: {:?}", source_fps);

        // Step 7: Generate manifests
        let mut output_files = vec![atlas_path.clone()];
        let mut metadata = serde_json::json!({
            "operation": "sprite_generate",
            "source_asset_id": payload.source_asset_id,
            "frame_count": deduped_frames.len(),
            "fps": payload.fps,
            "source_fps": source_fps,
            "dedup_removed": dedup_removed,
            "atlas_width": atlas_img.width(),
            "atlas_height": atlas_img.height(),
            "format": format!("{:?}", payload.output_format).to_lowercase(),
            "animation_name": payload.animation_name,
            "project_id": payload.project_id,
        });

        // Use relative paths for manifest files (relative to output_dir)
        let manifest_filename = format!("{}.json", atlas_filename.trim_end_matches(".png"));
        let aseprite_filename = format!("{}_aseprite.json", atlas_filename.trim_end_matches(".png"));

        let manifest_path_str: Option<String>;
        let aseprite_path_str: Option<String>;

        match payload.output_format {
            OutputFormat::Json | OutputFormat::Both => {
                let manifest_path = output_dir.join(&manifest_filename);
                write_canonical_manifest(
                    &deduped_frames,
                    &frame_rects,
                    payload.fps,
                    source_fps,
                    dedup_removed,
                    &payload.animation_name,
                    &atlas_filename,
                    &manifest_path.to_string_lossy(),
                )?;
                manifest_path_str = Some(manifest_filename.clone());
                output_files.push(manifest_path);
            }
            OutputFormat::Aseprite => {
                manifest_path_str = None;
            }
        }

        match payload.output_format {
            OutputFormat::Aseprite | OutputFormat::Both => {
                let aseprite_path = output_dir.join(&aseprite_filename);
                write_aseprite_json(
                    &deduped_frames,
                    &frame_rects,
                    payload.fps,
                    &atlas_filename,
                    &aseprite_path.to_string_lossy(),
                )?;
                aseprite_path_str = Some(aseprite_filename.clone());
                output_files.push(aseprite_path);
            }
            OutputFormat::Json => {
                aseprite_path_str = None;
            }
        }

        // Add relative paths to metadata
        if let Some(ref mp) = manifest_path_str {
            metadata["manifest_path"] = serde_json::json!(mp);
        }
        if let Some(ref ap) = aseprite_path_str {
            metadata["aseprite_path"] = serde_json::json!(ap);
        }

        // Clean up temp directory
        drop(temp_dir);

        Ok(JobResult::with_metadata(output_files, metadata))
    }
}

impl JobWorker for SpriteWorker {
    fn can_handle(&self, job_type: &str) -> bool {
        job_type == "sprite_generate"
    }

    fn process(&self, job: &Job) -> JobFuture {
        let assets_dir = self.assets_dir.clone();
        let job_id = job.id;
        let project_id = job.project_id;
        let operation = job.operation.clone();

        Box::pin(async move {
            // Deserialize operation JSON
            let payload: SpriteOperation = serde_json::from_value(operation)
                .map_err(|e| AppError::validation(format!("Invalid sprite operation payload: {}", e)))?;

            tracing::info!(
                "SpriteWorker processing job {} for project {}",
                job_id.into_uuid(),
                project_id.into_uuid()
            );

            let worker = SpriteWorker::new(assets_dir);
            worker.process_sprite_job(job_id, project_id, payload).await
        })
    }
}

// ============================================================================
// Frame Extraction
// ============================================================================

/// Extracts frames from a video file using ffmpeg.
///
/// Creates a temp directory and extracts frames as PNG files.
/// Returns the sorted list of frame file paths.
async fn extract_frames(
    source: &Path,
    fps: u8,
    output_dir: &Path,
) -> Result<Vec<PathBuf>, AppError> {
    // Verify source exists
    if !source.exists() {
        return Err(AppError::validation(format!(
            "Source video file does not exist: {}",
            source.display()
        )));
    }

    // Check ffmpeg is available
    let ffmpeg_ver = ffmpeg_version()
        .map_err(|_| AppError::validation("FFmpeg not found. Please install FFmpeg to process videos.".to_string()))?;

    tracing::info!("Using FFmpeg version: {}", ffmpeg_ver);

    let output_pattern = output_dir.join("frame_%04d.png");

    // Run ffmpeg to extract frames
    let ffmpeg_result = FfmpegCommand::new()
        .input(source.to_str().unwrap())
        .overwrite()
        .args(["-vf", &format!("fps={}", fps)])
        .output(output_pattern.to_str().unwrap())
        .spawn()
        .map_err(|e| AppError::internal(format!("FFmpeg spawn error: {}", e)))?
        .wait();

    // Explicitly check FFmpeg exit status
    match ffmpeg_result {
        Ok(status) => {
            if !status.success() {
                return Err(AppError::validation(format!(
                    "FFmpeg failed with exit code {}: {}",
                    status.code().map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string()),
                    source.display()
                )));
            }
        }
        Err(e) => {
            return Err(AppError::internal(format!("FFmpeg execution error: {}", e)));
        }
    }

    // Collect and sort frame paths using std::fs::read_dir since ffmpeg-sidecar blocks
    let mut frames: Vec<PathBuf> = std::fs::read_dir(output_dir)
        .map_err(|e| AppError::io_error(format!("Failed to read temp frame directory: {}", e)))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "png"))
        .collect();

    frames.sort();

    if frames.is_empty() {
        return Err(AppError::internal("No frames extracted from video".to_string()));
    }

    tracing::info!("Extracted {} frames from video", frames.len());
    Ok(frames)
}

/// Extracts the FPS of the source video using ffprobe.
///
/// Returns the frame rate as a floating point number (e.g., 30.0, 59.94).
/// Returns None if the FPS cannot be determined.
fn extract_source_fps(source: &Path) -> Option<f64> {
    // Try using ffprobe first (more reliable for getting FPS)
    let output = std::process::Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=r_frame_rate",
            "-of", "csv=p=0",
            source.to_str()?,
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let fps_str = String::from_utf8_lossy(&output.stdout);
        // Parse fps string like "30000/1001" or "30"
        if let Some((num, den)) = fps_str.trim().split_once('/') {
            let num: f64 = num.parse().ok()?;
            let den: f64 = den.parse().ok()?;
            if den > 0.0 {
                return Some(num / den);
            }
        }
        // Try parsing as simple float
        if let Ok(fps) = fps_str.trim().parse::<f64>() {
            return Some(fps);
        }
    }

    // Fallback: use ffmpeg to get video info
    let output = std::process::Command::new("ffmpeg")
        .args(["-i", source.to_str()?])
        .output()
        .ok()?;

    if output.status.success() {
        return None;
    }

    // Parse ffmpeg output for FPS
    let stderr = String::from_utf8_lossy(&output.stderr);
    for line in stderr.lines() {
        if line.contains("fps") || line.contains("fps ") {
            // Look for patterns like "30 fps" or "2997/1001 fps"
            for part in line.split_whitespace() {
                if part.contains("/") {
                    let parts: Vec<&str> = part.split('/').collect();
                    if parts.len() == 2 {
                        if let (Ok(num), Ok(den)) = (
                            parts[0].parse::<f64>(),
                            parts[1].parse::<f64>(),
                        ) {
                            if den > 0.0 {
                                return Some(num / den);
                            }
                        }
                    }
                } else if let Ok(fps) = part.parse::<f64>() {
                    if fps > 0.0 && fps < 1000.0 {
                        return Some(fps);
                    }
                }
            }
        }
    }

    None
}

// ============================================================================
// Perceptual Dedup
// ============================================================================

/// Computes perceptual difference between two images.
///
/// 1. Downscales both to 32x32
/// 2. Converts to grayscale
/// 3. Computes mean absolute pixel difference
/// 4. Normalizes to 0.0-1.0 range
fn perceptual_diff(img_a: &DynamicImage, img_b: &DynamicImage) -> f32 {
    // Downscale to 32x32 using nearest neighbor
    let a = img_a.resize_exact(32, 32, image::imageops::FilterType::Nearest);
    let b = img_b.resize_exact(32, 32, image::imageops::FilterType::Nearest);

    // Convert to grayscale (luma)
    let a_luma = a.to_luma8();
    let b_luma = b.to_luma8();

    // Compute mean absolute difference
    let diff: f32 = a_luma
        .pixels()
        .zip(b_luma.pixels())
        .map(|(p1, p2)| (p1[0] as f32 - p2[0] as f32).abs())
        .sum();

    diff / (32.0 * 32.0 * 255.0)
}

/// Deduplicates frames using perceptual similarity.
///
/// Keeps the first frame and subsequent frames if their difference
/// from the previous kept frame exceeds the threshold.
fn dedup_frames(frames: &[DynamicImage], threshold: f32) -> (Vec<DynamicImage>, u32) {
    let mut kept = Vec::with_capacity(frames.len());
    let mut removed = 0u32;

    for frame in frames {
        if kept.is_empty() {
            kept.push(frame.clone());
        } else {
            let diff = perceptual_diff(kept.last().unwrap(), frame);
            if diff >= threshold {
                kept.push(frame.clone());
            } else {
                removed += 1;
            }
        }
    }

    (kept, removed)
}

// ============================================================================
// Atlas Packing
// ============================================================================

/// Rectangle information for a packed frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameRect {
    pub index: usize,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// Packs frames into a sprite atlas using texture_packer.
///
/// Returns the atlas image and the list of frame rectangles.
async fn pack_atlas(
    frames: &[DynamicImage],
    max_size: u32,
    padding: u8,
) -> Result<(DynamicImage, Vec<FrameRect>), AppError> {
    // frames parameter is used for the initial capacity of the Vec
    if frames.is_empty() {
        return Err(AppError::validation("No frames to pack".to_string()));
    }

    let config = TexturePackerConfig {
        max_width: max_size,
        max_height: max_size,
        allow_rotation: false,
        border_padding: padding as u32,
        ..Default::default()
    };

    let mut packer = TexturePacker::new_skyline(config);

    for (i, frame) in frames.iter().enumerate() {
        let name = format!("frame_{:04}", i);
        if let Err(e) = packer.pack_own(name, frame.clone()) {
            // Check if this is a geometry error (frame doesn't fit in atlas)
            let err_msg = format!("{:?}", e);
            if err_msg.contains("Geometry") || err_msg.contains("overflow") || err_msg.contains("too large") {
                return Err(AppError::validation(format!(
                    "Atlas too small: frame {} ({}x{}) cannot fit in atlas of {}x{} with {}px padding. Try increasing atlas_max_size or reducing frame count/size.",
                    i,
                    frame.width(),
                    frame.height(),
                    max_size,
                    max_size,
                    padding
                )));
            }
            return Err(AppError::validation(format!(
                "Failed to pack frame {}: {:?}",
                i,
                e
            )));
        }
    }

    // Export the packed atlas using ImageExporter
    let atlas = ImageExporter::export(&packer, None)
        .map_err(|e| AppError::internal(format!("Failed to export atlas: {:?}", e)))?;

    // Collect frame rects from packer's frames
    let frames_map = packer.get_frames();
    let frame_rects: Vec<FrameRect> = frames
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let name = format!("frame_{:04}", i);
            if let Some(frame) = frames_map.get(&name) {
                FrameRect {
                    index: i,
                    x: frame.frame.x,
                    y: frame.frame.y,
                    w: frame.frame.w,
                    h: frame.frame.h,
                }
            } else {
                FrameRect {
                    index: i,
                    x: 0,
                    y: 0,
                    w: 0,
                    h: 0,
                }
            }
        })
        .collect();

    Ok((atlas, frame_rects))
}

// ============================================================================
// Manifest Generation
// ============================================================================

/// Canonical JSON manifest structure.
#[derive(Debug, Serialize)]
struct Manifest {
    atlas: String,
    frames: Vec<FrameEntry>,
    fps: u8,
    source_fps: Option<f64>,
    frame_count: usize,
    dedup_removed: u32,
    animation_name: String,
}

#[derive(Debug, Serialize)]
struct FrameEntry {
    index: usize,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    duration_ms: u32,
}

/// Writes the canonical JSON manifest to a file.
#[allow(clippy::too_many_arguments)]
fn write_canonical_manifest(
    frames: &[DynamicImage],
    rects: &[FrameRect],
    fps: u8,
    source_fps: Option<f64>,
    dedup_removed: u32,
    animation_name: &str,
    atlas_filename: &str,
    path: &str,
) -> Result<(), AppError> {
    let duration_ms = if fps > 0 { 1000 / fps as u32 } else { 100 };

    let frame_entries: Vec<FrameEntry> = rects
        .iter()
        .enumerate()
        .map(|(i, rect)| FrameEntry {
            index: i,
            x: rect.x,
            y: rect.y,
            w: rect.w,
            h: rect.h,
            duration_ms,
        })
        .collect();

    let manifest = Manifest {
        atlas: atlas_filename.to_string(),
        frames: frame_entries,
        fps,
        source_fps,
        frame_count: frames.len(),
        dedup_removed,
        animation_name: animation_name.to_string(),
    };

    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| AppError::internal(format!("Failed to serialize manifest: {}", e)))?;

    std::fs::write(path, json)
        .map_err(|e| AppError::io_error(format!("Failed to write manifest: {}", e)))?;

    tracing::info!("Wrote canonical manifest to {}", path);
    Ok(())
}

/// Aseprite JSON frame structure.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AsepriteFrame {
    frame: AsepriteFrameRect,
    duration: u32,
}

#[derive(Debug, Serialize)]
struct AsepriteFrameRect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

/// Aseprite JSON meta structure.
#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct AsepriteMeta {
    image: String,
    size: AsepriteSize,
    frame_tags: Vec<()>,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct AsepriteSize {
    w: u32,
    h: u32,
}

/// Writes Aseprite-compatible JSON to a file.
fn write_aseprite_json(
    _frames: &[DynamicImage],
    rects: &[FrameRect],
    fps: u8,
    atlas_filename: &str,
    path: &str,
) -> Result<(), AppError> {
    let duration_ms = if fps > 0 { 1000 / fps as u32 } else { 100 };

    let mut frames_map: IndexMap<String, AsepriteFrame> = IndexMap::new();

    for (i, rect) in rects.iter().enumerate() {
        let name = format!("{}_{}", atlas_filename.trim_end_matches(".png"), i);
        frames_map.insert(
            name,
            AsepriteFrame {
                frame: AsepriteFrameRect {
                    x: rect.x,
                    y: rect.y,
                    w: rect.w,
                    h: rect.h,
                },
                duration: duration_ms,
            },
        );
    }

    // Calculate atlas size from the last rect (should cover the full atlas)
    let (atlas_w, atlas_h) = if let Some(last_rect) = rects.last() {
        (last_rect.x + last_rect.w, last_rect.y + last_rect.h)
    } else {
        (0, 0)
    };

    let aseprite_json = serde_json::json!({
        "frames": frames_map,
        "meta": {
            "image": atlas_filename,
            "size": { "w": atlas_w, "h": atlas_h },
            "frameTags": []
        }
    });

    let json = serde_json::to_string_pretty(&aseprite_json)
        .map_err(|e| AppError::internal(format!("Failed to serialize Aseprite JSON: {}", e)))?;

    std::fs::write(path, json)
        .map_err(|e| AppError::io_error(format!("Failed to write Aseprite JSON: {}", e)))?;

    tracing::info!("Wrote Aseprite JSON to {}", path);
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle() {
        let worker = SpriteWorker::new("/tmp".to_string());
        assert!(worker.can_handle("sprite_generate"));
        assert!(!worker.can_handle("image_generate"));
    }

    #[test]
    fn test_output_format_default() {
        assert_eq!(OutputFormat::default(), OutputFormat::Both);
    }

    #[test]
    fn test_perceptual_diff_identical_images() {
        // Create two identical 64x64 red images
        let img1 = DynamicImage::new_rgb8(64, 64);
        let img2 = DynamicImage::new_rgb8(64, 64);

        let diff = perceptual_diff(&img1, &img2);
        assert!(diff < 0.01, "Identical images should have diff near 0, got {}", diff);
    }

    #[test]
    fn test_perceptual_diff_different_images() {
        use image::{Rgba, RgbaImage};

        let mut img1 = RgbaImage::new(64, 64);
        for pixel in img1.pixels_mut() {
            *pixel = Rgba([255, 0, 0, 255]); // Red
        }

        let mut img2 = RgbaImage::new(64, 64);
        for pixel in img2.pixels_mut() {
            *pixel = Rgba([0, 0, 255, 255]); // Blue
        }

        let diff = perceptual_diff(&DynamicImage::ImageRgba8(img1), &DynamicImage::ImageRgba8(img2));
        assert!(diff > 0.1, "Different images should have diff > 0.1, got {}", diff);
    }

    #[test]
    fn test_dedup_frames_keeps_all_above_threshold() {
        use image::{Rgba, RgbaImage};

        let mut img1 = RgbaImage::new(32, 32);
        for pixel in img1.pixels_mut() {
            *pixel = Rgba([255, 0, 0, 255]);
        }
        let mut img2 = RgbaImage::new(32, 32);
        for pixel in img2.pixels_mut() {
            *pixel = Rgba([0, 255, 0, 255]);
        }
        let mut img3 = RgbaImage::new(32, 32);
        for pixel in img3.pixels_mut() {
            *pixel = Rgba([0, 0, 255, 255]);
        }

        let frames = vec![
            DynamicImage::ImageRgba8(img1),
            DynamicImage::ImageRgba8(img2),
            DynamicImage::ImageRgba8(img3),
        ];

        let (kept, removed) = dedup_frames(&frames, 0.1);
        assert_eq!(kept.len(), 3, "All frames should be kept with low threshold");
        assert_eq!(removed, 0);
    }

    #[test]
    fn test_dedup_frames_removes_similar() {
        use image::{Rgba, RgbaImage};

        let mut img1 = RgbaImage::new(32, 32);
        for pixel in img1.pixels_mut() {
            *pixel = Rgba([100, 100, 100, 255]);
        }
        let mut img2 = RgbaImage::new(32, 32);
        for pixel in img2.pixels_mut() {
            *pixel = Rgba([101, 100, 100, 255]); // Very similar to img1
        }
        let mut img3 = RgbaImage::new(32, 32);
        for pixel in img3.pixels_mut() {
            *pixel = Rgba([200, 200, 200, 255]); // Different
        }

        let frames = vec![
            DynamicImage::ImageRgba8(img1),
            DynamicImage::ImageRgba8(img2),
            DynamicImage::ImageRgba8(img3),
        ];

        let (kept, removed) = dedup_frames(&frames, 0.03);
        assert!(kept.len() < 3, "Similar frames should be deduplicated");
        assert_eq!(removed, 1);
    }

    #[test]
    fn test_frame_rect_serialization() {
        let rect = FrameRect {
            index: 0,
            x: 10,
            y: 20,
            w: 64,
            h: 128,
        };
        let json = serde_json::to_string(&rect).unwrap();
        assert!(json.contains("\"index\":0"));
        assert!(json.contains("\"x\":10"));
        assert!(json.contains("\"y\":20"));
        assert!(json.contains("\"w\":64"));
        assert!(json.contains("\"h\":128"));
    }
}
