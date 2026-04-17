//! Sprite sheet slicing worker.
//!
//! Handles slicing of sprite sheet images into individual frames.

use std::collections::VecDeque;
use std::path::PathBuf;

use artifex_job_queue::Job;
use artifex_shared_kernel::AppError;
#[allow(unused_imports)]
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

use super::traits::{JobFuture, JobResult, JobWorker};

/// Payload for sprite slice jobs.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SliceOperation {
    pub source_file_path: String,
    pub mode: SliceMode,
    pub grid_params: GridSliceParams,
    pub auto_detect_params: AutoDetectSliceParams,
    pub source_asset_id: String,
    pub project_id: String,
}

/// Slice mode for sprite sheet slicing.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum SliceMode {
    #[default]
    Grid,
    AutoDetect,
}

/// Grid parameters for grid-based sprite sheet slicing.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GridSliceParams {
    #[serde(default = "default_rows")]
    pub rows: u32,
    #[serde(default = "default_cols")]
    pub cols: u32,
    #[serde(default)]
    pub margin: u32,
}

impl Default for GridSliceParams {
    fn default() -> Self {
        Self {
            rows: default_rows(),
            cols: default_cols(),
            margin: 0,
        }
    }
}

fn default_rows() -> u32 {
    4
}

fn default_cols() -> u32 {
    4
}

/// Auto-detect parameters for content-aware sprite sheet slicing.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDetectSliceParams {
    #[serde(default = "default_min_area")]
    pub min_area: u32,
    #[serde(default)]
    pub sort_order: SortOrder,
}

impl Default for AutoDetectSliceParams {
    fn default() -> Self {
        Self {
            min_area: default_min_area(),
            sort_order: SortOrder::default(),
        }
    }
}

fn default_min_area() -> u32 {
    100
}

/// Sort order for auto-detect slicing.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum SortOrder {
    LeftToRight,
    #[default]
    TopToBottom,
}

/// Worker for sprite sheet slicing jobs.
pub struct SliceWorker {
    /// Base directory for saving output assets.
    assets_dir: String,
}

impl SliceWorker {
    /// Creates a new SliceWorker.
    pub fn new(assets_dir: String) -> Self {
        Self { assets_dir }
    }

    /// Main processing function for sprite slicing.
    async fn process_slice_job(
        &self,
        _job_id: artifex_shared_kernel::JobId,
        project_id: artifex_shared_kernel::ProjectId,
        payload: SliceOperation,
    ) -> Result<JobResult, AppError> {
        // Extract source file stem for naming output directory
        let source_stem = std::path::Path::new(&payload.source_file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("sprite")
            .to_string();

        // Create output directory: <assets_dir>/<project_id>/<source_stem>_frames/
        let output_dir = PathBuf::from(&self.assets_dir)
            .join(project_id.into_uuid().to_string())
            .join(format!("{}_frames", source_stem));

        tokio::fs::create_dir_all(&output_dir)
            .await
            .map_err(|e| AppError::io_error(format!("Failed to create output directory: {}", e)))?;

        // Load source image
        let source_img = image::open(&payload.source_file_path)
            .map_err(|e| AppError::internal(format!("Failed to open source image {}: {}", payload.source_file_path, e)))?;

        let (img_width, img_height) = source_img.dimensions();

        // Perform slicing based on mode
        let frames = match payload.mode {
            SliceMode::Grid => self.slice_grid(&source_img, &payload.grid_params)?,
            SliceMode::AutoDetect => self.slice_auto_detect(&source_img, &payload.auto_detect_params)?,
        };

        // Save each frame
        let mut output_files = Vec::new();
        let mut frame_infos = Vec::new();

        for (index, frame_info) in frames.iter().enumerate() {
            let frame_filename = format!("frame_{:03}.png", index);
            let frame_path = output_dir.join(&frame_filename);

            frame_info.image
                .save(&frame_path)
                .map_err(|e| AppError::internal(format!("Failed to save frame {}: {}", frame_filename, e)))?;

            output_files.push(frame_path.clone());
            frame_infos.push(FrameInfo {
                index,
                path: frame_filename,
                bbox: frame_info.bbox.clone(),
            });
        }

        // Generate manifest
        let manifest_filename = format!("{}_manifest.json", source_stem);
        let manifest_path = output_dir.join(&manifest_filename);
        let manifest = Manifest {
            source_asset_id: payload.source_asset_id.clone(),
            source_width: img_width,
            source_height: img_height,
            mode: payload.mode,
            frame_count: frames.len(),
            frames: frame_infos,
        };

        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| AppError::internal(format!("Failed to serialize manifest: {}", e)))?;

        // Write manifest as sidecar file (NOT registered as asset)
        tokio::fs::write(&manifest_path, manifest_json)
            .await
            .map_err(|e| AppError::io_error(format!("Failed to write manifest: {}", e)))?;

        tracing::info!(
            "Sliced sprite sheet into {} frames (mode: {:?})",
            frames.len(),
            payload.mode
        );

        // Build metadata
        let metadata = serde_json::json!({
            "operation": "sprite_slice",
            "source_asset_id": payload.source_asset_id,
            "frame_count": frames.len(),
            "mode": format!("{:?}", payload.mode).to_lowercase(),
            "manifest_path": manifest_filename,
            "project_id": project_id.into_uuid().to_string(),
        });

        Ok(JobResult::with_metadata(output_files, metadata))
    }

