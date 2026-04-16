//! Text generation provider trait and types (stub implementation).

use serde::{Deserialize, Serialize};

use super::provider::{ProviderError, ProviderMetadata};

/// Parameters for text generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextParams {
    /// The prompt for text generation.
    pub prompt: String,
    /// Maximum number of tokens to generate.
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    /// Temperature for generation (0.0 to 2.0).
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// Optional stop sequences.
    #[serde(default)]
    pub stop_sequences: Option<Vec<String>>,
    /// Whether to stream the response.
    #[serde(default)]
    pub stream: bool,
}

fn default_max_tokens() -> u32 {
    1024
}

fn default_temperature() -> f32 {
    0.7
}

impl TextParams {
    /// Validates the text generation parameters.
    pub fn validate(&self) -> Result<(), String> {
        if self.prompt.is_empty() {
            return Err("Prompt cannot be empty".to_string());
        }
        if self.max_tokens == 0 {
            return Err("Max tokens must be greater than 0".to_string());
        }
        if self.temperature < 0.0 || self.temperature > 2.0 {
            return Err("Temperature must be between 0.0 and 2.0".to_string());
        }
        Ok(())
    }
}

/// Result of text generation.
#[derive(Debug, Clone)]
pub struct TextResult {
    /// Generated text.
    pub text: String,
    /// Number of tokens generated.
    pub tokens_used: u32,
    /// Whether the response was truncated.
    pub truncated: bool,
}

impl TextResult {
    /// Creates a new TextResult.
    pub fn new(text: String, tokens_used: u32, truncated: bool) -> Self {
        Self {
            text,
            tokens_used,
            truncated,
        }
    }
}

/// Provider trait for text generation services.
#[async_trait::async_trait]
pub trait TextProvider: Send + Sync {
    /// Generates text based on the given parameters.
    async fn complete(
        &self,
        params: &TextParams,
        api_key: &str,
    ) -> Result<TextResult, ProviderError>;

    /// Returns the provider metadata.
    fn metadata(&self) -> &ProviderMetadata;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_params_validate_valid() {
        let params = TextParams {
            prompt: "Once upon a time".to_string(),
            max_tokens: 100,
            temperature: 0.7,
            stop_sequences: None,
            stream: false,
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_text_params_validate_empty_prompt() {
        let params = TextParams {
            prompt: "".to_string(),
            ..Default::default()
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn test_text_params_validate_invalid_temperature() {
        let params = TextParams {
            prompt: "Hello".to_string(),
            temperature: -0.1,
            ..Default::default()
        };
        assert!(params.validate().is_err());

        let params2 = TextParams {
            prompt: "Hello".to_string(),
            temperature: 2.5,
            ..Default::default()
        };
        assert!(params2.validate().is_err());
    }

    #[test]
    fn test_text_result_new() {
        let result = TextResult::new("Hello, world!".to_string(), 50, false);
        assert_eq!(result.text, "Hello, world!");
        assert_eq!(result.tokens_used, 50);
        assert!(!result.truncated);
    }

    impl Default for TextParams {
        fn default() -> Self {
            Self {
                prompt: "Default prompt".to_string(),
                max_tokens: 1024,
                temperature: 0.7,
                stop_sequences: None,
                stream: false,
            }
        }
    }
}