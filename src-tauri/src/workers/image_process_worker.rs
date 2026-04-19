//! Image processing worker.
//!
//! Handles background removal and pixel art conversion jobs.

use std::path::PathBuf;
use std::sync::Arc;

use artifex_job_queue::Job;
use artifex_model_config::credential_store::CredentialStore;
use artifex_model_config::image_provider::ImageEditParams;
use artifex_model_config::ModelRouter;
use artifex_shared_kernel::AppError;
use image::{Rgb, RgbImage};
use serde::Deserialize;

use super::traits::{JobFuture, JobResult, JobWorker, WorkerCategory};

/// Payload for background removal jobs.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RemoveBackgroundPayload {
    source_asset_id: String,
    source_file_path: String,
    provider_mode: Option<String>,
}

/// Payload for pixel art conversion jobs.
#[derive(Debug, Deserialize)]
struct PixelArtPayload {
    source_asset_id: String,
    source_file_path: String,
    target_width: u32,
    target_height: u32,
    palette: PaletteMode,
    dithering: DitheringMode,
    outline: bool,
    outline_threshold: u8,
}

/// Palette mode for pixel art conversion.
#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize)]
enum PaletteMode {
    Pico8,
    GameBoy,
    Nes,
    Custom,
}

/// Dithering mode for pixel art conversion.
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Deserialize)]
enum DitheringMode {
    #[default]
    None,
    FloydSteinberg,
    Bayer,
    Atkinson,
}

/// Direction for outpainting canvas extension.
#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum OutpaintDirection {
    Left,
    Right,
    Top,
    Bottom,
}

/// Payload for inpaint jobs.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct InpaintPayload {
    source_asset_id: String,
    source_file_path: String,
    mask_path: String,
    prompt: String,
    negative_prompt: Option<String>,
    strength: f32,
    guidance_scale: f32,
    steps: u32,
}

/// Payload for outpaint jobs.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OutpaintPayload {
    source_asset_id: String,
    source_file_path: String,
    direction: OutpaintDirection,
    extend_pixels: u32,
    prompt: String,
    negative_prompt: Option<String>,
    strength: f32,
    guidance_scale: f32,
    steps: u32,
}

// =============================================================================
// Predefined Palettes
// =============================================================================

/// Pico-8 palette (16 colors)
const PALETTE_PICO8: &[(u8, u8, u8)] = &[
    (0x00, 0x00, 0x00), // black
    (0x1D, 0x2B, 0x53), // dark-blue
    (0x7E, 0x25, 0x53), // dark-purple
    (0x00, 0x87, 0x51), // dark-green
    (0xAB, 0x52, 0x36), // brown
    (0x5F, 0x57, 0x4F), // dark-grey
    (0xC2, 0xC3, 0xC7), // light-grey
    (0xFF, 0xF1, 0xE8), // white
    (0xFF, 0x00, 0x4D), // red
    (0xFF, 0xA3, 0x00), // orange
    (0xFF, 0xEC, 0x27), // yellow
    (0x00, 0xE4, 0x36), // green
    (0x29, 0xAD, 0xFF), // blue
    (0x83, 0x76, 0x9C), // lavender
    (0xFF, 0x77, 0xA8), // pink
    (0xFF, 0xCC, 0xAA), // peach
];

/// GameBoy palette (4 colors - original green-tinted)
const PALETTE_GAMEBOY: &[(u8, u8, u8)] = &[
    (0x0F, 0x38, 0x0F), // darkest
    (0x30, 0x62, 0x30), // dark
    (0x8B, 0xAC, 0x0F), // light
    (0x9B, 0xBC, 0x0F), // lightest
];

