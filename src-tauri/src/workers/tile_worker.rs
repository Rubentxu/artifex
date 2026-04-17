//! Tile generation worker.
//!
//! Handles seamless tile and texture generation jobs.

use std::path::PathBuf;
use std::sync::Arc;

use artifex_job_queue::Job;
use artifex_model_config::credential_store::CredentialStore;
use artifex_model_config::image_provider::ImageGenParams;
use artifex_model_config::ModelRouter;
use artifex_shared_kernel::AppError;
use serde::Deserialize;

use super::traits::{JobFuture, JobWorker};

/// Payload for tile generation jobs.
#[derive(Debug, Deserialize)]
struct TileGeneratePayload {
    prompt: String,
    width: u32,
    height: u32,
    biome: Option<String>,
    seamless: bool,
}

/// Worker for tile generation jobs.
pub struct TileWorker {
    /// Model router for resolving providers and fallback chain.
    router: Arc<ModelRouter>,
    /// Credential store for API keys.
    credential_store: Arc<dyn CredentialStore>,
    /// Base directory for saving output assets.
    assets_dir: String,
}

impl TileWorker {
    /// Creates a new TileWorker.
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

impl JobWorker for TileWorker {
    fn can_handle(&self, job_type: &str) -> bool {
        job_type == "tile_generate"
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
            let payload: TileGeneratePayload = serde_json::from_value(operation)
                .map_err(|e| AppError::validation(format!("Invalid tile generation payload: {}", e)))?;

            // Clone values we need later since ImageGenParams takes ownership
            let prompt_clone = payload.prompt.clone();
            let biome_clone = payload.biome.clone();

            // Build ImageGenParams for the provider
            let params = ImageGenParams {
                prompt: payload.prompt,
                negative_prompt: None,
                width: payload.width,
                height: payload.height,
                steps: 20, // Default steps for tile generation
                seed: None,
                num_images: 1,
                guidance_scale: 7.5,
                model_id: None,
            };

            // Resolve provider using routing key based on seamless flag
            let routing_key = if payload.seamless {
                "tilegen.seamless"
            } else {
                "tilegen.basic"
            };
            let resolved = router
                .resolve_image(routing_key)
                .await
                .map_err(|e| AppError::internal(format!("Failed to resolve tile provider: {}", e)))?;

            // Get credential
            let credential_id = format!("{}::api_key", resolved.profile.provider_name);
            let api_key = credential_store
                .get(&credential_id)
                .map_err(|_| AppError::internal(format!("Credential not found for {}", resolved.profile.provider_name)))?;

            // Call provider to generate tile
            let result = resolved
                .provider
                .generate(&params, &api_key)
                .await
                .map_err(|e| AppError::internal(format!("Provider error: {}", e)))?;

            // Build output path
            let output_dir = PathBuf::from(&assets_dir)
                .join(project_id.into_uuid().to_string())
                .join("images");

            tokio::fs::create_dir_all(&output_dir)
                .await
                .map_err(|e| AppError::io_error(format!("Failed to create output directory: {}", e)))?;

            let output_file = output_dir.join(format!("{}_tile.png", job_id.into_uuid()));

            // Save result
            tokio::fs::write(&output_file, &result.image_data)
                .await
                .map_err(|e| AppError::io_error(format!("Failed to write output file: {}", e)))?;

            // Calculate seam_score for tileability verification
            // This is a simplified check - compare edge pixels
            let seam_score = calculate_seamlessness(&result.image_data, result.width, result.height)?;

            // Return result with metadata
            Ok(super::traits::JobResult::with_metadata(
                vec![output_file.clone()],
                serde_json::json!({
                    "operation": "tile_generate",
                    "prompt": prompt_clone,
                    "biome": biome_clone,
                    "seamless": payload.seamless,
                    "width": result.width,
                    "height": result.height,
                    "format": result.format,
                    "model": resolved.profile.model_id,
                    "provider": resolved.profile.provider_name,
                    "seam_score": seam_score,
                    "project_id": project_id.into_uuid().to_string(),
                }),
            ))
        })
    }
}

