//! Seamless texture generation worker.
//!
//! Handles seamless texture generation jobs using mirror-padding algorithm.

use std::path::PathBuf;
use std::sync::Arc;

use artifex_job_queue::Job;
use artifex_model_config::credential_store::CredentialStore;
use artifex_model_config::image_provider::ImageGenParams;
use artifex_model_config::ModelRouter;
use artifex_shared_kernel::AppError;
use image::{DynamicImage, GenericImageView, Rgb, RgbImage};
use serde::Deserialize;

use super::traits::{JobFuture, JobResult, JobWorker};

/// Default seam threshold (MAE-based).
const DEFAULT_SEAM_THRESHOLD: f32 = 0.05;

/// Default padding pixels for mirror-padding.
const DEFAULT_PADDING_PIXELS: u32 = 16;

/// Payload for seamless texture generation jobs.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeamlessTexturePayload {
    source_asset_id: String,
    #[allow(dead_code)]
    secondary_asset_id: Option<String>,
    source_file_path: String,
    #[allow(dead_code)]
    secondary_file_path: Option<String>,
    mode: SeamlessMode,
    /// Image generation params (for FromPrompt mode).
    image_gen_params: Option<ImageGenParams>,
    /// Seam threshold (0.0-1.0). Default 0.05.
    seam_threshold: Option<f32>,
    /// Padding pixels for mirror-padding. Default 16.
    padding_pixels: Option<u32>,
    /// Blend fraction for overlap zones (0.0-1.0). Default 0.5.
    blend_fraction: Option<f32>,
}

/// Generation mode for seamless texture.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum SeamlessMode {
    /// Generate from prompt, then apply mirror-padding.
    FromPrompt,
    /// Process existing asset with mirror-padding.
    FromAsset,
}

/// Worker for seamless texture generation jobs.
pub struct SeamlessTextureWorker {
    /// Model router for resolving providers and fallback chain.
    router: Arc<ModelRouter>,
    /// Credential store for API keys.
    credential_store: Arc<dyn CredentialStore>,
    /// Base directory for saving output assets.
    assets_dir: String,
}

impl SeamlessTextureWorker {
    /// Creates a new SeamlessTextureWorker.
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

impl JobWorker for SeamlessTextureWorker {
    fn can_handle(&self, job_type: &str) -> bool {
        job_type == "seamless_texture"
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
            let payload: SeamlessTexturePayload = serde_json::from_value(operation)
                .map_err(|e| AppError::validation(format!("Invalid seamless texture payload: {}", e)))?;

            // Get parameters with defaults
            let seam_threshold = payload.seam_threshold.unwrap_or(DEFAULT_SEAM_THRESHOLD);
            let padding_pixels = payload.padding_pixels.unwrap_or(DEFAULT_PADDING_PIXELS);
            let blend_fraction = payload.blend_fraction.unwrap_or(0.5);

            // Process based on mode
            let result_image = match payload.mode {
                SeamlessMode::FromPrompt => {
                    // Generate image from prompt
                    let gen_params = payload.image_gen_params.ok_or_else(|| {
                        AppError::validation("FromPrompt mode requires image_gen_params".to_string())
                    })?;

                    // Resolve provider using routing key
                    let resolved = router
                        .resolve_image("imagegen.seamless")
                        .await
                        .map_err(|e| AppError::internal(format!("Failed to resolve seamless image provider: {}", e)))?;

                    // Get credential
                    let credential_id = format!("{}::api_key", resolved.profile.provider_name);
                    let api_key = credential_store
                        .get(&credential_id)
                        .map_err(|_| AppError::internal(format!("Credential not found for {}", resolved.profile.provider_name)))?;

                    // Generate image
                    let gen_result = resolved
                        .provider
                        .generate(&gen_params, &api_key)
                        .await
                        .map_err(|e| AppError::internal(format!("Provider error: {}", e)))?;

                    // Decode generated image
                    let img = image::load_from_memory(&gen_result.image_data)
                        .map_err(|e| AppError::internal(format!("Failed to decode generated image: {}", e)))?;

                    img
                }
                SeamlessMode::FromAsset => {
                    // Load existing image
                    let source_bytes = tokio::fs::read(&payload.source_file_path)
                        .await
                        .map_err(|e| AppError::io_error(format!("Failed to read source image: {}", e)))?;

                    image::load_from_memory(&source_bytes)
                        .map_err(|e| AppError::internal(format!("Failed to decode source image: {}", e)))?
                }
            };

            // Apply mirror-padding
            let padded = mirror_pad(&result_image, padding_pixels, blend_fraction);

            // Verify seams
            let seam_result = verify_seam(&padded, seam_threshold);

            // Build output path
            let output_dir = PathBuf::from(&assets_dir)
                .join(project_id.into_uuid().to_string())
                .join("images");

            tokio::fs::create_dir_all(&output_dir)
                .await
                .map_err(|e| AppError::io_error(format!("Failed to create output directory: {}", e)))?;

            let output_file = output_dir.join(format!("{}_seamless.png", job_id.into_uuid()));

            // Save result as PNG
            padded
                .save(&output_file)
                .map_err(|e| AppError::internal(format!("Failed to save seamless texture: {}", e)))?;

            // Return result with metadata
            Ok(JobResult::with_metadata(
                vec![output_file.clone()],
                serde_json::json!({
                    "operation": "seamless_texture",
                    "mode": format!("{:?}", payload.mode),
                    "source_asset_id": payload.source_asset_id,
                    "seam_threshold": seam_threshold,
                    "padding_pixels": padding_pixels,
                    "horizontal_mae": seam_result.horizontal_mae,
                    "vertical_mae": seam_result.vertical_mae,
                    "overall_mae": seam_result.overall_mae,
                    "passed": seam_result.passed,
                    "seam_warning": !seam_result.passed,
                    "project_id": project_id.into_uuid().to_string(),
                }),
            ))
        })
    }
}