    /// Slices the image using grid mode.
    fn slice_grid(
        &self,
        source_img: &DynamicImage,
        params: &GridSliceParams,
    ) -> Result<Vec<FrameWithBBox>, AppError> {
        let (img_width, img_height) = source_img.dimensions();

        if params.rows == 0 || params.cols == 0 {
            return Err(AppError::validation("Rows and cols must be greater than 0".to_string()));
        }

        let cell_width = img_width / params.cols;
        let cell_height = img_height / params.rows;

        if cell_width == 0 || cell_height == 0 {
            return Err(AppError::validation(format!(
                "Cell dimensions too small: {}x{} (image: {}x{}, grid: {}x{})",
                cell_width, cell_height, img_width, img_height, params.cols, params.rows
            )));
        }

        let mut frames = Vec::new();

        for row in 0..params.rows {
            for col in 0..params.cols {
                let x = col * cell_width + params.margin;
                let y = row * cell_height + params.margin;
                let w = cell_width.saturating_sub(2 * params.margin);
                let h = cell_height.saturating_sub(2 * params.margin);

                if w == 0 || h == 0 {
                    continue;
                }

                // Crop the cell
                let cell = source_img.crop_imm(x, y, w, h);

                // Skip fully transparent cells
                if is_fully_transparent(&cell) {
                    continue;
                }

                frames.push(FrameWithBBox {
                    image: cell,
                    bbox: BoundingBox { x, y, w, h },
                });
            }
        }

        Ok(frames)
    }

    /// Slices the image using auto-detect mode.
    fn slice_auto_detect(
        &self,
        source_img: &DynamicImage,
        params: &AutoDetectSliceParams,
    ) -> Result<Vec<FrameWithBBox>, AppError> {
        // Build alpha mask
        let alpha_mask = build_alpha_mask(source_img);

        // BFS labeling to find connected components
        let labels = bfs_labeling(&alpha_mask);

        // Compute bounding boxes and filter by min_area
        let mut components: Vec<BoundingBox> = labels
            .into_iter()
            .filter_map(|(_, bbox)| {
                let area = bbox.w * bbox.h;
                if area >= params.min_area {
                    Some(bbox)
                } else {
                    None
                }
            })
            .collect();

        // Sort components
        match params.sort_order {
            SortOrder::LeftToRight => {
                components.sort_by_key(|b| b.x);
            }
            SortOrder::TopToBottom => {
                components.sort_by_key(|b| b.y * 10000 + b.x);
            }
        }

        // Crop frames based on detected bounding boxes
        let mut frames = Vec::new();
        for bbox in components {
            let cell = source_img.crop_imm(bbox.x, bbox.y, bbox.w, bbox.h);
            frames.push(FrameWithBBox {
                image: cell,
                bbox,
            });
        }

        Ok(frames)
    }
}

