//! Image generation provider trait and types.

use serde::{Deserialize, Serialize};

use artifex_shared_kernel::AppError;

/// Parameters for image generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenParams {
    /// The prompt describing the image to generate.
    pub prompt: String,
    /// Optional negative prompt (things to avoid).
    #[serde(default)]
    pub negative_prompt: Option<String>,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Number of inference steps.
    pub steps: u32,
    /// Optional seed for reproducibility.
    pub seed: Option<u64>,
    /// Model identifier to use.
    pub model_id: String,
}

impl ImageGenParams {
    /// Validates the image generation parameters.
    pub fn validate(&self) -> Result<(), String> {
        if self.prompt.is_empty() {
            return Err("Prompt cannot be empty".to_string());
        }
        if self.width == 0 || self.height == 0 {
            return Err("Width and height must be non-zero".to_string());
        }
        if self.width > 4096 || self.height > 4096 {
            return Err("Width and height must be at most 4096".to_string());
        }
        if self.steps == 0 || self.steps > 500 {
            return Err("Steps must be between 1 and 500".to_string());
        }
        Ok(())
    }
}

/// Result of image generation.
#[derive(Debug, Clone)]
pub struct ImageGenResult {
    /// Raw image data (e.g., PNG bytes).
    pub image_data: Vec<u8>,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Image format (e.g., "png", "jpeg", "webp").
    pub format: String,
}

impl ImageGenResult {
    /// Creates a new ImageGenResult.
    pub fn new(image_data: Vec<u8>, width: u32, height: u32, format: impl Into<String>) -> Self {
        Self {
            image_data,
            width,
            height,
            format: format.into(),
        }
    }
}

/// Provider trait for image generation services.
///
/// Implementors provide the actual image generation capability,
/// whether from a remote API or a local model.
#[async_trait::async_trait]
pub trait ImageGenProvider: Send + Sync {
    /// Generates an image based on the given parameters.
    ///
    /// # Errors
    /// Returns an error if generation fails.
    async fn generate(&self, params: ImageGenParams) -> Result<ImageGenResult, AppError>;
}