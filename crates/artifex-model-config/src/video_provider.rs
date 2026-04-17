//! Video generation provider trait and types.

use serde::{Deserialize, Serialize};

use super::provider::{ProviderError, ProviderMetadata};

/// Parameters for video generation from an image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoGenParams {
    /// The source image as a URL or base64 data URI.
    /// The command/worker converts the source asset file to a data URI.
    pub source_image_url: String,
    /// The prompt describing the desired video motion.
    pub prompt: String,
    /// Duration in seconds (2-8).
    pub duration_secs: u8,
    /// Optional negative prompt (things to avoid).
    #[serde(default)]
    pub negative_prompt: Option<String>,
    /// Optional seed for reproducibility.
    #[serde(default)]
    pub seed: Option<u64>,
    /// Model ID to use (resolved from routing profile).
    /// This is set by the VideoGenWorker before calling the provider.
    #[serde(default)]
    pub model_id: Option<String>,
}

impl VideoGenParams {
    /// Validates the video generation parameters.
    pub fn validate(&self) -> Result<(), String> {
        if self.source_image_url.is_empty() {
            return Err("Source image URL is required".to_string());
        }
        if self.prompt.is_empty() {
            return Err("Prompt cannot be empty".to_string());
        }
        if self.duration_secs < 2 || self.duration_secs > 8 {
            return Err("Duration must be between 2 and 8 seconds".to_string());
        }
        Ok(())
    }
}

impl Default for VideoGenParams {
    fn default() -> Self {
        Self {
            source_image_url: String::new(),
            prompt: "Default video prompt".to_string(),
            duration_secs: 4,
            negative_prompt: None,
            seed: None,
            model_id: None,
        }
    }
}

/// Result of video generation.
#[derive(Debug, Clone)]
pub struct VideoGenResult {
    /// Raw video data (MP4 bytes).
    pub video_data: Vec<u8>,
    /// Video duration in seconds.
    pub duration_secs: f32,
    /// Video format (e.g., "mp4").
    pub format: String,
}

impl VideoGenResult {
    /// Creates a new VideoGenResult.
    pub fn new(video_data: Vec<u8>, duration_secs: f32, format: impl Into<String>) -> Self {
        Self {
            video_data,
            duration_secs,
            format: format.into(),
        }
    }
}

/// Provider trait for video generation services.
#[async_trait::async_trait]
pub trait VideoProvider: Send + Sync {
    /// Generates a video from a source image based on the given parameters.
    ///
    /// # Errors
    /// Returns an error if generation fails.
    async fn generate_video(
        &self,
        params: &VideoGenParams,
        api_key: &str,
    ) -> Result<VideoGenResult, ProviderError>;

    /// Returns the provider metadata.
    fn metadata(&self) -> &ProviderMetadata;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_gen_params_validate_valid() {
        let params = VideoGenParams {
            source_image_url: "data:image/png;base64,abc123".to_string(),
            prompt: "A car driving through the city".to_string(),
            duration_secs: 4,
            negative_prompt: Some("blurry".to_string()),
            seed: Some(42),
            model_id: None,
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_video_gen_params_validate_empty_prompt() {
        let params = VideoGenParams {
            source_image_url: "data:image/png;base64,abc123".to_string(),
            prompt: "".to_string(),
            duration_secs: 4,
            negative_prompt: None,
            seed: None,
            model_id: None,
        };
        assert!(params.validate().is_err());
        assert!(params.validate().unwrap_err().contains("empty"));
    }

    #[test]
    fn test_video_gen_params_validate_duration_too_low() {
        let params = VideoGenParams {
            source_image_url: "data:image/png;base64,abc123".to_string(),
            prompt: "Test".to_string(),
            duration_secs: 1,
            negative_prompt: None,
            seed: None,
            model_id: None,
        };
        assert!(params.validate().is_err());
        assert!(params.validate().unwrap_err().contains("2 and 8"));
    }

    #[test]
    fn test_video_gen_params_validate_duration_too_high() {
        let params = VideoGenParams {
            source_image_url: "data:image/png;base64,abc123".to_string(),
            prompt: "Test".to_string(),
            duration_secs: 10,
            negative_prompt: None,
            seed: None,
            model_id: None,
        };
        assert!(params.validate().is_err());
        assert!(params.validate().unwrap_err().contains("2 and 8"));
    }

    #[test]
    fn test_video_gen_params_validate_empty_source_image() {
        let params = VideoGenParams {
            source_image_url: "".to_string(),
            prompt: "Test".to_string(),
            duration_secs: 4,
            negative_prompt: None,
            seed: None,
            model_id: None,
        };
        assert!(params.validate().is_err());
        assert!(params.validate().unwrap_err().contains("Source image"));
    }

    #[test]
    fn test_video_gen_result_new() {
        let result = VideoGenResult::new(vec![1, 2, 3], 5.0, "mp4");
        assert_eq!(result.video_data, vec![1, 2, 3]);
        assert_eq!(result.duration_secs, 5.0);
        assert_eq!(result.format, "mp4");
    }

    #[test]
    fn test_video_gen_params_default() {
        let params = VideoGenParams::default();
        assert!(params.source_image_url.is_empty());
        assert_eq!(params.duration_secs, 4);
        assert!(params.negative_prompt.is_none());
        assert!(params.seed.is_none());
        assert!(params.model_id.is_none());
    }
}