/// NES palette (54 colors - every other color from the full 108 color palette)
const PALETTE_NES: &[(u8, u8, u8)] = &[
    (0x7C, 0x7C, 0x7C), (0x00, 0x00, 0xFC), (0x00, 0x00, 0xBC), (0x44, 0x28, 0xFC),
    (0x94, 0x00, 0xFC), (0xA8, 0x00, 0x7C), (0xA8, 0x10, 0x00), (0x88, 0x18, 0x00),
    (0x50, 0x30, 0x00), (0x00, 0x68, 0x00), (0x00, 0x58, 0x00), (0x00, 0x40, 0x40),
    (0x00, 0x00, 0xFC), (0x00, 0x00, 0xBC), (0x68, 0x54, 0xFC), (0xD8, 0xD8, 0xD8),
    (0x3C, 0xBC, 0xFC), (0x68, 0x88, 0xFC), (0x98, 0x78, 0xFC), (0xF8, 0x78, 0xF8),
    (0xF8, 0x58, 0x98), (0xF8, 0x38, 0x48), (0xF8, 0x3C, 0x18), (0xF8, 0x5C, 0x00),
    (0xF8, 0x7C, 0x34), (0x54, 0xD8, 0x54), (0x74, 0xD8, 0x58), (0xD8, 0xD8, 0x78),
    (0xD8, 0xF8, 0x78), (0x58, 0xF8, 0xB8), (0x00, 0xFC, 0xD8), (0x00, 0xD8, 0xF8),
    (0x00, 0xBC, 0xFC), (0x00, 0x78, 0xE8), (0x30, 0x78, 0xFC), (0x80, 0x58, 0xFC),
    (0xF8, 0x58, 0xFC), (0xF8, 0x78, 0xD8), (0xF8, 0x98, 0xA8), (0xF8, 0xB8, 0x78),
    (0xFC, 0xE8, 0x48), (0xFC, 0xF8, 0x58), (0xFC, 0xFC, 0x78), (0x00, 0xFC, 0x58),
    (0x00, 0xD8, 0x00), (0x00, 0xBC, 0x00), (0x00, 0xA8, 0x00), (0x00, 0x88, 0x00),
    (0x30, 0x30, 0xFC), (0x00, 0x00, 0xA8), (0x68, 0x00, 0x88), (0xA8, 0x00, 0x58),
    (0xA8, 0x10, 0x00), (0x88, 0x18, 0x00), (0x00, 0x38, 0x00), (0x00, 0x2C, 0x00),
];

/// Returns the palette for the given mode.
fn get_palette(mode: &PaletteMode) -> &'static [(u8, u8, u8)] {
    match mode {
        PaletteMode::Pico8 => PALETTE_PICO8,
        PaletteMode::GameBoy => PALETTE_GAMEBOY,
        PaletteMode::Nes => PALETTE_NES,
        PaletteMode::Custom => PALETTE_PICO8, // Default to Pico8 for custom
    }
}

/// Finds the nearest color in a palette to a given RGB color.
fn nearest_palette_color(r: u8, g: u8, b: u8, palette: &[(u8, u8, u8)]) -> (u8, u8, u8) {
    let mut best_idx = 0;
    let mut best_dist = u32::MAX;

    for (i, &color) in palette.iter().enumerate() {
        let dr = (r as i32 - color.0 as i32).unsigned_abs();
        let dg = (g as i32 - color.1 as i32).unsigned_abs();
        let db = (b as i32 - color.2 as i32).unsigned_abs();
        // Weighted Euclidean distance (human eye sensitivity)
        let dist = 2 * dr * dr + 4 * dg * dg + 3 * db * db;
        if dist < best_dist {
            best_dist = dist;
            best_idx = i;
        }
    }

    palette[best_idx]
}

/// Calculates the brightness of an RGB color.
fn brightness(r: u8, g: u8, b: u8) -> f32 {
    0.299 * (r as f32) + 0.587 * (g as f32) + 0.114 * (b as f32)
}

/// Worker for image processing jobs (background removal, pixel art conversion).
pub struct ImageProcessWorker {
    /// Model router for resolving providers and fallback chain.
    router: Arc<ModelRouter>,
    /// Credential store for API keys.
    credential_store: Arc<dyn CredentialStore>,
    /// Base directory for saving output assets.
    assets_dir: String,
}

impl ImageProcessWorker {
    /// Creates a new ImageProcessWorker.
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

impl JobWorker for ImageProcessWorker {
    fn can_handle(&self, job_type: &str) -> bool {
        matches!(
            job_type,
            "image_remove_background"
                | "pixel_art_convert"
                | "image_inpaint"
                | "image_outpaint"
        )
    }

    fn category(&self) -> WorkerCategory {
        WorkerCategory::CpuIntensive
    }