impl JobWorker for SliceWorker {
    fn can_handle(&self, job_type: &str) -> bool {
        job_type == "sprite_slice"
    }

    fn process(&self, job: &Job) -> JobFuture {
        let assets_dir = self.assets_dir.clone();
        let job_id = job.id;
        let project_id = job.project_id;
        let operation = job.operation.clone();

        Box::pin(async move {
            // Deserialize operation JSON
            let payload: SliceOperation = serde_json::from_value(operation)
                .map_err(|e| AppError::validation(format!("Invalid slice operation payload: {}", e)))?;

            tracing::info!(
                "SliceWorker processing job {} for project {}",
                job_id.into_uuid(),
                project_id.into_uuid()
            );

            let worker = SliceWorker::new(assets_dir);
            worker.process_slice_job(job_id, project_id, payload).await
        })
    }
}

// ============================================================================
// Slicing Logic
// ============================================================================

/// Frame with its bounding box.
#[derive(Debug, Clone)]
struct FrameWithBBox {
    image: DynamicImage,
    bbox: BoundingBox,
}

/// Bounding box for a frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// Frame info for manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FrameInfo {
    index: usize,
    path: String,
    bbox: BoundingBox,
}

/// Manifest structure for sliced sprite sheet.
#[derive(Debug, Serialize)]
struct Manifest {
    source_asset_id: String,
    source_width: u32,
    source_height: u32,
    mode: SliceMode,
    frame_count: usize,
    frames: Vec<FrameInfo>,
}

/// Checks if an image is fully transparent (alpha = 0 for all pixels).
fn is_fully_transparent(img: &DynamicImage) -> bool {
    let rgba = img.to_rgba8();
    for pixel in rgba.pixels() {
        if pixel[3] != 0 {
            return false;
        }
    }
    true
}

/// Builds an alpha mask from the source image.
/// Returns a 2D boolean grid where true = opaque (non-transparent) and false = transparent.
fn build_alpha_mask(img: &DynamicImage) -> Vec<Vec<bool>> {
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    let mut mask = vec![vec![false; width as usize]; height as usize];

    for y in 0..height {
        for x in 0..width {
            let pixel = rgba.get_pixel(x, y);
            mask[y as usize][x as usize] = pixel[3] != 0;
        }
    }

    mask
}