/// Applies mirror-padding to create a seamless texture.
///
/// The algorithm:
/// 1. Expands canvas to 2W × 2H (50% padding each side)
/// 2. Places original centered at (W/2, H/2)
/// 3. Mirrors left/right/top/bottom edges
/// 4. Mirrors 4 corners
/// 5. Blends overlap zones (linear blend)
/// 6. Crops back to W × H
fn mirror_pad(img: &DynamicImage, padding_pixels: u32, blend_fraction: f32) -> DynamicImage {
    let (w, h) = img.dimensions();
    let pad = padding_pixels;

    // Output dimensions: original + 2*pad on each axis
    let out_w = w + 2 * pad;
    let out_h = h + 2 * pad;

    // Start with the original image in RGB format
    let rgb = img.to_rgb8();
    let mut out = RgbImage::new(out_w, out_h);

    // Step 1: Copy original centered
    for y in 0..h {
        for x in 0..w {
            out.put_pixel(pad + x, pad + y, *rgb.get_pixel(x, y));
        }
    }

    // Step 2: Mirror top edge (rows above original, reflect y)
    for y in 0..pad {
        let src_y = pad - 1 - y; // reflected row
        for x in 0..w {
            let pixel = *rgb.get_pixel(x, src_y);
            out.put_pixel(pad + x, pad - 1 - y, pixel);
        }
    }

    // Step 3: Mirror bottom edge (rows below original, reflect y)
    for y in 0..pad {
        let src_y = h - 1 - y; // reflected row
        for x in 0..w {
            let pixel = *rgb.get_pixel(x, src_y);
            out.put_pixel(pad + x, pad + h + y, pixel);
        }
    }

    // Step 4: Mirror left edge (columns left of original, reflect x)
    for y in 0..h {
        for x in 0..pad {
            let reflected_x = pad - 1 - x;
            if reflected_x < w {
                let pixel = *rgb.get_pixel(reflected_x, y);
                out.put_pixel(pad - 1 - x, pad + y, pixel);
            }
        }
    }

    // Step 5: Mirror right edge (columns right of original, reflect x)
    for y in 0..h {
        for x in 0..pad {
            let reflected_x = w - 1 - x;
            if reflected_x < w {
                let pixel = *rgb.get_pixel(reflected_x, y);
                out.put_pixel(pad + w + x, pad + y, pixel);
            }
        }
    }

    // Step 6: Mirror corners (both axes)
    // Top-left corner
    for y in 0..pad {
        for x in 0..pad {
            let src_x = pad - 1 - x;
            let src_y = pad - 1 - y;
            if src_x < w && src_y < h {
                let pixel = *rgb.get_pixel(src_x, src_y);
                out.put_pixel(pad - 1 - x, pad - 1 - y, pixel);
            }
        }
    }

    // Top-right corner
    for y in 0..pad {
        for x in 0..pad {
            let src_x = w - 1 - x;
            let src_y = pad - 1 - y;
            if src_x < w && src_y < h {
                let pixel = *rgb.get_pixel(src_x, src_y);
                out.put_pixel(pad + w + x, pad - 1 - y, pixel);
            }
        }
    }

    // Bottom-left corner
    for y in 0..pad {
        for x in 0..pad {
            let src_x = pad - 1 - x;
            let src_y = h - 1 - y;
            if src_x < w && src_y < h {
                let pixel = *rgb.get_pixel(src_x, src_y);
                out.put_pixel(pad - 1 - x, pad + h + y, pixel);
            }
        }
    }

    // Bottom-right corner
    for y in 0..pad {
        for x in 0..pad {
            let src_x = w - 1 - x;
            let src_y = h - 1 - y;
            if src_x < w && src_y < h {
                let pixel = *rgb.get_pixel(src_x, src_y);
                out.put_pixel(pad + w + x, pad + h + y, pixel);
            }
        }
    }

    // Step 7: Blend overlap zones (linear blend)
    // The overlap zones are at the edges of the original image where mirrored regions meet
    // We apply a simple linear blend in these zones
    blend_overlap_zones(&mut out, w, h, pad, blend_fraction);

    // Step 8: Crop back to original dimensions (center crop)
    let mut cropped = RgbImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            cropped.put_pixel(x, y, *out.get_pixel(pad + x, pad + y));
        }
    }

    DynamicImage::ImageRgb8(cropped)
}

