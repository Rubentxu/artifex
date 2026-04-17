//! Fal.ai image generation provider.
//!
//! Implements the ImageProvider trait for Fal's API.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use artifex_model_config::{
    image_provider::{ImageEditParams, ImageGenParams, ImageGenResult, ImageProvider, MapKind, MaterialGenParams, MaterialResult},
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
                supported_capabilities: vec![ModelCapability::ImageGen, ModelCapability::ImageEdit],
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
                supported_capabilities: vec![ModelCapability::ImageGen, ModelCapability::ImageEdit],
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

    async fn remove_background(
        &self,
        image_data: &[u8],
        api_key: &str,
    ) -> Result<ImageGenResult, ProviderError> {
        use base64::Engine;
        // Fal has a background removal endpoint
        let model_id = "fal-ai/rmbg";

        // Encode image to base64
        let image_b64 = base64::engine::general_purpose::STANDARD.encode(image_data);

        // Create request for background removal
        let request_body = serde_json::json!({
            "model": model_id,
            "input": {
                "image_url": format!("data:image/png;base64,{}", image_b64)
            }
        });

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

        let result_data = image_response
            .bytes()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?
            .to_vec();

        Ok(ImageGenResult::new(result_data, 0, 0, "png"))
    }

    async fn inpaint(
        &self,
        image_data: &[u8],
        mask_data: &[u8],
        params: &ImageEditParams,
        api_key: &str,
    ) -> Result<ImageGenResult, ProviderError> {
        use base64::Engine;

        // Validate params
        params.validate().map_err(|e| {
            ProviderError::ProviderSpecific("fal".to_string(), e)
        })?;

        // Use flux-fill-dev model for inpainting
        let model_id = params
            .model_id
            .clone()
            .unwrap_or_else(|| "fal-ai/flux-fill-dev".to_string());

        // Encode image and mask to base64
        let image_b64 = base64::engine::general_purpose::STANDARD.encode(image_data);
        let mask_b64 = base64::engine::general_purpose::STANDARD.encode(mask_data);

        // Create request for inpainting using Fal's flux-fill endpoint
        // The input format expects base64 images as data URIs
        let request_body = FalInpaintRequest {
            model: model_id.clone(),
            input: FalInpaintInput {
                image_url: format!("data:image/png;base64,{}", image_b64),
                mask_url: format!("data:image/png;base64,{}", mask_b64),
                prompt: params.prompt.clone(),
                negative_prompt: params.negative_prompt.clone(),
                guidance_scale: params.guidance_scale,
                num_inference_steps: params.num_inference_steps,
                seed: params.seed,
            },
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

        let result_data = image_response
            .bytes()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?
            .to_vec();

        // Get image dimensions from the result
        // We don't know the exact dimensions without decoding, so use 0 as placeholder
        Ok(ImageGenResult::new(result_data, 0, 0, "png"))
    }

    async fn generate_material(
        &self,
        image_data: &[u8],
        params: &MaterialGenParams,
        api_key: &str,
    ) -> Result<MaterialResult, ProviderError> {
        use base64::Engine;
        use std::collections::HashMap;

        // Fal PATINA material endpoint
        let model_id = "fal-ai/patina/material";

        // Encode image to base64
        let image_b64 = base64::engine::general_purpose::STANDARD.encode(image_data);

        // Build request body
        let mut request_body = serde_json::json!({
            "model": model_id,
            "input": {
                "image_url": format!("data:image/png;base64,{}", image_b64)
            }
        });

        // Add resolution if provided
        if let Some(resolution) = params.resolution {
            // PATINA expects resolution as width/height
            request_body["input"]["resolution"] = serde_json::json!(resolution);
        }

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

        // Fal PATINA returns multiple images, one per map type
        // The images array contains [basecolor, normal, roughness, metalness, height] in order
        let mut maps = HashMap::new();
        let map_kinds = [
            MapKind::Basecolor,
            MapKind::Normal,
            MapKind::Roughness,
            MapKind::Metalness,
            MapKind::Height,
        ];

        for (i, img) in final_result.images.iter().enumerate() {
            if i < map_kinds.len() {
                // Download the image
                let image_response = self
                    .http_client
                    .get(&img.url)
                    .send()
                    .await
                    .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

                let image_bytes = image_response
                    .bytes()
                    .await
                    .map_err(|e| ProviderError::NetworkError(e.to_string()))?
                    .to_vec();

                maps.insert(map_kinds[i], image_bytes);
            }
        }

        if maps.is_empty() {
            return Err(ProviderError::ProviderSpecific(
                "fal".to_string(),
                "No material maps returned from PATINA".to_string(),
            ));
        }

        Ok(MaterialResult::new(maps))
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
struct FalResultResponse {
    images: Vec<FalImage>,
}

#[derive(Debug, Deserialize)]
struct FalImage {
    url: String,
}

/// Request body for Fal inpainting API.
#[derive(Debug, Serialize)]
struct FalInpaintRequest {
    model: String,
    input: FalInpaintInput,
}

/// Input parameters for Fal inpainting API.
#[derive(Debug, Serialize)]
struct FalInpaintInput {
    /// Base64-encoded image or URL to the image.
    image_url: String,
    /// Base64-encoded mask or URL to the mask (white = edit region).
    mask_url: String,
    /// Text prompt for the edit.
    prompt: String,
    /// Optional negative prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    negative_prompt: Option<String>,
    /// Guidance scale.
    #[serde(rename = "guidance_scale")]
    guidance_scale: f32,
    /// Number of inference steps.
    #[serde(rename = "num_inference_steps")]
    num_inference_steps: u32,
    /// Optional seed for reproducibility.
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
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