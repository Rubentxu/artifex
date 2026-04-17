//! Image generation provider trait and types.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::provider::{ProviderError, ProviderMetadata};

/// PBR material map types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MapKind {
    Basecolor,
    Normal,
    Roughness,
    Metalness,
    Height,
}

impl MapKind {
    /// Returns the map kind as a lowercase string.
    pub fn as_str(&self) -> &'static str {
        match self {
            MapKind::Basecolor => "basecolor",
            MapKind::Normal => "normal",
            MapKind::Roughness => "roughness",
            MapKind::Metalness => "metalness",
            MapKind::Height => "height",
        }
    }

    /// Parses a string to MapKind.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "basecolor" => Some(MapKind::Basecolor),
            "normal" => Some(MapKind::Normal),
            "roughness" => Some(MapKind::Roughness),
            "metalness" => Some(MapKind::Metalness),
            "height" => Some(MapKind::Height),
            _ => None,
        }
    }
}

/// Result of PBR material generation.
#[derive(Debug, Clone)]
pub struct MaterialResult {
    /// Map from MapKind to raw image bytes (PNG format).
    pub maps: HashMap<MapKind, Vec<u8>>,
}

impl MaterialResult {
    /// Creates a new MaterialResult with the given maps.
    pub fn new(maps: HashMap<MapKind, Vec<u8>>) -> Self {
        Self { maps }
    }

    /// Returns true if at least one map is present.
    pub fn is_valid(&self) -> bool {
        !self.maps.is_empty()
    }

    /// Gets a map by kind, returning None if not present.
    pub fn get(&self, kind: MapKind) -> Option<&Vec<u8>> {
        self.maps.get(&kind)
    }
}

/// Parameters for material generation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaterialGenParams {
    /// Optional output resolution (width/height). Provider may override.
    #[serde(default)]
    pub resolution: Option<u32>,
}

/// Parameters for image editing (inpainting/outpainting).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageEditParams {
    /// The prompt describing the desired edit.
    pub prompt: String,
    /// Optional negative prompt (things to avoid).
    #[serde(default)]
    pub negative_prompt: Option<String>,
    /// How strongly to follow the mask (0.0–1.0). Default 0.8.
    #[serde(default = "default_edit_strength")]
    pub strength: f32,
    /// Guidance scale (1.0–20.0). Default 7.5.
    #[serde(default = "default_edit_guidance_scale")]
    pub guidance_scale: f32,
    /// Inference steps. Default 30.
    #[serde(default = "default_edit_steps")]
    pub num_inference_steps: u32,
    /// Optional seed for reproducibility.
    #[serde(default)]
    pub seed: Option<u64>,
    /// Optional model ID to use.
    #[serde(default)]
    pub model_id: Option<String>,
}

fn default_edit_strength() -> f32 {
    0.8
}

fn default_edit_guidance_scale() -> f32 {
    7.5
}

fn default_edit_steps() -> u32 {
    30
}

impl ImageEditParams {
    /// Validates the image edit parameters.
    pub fn validate(&self) -> Result<(), String> {
        if self.prompt.is_empty() {
            return Err("Prompt cannot be empty".to_string());
        }
        if self.strength < 0.0 || self.strength > 1.0 {
            return Err("Strength must be between 0.0 and 1.0".to_string());
        }
        if self.guidance_scale < 1.0 || self.guidance_scale > 20.0 {
            return Err("Guidance scale must be between 1.0 and 20.0".to_string());
        }
        if self.num_inference_steps == 0 || self.num_inference_steps > 500 {
            return Err("Steps must be between 1 and 500".to_string());
        }
        Ok(())
    }
}