/// Performs connected-component labeling using BFS (4-connectivity).
/// Returns a vector of (label, bounding_box) for each component.
fn bfs_labeling(mask: &[Vec<bool>]) -> Vec<(u32, BoundingBox)> {
    let height = mask.len();
    if height == 0 {
        return vec![];
    }
    let width = mask[0].len();
    if width == 0 {
        return vec![];
    }

    let mut visited = vec![vec![false; width]; height];
    let mut components = Vec::new();
    let mut current_label: u32 = 1;

    // Directions: up, right, down, left (4-connectivity)
    let directions = [(0i32, -1), (1, 0), (0, 1), (-1, 0)];

    for y in 0..height {
        for x in 0..width {
            if mask[y][x] && !visited[y][x] {
                // Found a new component, do BFS
                let mut queue = VecDeque::new();
                queue.push_back((x as i32, y as i32));
                visited[y][x] = true;

                let mut min_x = x;
                let mut max_x = x;
                let mut min_y = y;
                let mut max_y = y;

                while let Some((cx, cy)) = queue.pop_front() {
                    // Update bounding box
                    min_x = min_x.min(cx as usize);
                    max_x = max_x.max(cx as usize);
                    min_y = min_y.min(cy as usize);
                    max_y = max_y.max(cy as usize);

                    // Explore neighbors (4-connectivity)
                    for &(dx, dy) in &directions {
                        let nx = cx + dx;
                        let ny = cy + dy;

                        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                            let ny = ny as usize;
                            let nx = nx as usize;
                            if mask[ny][nx] && !visited[ny][nx] {
                                visited[ny][nx] = true;
                                queue.push_back((nx as i32, ny as i32));
                            }
                        }
                    }
                }

                let bbox = BoundingBox {
                    x: min_x as u32,
                    y: min_y as u32,
                    w: (max_x - min_x + 1) as u32,
                    h: (max_y - min_y + 1) as u32,
                };

                components.push((current_label, bbox));
                current_label += 1;
            }
        }
    }

    components
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle() {
        let worker = SliceWorker::new("/tmp".to_string());
        assert!(worker.can_handle("sprite_slice"));
        assert!(!worker.can_handle("sprite_generate"));
        assert!(!worker.can_handle("image_generate"));
    }

    #[test]
    fn test_is_fully_transparent() {
        // Create a fully transparent 10x10 image
        let img = RgbaImage::new(10, 10);
        let dynamic = DynamicImage::ImageRgba8(img);
        assert!(is_fully_transparent(&dynamic));

        // Create a 10x10 image with one opaque pixel
        let mut img = RgbaImage::new(10, 10);
        img.put_pixel(5, 5, Rgba([0, 0, 0, 255]));
        let dynamic = DynamicImage::ImageRgba8(img);
        assert!(!is_fully_transparent(&dynamic));
    }

    #[test]
    fn test_build_alpha_mask() {
        // Create a 3x3 image with center pixel opaque
        let mut img = RgbaImage::new(3, 3);
        img.put_pixel(1, 1, Rgba([0, 0, 0, 255])); // Center opaque
        // Other pixels are transparent (alpha = 0)

        let dynamic = DynamicImage::ImageRgba8(img);
        let mask = build_alpha_mask(&dynamic);

        assert_eq!(mask.len(), 3);
        assert_eq!(mask[0].len(), 3);
        assert!(!mask[0][0]);
        assert!(!mask[0][1]);
        assert!(!mask[0][2]);
        assert!(!mask[1][0]);
        assert!(mask[1][1]); // Center is opaque
        assert!(!mask[1][2]);
        assert!(!mask[2][0]);
        assert!(!mask[2][1]);
        assert!(!mask[2][2]);
    }

    #[test]
    fn test_bfs_labeling_single_component() {
        // 3x3 with center pixel opaque
        let mask = vec![
            vec![false, false, false],
            vec![false, true, false],
            vec![false, false, false],
        ];

        let components = bfs_labeling(&mask);
        assert_eq!(components.len(), 1);
        let (_, bbox) = &components[0];
        assert_eq!(bbox.x, 1);
        assert_eq!(bbox.y, 1);
        assert_eq!(bbox.w, 1);
        assert_eq!(bbox.h, 1);
    }

    #[test]
    fn test_bfs_labeling_multiple_components() {
        // 5x5 with two separate 2x2 blocks
        let mask = vec![
            vec![true, true, false, false, false],
            vec![true, true, false, false, false],
            vec![false, false, false, true, true],
            vec![false, false, false, true, true],
            vec![false, false, false, false, false],
        ];

        let components = bfs_labeling(&mask);
        assert_eq!(components.len(), 2);

        // Sort by x for consistent testing
        let mut sorted = components.clone();
        sorted.sort_by_key(|(label, _)| *label);

        let (_, bbox1) = &sorted[0];
        assert_eq!(bbox1.x, 0);
        assert_eq!(bbox1.y, 0);
        assert_eq!(bbox1.w, 2);
        assert_eq!(bbox1.h, 2);

        let (_, bbox2) = &sorted[1];
        assert_eq!(bbox2.x, 3);
        assert_eq!(bbox2.y, 2);
        assert_eq!(bbox2.w, 2);
        assert_eq!(bbox2.h, 2);
    }

    #[test]
    fn test_bfs_labeling_empty() {
        let mask: Vec<Vec<bool>> = vec![];
        let components = bfs_labeling(&mask);
        assert!(components.is_empty());

        let mask = vec![vec![false, false], vec![false, false]];
        let components = bfs_labeling(&mask);
        assert!(components.is_empty());
    }

    #[test]
    fn test_bfs_labeling_l_shaped() {
        // L-shaped component
        let mask = vec![
            vec![true, false, false],
            vec![true, false, false],
            vec![true, true, false],
        ];

        let components = bfs_labeling(&mask);
        assert_eq!(components.len(), 1);
        let (_, bbox) = &components[0];
        // Bounding box should encompass the entire L shape
        assert_eq!(bbox.x, 0);
        assert_eq!(bbox.y, 0);
        assert_eq!(bbox.w, 2);
        assert_eq!(bbox.h, 3);
    }

    #[test]
    fn test_grid_slice_basic() {
        let worker = SliceWorker::new("/tmp".to_string());

        // Create a 100x100 red image
        let mut img = RgbaImage::new(100, 100);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([255, 0, 0, 255]);
        }
        let source = DynamicImage::ImageRgba8(img);

        let params = GridSliceParams {
            rows: 2,
            cols: 2,
            margin: 0,
        };

        let frames = worker.slice_grid(&source, &params).unwrap();
        assert_eq!(frames.len(), 4);

        // All frames should be 50x50
        for frame in &frames {
            assert_eq!(frame.bbox.w, 50);
            assert_eq!(frame.bbox.h, 50);
        }
    }

    #[test]
    fn test_grid_slice_with_margin() {
        let worker = SliceWorker::new("/tmp".to_string());

        // Create a 100x100 red image
        let mut img = RgbaImage::new(100, 100);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([255, 0, 0, 255]);
        }
        let source = DynamicImage::ImageRgba8(img);

        let params = GridSliceParams {
            rows: 2,
            cols: 2,
            margin: 10,
        };

        let frames = worker.slice_grid(&source, &params).unwrap();
        assert_eq!(frames.len(), 4);

        // All frames should be 30x30 (50 - 2*10)
        for frame in &frames {
            assert_eq!(frame.bbox.w, 30);
            assert_eq!(frame.bbox.h, 30);
        }
    }

    #[test]
    fn test_grid_slice_skips_transparent() {
        let worker = SliceWorker::new("/tmp".to_string());

        // Create a 100x100 image where top-left quadrant is transparent
        let mut img = RgbaImage::new(100, 100);
        // Top-left quadrant (0-49, 0-49) - transparent
        for y in 0..50 {
            for x in 0..50 {
                img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            }
        }
        // Top-right quadrant (50-99, 0-49) - red
        for y in 0..50 {
            for x in 50..100 {
                img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        // Bottom-left quadrant (0-49, 50-99) - red
        for y in 50..100 {
            for x in 0..50 {
                img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        // Bottom-right quadrant (50-99, 50-99) - red
        for y in 50..100 {
            for x in 50..100 {
                img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        let source = DynamicImage::ImageRgba8(img);

        let params = GridSliceParams {
            rows: 2,
            cols: 2,
            margin: 0,
        };

        let frames = worker.slice_grid(&source, &params).unwrap();
        // Should skip the transparent top-left cell
        assert_eq!(frames.len(), 3);
    }

    #[test]
    fn test_auto_detect_single_sprite() {
        let worker = SliceWorker::new("/tmp".to_string());

        // Create a 100x100 image with a 30x30 red sprite at (10, 10)
        let mut img = RgbaImage::new(100, 100);
        // Make all transparent first
        for pixel in img.pixels_mut() {
            *pixel = Rgba([0, 0, 0, 0]);
        }
        // Draw a red rectangle at (10,10) size 30x30
        for y in 10..40 {
            for x in 10..40 {
                img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        let source = DynamicImage::ImageRgba8(img);

        let params = AutoDetectSliceParams {
            min_area: 100,
            sort_order: SortOrder::TopToBottom,
        };

        let frames = worker.slice_auto_detect(&source, &params).unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].bbox.x, 10);
        assert_eq!(frames[0].bbox.y, 10);
        assert_eq!(frames[0].bbox.w, 30);
        assert_eq!(frames[0].bbox.h, 30);
    }

    #[test]
    fn test_auto_detect_multiple_sprites() {
        let worker = SliceWorker::new("/tmp".to_string());

        // Create a 200x100 image with two 30x30 sprites
        let mut img = RgbaImage::new(200, 100);
        // Make all transparent first
        for pixel in img.pixels_mut() {
            *pixel = Rgba([0, 0, 0, 0]);
        }
        // Draw first sprite at (10, 10)
        for y in 10..40 {
            for x in 10..40 {
                img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        // Draw second sprite at (100, 10)
        for y in 10..40 {
            for x in 100..130 {
                img.put_pixel(x, y, Rgba([0, 255, 0, 255]));
            }
        }
        let source = DynamicImage::ImageRgba8(img);

        let params = AutoDetectSliceParams {
            min_area: 100,
            sort_order: SortOrder::LeftToRight,
        };

        let frames = worker.slice_auto_detect(&source, &params).unwrap();
        assert_eq!(frames.len(), 2);

        // First should be left-most (by x)
        assert_eq!(frames[0].bbox.x, 10);
        assert_eq!(frames[1].bbox.x, 100);
    }

    #[test]
    fn test_auto_detect_filters_small() {
        let worker = SliceWorker::new("/tmp".to_string());

        // Create a 100x100 image with a tiny 5x5 sprite
        let mut img = RgbaImage::new(100, 100);
        // Make all transparent first
        for pixel in img.pixels_mut() {
            *pixel = Rgba([0, 0, 0, 0]);
        }
        // Draw a small 5x5 red rectangle at (10, 10)
        for y in 10..15 {
            for x in 10..15 {
                img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        let source = DynamicImage::ImageRgba8(img);

        let params = AutoDetectSliceParams {
            min_area: 100, // 5x5 = 25 < 100, should be filtered
            sort_order: SortOrder::TopToBottom,
        };

        let frames = worker.slice_auto_detect(&source, &params).unwrap();
        assert_eq!(frames.len(), 0);
    }

    #[test]
    fn test_slice_mode_default() {
        assert_eq!(SliceMode::default(), SliceMode::Grid);
    }

    #[test]
    fn test_sort_order_default() {
        assert_eq!(SortOrder::default(), SortOrder::TopToBottom);
    }

    #[test]
    fn test_grid_params_default() {
        let params = GridSliceParams::default();
        assert_eq!(params.rows, 4);
        assert_eq!(params.cols, 4);
        assert_eq!(params.margin, 0);
    }

    #[test]
    fn test_auto_detect_params_default() {
        let params = AutoDetectSliceParams::default();
        assert_eq!(params.min_area, 100);
        assert_eq!(params.sort_order, SortOrder::TopToBottom);
    }

    #[test]
    fn test_bounding_box_serialization() {
        let bbox = BoundingBox {
            x: 10,
            y: 20,
            w: 30,
            h: 40,
        };
        let json = serde_json::to_string(&bbox).unwrap();
        assert!(json.contains("\"x\":10"));
        assert!(json.contains("\"y\":20"));
        assert!(json.contains("\"w\":30"));
        assert!(json.contains("\"h\":40"));
    }
}
