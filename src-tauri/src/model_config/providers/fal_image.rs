//! Fal.ai image generation provider.
//!
//! Implements the ImageProvider trait for Fal's API.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use artifex_model_config::{
    image_provider::{ImageGenParams, ImageGenResult, ImageProvider},
    provider::{AuthType, ModelCapability, ProviderError, ProviderKind, ProviderMetadata},
};

/// Fal API client for image generation.
#[derive(Debug, Clone)]
pub struct FalImageProvider {
    http_client: Client,
    metadata: ProviderMetadata,
}

impl FalImageProvider {
    /// Creates a new FalImageProvider.
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
            metadata: ProviderMetadata {
                id: "fal".to_string(),
                name: "Fal".to_string(),
                kind: ProviderKind::Fal,
                base_url: "https://queue.fal.run".to_string(),
                supported_capabilities: vec![ModelCapability::ImageGen],
                auth_type: AuthType::ApiKey,
            },
        }
    }

    /// Creates a new FalImageProvider with a custom HTTP client.
    pub fn with_client(http_client: Client) -> Self {
        Self {
            http_client,
            metadata: ProviderMetadata {
                id: "fal".to_string(),
                name: "Fal".to_string(),
                kind: ProviderKind::Fal,
                base_url: "https://queue.fal.run".to_string(),
                supported_capabilities: vec![ModelCapability::ImageGen],
                auth_type: AuthType::ApiKey,
            },
        }
    }
}

impl Default for FalImageProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ImageProvider for FalImageProvider {
    async fn generate(
        &self,
        params: &ImageGenParams,
        api_key: &str,
    ) -> Result<ImageGenResult, ProviderError> {
        // Validate params
        params.validate().map_err(|e| {
            ProviderError::ProviderSpecific("fal".to_string(), e)
        })?;

        // Use model_id from params if provided, otherwise use default
        let model_id = params.model_id.clone()
            .unwrap_or_else(|| "fal-ai/flux-dev".to_string());

        // Create a request to the Fal queue
        let request_body = FalQueueRequest {
            model: model_id.clone(),
            input: FalInput {
                prompt: params.prompt.clone(),
                image_size: FalImageSize {
                    width: params.width,
                    height: params.height,
                },
                num_images: params.num_images,
                guidance_scale: params.guidance_scale,
                num_inference_steps: params.steps,
                seed: params.seed,
                enable_safety_checker: Some(true),
            },
            webhook_url: None,
        };

        // Submit to queue
        let response = self
            .http_client
            .post(format!("https://queue.fal.run/{}", model_id))
            .header("Authorization", format!("Key {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(map_fal_error(response.status().as_u16(), response.text().await.unwrap_or_default()).await);
        }

        let queue_response: FalQueueResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ProviderSpecific("fal".to_string(), e.to_string()))?;

        // Poll for completion using the request ID
        let final_result = self.poll_result(&queue_response.request_id, api_key).await?;

        // Fetch the actual image data
        let image_url = final_result
            .images
            .first()
            .map(|img| img.url.as_str())
            .ok_or_else(|| {
                ProviderError::ProviderSpecific(
                    "fal".to_string(),
                    "No images in response".to_string(),
                )
            })?;

        let image_response = self
            .http_client
            .get(image_url)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let image_data = image_response
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

    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }
}