/// Blends overlap zones between original and mirrored regions using linear blend.
fn blend_overlap_zones(img: &mut RgbImage, w: u32, h: u32, pad: u32, blend_fraction: f32) {
    // Blend zone size (half of padding)
    let blend_zone = (pad as f32 * blend_fraction).ceil() as u32;

    if blend_zone == 0 {
        return;
    }

    // Horizontal blend zones (left and right edges of original)
    for y in 0..h {
        for bx in 0..blend_zone {
            let left_weight = bx as f32 / blend_zone as f32;
            let right_weight = 1.0 - left_weight;

            // Left blend zone: blend original left edge with mirrored right edge
            let orig_x = pad + bx;
            let mir_x = pad + w - 1 - bx;

            if orig_x < pad + w && mir_x < img.width() {
                let orig_pixel = *img.get_pixel(orig_x, pad + y);
                let mir_pixel = *img.get_pixel(mir_x, pad + y);

                let blended = blend_pixels(&orig_pixel, &mir_pixel, right_weight);
                img.put_pixel(orig_x, pad + y, blended);
            }

            // Right blend zone: blend original right edge with mirrored left edge
            let orig_x = pad + w - 1 - bx;
            let mir_x = pad + bx;

            if orig_x >= pad && orig_x < pad + w && mir_x < pad + w {
                let orig_pixel = *img.get_pixel(orig_x, pad + y);
                let mir_pixel = *img.get_pixel(mir_x, pad + y);

                let blended = blend_pixels(&orig_pixel, &mir_pixel, left_weight);
                img.put_pixel(orig_x, pad + y, blended);
            }
        }
    }

    // Vertical blend zones (top and bottom edges of original)
    for x in 0..w {
        for by in 0..blend_zone {
            let top_weight = by as f32 / blend_zone as f32;
            let bottom_weight = 1.0 - top_weight;

            // Top blend zone: blend original top edge with mirrored bottom edge
            let orig_y = pad + by;
            let mir_y = pad + h - 1 - by;

            if orig_y < pad + h && mir_y < img.height() {
                let orig_pixel = *img.get_pixel(pad + x, orig_y);
                let mir_pixel = *img.get_pixel(pad + x, mir_y);

                let blended = blend_pixels(&orig_pixel, &mir_pixel, bottom_weight);
                img.put_pixel(pad + x, orig_y, blended);
            }

            // Bottom blend zone: blend original bottom edge with mirrored top edge
            let orig_y = pad + h - 1 - by;
            let mir_y = pad + by;

            if orig_y >= pad && orig_y < pad + h && mir_y < pad + h {
                let orig_pixel = *img.get_pixel(pad + x, orig_y);
                let mir_pixel = *img.get_pixel(pad + x, mir_y);

                let blended = blend_pixels(&orig_pixel, &mir_pixel, top_weight);
                img.put_pixel(pad + x, orig_y, blended);
            }
        }
    }
}

/// Blends two pixels with given weight for the second pixel.
fn blend_pixels(p1: &Rgb<u8>, p2: &Rgb<u8>, weight: f32) -> Rgb<u8> {
    let w1 = 1.0 - weight;
    Rgb([
        ((p1[0] as f32 * w1) + (p2[0] as f32 * weight)).round() as u8,
        ((p1[1] as f32 * w1) + (p2[1] as f32 * weight)).round() as u8,
        ((p1[2] as f32 * w1) + (p2[2] as f32 * weight)).round() as u8,
    ])
}

