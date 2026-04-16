//! Audio generation provider trait and types (stub implementation).

use serde::{Deserialize, Serialize};

use super::provider::{ProviderError, ProviderMetadata};

/// Parameters for audio generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioGenParams {
    /// The prompt describing the audio to generate.
    pub prompt: String,
    /// Duration in seconds.
    #[serde(default)]
    pub duration_secs: Option<f32>,
    /// Sample rate.
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,
    /// Kind of audio: "sfx" or "music".
    #[serde(default)]
    pub kind: Option<String>,
    /// Model ID to use (resolved from routing profile).
    #[serde(default)]
    pub model_id: Option<String>,
    /// Seed for reproducibility.
    #[serde(default)]
    pub seed: Option<u64>,
    /// Output format: "mp3", "wav", etc.
    #[serde(default)]
    pub output_format: Option<String>,
}

fn default_sample_rate() -> u32 {
    44100
}

impl AudioGenParams {
    /// Validates the audio generation parameters.
    pub fn validate(&self) -> Result<(), String> {
        if self.prompt.is_empty() {
            return Err("Prompt cannot be empty".to_string());
        }
        Ok(())
    }
}

/// Result of audio generation.
#[derive(Debug, Clone)]
pub struct AudioGenResult {
    /// Raw audio data.
    pub audio_data: Vec<u8>,
    /// Duration in seconds.
    pub duration_secs: f32,
    /// Audio format (e.g., "wav", "mp3").
    pub format: String,
}

impl AudioGenResult {
    /// Creates a new AudioGenResult.
    pub fn new(audio_data: Vec<u8>, duration_secs: f32, format: impl Into<String>) -> Self {
        Self {
            audio_data,
            duration_secs,
            format: format.into(),
        }
    }
}

/// Provider trait for audio generation services.
#[async_trait::async_trait]
pub trait AudioProvider: Send + Sync {
    /// Generates audio based on the given parameters.
    async fn generate(
        &self,
        params: &AudioGenParams,
        api_key: &str,
    ) -> Result<AudioGenResult, ProviderError>;

    /// Returns the provider metadata.
    fn metadata(&self) -> &ProviderMetadata;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_gen_params_validate_valid() {
        let params = AudioGenParams {
            prompt: "A beautiful piano melody".to_string(),
            duration_secs: Some(30.0),
            sample_rate: 44100,
            kind: Some("sfx".to_string()),
            model_id: Some("eleven_turbo_v2".to_string()),
            seed: Some(42),
            output_format: Some("mp3".to_string()),
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_audio_gen_params_validate_empty_prompt() {
        let params = AudioGenParams {
            prompt: "".to_string(),
            duration_secs: None,
            sample_rate: 44100,
            kind: None,
            model_id: None,
            seed: None,
            output_format: None,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn test_audio_gen_result_new() {
        let result = AudioGenResult::new(vec![1, 2, 3], 30.0, "wav");
        assert_eq!(result.audio_data, vec![1, 2, 3]);
        assert_eq!(result.duration_secs, 30.0);
        assert_eq!(result.format, "wav");
    }
}