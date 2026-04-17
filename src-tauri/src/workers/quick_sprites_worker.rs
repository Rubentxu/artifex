//! Quick sprites generation worker.
//!
//! Handles combined image/video-to-sprite-sheet pipeline:
//! - FromImage: existing image → VideoGen → Sprite frames → Atlas
//! - FromPrompt: prompt → ImageGen → VideoGen → Sprite frames → Atlas
//!
//! This is a composite worker that chains AI generation with FFmpeg processing.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use artifex_job_queue::Job;
use artifex_model_config::credential_store::CredentialStore;
use artifex_model_config::image_provider::ImageGenParams;
use artifex_model_config::video_provider::VideoGenParams;
use artifex_model_config::ModelRouter;
use artifex_shared_kernel::AppError;
use base64::Engine;
use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::version::ffmpeg_version;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;
use texture_packer::exporter::ImageExporter;
use texture_packer::{TexturePacker, TexturePackerConfig};

use super::traits::{JobFuture, JobResult, JobWorker};

/// Payload for quick sprites generation jobs.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickSpritesOperation {
    /// Generation mode.
    pub mode: QuickSpritesMode,
    /// Source asset ID (for FromImage mode).
    pub source_asset_id: Option<String>,
    /// Source file path (resolved by command from asset).
    pub source_file_path: Option<String>,
    /// Motion prompt for video generation.
    pub motion_prompt: String,
    /// Optional negative prompt for video generation.
    pub negative_prompt: Option<String>,
    /// Image generation params (for FromPrompt mode).
    pub image_gen_params: Option<ImageGenParams>,
    /// Video generation params (overrides defaults if provided).
    pub video_duration_secs: Option<u8>,
    pub video_seed: Option<u64>,
    /// Sprite options.
    pub fps: u8,
    pub dedup_threshold: f32,
    pub atlas_max_size: u32,
    pub padding: u8,
    pub animation_name: String,
    /// Output format for manifests.
    pub output_format: QuickSpritesOutputFormat,
    pub project_id: String,
}

/// Generation mode for quick sprites.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuickSpritesMode {
    /// Generate from existing image asset.
    FromImage,
    /// Generate from prompt (image gen first).
    FromPrompt,
}

/// Output format for sprite sheet manifests.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum QuickSpritesOutputFormat {
    Json,
    Aseprite,
    #[default]
    Both,
}

/// Worker for quick sprites generation jobs.
pub struct QuickSpritesWorker {
    /// Model router for resolving providers and fallback chain.
    router: Arc<ModelRouter>,
    /// Credential store for API keys.
    credential_store: Arc<dyn CredentialStore>,
    /// Base directory for saving output assets.
    assets_dir: String,
}

impl QuickSpritesWorker {
    /// Creates a new QuickSpritesWorker.
    pub fn new(
        router: Arc<ModelRouter>,
        credential_store: Arc<dyn CredentialStore>,
        assets_dir: String,
    ) -> Self {
        Self {
            router,
            credential_store,
            assets_dir,
        }
    }
}

impl JobWorker for QuickSpritesWorker {
    fn can_handle(&self, job_type: &str) -> bool {
        job_type == "quick_sprites"
    }

    fn process(&self, job: &Job) -> JobFuture {
        let router = self.router.clone();
        let credential_store = self.credential_store.clone();
        let assets_dir = self.assets_dir.clone();
        let job_id = job.id;
        let project_id = job.project_id;
        let operation = job.operation.clone();

        Box::pin(async move {
            // Deserialize operation JSON
            let payload: QuickSpritesOperation = serde_json::from_value(operation)
                .map_err(|e| AppError::validation(format!("Invalid quick sprites payload: {}", e)))?;

            tracing::info!(
                "QuickSpritesWorker processing job {} for project {} in mode {:?}",
                job_id.into_uuid(),
                project_id.into_uuid(),
                payload.mode
            );

            let worker = QuickSpritesWorker::new(router, credential_store, assets_dir);
            worker.process_quick_sprites_job(job_id, project_id, payload).await
        })
    }
}

