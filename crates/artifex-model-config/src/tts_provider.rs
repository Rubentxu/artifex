//! Text-to-Speech provider trait and types (stub implementation).

use serde::{Deserialize, Serialize};

use super::provider::{ProviderError, ProviderMetadata};

/// Parameters for text-to-speech synthesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsParams {
    /// The text to synthesize.
    pub text: String,
    /// Voice ID (provider-specific).
    #[serde(default)]
    pub voice_id: Option<String>,
    /// Language code (e.g., "en-US").
    #[serde(default = "default_language")]
    pub language: String,
    /// Speech rate (0.5 to 2.0).
    #[serde(default = "default_speed")]
    pub speed: f32,
    /// Model ID to use (resolved from routing profile).
    #[serde(default)]
    pub model_id: Option<String>,
    /// Voice stability setting (ElevenLabs).
    #[serde(default)]
    pub stability: Option<f32>,
    /// Voice similarity boost setting (ElevenLabs).
    #[serde(default)]
    pub similarity_boost: Option<f32>,
    /// Output format: "mp3", "wav", etc.
    #[serde(default)]
    pub output_format: Option<String>,
}

fn default_language() -> String {
    "en-US".to_string()
}

fn default_speed() -> f32 {
    1.0
}

impl TtsParams {
    /// Validates the TTS parameters.
    pub fn validate(&self) -> Result<(), String> {
        if self.text.is_empty() {
            return Err("Text cannot be empty".to_string());
        }
        if self.speed < 0.5 || self.speed > 2.0 {
            return Err("Speed must be between 0.5 and 2.0".to_string());
        }
        Ok(())
    }
}

/// Result of text-to-speech synthesis.
#[derive(Debug, Clone)]
pub struct TtsResult {
    /// Raw audio data.
    pub audio_data: Vec<u8>,
    /// Duration in seconds.
    pub duration_secs: f32,
    /// Audio format (e.g., "mp3", "wav").
    pub format: String,
}

impl TtsResult {
    /// Creates a new TtsResult.
    pub fn new(audio_data: Vec<u8>, duration_secs: f32, format: impl Into<String>) -> Self {
        Self {
            audio_data,
            duration_secs,
            format: format.into(),
        }
    }
}

/// Provider trait for text-to-speech services.
#[async_trait::async_trait]
pub trait TtsProvider: Send + Sync {
    /// Synthesizes speech from text.
    async fn synthesize(
        &self,
        params: &TtsParams,
        api_key: &str,
    ) -> Result<TtsResult, ProviderError>;

    /// Returns the provider metadata.
    fn metadata(&self) -> &ProviderMetadata;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tts_params_validate_valid() {
        let params = TtsParams {
            text: "Hello, world!".to_string(),
            voice_id: Some("voice_1".to_string()),
            language: "en-US".to_string(),
            speed: 1.0,
            model_id: Some("eleven_turbo_v2".to_string()),
            stability: Some(0.5),
            similarity_boost: Some(0.75),
            output_format: Some("mp3".to_string()),
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_tts_params_validate_empty_text() {
        let params = TtsParams {
            text: "".to_string(),
            voice_id: None,
            language: "en-US".to_string(),
            speed: 1.0,
            model_id: None,
            stability: None,
            similarity_boost: None,
            output_format: None,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn test_tts_params_validate_invalid_speed() {
        let params = TtsParams {
            text: "Hello".to_string(),
            speed: 0.1,
            ..Default::default()
        };
        assert!(params.validate().is_err());

        let params2 = TtsParams {
            text: "Hello".to_string(),
            speed: 3.0,
            ..Default::default()
        };
        assert!(params2.validate().is_err());
    }

    impl Default for TtsParams {
        fn default() -> Self {
            Self {
                text: "Default text".to_string(),
                voice_id: None,
                language: "en-US".to_string(),
                speed: 1.0,
                model_id: None,
                stability: None,
                similarity_boost: None,
                output_format: None,
            }
        }
    }
}