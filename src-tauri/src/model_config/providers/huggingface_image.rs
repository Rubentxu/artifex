//! HuggingFace image generation provider.
//!
//! Implements the ImageProvider trait for HuggingFace's Inference API.

use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;

use artifex_model_config::{
    image_provider::{ImageGenParams, ImageGenResult, ImageProvider},
    provider::{AuthType, ModelCapability, ProviderError, ProviderKind, ProviderMetadata},
};

/// HuggingFace Inference API client for image generation.
#[derive(Debug, Clone)]
pub struct HuggingFaceImageProvider {
    http_client: Client,
    metadata: ProviderMetadata,
    /// Default model ID to use if not specified in config.
    default_model: String,
}

impl HuggingFaceImageProvider {
    /// Creates a new HuggingFaceImageProvider with the default model.
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
            default_model: "stabilityai/stable-diffusion-xl-base-1.0".to_string(),
            metadata: ProviderMetadata {
                id: "huggingface".to_string(),
                name: "HuggingFace".to_string(),
                kind: ProviderKind::HuggingFace,
                base_url: "https://api-inference.huggingface.co".to_string(),
                supported_capabilities: vec![ModelCapability::ImageGen],
                auth_type: AuthType::ApiKey,
            },
        }
    }

    /// Creates a new HuggingFaceImageProvider with a custom HTTP client.
    pub fn with_client(http_client: Client) -> Self {
        Self {
            http_client,
            default_model: "stabilityai/stable-diffusion-xl-base-1.0".to_string(),
            metadata: ProviderMetadata {
                id: "huggingface".to_string(),
                name: "HuggingFace".to_string(),
                kind: ProviderKind::HuggingFace,
                base_url: "https://api-inference.huggingface.co".to_string(),
                supported_capabilities: vec![ModelCapability::ImageGen],
                auth_type: AuthType::ApiKey,
            },
        }
    }

    /// Creates a new HuggingFaceImageProvider with a specific default model.
    pub fn with_model(model_id: impl Into<String>) -> Self {
        let model = model_id.into();
        Self {
            http_client: Client::new(),
            default_model: model.clone(),
            metadata: ProviderMetadata {
                id: "huggingface".to_string(),
                name: "HuggingFace".to_string(),
                kind: ProviderKind::HuggingFace,
                base_url: "https://api-inference.huggingface.co".to_string(),
                supported_capabilities: vec![ModelCapability::ImageGen],
                auth_type: AuthType::ApiKey,
            },
        }
    }
}

impl Default for HuggingFaceImageProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ImageProvider for HuggingFaceImageProvider {
    async fn generate(
        &self,
        params: &ImageGenParams,
        api_key: &str,
    ) -> Result<ImageGenResult, ProviderError> {
        // Validate params
        params.validate().map_err(|e| {
            ProviderError::ProviderSpecific("huggingface".to_string(), e)
        })?;

        // Use model_id from params if provided, otherwise use default
        let model_id = params.model_id.clone()
            .unwrap_or_else(|| self.default_model.clone());

        // HuggingFace Inference API endpoint for text-to-image
        let url = format!(
            "https://api-inference.huggingface.co/models/{}",
            model_id
        );

        // Build request body
        let request_body = HuggingFaceRequest {
            inputs: params.prompt.clone(),
            parameters: Some(HuggingFaceParameters {
                negative_prompt: params.negative_prompt.clone(),
                guidance_scale: Some(params.guidance_scale),
                num_inference_steps: Some(params.steps),
                seed: params.seed,
                width: Some(params.width),
                height: Some(params.height),
            }),
        };

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let status = response.status();

        // Handle error responses
        if !status.is_success() {
            return Err(map_huggingface_error(status.as_u16(), response.text().await.unwrap_or_default()).await);
        }

        // HuggingFace returns image bytes directly on success
        let image_data = response
            .bytes()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?
            .to_vec();

        Ok(ImageGenResult::new(
            image_data,
            params.width,
            params.height,
            "png",
        ))
    }

