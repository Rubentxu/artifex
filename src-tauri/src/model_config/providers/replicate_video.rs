//! Replicate video generation provider.
//!
//! Implements the VideoProvider trait for Replicate's image-to-video API.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use artifex_model_config::{
    provider::{AuthType, ModelCapability, ProviderError, ProviderKind, ProviderMetadata},
    video_provider::{VideoGenParams, VideoGenResult, VideoProvider},
};

/// Replicate API client for video generation.
#[derive(Debug, Clone)]
pub struct ReplicateVideoProvider {
    http_client: Client,
    metadata: ProviderMetadata,
}

impl ReplicateVideoProvider {
    /// Creates a new ReplicateVideoProvider.
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
            metadata: ProviderMetadata {
                id: "replicate".to_string(),
                name: "Replicate".to_string(),
                kind: ProviderKind::Replicate,
                base_url: "https://api.replicate.com/v1".to_string(),
                supported_capabilities: vec![ModelCapability::VideoGen],
                auth_type: AuthType::ApiKey,
            },
        }
    }

    /// Creates a new ReplicateVideoProvider with a custom HTTP client.
    pub fn with_client(http_client: Client) -> Self {
        Self {
            http_client,
            metadata: ProviderMetadata {
                id: "replicate".to_string(),
                name: "Replicate".to_string(),
                kind: ProviderKind::Replicate,
                base_url: "https://api.replicate.com/v1".to_string(),
                supported_capabilities: vec![ModelCapability::VideoGen],
                auth_type: AuthType::ApiKey,
            },
        }
    }
}

impl Default for ReplicateVideoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VideoProvider for ReplicateVideoProvider {
    async fn generate_video(
        &self,
        params: &VideoGenParams,
        api_key: &str,
    ) -> Result<VideoGenResult, ProviderError> {
        // Validate params
        params.validate().map_err(|e| {
            ProviderError::ProviderSpecific("replicate".to_string(), e)
        })?;

        // Use model_id from params if provided, otherwise use default Wan 2.5 model
        let model_version = params
            .model_id
            .clone()
            .unwrap_or_else(|| "minimax/minimax-hyper-svd-video-256w".to_string());

        // Create a prediction request for video generation
        let request_body = ReplicateVideoPredictionRequest {
            version: model_version.clone(),
            input: ReplicateVideoInput {
                image: params.source_image_url.clone(),
                prompt: params.prompt.clone(),
                negative_prompt: params.negative_prompt.clone(),
                duration: params.duration_secs,
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

        // Get output video URL
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

        // Fetch the video data
        let video_response = self
            .http_client
            .get(&output_url)
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

impl ReplicateVideoProvider {
    /// Polls a prediction until it completes or fails.
    async fn poll_prediction(
        &self,
        poll_url: &str,
        api_key: &str,
    ) -> Result<ReplicatePredictionResponse, ProviderError> {
        let max_attempts = 90; // Video generation can take longer
        let delay = std::time::Duration::from_secs(3);

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
            message: "Video generation timed out after maximum polling attempts".to_string(),
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
struct ReplicateVideoPredictionRequest {
    version: String,
    input: ReplicateVideoInput,
}

#[derive(Debug, Serialize)]
struct ReplicateVideoInput {
    /// Source image URL or base64 data URI.
    image: String,
    /// Text prompt for video generation.
    prompt: String,
    /// Optional negative prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    negative_prompt: Option<String>,
    /// Duration in seconds (2-8).
    duration: u8,
    /// Optional seed for reproducibility.
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ReplicatePredictionResponse {
    id: String,
    status: String,
    output: Option<serde_json::Value>,
    error: Option<String>,
    urls: ReplicateUrls,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ReplicateUrls {
    poll: String,
    get: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use artifex_model_config::provider::ProviderError;

    #[test]
    fn test_replicate_video_provider_metadata() {
        let provider = ReplicateVideoProvider::new();
        let metadata = provider.metadata();

        assert_eq!(metadata.name, "Replicate");
        assert_eq!(metadata.kind, ProviderKind::Replicate);
        assert_eq!(metadata.base_url, "https://api.replicate.com/v1");
        assert!(metadata.supported_capabilities.contains(&ModelCapability::VideoGen));
        assert_eq!(metadata.auth_type, AuthType::ApiKey);
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
    async fn test_video_gen_params_validation_empty_prompt() {
        let provider = ReplicateVideoProvider::new();
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