    fn process(&self, job: &Job) -> JobFuture {
        let router = self.router.clone();
        let credential_store = self.credential_store.clone();
        let assets_dir = self.assets_dir.clone();
        let job_type = job.job_type.clone();
        let job_id = job.id;
        let project_id = job.project_id;
        let operation = job.operation.clone();

        Box::pin(async move {
            match job_type.as_str() {
                "image_remove_background" => {
                    // Deserialize operation JSON
                    let payload: RemoveBackgroundPayload = serde_json::from_value(operation)
                        .map_err(|e| AppError::validation(format!("Invalid job payload: {}", e)))?;

                    // Read source image
                    let source_bytes = tokio::fs::read(&payload.source_file_path)
                        .await
                        .map_err(|e| AppError::io_error(format!("Failed to read source image: {}", e)))?;

                    // Resolve provider using routing key
                    let resolved = router
                        .resolve_image("imageproc.remove_bg")
                        .await
                        .map_err(|e| AppError::internal(format!("Failed to resolve provider: {}", e)))?;

                    // Get credential
                    let credential_id = format!("{}::api_key", resolved.profile.provider_name);
                    let api_key = credential_store
                        .get(&credential_id)
                        .map_err(|_| AppError::internal(format!("Credential not found for {}", resolved.profile.provider_name)))?;

                    // Call provider to remove background
                    let result = resolved
                        .provider
                        .remove_background(&source_bytes, &api_key)
                        .await
                        .map_err(|e| AppError::internal(format!("Provider error: {}", e)))?;

                    // Build output path
                    let output_dir = PathBuf::from(&assets_dir)
                        .join(project_id.into_uuid().to_string())
                        .join("images");

                    tokio::fs::create_dir_all(&output_dir)
                        .await
                        .map_err(|e| AppError::io_error(format!("Failed to create output directory: {}", e)))?;

                    let output_file = output_dir.join(format!("{}_nobg.png", job_id.into_uuid()));

                    // Save result
                    tokio::fs::write(&output_file, &result.image_data)
                        .await
                        .map_err(|e| AppError::io_error(format!("Failed to write output file: {}", e)))?;

                    // Return result with metadata
                    Ok(JobResult::with_metadata(
                        vec![output_file.clone()],
                        serde_json::json!({
                            "operation": "remove_background",
                            "source_asset_id": payload.source_asset_id,
                            "provider": resolved.profile.provider_name,
                            "model": resolved.profile.model_id,
                            "project_id": project_id.into_uuid().to_string(),
                        }),
                    ))
                }
                "pixel_art_convert" => {
                    // Deserialize operation JSON
                    let payload: PixelArtPayload = serde_json::from_value(operation)
                        .map_err(|e| AppError::validation(format!("Invalid job payload: {}", e)))?;

                    // Read source image
                    let source_img = image::load_from_memory(
                        &tokio::fs::read(&payload.source_file_path).await.map_err(|e| {
                            AppError::io_error(format!("Failed to read source image: {}", e))
                        })?,
                    )
                    .map_err(|e| AppError::internal(format!("Failed to decode image: {}", e)))?;

                    // Resize to target dimensions using nearest neighbor
                    let resized = source_img.resize_exact(
                        payload.target_width,
                        payload.target_height,
                        image::imageops::FilterType::Nearest,
                    );

                    // Convert to RGB
                    let rgb_img = resized.to_rgb8();

                    // Get palette
                    let palette = get_palette(&payload.palette);

                    // Apply dithering or direct color mapping
                    let processed: RgbImage = if payload.dithering == DitheringMode::None {
                        // Direct color mapping without dithering
                        let (width, height) = rgb_img.dimensions();
                        let mut output = RgbImage::new(width, height);
                        for (x, y, pixel) in rgb_img.enumerate_pixels() {
                            let nearest = nearest_palette_color(pixel[0], pixel[1], pixel[2], palette);
                            output.put_pixel(x, y, Rgb([nearest.0, nearest.1, nearest.2]));
                        }
                        output
                    } else {
                        // Floyd-Steinberg dithering
                        apply_floyd_steinberg(&rgb_img, palette)
                    };

                    // Apply outline if requested
                    let final_img: RgbImage = if payload.outline {
                        apply_outline(&processed, payload.outline_threshold)
                    } else {
                        processed
                    };

                    // Build output path
                    let output_dir = PathBuf::from(&assets_dir)
                        .join(project_id.into_uuid().to_string())
                        .join("images");

                    tokio::fs::create_dir_all(&output_dir)
                        .await
                        .map_err(|e| AppError::io_error(format!("Failed to create output directory: {}", e)))?;

                    let output_file = output_dir.join(format!("{}_pixelart.png", job_id.into_uuid()));

                    // Save result as PNG
                    final_img
                        .save(&output_file)
                        .map_err(|e| AppError::internal(format!("Failed to save pixel art: {}", e)))?;

                    // Return result with metadata
                    Ok(JobResult::with_metadata(
                        vec![output_file.clone()],
                        serde_json::json!({
                            "operation": "pixel_art_convert",
                            "source_asset_id": payload.source_asset_id,
                            "target_size": {
                                "width": payload.target_width,
                                "height": payload.target_height,
                            },
                            "palette": format!("{:?}", payload.palette),
                            "dithering": format!("{:?}", payload.dithering),
                            "outline": payload.outline,
                            "project_id": project_id.into_uuid().to_string(),
                        }),
                    ))
                }
                "image_inpaint" => {
                    // Deserialize operation JSON
                    let payload: InpaintPayload = serde_json::from_value(operation)
                        .map_err(|e| AppError::validation(format!("Invalid job payload: {}", e)))?;

                    // Read source image and mask
                    let source_bytes = tokio::fs::read(&payload.source_file_path)
                        .await
                        .map_err(|e| AppError::io_error(format!("Failed to read source image: {}", e)))?;

                    let mask_bytes = tokio::fs::read(&payload.mask_path)
                        .await
                        .map_err(|e| AppError::io_error(format!("Failed to read mask image: {}", e)))?;

                    // Resolve provider using routing key
                    let resolved = router
                        .resolve_image("imageedit.inpaint")
                        .await
                        .map_err(|e| AppError::internal(format!("Failed to resolve provider: {}", e)))?;

                    // Get credential
                    let credential_id = format!("{}::api_key", resolved.profile.provider_name);
                    let api_key = credential_store
                        .get(&credential_id)
                        .map_err(|_| AppError::internal(format!("Credential not found for {}", resolved.profile.provider_name)))?;

                    // Build edit params
                    let edit_params = ImageEditParams {
                        prompt: payload.prompt,
                        negative_prompt: payload.negative_prompt,
                        strength: payload.strength,
                        guidance_scale: payload.guidance_scale,
                        num_inference_steps: payload.steps,
                        seed: None,
                        model_id: resolved.profile.model_id.clone().into(),
                    };

                    // Call provider to inpaint
                    let result = resolved
                        .provider
                        .inpaint(&source_bytes, &mask_bytes, &edit_params, &api_key)
                        .await
                        .map_err(|e| AppError::internal(format!("Provider error: {}", e)))?;

                    // Build output path
                    let output_dir = PathBuf::from(&assets_dir)
                        .join(project_id.into_uuid().to_string())
                        .join("images");

                    tokio::fs::create_dir_all(&output_dir)
                        .await
                        .map_err(|e| AppError::io_error(format!("Failed to create output directory: {}", e)))?;

                    let output_file = output_dir.join(format!("{}_inpainted.png", job_id.into_uuid()));

                    // Save result
                    tokio::fs::write(&output_file, &result.image_data)
                        .await
                        .map_err(|e| AppError::io_error(format!("Failed to write output file: {}", e)))?;

                    // Return result with metadata
                    Ok(JobResult::with_metadata(
                        vec![output_file.clone()],
                        serde_json::json!({
                            "operation": "inpaint",
                            "source_asset_id": payload.source_asset_id,
                            "provider": resolved.profile.provider_name,
                            "model": resolved.profile.model_id,
                            "project_id": project_id.into_uuid().to_string(),
                        }),
                    ))
                }
                "image_outpaint" => {
                    // Deserialize operation JSON
                    let payload: OutpaintPayload = serde_json::from_value(operation)
                        .map_err(|e| AppError::validation(format!("Invalid job payload: {}", e)))?;

                    use image::GenericImageView;

                    // Read source image
                    let source_bytes = tokio::fs::read(&payload.source_file_path)
                        .await
                        .map_err(|e| AppError::io_error(format!("Failed to read source image: {}", e)))?;

                    let source_img = image::load_from_memory(&source_bytes)
                        .map_err(|e| AppError::internal(format!("Failed to decode source image: {}", e)))?;

                    let (_src_width, _src_height) = source_img.dimensions();

                    // Expand canvas and create mask for new region
                    let (expanded_img, mask_img) = expand_canvas_with_mask(
                        &source_img,
                        &payload.direction,
                        payload.extend_pixels,
                    )?;

                    // Save expanded image and mask to temp files for the provider
                    let temp_dir = std::env::temp_dir();
                    let expanded_path = temp_dir.join(format!("{}_expanded.png", job_id.into_uuid()));
                    let mask_path = temp_dir.join(format!("{}_mask.png", job_id.into_uuid()));

                    expanded_img
                        .save(&expanded_path)
                        .map_err(|e| AppError::internal(format!("Failed to save expanded image: {}", e)))?;

                    mask_img
                        .save(&mask_path)
                        .map_err(|e| AppError::internal(format!("Failed to save mask: {}", e)))?;

                    // Read back the files
                    let expanded_bytes = tokio::fs::read(&expanded_path)
                        .await
                        .map_err(|e| AppError::io_error(format!("Failed to read expanded image: {}", e)))?;

                    let mask_bytes = tokio::fs::read(&mask_path)
                        .await
                        .map_err(|e| AppError::io_error(format!("Failed to read mask: {}", e)))?;

                    // Clean up temp files
                    let _ = tokio::fs::remove_file(&expanded_path).await;
                    let _ = tokio::fs::remove_file(&mask_path).await;

                    // Resolve provider using routing key
                    let resolved = router
                        .resolve_image("imageedit.outpaint")
                        .await
                        .map_err(|e| AppError::internal(format!("Failed to resolve provider: {}", e)))?;

                    // Get credential
                    let credential_id = format!("{}::api_key", resolved.profile.provider_name);
                    let api_key = credential_store
                        .get(&credential_id)
                        .map_err(|_| AppError::internal(format!("Credential not found for {}", resolved.profile.provider_name)))?;

                    // Build edit params
                    let edit_params = ImageEditParams {
                        prompt: payload.prompt,
                        negative_prompt: payload.negative_prompt,
                        strength: payload.strength,
                        guidance_scale: payload.guidance_scale,
                        num_inference_steps: payload.steps,
                        seed: None,
                        model_id: resolved.profile.model_id.clone().into(),
                    };

                    // Call provider to inpaint
                    let result = resolved
                        .provider
                        .inpaint(&expanded_bytes, &mask_bytes, &edit_params, &api_key)
                        .await
                        .map_err(|e| AppError::internal(format!("Provider error: {}", e)))?;

                    // Build output path
                    let output_dir = PathBuf::from(&assets_dir)
                        .join(project_id.into_uuid().to_string())
                        .join("images");

                    tokio::fs::create_dir_all(&output_dir)
                        .await
                        .map_err(|e| AppError::io_error(format!("Failed to create output directory: {}", e)))?;

                    let output_file = output_dir.join(format!("{}_outpainted.png", job_id.into_uuid()));

                    // Save result
                    tokio::fs::write(&output_file, &result.image_data)
                        .await
                        .map_err(|e| AppError::io_error(format!("Failed to write output file: {}", e)))?;

                    // Return result with metadata
                    Ok(JobResult::with_metadata(
                        vec![output_file.clone()],
                        serde_json::json!({
                            "operation": "outpaint",
                            "source_asset_id": payload.source_asset_id,
                            "direction": format!("{:?}", payload.direction),
                            "extend_px": payload.extend_pixels,
                            "provider": resolved.profile.provider_name,
                            "model": resolved.profile.model_id,
                            "project_id": project_id.into_uuid().to_string(),
                        }),
                    ))
                }
                _ => Err(AppError::validation(format!(
                    "Unknown job type for ImageProcessWorker: {}",
                    job_type
                ))),
            }
        })
    }
}

/// Applies Floyd-Steinberg dithering to an RGB image using the given palette.
fn apply_floyd_steinberg(img: &RgbImage, palette: &[(u8, u8, u8)]) -> RgbImage {
    let (width, height) = img.dimensions();
    let mut result = img.clone();
    let mut errors: Vec<Vec<(i32, i32, i32)>> = vec![vec![(0, 0, 0); width as usize]; height as usize];

    for y in 0..height {
        for x in 0..width {
            let pixel = *result.get_pixel(x, y);
            let old_r = pixel[0] as i32;
            let old_g = pixel[1] as i32;
            let old_b = pixel[2] as i32;

            // Add accumulated error
            let (err_r, err_g, err_b) = errors[y as usize][x as usize];
            let new_r = (old_r + err_r).clamp(0, 255) as u8;
            let new_g = (old_g + err_g).clamp(0, 255) as u8;
            let new_b = (old_b + err_b).clamp(0, 255) as u8;

            // Find nearest palette color
            let nearest = nearest_palette_color(new_r, new_g, new_b, palette);

            // Set the pixel to the nearest palette color
            result.put_pixel(x, y, Rgb([nearest.0, nearest.1, nearest.2]));

            // Calculate quantization error
            let quant_err_r = new_r as i32 - nearest.0 as i32;
            let quant_err_g = new_g as i32 - nearest.1 as i32;
            let quant_err_b = new_b as i32 - nearest.2 as i32;

            // Distribute error to neighboring pixels (Floyd-Steinberg pattern)
            // Right neighbor (1, 0) - 7/16
            if x + 1 < width {
                let e = &mut errors[y as usize][(x + 1) as usize];
                e.0 += quant_err_r * 7 / 16;
                e.1 += quant_err_g * 7 / 16;
                e.2 += quant_err_b * 7 / 16;
            }
            // Bottom-left neighbor (-1, 1) - 3/16
            if x > 0 && y + 1 < height {
                let e = &mut errors[(y + 1) as usize][(x - 1) as usize];
                e.0 += quant_err_r * 3 / 16;
                e.1 += quant_err_g * 3 / 16;
                e.2 += quant_err_b * 3 / 16;
            }
            // Bottom neighbor (0, 1) - 5/16
            if y + 1 < height {
                let e = &mut errors[(y + 1) as usize][x as usize];
                e.0 += quant_err_r * 5 / 16;
                e.1 += quant_err_g * 5 / 16;
                e.2 += quant_err_b * 5 / 16;
            }
            // Bottom-right neighbor (1, 1) - 1/16
            if x + 1 < width && y + 1 < height {
                let e = &mut errors[(y + 1) as usize][(x + 1) as usize];
                e.0 += quant_err_r / 16;
                e.1 += quant_err_g / 16;
                e.2 += quant_err_b / 16;
            }
        }
    }

    result
}

/// Applies edge-based outline to a pixel art image.
fn apply_outline(img: &RgbImage, threshold: u8) -> RgbImage {
    let (width, height) = img.dimensions();
    let mut result = img.clone();

    for y in 0..height {
        for x in 0..width {
            let brightness_val = brightness(img.get_pixel(x, y)[0], img.get_pixel(x, y)[1], img.get_pixel(x, y)[2]);

            // Check if this pixel is an edge (compare with neighbors)
            let mut max_diff = 0u8;

            // Left neighbor
            if x > 0 {
                let neighbor_brightness = brightness(
                    img.get_pixel(x - 1, y)[0],
                    img.get_pixel(x - 1, y)[1],
                    img.get_pixel(x - 1, y)[2],
                );
                max_diff = max_diff.max((brightness_val - neighbor_brightness).abs() as u8);
            }
            // Right neighbor
            if x + 1 < width {
                let neighbor_brightness = brightness(
                    img.get_pixel(x + 1, y)[0],
                    img.get_pixel(x + 1, y)[1],
                    img.get_pixel(x + 1, y)[2],
                );
                max_diff = max_diff.max((brightness_val - neighbor_brightness).abs() as u8);
            }
            // Top neighbor
            if y > 0 {
                let neighbor_brightness = brightness(
                    img.get_pixel(x, y - 1)[0],
                    img.get_pixel(x, y - 1)[1],
                    img.get_pixel(x, y - 1)[2],
                );
                max_diff = max_diff.max((brightness_val - neighbor_brightness).abs() as u8);
            }
            // Bottom neighbor
            if y + 1 < height {
                let neighbor_brightness = brightness(
                    img.get_pixel(x, y + 1)[0],
                    img.get_pixel(x, y + 1)[1],
                    img.get_pixel(x, y + 1)[2],
                );
                max_diff = max_diff.max((brightness_val - neighbor_brightness).abs() as u8);
            }

            // If edge is detected, set pixel to black
            if max_diff > threshold {
                result.put_pixel(x, y, Rgb([0, 0, 0]));
            }
        }
    }

    result
}

/// Expands an image canvas in the given direction and creates a mask
/// where the new region is white (to be regenerated) and original is black (to keep).
///
/// Returns (expanded_image, mask_image).
fn expand_canvas_with_mask(
    source: &image::DynamicImage,
    direction: &OutpaintDirection,
    extend_px: u32,
) -> Result<(image::DynamicImage, RgbImage), AppError> {
    use image::GenericImageView;

    let (src_width, src_height) = source.dimensions();

    // Calculate new dimensions based on direction
    let (new_width, new_height, offset_x, offset_y) = match direction {
        OutpaintDirection::Left => {
            (src_width + extend_px, src_height, extend_px, 0)
        }
        OutpaintDirection::Right => {
            (src_width + extend_px, src_height, 0, 0)
        }
        OutpaintDirection::Top => {
            (src_width, src_height + extend_px, 0, extend_px)
        }
        OutpaintDirection::Bottom => {
            (src_width, src_height + extend_px, 0, 0)
        }
    };

    // Create a new image with the expanded dimensions
    // Fill with edge pixels from source (padding)
    let mut expanded: RgbImage = RgbImage::new(new_width, new_height);

    // Fill the expanded canvas with edge-padding from the source image
    for y in 0..new_height {
        for x in 0..new_width {
            // Calculate corresponding source pixel coordinates
            let src_x = ((x as i32) - (offset_x as i32)).clamp(0, src_width as i32 - 1) as u32;
            let src_y = ((y as i32) - (offset_y as i32)).clamp(0, src_height as i32 - 1) as u32;

            let pixel = source.get_pixel(src_x, src_y);
            // Convert RGBA to RGB if needed
            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];
            expanded.put_pixel(x, y, Rgb([r, g, b]));
        }
    }