/// Seam verification result.
struct SeamResult {
    /// Horizontal MAE (left vs right edges).
    horizontal_mae: f32,
    /// Vertical MAE (top vs bottom edges).
    vertical_mae: f32,
    /// Overall MAE (max of horizontal and vertical).
    overall_mae: f32,
    /// Whether the seam passes the threshold.
    passed: bool,
}

/// Verifies seam quality by comparing opposite edges.
///
/// Sample 4px-wide strips at all edges and compute per-channel MAE
/// between opposite edges.
fn verify_seam(img: &DynamicImage, threshold: f32) -> SeamResult {
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();

    // Strip width for comparison
    let strip_width = 4;

    if w < strip_width * 2 || h < strip_width * 2 {
        // Image too small to verify
        return SeamResult {
            horizontal_mae: 0.0,
            vertical_mae: 0.0,
            overall_mae: 0.0,
            passed: true,
        };
    }

    // Horizontal seam: compare left strip with horizontally-flipped right strip
    let mut h_mae_sum = 0.0_f32;
    let strip_area_h = (strip_width * h) as f32;

    for y in 0..h {
        for x in 0..strip_width {
            let left_pixel = rgb.get_pixel(x, y);
            let right_pixel = rgb.get_pixel(w - 1 - x, y);
            h_mae_sum += per_channel_mae(left_pixel, right_pixel);
        }
    }
    let horizontal_mae = h_mae_sum / strip_area_h;

    // Vertical seam: compare top strip with vertically-flipped bottom strip
    let mut v_mae_sum = 0.0_f32;
    let strip_area_v = (strip_width * w) as f32;

    for y in 0..strip_width {
        for x in 0..w {
            let top_pixel = rgb.get_pixel(x, y);
            let bottom_pixel = rgb.get_pixel(x, h - 1 - y);
            v_mae_sum += per_channel_mae(top_pixel, bottom_pixel);
        }
    }
    let vertical_mae = v_mae_sum / strip_area_v;

    let overall_mae = horizontal_mae.max(vertical_mae);
    let passed = overall_mae <= threshold;

    SeamResult {
        horizontal_mae,
        vertical_mae,
        overall_mae,
        passed,
    }
}

/// Computes per-channel MAE between two pixels.
fn per_channel_mae(a: &Rgb<u8>, b: &Rgb<u8>) -> f32 {
    ((a[0] as f32 - b[0] as f32).abs()
     + (a[1] as f32 - b[1] as f32).abs()
     + (a[2] as f32 - b[2] as f32).abs()) / 3.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle() {
        let worker = SeamlessTextureWorker::new(
            Arc::new(ModelRouter::new(
                Arc::new(artifex_model_config::ProviderRegistry::new()),
                Arc::new(TestRepo),
                Arc::new(artifex_model_config::credential_store::InMemoryCredentialStore::new()),
            )),
            Arc::new(artifex_model_config::credential_store::InMemoryCredentialStore::new()),
            "/tmp".to_string(),
        );

        assert!(worker.can_handle("seamless_texture"));
        assert!(!worker.can_handle("tile_generate"));
        assert!(!worker.can_handle("image_generate"));
    }

    #[test]
    fn test_verify_seam_perfect_seam() {
        // Create a small 8x8 RGB image with solid color (edges match perfectly)
        let mut img = RgbImage::new(8, 8);
        for pixel in img.pixels_mut() {
            *pixel = Rgb([100, 150, 200]);
        }

        let result = verify_seam(&DynamicImage::ImageRgb8(img), 0.05);

        // Perfect seam should have MAE = 0
        assert_eq!(result.horizontal_mae, 0.0);
        assert_eq!(result.vertical_mae, 0.0);
        assert!(result.passed);
    }

    #[test]
    fn test_verify_seam_poor_seam() {
        // Create a small 8x8 RGB image with different edge colors
        let mut img = RgbImage::new(8, 8);
        for y in 0..8 {
            for x in 0..8 {
                if x < 4 {
                    img.put_pixel(x, y, Rgb([100, 150, 200]));
                } else {
                    img.put_pixel(x, y, Rgb([50, 100, 150]));
                }
            }
        }

        let result = verify_seam(&DynamicImage::ImageRgb8(img), 0.05);

        // Different edges should have higher MAE
        assert!(result.horizontal_mae > 0.0);
        assert!(!result.passed);
    }

    #[test]
    fn test_blend_pixels() {
        let p1 = Rgb([100, 100, 100]);
        let p2 = Rgb([200, 200, 200]);

        let blended = blend_pixels(&p1, &p2, 0.5);

        // 50% blend should give average
        assert_eq!(blended[0], 150);
        assert_eq!(blended[1], 150);
        assert_eq!(blended[2], 150);
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