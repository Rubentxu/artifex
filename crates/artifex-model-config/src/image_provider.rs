//! Image generation provider trait and types.

use serde::{Deserialize, Serialize};

use super::provider::{ProviderError, ProviderMetadata};

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
    #[serde(default)]
    pub seed: Option<u64>,
    /// Number of images to generate.
    #[serde(default = "default_num_images")]
    pub num_images: u32,
    /// Guidance scale for generation.
    #[serde(default = "default_guidance_scale")]
    pub guidance_scale: f32,
    /// Model ID to use (resolved from routing profile).
    /// This is set by the ImageGenWorker before calling the provider.
    #[serde(default)]
    pub model_id: Option<String>,
}

fn default_num_images() -> u32 {
    1
}

fn default_guidance_scale() -> f32 {
    7.5
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
        if self.num_images == 0 || self.num_images > 10 {
            return Err("Number of images must be between 1 and 10".to_string());
        }
        Ok(())
    }

    /// Creates minimal params for connection testing.
    /// Uses smallest possible dimensions to minimize API usage.
    pub fn test_params() -> Self {
        Self {
            prompt: "test".to_string(),
            negative_prompt: None,
            width: 64,
            height: 64,
            steps: 1,
            seed: None,
            num_images: 1,
            guidance_scale: 7.5,
            model_id: None,
        }
    }
}

impl Default for ImageGenParams {
    fn default() -> Self {
        Self {
            prompt: "Default prompt".to_string(),
            negative_prompt: None,
            width: 512,
            height: 512,
            steps: 20,
            seed: None,
            num_images: 1,
            guidance_scale: 7.5,
            model_id: None,
        }
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
#[async_trait::async_trait]
pub trait ImageProvider: Send + Sync {
    /// Generates an image based on the given parameters.
    ///
    /// # Errors
    /// Returns an error if generation fails.
    async fn generate(
        &self,
        params: &ImageGenParams,
        api_key: &str,
    ) -> Result<ImageGenResult, ProviderError>;

    /// Returns the provider metadata.
    fn metadata(&self) -> &ProviderMetadata;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_gen_params_validate_valid() {
        let params = ImageGenParams {
            prompt: "A beautiful sunset".to_string(),
            negative_prompt: Some("blurry".to_string()),
            width: 512,
            height: 512,
            steps: 20,
            seed: Some(42),
            num_images: 1,
            guidance_scale: 7.5,
            model_id: None,
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_image_gen_params_validate_empty_prompt() {
        let params = ImageGenParams {
            prompt: "".to_string(),
            width: 512,
            height: 512,
            steps: 20,
            ..Default::default()
        };
        assert!(params.validate().is_err());
        assert!(params.validate().unwrap_err().contains("empty"));
    }

    #[test]
    fn test_image_gen_params_validate_zero_dimensions() {
        let params = ImageGenParams {
            prompt: "Test".to_string(),
            width: 0,
            height: 512,
            steps: 20,
            ..Default::default()
        };
        assert!(params.validate().is_err());

        let params2 = ImageGenParams {
            prompt: "Test".to_string(),
            width: 512,
            height: 0,
            steps: 20,
            ..Default::default()
        };
        assert!(params2.validate().is_err());
    }

    #[test]
    fn test_image_gen_params_validate_too_large() {
        let params = ImageGenParams {
            prompt: "Test".to_string(),
            width: 4097,
            height: 512,
            steps: 20,
            ..Default::default()
        };
        assert!(params.validate().is_err());
        assert!(params.validate().unwrap_err().contains("4096"));
    }

    #[test]
    fn test_image_gen_params_validate_invalid_steps() {
        let params = ImageGenParams {
            prompt: "Test".to_string(),
            width: 512,
            height: 512,
            steps: 0,
            ..Default::default()
        };
        assert!(params.validate().is_err());

        let params2 = ImageGenParams {
            prompt: "Test".to_string(),
            width: 512,
            height: 512,
            steps: 501,
            ..Default::default()
        };
        assert!(params2.validate().is_err());
    }

    #[test]
    fn test_image_gen_result_new() {
        let result = ImageGenResult::new(vec![1, 2, 3], 512, 512, "png");
        assert_eq!(result.image_data, vec![1, 2, 3]);
        assert_eq!(result.width, 512);
        assert_eq!(result.height, 512);
        assert_eq!(result.format, "png");
    }
}