    // Create mask: white (255) for new region, black (0) for original region
    let mut mask: RgbImage = RgbImage::new(new_width, new_height);

    // The new region is the area that was added
    match direction {
        OutpaintDirection::Left => {
            // New region is on the left (x < offset_x)
            for y in 0..new_height {
                for x in 0..offset_x {
                    mask.put_pixel(x, y, Rgb([255, 255, 255]));
                }
                for x in offset_x..new_width {
                    mask.put_pixel(x, y, Rgb([0, 0, 0]));
                }
            }
        }
        OutpaintDirection::Right => {
            // New region is on the right (x >= src_width)
            for y in 0..new_height {
                for x in 0..src_width {
                    mask.put_pixel(x, y, Rgb([0, 0, 0]));
                }
                for x in src_width..new_width {
                    mask.put_pixel(x, y, Rgb([255, 255, 255]));
                }
            }
        }
        OutpaintDirection::Top => {
            // New region is on top (y < offset_y)
            for y in 0..offset_y {
                for x in 0..new_width {
                    mask.put_pixel(x, y, Rgb([255, 255, 255]));
                }
            }
            for y in offset_y..new_height {
                for x in 0..new_width {
                    mask.put_pixel(x, y, Rgb([0, 0, 0]));
                }
            }
        }
        OutpaintDirection::Bottom => {
            // New region is on bottom (y >= src_height)
            for y in 0..src_height {
                for x in 0..new_width {
                    mask.put_pixel(x, y, Rgb([0, 0, 0]));
                }
            }
            for y in src_height..new_height {
                for x in 0..new_width {
                    mask.put_pixel(x, y, Rgb([255, 255, 255]));
                }
            }
        }
    }