/// Calculates a seamlessness score by comparing opposite edges.
/// Returns a value between 0.0 (not seamless) and 1.0 (perfectly seamless).
fn calculate_seamlessness(image_data: &[u8], _width: u32, _height: u32) -> Result<f32, AppError> {

    // Decode the image to check edges
    let img = image::load_from_memory(image_data)
        .map_err(|e| AppError::internal(format!("Failed to decode image for seam check: {}", e)))?;

    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();

    if w < 2 || h < 2 {
        return Ok(1.0); // Too small to check
    }

    // Take 1px strips from left and right edges
    let left_strip: Vec<f32> = (0..h)
        .map(|y| {
            let pixel = rgb.get_pixel(0, y);
            brightness(pixel[0], pixel[1], pixel[2])
        })
        .collect();

    let right_strip: Vec<f32> = (0..h)
        .map(|y| {
            let pixel = rgb.get_pixel(w - 1, y);
            brightness(pixel[0], pixel[1], pixel[2])
        })
        .collect();

    // Take 1px strips from top and bottom edges
    let top_strip: Vec<f32> = (0..w)
        .map(|x| {
            let pixel = rgb.get_pixel(x, 0);
            brightness(pixel[0], pixel[1], pixel[2])
        })
        .collect();

    let bottom_strip: Vec<f32> = (0..w)
        .map(|x| {
            let pixel = rgb.get_pixel(x, h - 1);
            brightness(pixel[0], pixel[1], pixel[2])
        })
        .collect();

    // Calculate mean difference for horizontal edges
    let h_diff: f32 = left_strip
        .iter()
        .zip(right_strip.iter())
        .map(|(l, r)| (l - r).abs())
        .sum::<f32>()
        / h as f32;

    // Calculate mean difference for vertical edges
    let v_diff: f32 = top_strip
        .iter()
        .zip(bottom_strip.iter())
        .map(|(t, b)| (t - b).abs())
        .sum::<f32>()
        / w as f32;

    let avg_diff = (h_diff + v_diff) / 2.0;

    // Convert to a seamlessness score (1.0 = perfect, 0.0 = very different)
    // If avg_diff < threshold (0.15), mark as seamless
    let score = if avg_diff < 0.15 {
        1.0 - (avg_diff / 0.15)
    } else {
        0.0
    };

    Ok(score)
}

/// Calculates the brightness of an RGB color.
fn brightness(r: u8, g: u8, b: u8) -> f32 {
    0.299 * (r as f32) + 0.587 * (g as f32) + 0.114 * (b as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle() {
        let worker = TileWorker::new(
            Arc::new(ModelRouter::new(
                Arc::new(artifex_model_config::ProviderRegistry::new()),
                Arc::new(TestRepo),
                Arc::new(artifex_model_config::credential_store::InMemoryCredentialStore::new()),
            )),
            Arc::new(artifex_model_config::credential_store::InMemoryCredentialStore::new()),
            "/tmp".to_string(),
        );

        assert!(worker.can_handle("tile_generate"));
        assert!(!worker.can_handle("image_generate"));
        assert!(!worker.can_handle("image_remove_background"));
    }

    #[test]
    fn test_seam_score_returns_value_between_0_and_1() {
        use image::{Rgb, RgbImage, ImageEncoder};

        // Create a small 4x4 RGB image with solid color (perfectly seamless)
        let mut img = RgbImage::new(4, 4);
        for pixel in img.pixels_mut() {
            *pixel = Rgb([128, 128, 128]);
        }

        // Encode to PNG bytes
        let mut png_bytes = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
        encoder.write_image(&img, 4, 4, image::ExtendedColorType::Rgb8).unwrap();

        // Calculate seamlessness score
        let score = calculate_seamlessness(&png_bytes, 4, 4).unwrap();

        // Score must be between 0.0 and 1.0 (inclusive)
        assert!(score >= 0.0 && score <= 1.0, "seam_score {} should be between 0.0 and 1.0", score);
    }

    #[test]
    fn test_seam_score_perfect_seam_is_high() {
        use image::{Rgb, RgbImage, ImageEncoder};

        // Create a small 4x4 RGB image with solid color (edges match perfectly)
        let mut img = RgbImage::new(4, 4);
        for pixel in img.pixels_mut() {
            *pixel = Rgb([100, 150, 200]);
        }

        // Encode to PNG bytes
        let mut png_bytes = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
        encoder.write_image(&img, 4, 4, image::ExtendedColorType::Rgb8).unwrap();

        // Calculate seamlessness score
        let score = calculate_seamlessness(&png_bytes, 4, 4).unwrap();

        // A perfectly seamless image (all edges identical) should have score close to 1.0
        assert!(score > 0.9, "Perfectly seamless image should have high score, got {}", score);
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