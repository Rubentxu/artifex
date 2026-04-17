//! Together AI text generation provider.
//!
//! Implements the TextProvider trait for Together's API.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use artifex_model_config::{
    provider::{AuthType, ModelCapability, ProviderError, ProviderKind, ProviderMetadata},
    text_provider::{TextParams, TextProvider, TextResult},
};

/// Together AI API client for text generation.
#[derive(Debug, Clone)]
pub struct TogetherTextProvider {
    http_client: Client,
    metadata: ProviderMetadata,
    /// Default model ID to use.
    default_model: String,
}

impl TogetherTextProvider {
    /// Creates a new TogetherTextProvider with the default model.
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
            default_model: "meta-llama/Llama-3-70b-chat-hf".to_string(),
            metadata: ProviderMetadata {
                id: "together".to_string(),
                name: "Together AI".to_string(),
                kind: ProviderKind::Together,
                base_url: "https://api.together.xyz/v1".to_string(),
                supported_capabilities: vec![
                    ModelCapability::TextComplete,
                    ModelCapability::CodeComplete,
                ],
                auth_type: AuthType::ApiKey,
            },
        }
    }

    /// Creates a new TogetherTextProvider with a custom HTTP client.
    pub fn with_client(http_client: Client) -> Self {
        Self {
            http_client,
            default_model: "meta-llama/Llama-3-70b-chat-hf".to_string(),
            metadata: ProviderMetadata {
                id: "together".to_string(),
                name: "Together AI".to_string(),
                kind: ProviderKind::Together,
                base_url: "https://api.together.xyz/v1".to_string(),
                supported_capabilities: vec![
                    ModelCapability::TextComplete,
                    ModelCapability::CodeComplete,
                ],
                auth_type: AuthType::ApiKey,
            },
        }
    }

    /// Creates a new TogetherTextProvider with a specific default model.
    pub fn with_model(model_id: impl Into<String>) -> Self {
        let model = model_id.into();
        Self {
            http_client: Client::new(),
            default_model: model.clone(),
            metadata: ProviderMetadata {
                id: "together".to_string(),
                name: "Together AI".to_string(),
                kind: ProviderKind::Together,
                base_url: "https://api.together.xyz/v1".to_string(),
                supported_capabilities: vec![
                    ModelCapability::TextComplete,
                    ModelCapability::CodeComplete,
                ],
                auth_type: AuthType::ApiKey,
            },
        }
    }
}

impl Default for TogetherTextProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TextProvider for TogetherTextProvider {
    async fn complete(
        &self,
        params: &TextParams,
        api_key: &str,
    ) -> Result<TextResult, ProviderError> {
        // Validate params
        params.validate().map_err(|e| {
            ProviderError::ProviderSpecific("together".to_string(), e)
        })?;

        // Together AI chat completions endpoint
        let url = "https://api.together.xyz/v1/chat/completions";

        // Build messages array
        let messages = vec![TogetherMessage {
            role: "user".to_string(),
            content: params.prompt.clone(),
        }];

        // Build request body
        let request_body = TogetherChatRequest {
            model: self.default_model.clone(),
            messages,
            max_tokens: params.max_tokens,
            temperature: params.temperature,
            stop: params.stop_sequences.clone(),
        };

        let response = self
            .http_client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let status = response.status();

        if !status.is_success() {
            return Err(map_together_error(status.as_u16(), response.text().await.unwrap_or_default()).await);
        }

        let completion: TogetherChatResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ProviderSpecific("together".to_string(), e.to_string()))?;

        let text = completion
            .choices
            .first()
            .and_then(|c| c.message.content.as_deref())
            .unwrap_or("")
            .to_string();

        let usage = completion.usage;
        let truncated = completion.choices.first().is_some_and(|c| c.finish_reason == "length");

        Ok(TextResult::new(
            text,
            usage.completion_tokens,
            truncated,
        ))
    }

    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }
}

/// Maps HTTP status codes to ProviderError variants.
async fn map_together_error(status: u16, body: String) -> ProviderError {
    match status {
        401 | 403 => ProviderError::AuthFailed {
            provider: "together".to_string(),
            message: "Invalid or missing API key".to_string(),
        },
        429 => ProviderError::RateLimited {
            provider: "together".to_string(),
            retry_after_secs: None,
        },
        400 => ProviderError::ProviderSpecific(
            "together".to_string(),
            format!("Bad request: {}", body),
        ),
        _ => ProviderError::ProviderSpecific(
            "together".to_string(),
            format!("HTTP {}: {}", status, body),
        ),
    }
}

// === Together AI API Types ===

#[derive(Debug, Serialize)]
struct TogetherChatRequest {
    model: String,
    messages: Vec<TogetherMessage>,
    #[serde(rename = "max_tokens")]
    max_tokens: u32,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct TogetherMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TogetherChatResponse {
    id: String,
    choices: Vec<TogetherChoice>,
    usage: TogetherUsage,
}

#[derive(Debug, Deserialize)]
struct TogetherChoice {
    message: TogetherChoiceMessage,
    #[serde(rename = "finish_reason")]
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TogetherChoiceMessage {
    role: String,
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TogetherUsage {
    #[serde(rename = "prompt_tokens")]
    prompt_tokens: u32,
    #[serde(rename = "completion_tokens")]
    completion_tokens: u32,
    #[serde(rename = "total_tokens")]
    total_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use artifex_model_config::provider::ProviderError;

    #[test]
    fn test_together_provider_metadata() {
        let provider = TogetherTextProvider::new();
        let metadata = provider.metadata();

        assert_eq!(metadata.name, "Together AI");
        assert_eq!(metadata.kind, ProviderKind::Together);
        assert_eq!(metadata.base_url, "https://api.together.xyz/v1");
        assert!(metadata.supported_capabilities.contains(&ModelCapability::TextComplete));
        assert_eq!(metadata.auth_type, AuthType::ApiKey);
    }

    #[test]
    fn test_together_provider_with_custom_model() {
        let provider = TogetherTextProvider::with_model("mistralai/Mistral-7B-Instruct-v0.1");
        assert_eq!(provider.metadata().name, "Together AI");
    }

    #[tokio::test]
    async fn test_map_together_error_401() {
        let err = map_together_error(401, "Unauthorized".to_string()).await;
        match err {
            ProviderError::AuthFailed { provider, .. } => {
                assert_eq!(provider, "together");
            }
            _ => panic!("Expected AuthFailed"),
        }
    }

    #[tokio::test]
    async fn test_map_together_error_429() {
        let err = map_together_error(429, "Rate limit exceeded".to_string()).await;
        match err {
            ProviderError::RateLimited { provider, .. } => {
                assert_eq!(provider, "together");
            }
            _ => panic!("Expected RateLimited"),
        }
    }

    #[tokio::test]
    async fn test_text_params_validation() {
        let provider = TogetherTextProvider::new();
        let params = TextParams {
            prompt: "".to_string(),
            max_tokens: 100,
            temperature: 0.7,
            stop_sequences: None,
            stream: false,
        };

        let result = provider.complete(&params, "fake-key").await;
        assert!(result.is_err());
    }
}