    async fn remove_background(
        &self,
        image_data: &[u8],
        api_key: &str,
    ) -> Result<ImageGenResult, ProviderError> {
        use base64::Engine;
        // HuggingFace background removal model
        let model_id = "XuCluster/BgRemoval";

        let url = format!(
            "https://api-inference.huggingface.co/models/{}",
            model_id
        );

        // Encode image to base64 for HF API
        let image_b64 = base64::engine::general_purpose::STANDARD.encode(image_data);

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "inputs": format!("data:image/png;base64,{}", image_b64)
            }))
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let status = response.status();

        if !status.is_success() {
            return Err(map_huggingface_error(status.as_u16(), response.text().await.unwrap_or_default()).await);
        }

        let result_data = response
            .bytes()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?
            .to_vec();

        Ok(ImageGenResult::new(result_data, 0, 0, "png"))
    }

    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }
}

/// Maps HTTP status codes to ProviderError variants.
async fn map_huggingface_error(status: u16, body: String) -> ProviderError {
    match status {
        401 | 403 => ProviderError::AuthFailed {
            provider: "huggingface".to_string(),
            message: "Invalid or missing API key".to_string(),
        },
        422 => ProviderError::ProviderSpecific(
            "huggingface".to_string(),
            format!("Invalid request: {}", body),
        ),
        503 => {
            // Model is loading or unavailable
            ProviderError::ProviderSpecific(
                "huggingface".to_string(),
                "Model is loading or temporarily unavailable. Please retry.".to_string(),
            )
        }
        429 => ProviderError::RateLimited {
            provider: "huggingface".to_string(),
            retry_after_secs: None,
        },
        _ => ProviderError::ProviderSpecific(
            "huggingface".to_string(),
            format!("HTTP {}: {}", status, body),
        ),
    }
}

// === HuggingFace API Types ===

#[derive(Debug, Serialize)]
struct HuggingFaceRequest {
    inputs: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<HuggingFaceParameters>,
}

#[derive(Debug, Serialize)]
struct HuggingFaceParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    negative_prompt: Option<String>,
    #[serde(rename = "guidance_scale", skip_serializing_if = "Option::is_none")]
    guidance_scale: Option<f32>,
    #[serde(rename = "num_inference_steps", skip_serializing_if = "Option::is_none")]
    num_inference_steps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use artifex_model_config::provider::ProviderError;

    #[test]
    fn test_huggingface_provider_metadata() {
        let provider = HuggingFaceImageProvider::new();
        let metadata = provider.metadata();

        assert_eq!(metadata.name, "HuggingFace");
        assert_eq!(metadata.kind, ProviderKind::HuggingFace);
        assert_eq!(metadata.base_url, "https://api-inference.huggingface.co");
        assert!(metadata.supported_capabilities.contains(&ModelCapability::ImageGen));
        assert_eq!(metadata.auth_type, AuthType::ApiKey);
    }

    #[test]
    fn test_huggingface_provider_with_custom_model() {
        let provider = HuggingFaceImageProvider::with_model("stabilityai/stable-diffusion-2-1");
        assert_eq!(provider.metadata().name, "HuggingFace");
    }

    #[tokio::test]
    async fn test_map_huggingface_error_401() {
        let err = map_huggingface_error(401, "Unauthorized".to_string()).await;
        match err {
            ProviderError::AuthFailed { provider, .. } => {
                assert_eq!(provider, "huggingface");
            }
            _ => panic!("Expected AuthFailed"),
        }
    }

    #[tokio::test]
    async fn test_map_huggingface_error_429() {
        let err = map_huggingface_error(429, "Rate limit exceeded".to_string()).await;
        match err {
            ProviderError::RateLimited { provider, .. } => {
                assert_eq!(provider, "huggingface");
            }
            _ => panic!("Expected RateLimited"),
        }
    }

    #[tokio::test]
    async fn test_map_huggingface_error_503() {
        let err = map_huggingface_error(503, "Model loading".to_string()).await;
        match err {
            ProviderError::ProviderSpecific(provider, _) => {
                assert_eq!(provider, "huggingface");
            }
            _ => panic!("Expected ProviderSpecific"),
        }
    }

    #[tokio::test]
    async fn test_image_gen_params_validation() {
        let provider = HuggingFaceImageProvider::new();
        let params = ImageGenParams {
            prompt: "".to_string(),
            width: 512,
            height: 512,
            steps: 20,
            ..Default::default()
        };

        let result = provider.generate(&params, "fake-key").await;
        assert!(result.is_err());
    }
}