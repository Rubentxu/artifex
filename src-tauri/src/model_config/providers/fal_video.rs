//! Fal.ai video generation provider.
//!
//! Implements the VideoProvider trait for Fal's image-to-video API.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use artifex_model_config::{
    provider::{AuthType, ModelCapability, ProviderError, ProviderKind, ProviderMetadata},
    video_provider::{VideoGenParams, VideoGenResult, VideoProvider},
};

/// Fal API client for video generation.
#[derive(Debug, Clone)]
pub struct FalVideoProvider {
    http_client: Client,
    metadata: ProviderMetadata,
}

impl FalVideoProvider {
    /// Creates a new FalVideoProvider.
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
            metadata: ProviderMetadata {
                id: "fal".to_string(),
                name: "Fal".to_string(),
                kind: ProviderKind::Fal,
                base_url: "https://queue.fal.run".to_string(),
                supported_capabilities: vec![ModelCapability::VideoGen],
                auth_type: AuthType::ApiKey,
            },
        }
    }

    /// Creates a new FalVideoProvider with a custom HTTP client.
    pub fn with_client(http_client: Client) -> Self {
        Self {
            http_client,
            metadata: ProviderMetadata {
                id: "fal".to_string(),
                name: "Fal".to_string(),
                kind: ProviderKind::Fal,
                base_url: "https://queue.fal.run".to_string(),
                supported_capabilities: vec![ModelCapability::VideoGen],
                auth_type: AuthType::ApiKey,
            },
        }
    }
}

impl Default for FalVideoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VideoProvider for FalVideoProvider {
    async fn generate_video(
        &self,
        params: &VideoGenParams,
        api_key: &str,
    ) -> Result<VideoGenResult, ProviderError> {
        // Validate params
        params.validate().map_err(|e| {
            ProviderError::ProviderSpecific("fal".to_string(), e)
        })?;

        // Use model_id from params if provided, otherwise use default Kling model
        let model_id = params
            .model_id
            .clone()
            .unwrap_or_else(|| "fal-ai/kling-video".to_string());

        // Build the request for Fal's video API
        // Fal uses the same queue API structure for video
        let request_body = FalVideoQueueRequest {
            model: model_id.clone(),
            input: FalVideoInput {
                image_url: params.source_image_url.clone(),
                prompt: params.prompt.clone(),
                duration: params.duration_secs as f32,
                negative_prompt: params.negative_prompt.clone(),
                seed: params.seed,
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

        // Poll for completion
        let final_result = self.poll_result(&queue_response.request_id, api_key).await?;

        // Fetch the video from the output URL
        let video_url = final_result
            .video
            .as_ref()
            .map(|v| v.url.as_str())
            .ok_or_else(|| {
                ProviderError::ProviderSpecific(
                    "fal".to_string(),
                    "No video in response".to_string(),
                )
            })?;

        let video_response = self
            .http_client
            .get(video_url)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let video_data = video_response
            .bytes()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?
            .to_vec();

        Ok(VideoGenResult::new(
            video_data,
            params.duration_secs as f32,
            "mp4",
        ))
    }

    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }
}

impl FalVideoProvider {
    /// Polls for the result of a queued request.
    async fn poll_result(
        &self,
        request_id: &str,
        api_key: &str,
    ) -> Result<FalVideoResultResponse, ProviderError> {
        let max_attempts = 90; // Video generation can take longer than image generation
        let delay = std::time::Duration::from_secs(3);

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
                        .get(format!("https://queue.fal.run/requests/{}/results", request_id))
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
            message: "Video generation timed out after maximum polling attempts".to_string(),
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
struct FalVideoQueueRequest {
    model: String,
    input: FalVideoInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    webhook_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct FalVideoInput {
    /// URL or data URI of the source image.
    image_url: String,
    /// Text prompt for video generation.
    prompt: String,
    /// Duration in seconds.
    duration: f32,
    /// Optional negative prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    negative_prompt: Option<String>,
    /// Optional seed for reproducibility.
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
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
struct FalVideoResultResponse {
    video: Option<FalVideo>,
}

#[derive(Debug, Deserialize)]
struct FalVideo {
    url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use artifex_model_config::provider::ProviderError;

    #[test]
    fn test_fal_video_provider_metadata() {
        let provider = FalVideoProvider::new();
        let metadata = provider.metadata();

        assert_eq!(metadata.name, "Fal");
        assert_eq!(metadata.kind, ProviderKind::Fal);
        assert_eq!(metadata.base_url, "https://queue.fal.run");
        assert!(metadata.supported_capabilities.contains(&ModelCapability::VideoGen));
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
    async fn test_video_gen_params_validation_empty_prompt() {
        let provider = FalVideoProvider::new();
        let params = VideoGenParams {
            source_image_url: "data:image/png;base64,abc123".to_string(),
            prompt: "".to_string(),
            duration_secs: 4,
            negative_prompt: None,
            seed: None,
            model_id: None,
        };

        let result = provider.generate_video(&params, "fake-key").await;
        assert!(result.is_err());
    }
}
