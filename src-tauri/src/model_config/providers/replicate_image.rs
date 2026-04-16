//! Replicate image generation provider.
//!
//! Implements the ImageProvider trait for Replicate's API.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use artifex_model_config::{
    image_provider::{ImageGenParams, ImageGenResult, ImageProvider},
    provider::{AuthType, ModelCapability, ProviderError, ProviderKind, ProviderMetadata},
};

/// Replicate API client for image generation.
#[derive(Debug, Clone)]
pub struct ReplicateImageProvider {
    http_client: Client,
    metadata: ProviderMetadata,
}

impl ReplicateImageProvider {
    /// Creates a new ReplicateImageProvider.
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
            metadata: ProviderMetadata {
                id: "replicate".to_string(),
                name: "Replicate".to_string(),
                kind: ProviderKind::Replicate,
                base_url: "https://api.replicate.com/v1".to_string(),
                supported_capabilities: vec![ModelCapability::ImageGen],
                auth_type: AuthType::ApiKey,
            },
        }
    }

    /// Creates a new ReplicateImageProvider with a custom HTTP client.
    pub fn with_client(http_client: Client) -> Self {
        Self {
            http_client,
            metadata: ProviderMetadata {
                id: "replicate".to_string(),
                name: "Replicate".to_string(),
                kind: ProviderKind::Replicate,
                base_url: "https://api.replicate.com/v1".to_string(),
                supported_capabilities: vec![ModelCapability::ImageGen],
                auth_type: AuthType::ApiKey,
            },
        }
    }
}

impl Default for ReplicateImageProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ImageProvider for ReplicateImageProvider {
    async fn generate(
        &self,
        params: &ImageGenParams,
        api_key: &str,
    ) -> Result<ImageGenResult, ProviderError> {
        // Validate params
        params.validate().map_err(|e| {
            ProviderError::ProviderSpecific("replicate".to_string(), e)
        })?;

        // Use model_id from params if provided, otherwise use default
        let model_version = params.model_id.clone()
            .unwrap_or_else(|| "stability-ai/sdxl:39ed52f2a78e934b3ba6e2a89f5b1c712de7dfea535525255b1aa35c5565e08b".to_string());

        // Create a prediction request
        let request_body = ReplicatePredictionRequest {
            version: model_version,
            input: ReplicateInput {
                prompt: params.prompt.clone(),
                negative_prompt: params.negative_prompt.clone(),
                width: params.width,
                height: params.height,
                num_outputs: params.num_images,
                guidance_scale: Some(params.guidance_scale),
                num_inference_steps: Some(params.steps),
                seed: params.seed,
            },
        };

        // Start prediction
        let response = self
            .http_client
            .post("https://api.replicate.com/v1/predictions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(map_replicate_error(response.status().as_u16(), response.text().await.unwrap_or_default()).await);
        }

        let prediction: ReplicatePredictionResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ProviderSpecific("replicate".to_string(), e.to_string()))?;

        // Poll for completion
        let final_prediction = self.poll_prediction(&prediction.urls.poll, api_key).await?;

        // Get output image
        // Output can be an array of URLs or a single URL string
        let output_url: String = if let Some(output) = final_prediction.output {
            if let Some(arr) = output.as_array() {
                arr.first()
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| {
                        ProviderError::ProviderSpecific(
                            "replicate".to_string(),
                            "Output array element is not a string".to_string(),
                        )
                    })?
            } else if let Some(s) = output.as_str() {
                s.to_string()
            } else {
                return Err(ProviderError::ProviderSpecific(
                    "replicate".to_string(),
                    "Output is not a valid URL string or array".to_string(),
                ));
            }
        } else {
            return Err(ProviderError::ProviderSpecific(
                "replicate".to_string(),
                "No output in prediction response".to_string(),
            ));
        };

        // Fetch the actual image data
        let image_response = self
            .http_client
            .get(output_url)
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