impl FalImageProvider {
    /// Polls for the result of a queued request.
    async fn poll_result(
        &self,
        request_id: &str,
        api_key: &str,
    ) -> Result<FalResultResponse, ProviderError> {
        let max_attempts = 60;
        let delay = std::time::Duration::from_secs(2);

        let status_url = format!("https://queue.fal.run/requests/{}", request_id);

        for _ in 0..max_attempts {
            let response = self
                .http_client
                .get(&status_url)
                .header("Authorization", format!("Key {}", api_key))
                .send()
                .await
                .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

            let status: FalStatusResponse = response
                .json()
                .await
                .map_err(|e| ProviderError::ProviderSpecific("fal".to_string(), e.to_string()))?;

            match status.status.as_str() {
                "COMPLETED" => {
                    // Fetch the actual result
                    let result_response = self
                        .http_client
                        .get(&format!("https://queue.fal.run/requests/{}/results", request_id))
                        .header("Authorization", format!("Key {}", api_key))
                        .send()
                        .await
                        .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

                    return result_response
                        .json()
                        .await
                        .map_err(|e| ProviderError::ProviderSpecific("fal".to_string(), e.to_string()));
                }
                "FAILED" => {
                    return Err(ProviderError::ProviderSpecific(
                        "fal".to_string(),
                        status.error.unwrap_or_else(|| "Request failed".to_string()),
                    ));
                }
                "CANCELLED" => {
                    return Err(ProviderError::ProviderSpecific(
                        "fal".to_string(),
                        "Request was cancelled".to_string(),
                    ));
                }
                _ => {
                    // Still processing, wait and poll again
                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(ProviderError::Timeout {
            provider: "fal".to_string(),
            message: "Request timed out after maximum polling attempts".to_string(),
        })
    }
}

/// Maps HTTP status codes to ProviderError variants.
async fn map_fal_error(status: u16, body: String) -> ProviderError {
    match status {
        401 | 403 => ProviderError::AuthFailed {
            provider: "fal".to_string(),
            message: "Invalid or missing API key".to_string(),
        },
        429 => ProviderError::RateLimited {
            provider: "fal".to_string(),
            retry_after_secs: None,
        },
        404 => ProviderError::ModelNotFound {
            model_id: "unknown".to_string(),
        },
        _ => ProviderError::ProviderSpecific(
            "fal".to_string(),
            format!("HTTP {}: {}", status, body),
        ),
    }
}

// === Fal API Types ===

#[derive(Debug, Serialize)]
struct FalQueueRequest {
    model: String,
    input: FalInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    webhook_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct FalInput {
    prompt: String,
    #[serde(rename = "image_size")]
    image_size: FalImageSize,
    #[serde(rename = "num_images")]
    num_images: u32,
    #[serde(rename = "guidance_scale")]
    guidance_scale: f32,
    #[serde(rename = "num_inference_steps")]
    num_inference_steps: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
    #[serde(rename = "enable_safety_checker")]
    enable_safety_checker: Option<bool>,
}

#[derive(Debug, Serialize)]
struct FalImageSize {
    width: u32,
    height: u32,
}

#[derive(Debug, Deserialize)]
struct FalQueueResponse {
    #[serde(rename = "requestId")]
    request_id: String,
    status: String,
}

#[derive(Debug, Deserialize)]
struct FalStatusResponse {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FalResultResponse {
    images: Vec<FalImage>,
}

#[derive(Debug, Deserialize)]
struct FalImage {
    url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use artifex_model_config::provider::ProviderError;

    #[test]
    fn test_fal_provider_metadata() {
        let provider = FalImageProvider::new();
        let metadata = provider.metadata();

        assert_eq!(metadata.name, "Fal");
        assert_eq!(metadata.kind, ProviderKind::Fal);
        assert_eq!(metadata.base_url, "https://queue.fal.run");
        assert!(metadata.supported_capabilities.contains(&ModelCapability::ImageGen));
        assert_eq!(metadata.auth_type, AuthType::ApiKey);
    }

    #[tokio::test]
    async fn test_map_fal_error_401() {
        let err = map_fal_error(401, "Unauthorized".to_string()).await;
        match err {
            ProviderError::AuthFailed { provider, .. } => {
                assert_eq!(provider, "fal");
            }
            _ => panic!("Expected AuthFailed"),
        }
    }

    #[tokio::test]
    async fn test_map_fal_error_429() {
        let err = map_fal_error(429, "Rate limit exceeded".to_string()).await;
        match err {
            ProviderError::RateLimited { provider, .. } => {
                assert_eq!(provider, "fal");
            }
            _ => panic!("Expected RateLimited"),
        }
    }

    #[tokio::test]
    async fn test_image_gen_params_validation() {
        let provider = FalImageProvider::new();
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