impl QuickSpritesWorker {
    /// Main processing function for quick sprites generation.
    async fn process_quick_sprites_job(
        &self,
        job_id: artifex_shared_kernel::JobId,
        project_id: artifex_shared_kernel::ProjectId,
        payload: QuickSpritesOperation,
    ) -> Result<JobResult, AppError> {
        // Step 1: Get source image (either from asset or generate from prompt)
        let source_image_data_uri = match payload.mode {
            QuickSpritesMode::FromImage => {
                // Read existing image file and convert to base64
                let source_path = payload.source_file_path.ok_or_else(|| {
                    AppError::validation("FromImage mode requires source_file_path".to_string())
                })?;
                read_file_as_data_uri(&source_path)?
            }
            QuickSpritesMode::FromPrompt => {
                // Generate image from prompt first
                let gen_params = payload.image_gen_params.ok_or_else(|| {
                    AppError::validation("FromPrompt mode requires image_gen_params".to_string())
                })?;

                // Resolve provider
                let resolved = self.router
                    .resolve_image("imagegen.txt2img")
                    .await
                    .map_err(|e| AppError::internal(format!("Failed to resolve image model: {}", e)))?;

                // Get credential
                let credential_id = format!("{}::api_key", resolved.profile.provider_name);
                let api_key = self.credential_store
                    .get(&credential_id)
                    .map_err(|_| AppError::internal(format!("Credential not found for {}", resolved.profile.provider_name)))?;

                // Generate image
                let gen_result = resolved
                    .provider
                    .generate(&gen_params, &api_key)
                    .await
                    .map_err(|e| AppError::internal(format!("Image generation error: {}", e)))?;

                // Convert to data URI
                let mime_type = "image/png"; // Default for generation
                let base64_data = base64::engine::general_purpose::STANDARD.encode(&gen_result.image_data);
                format!("data:{};base64,{}", mime_type, base64_data)
            }
        };

        // Step 2: Generate video from source image
        let video_duration = payload.video_duration_secs.unwrap_or(4);
        let mut video_params = VideoGenParams {
            source_image_url: source_image_data_uri.clone(),
            prompt: payload.motion_prompt.clone(),
            negative_prompt: payload.negative_prompt.clone(),
            duration_secs: video_duration,
            seed: payload.video_seed,
            model_id: None,
        };
        video_params.validate().map_err(AppError::validation)?;

        // Resolve video provider
        let resolved = self.router
            .resolve_video("videogen.img2video")
            .await
            .map_err(|e| AppError::internal(format!("Failed to resolve video model: {}", e)))?;

        // Get credential
        let credential_id = format!("{}::api_key", resolved.profile.provider_name);
        let api_key = self.credential_store
            .get(&credential_id)
            .map_err(|_| AppError::internal(format!("Credential not found for {}", resolved.profile.provider_name)))?;

        // Inject model_id
        video_params.model_id = Some(resolved.profile.model_id.clone());

        // Generate video
        let video_result = resolved
            .provider
            .generate_video(&video_params, &api_key)
            .await
            .map_err(|e| AppError::internal(format!("Video generation error: {}", e)))?;

        // Step 3: Save video to temp file
        let temp_dir = TempDir::new()
            .map_err(|e| AppError::io_error(format!("Failed to create temp directory: {}", e)))?;
        let video_path = temp_dir.path().join("input_video.mp4");
        tokio::fs::write(&video_path, &video_result.video_data)
            .await
            .map_err(|e| AppError::io_error(format!("Failed to write temp video file: {}", e)))?;

        // Step 4: Extract frames from video using FFmpeg
        let frame_paths = extract_frames(video_path.as_path(), payload.fps, temp_dir.path()).await?;

        // Step 5: Load frames as DynamicImages
        let mut frames: Vec<DynamicImage> = Vec::new();
        for frame_path in &frame_paths {
            let img = image::open(frame_path)
                .map_err(|e| AppError::internal(format!("Failed to load frame {}: {}", frame_path.display(), e)))?;
            frames.push(img);
        }

        // Step 6: Dedup frames
        let (deduped_frames, dedup_removed) = dedup_frames(&frames, payload.dedup_threshold);
        tracing::info!(
            "Dedup: {} frames removed (threshold={})",
            dedup_removed,
            payload.dedup_threshold
        );

        // Step 7: Pack atlas
        let (atlas_img, frame_rects) = pack_atlas(
            &deduped_frames,
            payload.atlas_max_size,
            payload.padding,
        )
        .await?;

        // Step 8: Save spritesheet PNG
        let output_dir = PathBuf::from(&self.assets_dir)
            .join(project_id.into_uuid().to_string())
            .join("sprites");
        tokio::fs::create_dir_all(&output_dir)
            .await
            .map_err(|e| AppError::io_error(format!("Failed to create output directory: {}", e)))?;

        let atlas_filename = format!("{}_{}.png", payload.animation_name, job_id.into_uuid());
        let atlas_path = output_dir.join(&atlas_filename);
        atlas_img
            .save(&atlas_path)
            .map_err(|e| AppError::internal(format!("Failed to save atlas: {}", e)))?;

        // Step 9: Extract source FPS from video
        let source_fps = extract_source_fps(video_path.as_path());
        tracing::info!("Source video FPS: {:?}", source_fps);

        // Step 10: Generate manifests
        let mut output_files = vec![atlas_path.clone()];
        let mut metadata = serde_json::json!({
            "operation": "quick_sprites",
            "mode": format!("{:?}", payload.mode).to_lowercase(),
            "fps": payload.fps,
            "source_fps": source_fps,
            "dedup_removed": dedup_removed,
            "atlas_width": atlas_img.width(),
            "atlas_height": atlas_img.height(),
            "frame_count": deduped_frames.len(),
            "format": format!("{:?}", payload.output_format).to_lowercase(),
            "animation_name": payload.animation_name,
            "project_id": payload.project_id,
            "video_duration_secs": video_result.duration_secs,
        });

        let manifest_filename = format!("{}.json", atlas_filename.trim_end_matches(".png"));
        let aseprite_filename = format!("{}_aseprite.json", atlas_filename.trim_end_matches(".png"));

        let manifest_path_str: Option<String>;
        let aseprite_path_str: Option<String>;

        match payload.output_format {
            QuickSpritesOutputFormat::Json | QuickSpritesOutputFormat::Both => {
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
            QuickSpritesOutputFormat::Aseprite => {
                manifest_path_str = None;
            }
        }

        match payload.output_format {
            QuickSpritesOutputFormat::Aseprite | QuickSpritesOutputFormat::Both => {
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
            QuickSpritesOutputFormat::Json => {
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

        // Clean up temp directory (includes temp video file)
        drop(temp_dir);

        Ok(JobResult::with_metadata(output_files, metadata))
    }
}

/// Reads a file and converts it to a base64 data URI.
fn read_file_as_data_uri(file_path: &str) -> Result<String, AppError> {
    let mut file = std::fs::File::open(file_path)
        .map_err(|e| AppError::io_error(format!("Failed to open file {}: {}", file_path, e)))?;
    let mut buffer = Vec::new();
    use std::io::Read;
    file.read_to_end(&mut buffer)
        .map_err(|e| AppError::io_error(format!("Failed to read file {}: {}", file_path, e)))?;

    // Detect MIME type from file extension
    let extension = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();

    let mime_type = match extension.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        _ => "image/png",
    };

    let base64_data = base64::engine::general_purpose::STANDARD.encode(&buffer);
    Ok(format!("data:{};base64,{}", mime_type, base64_data))
}

// ============================================================================
// Frame Extraction (mirrors SpriteWorker)
// ============================================================================

/// Extracts frames from a video file using ffmpeg.
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

    // Collect and sort frame paths
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
fn extract_source_fps(source: &Path) -> Option<f64> {
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
        if let Some((num, den)) = fps_str.trim().split_once('/') {
            let num: f64 = num.parse().ok()?;
            let den: f64 = den.parse().ok()?;
            if den > 0.0 {
                return Some(num / den);
            }
        }
        if let Ok(fps) = fps_str.trim().parse::<f64>() {
            return Some(fps);
        }
    }
    None
}

// ============================================================================
// Perceptual Dedup (mirrors SpriteWorker)
// ============================================================================

/// Computes perceptual difference between two images.
fn perceptual_diff(img_a: &DynamicImage, img_b: &DynamicImage) -> f32 {
    let a = img_a.resize_exact(32, 32, image::imageops::FilterType::Nearest);
    let b = img_b.resize_exact(32, 32, image::imageops::FilterType::Nearest);
    let a_luma = a.to_luma8();
    let b_luma = b.to_luma8();
    let diff: f32 = a_luma
        .pixels()
        .zip(b_luma.pixels())
        .map(|(p1, p2)| (p1[0] as f32 - p2[0] as f32).abs())
        .sum();
    diff / (32.0 * 32.0 * 255.0)
}

/// Deduplicates frames using perceptual similarity.
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
// Atlas Packing (mirrors SpriteWorker)
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
async fn pack_atlas(
    frames: &[DynamicImage],
    max_size: u32,
    padding: u8,
) -> Result<(DynamicImage, Vec<FrameRect>), AppError> {
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

    let atlas = ImageExporter::export(&packer, None)
        .map_err(|e| AppError::internal(format!("Failed to export atlas: {:?}", e)))?;

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
// Manifest Generation (mirrors SpriteWorker)
// ============================================================================

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

fn write_aseprite_json(
    _frames: &[DynamicImage],
    rects: &[FrameRect],
    fps: u8,
    atlas_filename: &str,
    path: &str,
) -> Result<(), AppError> {
    use indexmap::IndexMap;

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
        let worker = QuickSpritesWorker::new(
            Arc::new(ModelRouter::new(
                Arc::new(artifex_model_config::ProviderRegistry::new()),
                Arc::new(TestRepo),
                Arc::new(artifex_model_config::credential_store::InMemoryCredentialStore::new()),
            )),
            Arc::new(artifex_model_config::credential_store::InMemoryCredentialStore::new()),
            "/tmp".to_string(),
        );
        assert!(worker.can_handle("quick_sprites"));
        assert!(!worker.can_handle("sprite_generate"));
    }

    // Mock repository for testing
    struct TestRepo;

    #[async_trait::async_trait]
    impl artifex_model_config::router::ModelConfigRepository for TestRepo {
        async fn find_profile(
            &self,
            _id: &uuid::Uuid,
        ) -> Result<Option<artifex_model_config::ModelProfile>, String> {
            Ok(None)
        }

        async fn find_rule(
            &self,
            _operation_type: &str,
        ) -> Result<Option<artifex_model_config::RoutingRule>, String> {
            Ok(None)
        }

        async fn list_enabled_profiles(
            &self,
            _capability: artifex_model_config::ModelCapability,
        ) -> Result<Vec<artifex_model_config::ModelProfile>, String> {
            Ok(vec![])
        }
    }
}