impl Default for ImageEditParams {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            negative_prompt: None,
            strength: 0.8,
            guidance_scale: 7.5,
            num_inference_steps: 30,
            seed: None,
            model_id: None,
        }
    }
}

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

    /// Removes background from an image.
    ///
    /// # Arguments
    /// * `image_data` - Raw image bytes (PNG, JPEG, etc.)
    /// * `api_key` - API key for authentication
    ///
    /// # Errors
    /// Returns an error if background removal fails.
    async fn remove_background(
        &self,
        image_data: &[u8],
        api_key: &str,
    ) -> Result<ImageGenResult, ProviderError>;

    /// Inpaints or edits an image using a mask.
    ///
    /// White areas in the mask indicate regions to be regenerated.
    ///
    /// # Arguments
    /// * `image_data` - Original image bytes
    /// * `mask_data` - Mask bytes (white = edit region, black = keep)
    /// * `params` - Edit parameters including prompt
    /// * `api_key` - API key for authentication
    ///
    /// # Errors
    /// Returns an error if inpainting fails.
    async fn inpaint(
        &self,
        image_data: &[u8],
        mask_data: &[u8],
        params: &ImageEditParams,
        api_key: &str,
    ) -> Result<ImageGenResult, ProviderError> {
        let _ = (image_data, mask_data, params, api_key);
        Err(ProviderError::ProviderSpecific(
            self.metadata().id.clone(),
            "inpaint is not supported by this provider".to_string(),
        ))
    }

    /// Generates PBR material maps from a source image.
    ///
    /// # Arguments
    /// * `image_data` - Source image bytes (PNG, JPEG, etc.)
    /// * `params` - Generation parameters
    /// * `api_key` - API key for authentication
    ///
    /// # Errors
    /// Returns an error if material generation fails or is not supported.
    async fn generate_material(
        &self,
        image_data: &[u8],
        params: &MaterialGenParams,
        api_key: &str,
    ) -> Result<MaterialResult, ProviderError> {
        let _ = (image_data, params, api_key);
        Err(ProviderError::ProviderSpecific(
            self.metadata().id.clone(),
            "material generation is not supported by this provider".to_string(),
        ))
    }

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

    // === ImageEditParams tests ===

    #[test]
    fn test_image_edit_params_validate_valid() {
        let params = ImageEditParams {
            prompt: "Edit this region".to_string(),
            negative_prompt: Some("blurry".to_string()),
            strength: 0.8,
            guidance_scale: 7.5,
            num_inference_steps: 30,
            seed: Some(42),
            model_id: None,
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_image_edit_params_validate_empty_prompt() {
        let params = ImageEditParams {
            prompt: "".to_string(),
            ..Default::default()
        };
        assert!(params.validate().is_err());
        assert!(params.validate().unwrap_err().contains("empty"));
    }

    #[test]
    fn test_image_edit_params_validate_strength_out_of_range() {
        let params_too_low = ImageEditParams {
            prompt: "Test".to_string(),
            strength: -0.1,
            ..Default::default()
        };
        assert!(params_too_low.validate().is_err());
        assert!(params_too_low.validate().unwrap_err().contains("Strength"));

        let params_too_high = ImageEditParams {
            prompt: "Test".to_string(),
            strength: 1.5,
            ..Default::default()
        };
        assert!(params_too_high.validate().is_err());
        assert!(params_too_high.validate().unwrap_err().contains("Strength"));
    }

    #[test]
    fn test_image_edit_params_validate_guidance_scale_out_of_range() {
        let params_low = ImageEditParams {
            prompt: "Test".to_string(),
            guidance_scale: 0.5,
            ..Default::default()
        };
        assert!(params_low.validate().is_err());
        assert!(params_low.validate().unwrap_err().contains("Guidance"));

        let params_high = ImageEditParams {
            prompt: "Test".to_string(),
            guidance_scale: 25.0,
            ..Default::default()
        };
        assert!(params_high.validate().is_err());
        assert!(params_high.validate().unwrap_err().contains("Guidance"));
    }

    #[test]
    fn test_image_edit_params_validate_invalid_steps() {
        let params_zero = ImageEditParams {
            prompt: "Test".to_string(),
            num_inference_steps: 0,
            ..Default::default()
        };
        assert!(params_zero.validate().is_err());

        let params_too_high = ImageEditParams {
            prompt: "Test".to_string(),
            num_inference_steps: 501,
            ..Default::default()
        };
        assert!(params_too_high.validate().is_err());
    }

    #[test]
    fn test_image_edit_params_defaults() {
        let params = ImageEditParams::default();
        assert_eq!(params.prompt, "");
        assert!(params.negative_prompt.is_none());
        assert_eq!(params.strength, 0.8);
        assert_eq!(params.guidance_scale, 7.5);
        assert_eq!(params.num_inference_steps, 30);
        assert!(params.seed.is_none());
        assert!(params.model_id.is_none());
    }

    // === MapKind tests ===

    #[test]
    fn test_map_kind_as_str() {
        assert_eq!(MapKind::Basecolor.as_str(), "basecolor");
        assert_eq!(MapKind::Normal.as_str(), "normal");
        assert_eq!(MapKind::Roughness.as_str(), "roughness");
        assert_eq!(MapKind::Metalness.as_str(), "metalness");
        assert_eq!(MapKind::Height.as_str(), "height");
    }

    #[test]
    fn test_map_kind_from_str() {
        assert_eq!(MapKind::from_str("basecolor"), Some(MapKind::Basecolor));
        assert_eq!(MapKind::from_str("normal"), Some(MapKind::Normal));
        assert_eq!(MapKind::from_str("roughness"), Some(MapKind::Roughness));
        assert_eq!(MapKind::from_str("metalness"), Some(MapKind::Metalness));
        assert_eq!(MapKind::from_str("height"), Some(MapKind::Height));
        assert_eq!(MapKind::from_str("unknown"), None);
    }

    // === MaterialResult tests ===

    #[test]
    fn test_material_result_new() {
        let mut maps = HashMap::new();
        maps.insert(MapKind::Basecolor, vec![1, 2, 3]);
        maps.insert(MapKind::Normal, vec![4, 5, 6]);

        let result = MaterialResult::new(maps);
        assert!(result.is_valid());
        assert_eq!(result.get(MapKind::Basecolor), Some(&vec![1, 2, 3]));
        assert_eq!(result.get(MapKind::Normal), Some(&vec![4, 5, 6]));
        assert!(result.get(MapKind::Roughness).is_none());
    }

    #[test]
    fn test_material_result_empty_is_invalid() {
        let maps = HashMap::new();
        let result = MaterialResult::new(maps);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_material_gen_params_default() {
        let params = MaterialGenParams::default();
        assert!(params.resolution.is_none());
    }

    #[test]
    fn test_material_gen_params_with_resolution() {
        let params = MaterialGenParams {
            resolution: Some(1024),
        };
        assert_eq!(params.resolution, Some(1024));
    }
}