impl ReplicateImageProvider {
    /// Polls a prediction until it completes or fails.
    async fn poll_prediction(
        &self,
        poll_url: &str,
        api_key: &str,
    ) -> Result<ReplicatePredictionResponse, ProviderError> {
        let max_attempts = 60;
        let delay = std::time::Duration::from_secs(2);

        for _ in 0..max_attempts {
            let response = self
                .http_client
                .get(poll_url)
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await
                .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

            let prediction: ReplicatePredictionResponse = response
                .json()
                .await
                .map_err(|e| ProviderError::ProviderSpecific("replicate".to_string(), e.to_string()))?;

            match prediction.status.as_str() {
                "succeeded" => return Ok(prediction),
                "failed" => {
                    return Err(ProviderError::ProviderSpecific(
                        "replicate".to_string(),
                        prediction.error.unwrap_or_else(|| "Prediction failed".to_string()),
                    ));
                }
                "canceled" => {
                    return Err(ProviderError::ProviderSpecific(
                        "replicate".to_string(),
                        "Prediction was canceled".to_string(),
                    ));
                }
                _ => {
                    // Still processing, wait and poll again
                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(ProviderError::Timeout {
            provider: "replicate".to_string(),
            message: "Prediction timed out after maximum polling attempts".to_string(),
        })
    }
}

/// Maps HTTP status codes to ProviderError variants.
async fn map_replicate_error(status: u16, body: String) -> ProviderError {
    match status {
        401 | 403 => ProviderError::AuthFailed {
            provider: "replicate".to_string(),
            message: "Invalid or missing API key".to_string(),
        },
        429 => ProviderError::RateLimited {
            provider: "replicate".to_string(),
            retry_after_secs: None,
        },
        404 => ProviderError::ModelNotFound {
            model_id: "unknown".to_string(),
        },
        422 => ProviderError::ProviderSpecific(
            "replicate".to_string(),
            format!("Invalid request: {}", body),
        ),
        _ => ProviderError::ProviderSpecific(
            "replicate".to_string(),
            format!("HTTP {}: {}", status, body),
        ),
    }
}

// === Replicate API Types ===

#[derive(Debug, Serialize)]
struct ReplicatePredictionRequest {
    version: String,
    input: ReplicateInput,
}

#[derive(Debug, Serialize)]
struct ReplicateInput {
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    negative_prompt: Option<String>,
    width: u32,
    height: u32,
    #[serde(rename = "num_outputs")]
    num_outputs: u32,
    #[serde(rename = "guidance_scale")]
    #[serde(skip_serializing_if = "Option::is_none")]
    guidance_scale: Option<f32>,
    #[serde(rename = "num_inference_steps")]
    #[serde(skip_serializing_if = "Option::is_none")]
    num_inference_steps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ReplicatePredictionResponse {
    id: String,
    status: String,
    output: Option<serde_json::Value>,
    error: Option<String>,
    urls: ReplicateUrls,
}

#[derive(Debug, Deserialize)]
struct ReplicateUrls {
    poll: String,
    get: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use artifex_model_config::provider::ProviderError;

    #[test]
    fn test_replicate_provider_metadata() {
        let provider = ReplicateImageProvider::new();
        let metadata = provider.metadata();

        assert_eq!(metadata.name, "Replicate");
        assert_eq!(metadata.kind, ProviderKind::Replicate);
        assert_eq!(metadata.base_url, "https://api.replicate.com/v1");
        assert!(metadata.supported_capabilities.contains(&ModelCapability::ImageGen));
        assert_eq!(metadata.auth_type, AuthType::ApiKey);
    }

    #[test]
    fn test_provider_error_display() {
        let err = ProviderError::AuthFailed {
            provider: "replicate".to_string(),
            message: "Invalid API key".to_string(),
        };
        assert!(err.to_string().contains("Authentication failed"));
        assert!(err.to_string().contains("replicate"));

        let err = ProviderError::RateLimited {
            provider: "replicate".to_string(),
            retry_after_secs: Some(60),
        };
        assert!(err.to_string().contains("Rate limited"));
    }

    #[tokio::test]
    async fn test_map_replicate_error_401() {
        let err = map_replicate_error(401, "Unauthorized".to_string()).await;
        match err {
            ProviderError::AuthFailed { provider, .. } => {
                assert_eq!(provider, "replicate");
            }
            _ => panic!("Expected AuthFailed"),
        }
    }

    #[tokio::test]
    async fn test_map_replicate_error_429() {
        let err = map_replicate_error(429, "Rate limit exceeded".to_string()).await;
        match err {
            ProviderError::RateLimited { provider, .. } => {
                assert_eq!(provider, "replicate");
            }
            _ => panic!("Expected RateLimited"),
        }
    }

    #[tokio::test]
    async fn test_map_replicate_error_404() {
        let err = map_replicate_error(404, "Not found".to_string()).await;
        match err {
            ProviderError::ModelNotFound { model_id } => {
                assert_eq!(model_id, "unknown");
            }
            _ => panic!("Expected ModelNotFound"),
        }
    }

    #[tokio::test]
    async fn test_image_gen_params_validation() {
        let provider = ReplicateImageProvider::new();
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