    Ok((image::DynamicImage::ImageRgb8(expanded), mask))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle() {
        let worker = ImageProcessWorker::new(
            Arc::new(ModelRouter::new(
                Arc::new(artifex_model_config::ProviderRegistry::new()),
                Arc::new(TestRepo),
                Arc::new(artifex_model_config::credential_store::InMemoryCredentialStore::new()),
            )),
            Arc::new(artifex_model_config::credential_store::InMemoryCredentialStore::new()),
            "/tmp".to_string(),
        );

        assert!(worker.can_handle("image_remove_background"));
        assert!(worker.can_handle("pixel_art_convert"));
        assert!(worker.can_handle("image_inpaint"));
        assert!(worker.can_handle("image_outpaint"));
        assert!(!worker.can_handle("image_generate"));
        assert!(!worker.can_handle("tile_generate"));
    }

    #[test]
    fn test_palette_nearest_color() {
        // Test that we can find nearest colors
        let nearest = nearest_palette_color(255, 0, 0, PALETTE_PICO8);
        assert_eq!(nearest, (0xFF, 0x00, 0x4D)); // Should be red

        let nearest = nearest_palette_color(0, 0, 0, PALETTE_PICO8);
        assert_eq!(nearest, (0x00, 0x00, 0x00)); // Should be black

        let nearest = nearest_palette_color(255, 255, 255, PALETTE_PICO8);
        assert_eq!(nearest, (0xFF, 0xF1, 0xE8)); // Should be white
    }

