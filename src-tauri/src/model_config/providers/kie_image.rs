//! Kie AI image generation provider.
//!
//! Implements the ImageProvider trait for Kie AI's Flux Kontext API.
//! Uses polling-based async task API with 2s interval and 60-attempt max.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use artifex_model_config::{
    image_provider::{ImageGenParams, ImageGenResult, ImageProvider},
    provider::{AuthType, ModelCapability, ProviderError, ProviderKind, ProviderMetadata},
};

/// Kie AI API client for image generation.
#[derive(Debug, Clone)]
pub struct KieImageProvider {
    http_client: Client,
    metadata: ProviderMetadata,
}

impl KieImageProvider {
    /// Creates a new KieImageProvider.
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
            metadata: ProviderMetadata {
                id: "kie".to_string(),
                name: "Kie AI".to_string(),
                kind: ProviderKind::Kie,
                base_url: "https://api.kie.ai".to_string(),
                supported_capabilities: vec![ModelCapability::ImageGen],
                auth_type: AuthType::ApiKey,
            },
        }
    }

    /// Creates a new KieImageProvider with a custom HTTP client.
    pub fn with_client(http_client: Client) -> Self {
        Self {
            http_client,
            metadata: ProviderMetadata {
                id: "kie".to_string(),
                name: "Kie AI".to_string(),
                kind: ProviderKind::Kie,
                base_url: "https://api.kie.ai".to_string(),
                supported_capabilities: vec![ModelCapability::ImageGen],
                auth_type: AuthType::ApiKey,
            },
        }
    }
}

impl Default for KieImageProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ImageProvider for KieImageProvider {
    async fn generate(
        &self,
        params: &ImageGenParams,
        api_key: &str,
    ) -> Result<ImageGenResult, ProviderError> {
        // Validate params
        params.validate().map_err(|e| {
            ProviderError::ProviderSpecific("kie".to_string(), e)
        })?;

        // Log model_id if provided.
        // NOTE: Kie AI uses a fixed endpoint-based API (flux-kontext). The model_id from routing
        // is used for logging/debugging purposes and to select the appropriate endpoint variant.
        // Kie does not support per-request model selection via API parameters - all requests
        // go to the same /flux/kontext endpoint regardless of model_id.
        if let Some(ref model_id) = params.model_id {
            tracing::debug!(
                "Kie image generation with routed model_id '{}' (Kie uses fixed kontext endpoint)",
                model_id
            );
        } else {
            tracing::debug!("Kie image generation using default model (flux-kontext)");
        }

        // Step 1: Start generation task
        let task_id = self.start_generation(params, api_key).await?;

        // Step 2: Poll for completion
        let image_url = self.poll_for_result(&task_id, api_key).await?;

        // Step 3: Download the image
        let image_data = self.download_image(&image_url).await?;

        Ok(ImageGenResult::new(
            image_data,
            params.width,
            params.height,
            "png",
        ))
    }

    async fn remove_background(
        &self,
        _image_data: &[u8],
        _api_key: &str,
    ) -> Result<ImageGenResult, ProviderError> {
        // Kie AI does not support background removal
        Err(ProviderError::ProviderSpecific(
            "kie".to_string(),
            "Kie AI does not support background removal. Please use Replicate or Fal.".to_string(),
        ))
    }

    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }
}