    #[test]
    fn test_brightness() {
        assert_eq!(brightness(0, 0, 0), 0.0);
        // Brightness for white should be close to 255 (255 * 0.299 + 255 * 0.587 + 255 * 0.114 = 254.995)
        let white_brightness = brightness(255, 255, 255);
        assert!(white_brightness > 254.0 && white_brightness <= 255.0);
        // Green appears brightest, then red, then blue (due to human eye sensitivity weights)
        assert!(brightness(0, 255, 0) > brightness(255, 0, 0));
        assert!(brightness(255, 0, 0) > brightness(0, 0, 255));
    }

    #[test]
    fn test_pico8_palette_has_16_colors() {
        // Pico-8 palette must have exactly 16 colors
        assert_eq!(PALETTE_PICO8.len(), 16, "Pico-8 palette should have 16 colors");
    }

    #[test]
    fn test_gameboy_palette_has_4_colors() {
        // GameBoy palette must have exactly 4 colors
        assert_eq!(PALETTE_GAMEBOY.len(), 4, "GameBoy palette should have 4 colors");
    }

    #[test]
    fn test_expand_canvas_right() {
        // Create a small test image (red)
        let img = image::RgbImage::from_fn(2, 2, |_x, _y| {
            Rgb([255, 0, 0])
        });

        let direction = OutpaintDirection::Right;
        let extend_px = 2;

        let result = expand_canvas_with_mask(
            &image::DynamicImage::ImageRgb8(img.clone()),
            &direction,
            extend_px,
        );

        assert!(result.is_ok());
        let (expanded, mask) = result.unwrap();

        // New dimensions: 4x2
        assert_eq!(expanded.width(), 4);
        assert_eq!(expanded.height(), 2);
        assert_eq!(mask.width(), 4);
        assert_eq!(mask.height(), 2);

        // Left half (original) should be black in mask
        // Right half (new) should be white in mask
        for y in 0..2 {
            assert_eq!(mask.get_pixel(0, y)[0], 0); // black
            assert_eq!(mask.get_pixel(1, y)[0], 0); // black
            assert_eq!(mask.get_pixel(2, y)[0], 255); // white
            assert_eq!(mask.get_pixel(3, y)[0], 255); // white
        }
    }