impl KieImageProvider {
    /// Starts a generation task and returns the task ID.
    async fn start_generation(
        &self,
        params: &ImageGenParams,
        api_key: &str,
    ) -> Result<String, ProviderError> {
        // Kie API uses aspect_ratio, not image_size
        let aspect_ratio = match (params.width, params.height) {
            (1024, 1024) => "1:1".to_string(),
            (768, 1024) => "3:4".to_string(),
            (1024, 768) => "4:3".to_string(),
            (512, 512) => "1:1".to_string(),
            (640, 1536) => "5:12".to_string(),
            (1536, 640) => "12:5".to_string(),
            (640, 1280) => "1:2".to_string(),
            (1280, 640) => "2:1".to_string(),
            // Default fallback
            (w, h) => format!("{}:{}", w, h),
        };

        let request_body = KieGenerateRequest {
            prompt: params.prompt.clone(),
            aspect_ratio,
        };

        let response = self
            .http_client
            .post("https://api.kie.ai/api/v1/flux/kontext/generate")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let status = response.status().as_u16();
        if status == 401 || status == 403 {
            return Err(ProviderError::AuthFailed {
                provider: "kie".to_string(),
                message: "Invalid or missing API key".to_string(),
            });
        }
        if status == 429 {
            return Err(ProviderError::RateLimited {
                provider: "kie".to_string(),
                retry_after_secs: None,
            });
        }
        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::ProviderSpecific(
                "kie".to_string(),
                format!("HTTP {}: {}", status, body),
            ));
        }

        let resp: KieGenerateResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ProviderSpecific("kie".to_string(), e.to_string()))?;

        Ok(resp.task_id)
    }

    /// Polls for the generation result until completion.
    async fn poll_for_result(&self, task_id: &str, api_key: &str) -> Result<String, ProviderError> {
        let max_attempts = 60;
        let delay = Duration::from_secs(2);
        let poll_url = format!(
            "https://api.kie.ai/api/v1/flux/kontext/record-info?taskId={}",
            task_id
        );

        for _ in 0..max_attempts {
            let response = self
                .http_client
                .get(&poll_url)
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await
                .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

            let status = response.status().as_u16();
            if status == 401 || status == 403 {
                return Err(ProviderError::AuthFailed {
                    provider: "kie".to_string(),
                    message: "Invalid or missing API key".to_string(),
                });
            }
            if status == 429 {
                return Err(ProviderError::RateLimited {
                    provider: "kie".to_string(),
                    retry_after_secs: None,
                });
            }

            let status_resp: KieStatusResponse = response
                .json()
                .await
                .map_err(|e| ProviderError::ProviderSpecific("kie".to_string(), e.to_string()))?;

            // Kie API uses successFlag: 0 = processing, 1 = success, 2 = create-task-failed, 3 = generate-failed
            match status_resp.success_flag {
                0 => {
                    // Still processing, wait and poll again
                    tokio::time::sleep(delay).await;
                }
                1 => {
                    // Success - return the image URL
                    return status_resp
                        .image_url
                        .ok_or_else(|| {
                            ProviderError::ProviderSpecific(
                                "kie".to_string(),
                                "No image URL in successful response".to_string(),
                            )
                        });
                }
                2 | 3 => {
                    return Err(ProviderError::ProviderSpecific(
                        "kie".to_string(),
                        status_resp.error_message.unwrap_or_else(|| "Generation failed".to_string()),
                    ));
                }
                _ => {
                    return Err(ProviderError::ProviderSpecific(
                        "kie".to_string(),
                        format!("Unknown successFlag: {}", status_resp.success_flag),
                    ));
                }
            }
        }

        Err(ProviderError::Timeout {
            provider: "kie".to_string(),
            message: "Generation timed out after maximum polling attempts".to_string(),
        })
    }

    /// Downloads the image from the given URL.
    async fn download_image(&self, image_url: &str) -> Result<Vec<u8>, ProviderError> {
        let response = self
            .http_client
            .get(image_url)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ProviderError::NetworkError(format!(
                "Failed to download image: HTTP {}",
                response.status().as_u16()
            )));
        }

        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| ProviderError::NetworkError(e.to_string()))
    }
}

// === Kie API Types ===

#[derive(Debug, Serialize)]
struct KieGenerateRequest {
    prompt: String,
    #[serde(rename = "aspect_ratio")]
    aspect_ratio: String,
}

#[derive(Debug, Deserialize)]
struct KieGenerateResponse {
    #[serde(rename = "taskId")]
    task_id: String,
}

#[derive(Debug, Deserialize)]
struct KieStatusResponse {
    #[serde(rename = "successFlag")]
    success_flag: u32,
    #[serde(rename = "imageUrl")]
    #[serde(skip_serializing_if = "Option::is_none")]
    image_url: Option<String>,
    #[serde(rename = "errorMessage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    error_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kie_provider_metadata() {
        let provider = KieImageProvider::new();
        let metadata = provider.metadata();

        assert_eq!(metadata.name, "Kie AI");
        assert_eq!(metadata.id, "kie");
        assert_eq!(metadata.kind, ProviderKind::Kie);
        assert_eq!(metadata.base_url, "https://api.kie.ai");
        assert!(metadata.supported_capabilities.contains(&ModelCapability::ImageGen));
        assert_eq!(metadata.auth_type, AuthType::ApiKey);
    }

    #[tokio::test]
    async fn test_image_gen_params_validation() {
        let provider = KieImageProvider::new();
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