    #[test]
    fn test_expand_canvas_left() {
        // Create a small test image (green)
        let img = image::RgbImage::from_fn(2, 2, |_x, _y| {
            Rgb([0, 255, 0])
        });

        let direction = OutpaintDirection::Left;
        let extend_px = 2;

        let result = expand_canvas_with_mask(
            &image::DynamicImage::ImageRgb8(img.clone()),
            &direction,
            extend_px,
        );

        assert!(result.is_ok());
        let (expanded, mask) = result.unwrap();

        // New dimensions: 4x2
        assert_eq!(expanded.width(), 4);
        assert_eq!(expanded.height(), 2);

        // Left half (new) should be white in mask
        // Right half (original) should be black in mask
        for y in 0..2 {
            assert_eq!(mask.get_pixel(0, y)[0], 255); // white
            assert_eq!(mask.get_pixel(1, y)[0], 255); // white
            assert_eq!(mask.get_pixel(2, y)[0], 0); // black
            assert_eq!(mask.get_pixel(3, y)[0], 0); // black
        }
    }

    #[test]
    fn test_expand_canvas_top() {
        // Create a small test image (blue)
        let img = image::RgbImage::from_fn(2, 2, |_x, _y| {
            Rgb([0, 0, 255])
        });

        let direction = OutpaintDirection::Top;
        let extend_px = 2;

        let result = expand_canvas_with_mask(
            &image::DynamicImage::ImageRgb8(img.clone()),
            &direction,
            extend_px,
        );

        assert!(result.is_ok());
        let (expanded, mask) = result.unwrap();

        // New dimensions: 2x4
        assert_eq!(expanded.width(), 2);
        assert_eq!(expanded.height(), 4);

        // Top half (new) should be white in mask
        // Bottom half (original) should be black in mask
        for x in 0..2 {
            assert_eq!(mask.get_pixel(x, 0)[0], 255); // white
            assert_eq!(mask.get_pixel(x, 1)[0], 255); // white
            assert_eq!(mask.get_pixel(x, 2)[0], 0); // black
            assert_eq!(mask.get_pixel(x, 3)[0], 0); // black
        }
    }

    #[test]
    fn test_expand_canvas_bottom() {
        // Create a small test image
        let img = image::RgbImage::from_fn(2, 2, |_x, _y| {
            Rgb([128, 128, 128])
        });

        let direction = OutpaintDirection::Bottom;
        let extend_px = 2;

        let result = expand_canvas_with_mask(
            &image::DynamicImage::ImageRgb8(img.clone()),
            &direction,
            extend_px,
        );

        assert!(result.is_ok());
        let (expanded, mask) = result.unwrap();

        // New dimensions: 2x4
        assert_eq!(expanded.width(), 2);
        assert_eq!(expanded.height(), 4);

        // Top half (original) should be black in mask
        // Bottom half (new) should be white in mask
        for x in 0..2 {
            assert_eq!(mask.get_pixel(x, 0)[0], 0); // black
            assert_eq!(mask.get_pixel(x, 1)[0], 0); // black
            assert_eq!(mask.get_pixel(x, 2)[0], 255); // white
            assert_eq!(mask.get_pixel(x, 3)[0], 255); // white
        }
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