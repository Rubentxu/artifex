//! ElevenLabs audio provider implementation.

use reqwest::Client;
use serde_json::json;

use artifex_model_config::audio_provider::{AudioGenParams, AudioGenResult, AudioProvider};
use artifex_model_config::provider::{AuthType, ModelCapability, ProviderError, ProviderMetadata};
use artifex_model_config::tts_provider::{TtsParams, TtsResult, TtsProvider};
use async_trait::async_trait;

/// ElevenLabs provider for TTS and audio generation.
pub struct ElevenLabsProvider {
    client: Client,
    base_url: String,
    metadata: ProviderMetadata,
}

impl ElevenLabsProvider {
    /// Creates a new ElevenLabs provider.
    pub fn new() -> Self {
        let base_url = "https://api.elevenlabs.io/v1".to_string();
        let metadata = ProviderMetadata {
            id: "elevenlabs".to_string(),
            name: "ElevenLabs".to_string(),
            kind: artifex_model_config::provider::ProviderKind::ElevenLabs,
            base_url: base_url.clone(),
            supported_capabilities: vec![ModelCapability::Tts, ModelCapability::AudioGen],
            auth_type: AuthType::ApiKey,
        };
        Self {
            client: Client::new(),
            base_url,
            metadata,
        }
    }

    /// Maps HTTP status code to provider error.
    fn map_status(status: reqwest::StatusCode, body: &str) -> ProviderError {
        match status.as_u16() {
            401 => ProviderError::AuthFailed {
                provider: "elevenlabs".to_string(),
                message: body.to_string(),
            },
            429 => ProviderError::RateLimited {
                provider: "elevenlabs".to_string(),
                retry_after_secs: None,
            },
            402 => ProviderError::QuotaExceeded {
                provider: "elevenlabs".to_string(),
                message: body.to_string(),
            },
            422 => ProviderError::ProviderSpecific("elevenlabs".to_string(), body.to_string()),
            status_code => ProviderError::ProviderSpecific(
                "elevenlabs".to_string(),
                format!("HTTP {}: {}", status_code, body),
            ),
        }
    }

    /// Synthesizes speech using ElevenLabs TTS API.
    async fn synthesize_speech_impl(
        &self,
        params: &TtsParams,
        api_key: &str,
    ) -> Result<TtsResult, ProviderError> {
        let voice_id = params
            .voice_id
            .as_deref()
            .unwrap_or("21m00Tcm4TlvDq8ikWAM");

        let url = format!(
            "{}/text-to-speech/{}?output_format=mp3_44100_128",
            self.base_url, voice_id
        );

        // Build voice settings from params
        let voice_settings = json!({
            "stability": params.stability.unwrap_or(0.5),
            "similarity_boost": params.similarity_boost.unwrap_or(0.75),
        });

        let body = json!({
            "text": params.text,
            "model_id": params.model_id.as_deref().unwrap_or("eleven_multilingual_v2"),
            "voice_settings": voice_settings,
        });

        let response = self
            .client
            .post(&url)
            .header("xi-api-key", api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Self::map_status(status, &body));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        // Estimate duration: MP3 at 128kbps
        let duration_secs = (bytes.len() as f32) / (16000.0);

        Ok(TtsResult::new(bytes.to_vec(), duration_secs, "mp3"))
    }

    /// Generates SFX using ElevenLabs sound generation API.
    async fn generate_sfx_impl(
        &self,
        params: &AudioGenParams,
        api_key: &str,
    ) -> Result<AudioGenResult, ProviderError> {
        let url = format!("{}/sound-generation", self.base_url);

        let mut body = json!({
            "text": params.prompt,
        });

        if let Some(duration) = params.duration_secs {
            body["duration_seconds"] = serde_json::json!(duration);
        }

        if let Some(ref model_id) = params.model_id {
            body["model_id"] = serde_json::json!(model_id);
        }

        let response = self
            .client
            .post(&url)
            .header("xi-api-key", api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Self::map_status(status, &body));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        // Estimate duration for SFX
        let duration_secs = params.duration_secs.unwrap_or(5.0);

        Ok(AudioGenResult::new(bytes.to_vec(), duration_secs, "mp3"))
    }

    /// Generates music using ElevenLabs text-to-music API.
    async fn generate_music_impl(
        &self,
        params: &AudioGenParams,
        api_key: &str,
    ) -> Result<AudioGenResult, ProviderError> {
        let url = format!("{}/text-to-music", self.base_url);

        let music_length_ms = params
            .duration_secs
            .map(|d| (d * 1000.0) as u32)
            .unwrap_or(30000);

        let body = json!({
            "prompt": params.prompt,
            "music_length_ms": music_length_ms,
        });

        let response = self
            .client
            .post(&url)
            .header("xi-api-key", api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Self::map_status(status, &body));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let duration_secs = params.duration_secs.unwrap_or(30.0);

        Ok(AudioGenResult::new(bytes.to_vec(), duration_secs, "mp3"))
    }
}

impl Default for ElevenLabsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TtsProvider for ElevenLabsProvider {
    async fn synthesize(
        &self,
        params: &TtsParams,
        api_key: &str,
    ) -> Result<TtsResult, ProviderError> {
        self.synthesize_speech_impl(params, api_key).await
    }

    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }
}

#[async_trait]
impl AudioProvider for ElevenLabsProvider {
    async fn generate(
        &self,
        params: &AudioGenParams,
        api_key: &str,
    ) -> Result<AudioGenResult, ProviderError> {
        match params.kind.as_deref() {
            Some("music") => self.generate_music_impl(params, api_key).await,
            _ => self.generate_sfx_impl(params, api_key).await,
        }
    }

    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_metadata() {
        let provider = ElevenLabsProvider::new();
        // Use AudioProvider trait to disambiguate
        let metadata = <ElevenLabsProvider as AudioProvider>::metadata(&provider);

        assert_eq!(metadata.id, "elevenlabs");
        assert_eq!(metadata.name, "ElevenLabs");
        assert!(metadata
            .supported_capabilities
            .contains(&ModelCapability::Tts));
        assert!(metadata
            .supported_capabilities
            .contains(&ModelCapability::AudioGen));
    }

    #[test]
    fn test_error_mapping_auth_failed() {
        let error = ElevenLabsProvider::map_status(
            reqwest::StatusCode::UNAUTHORIZED,
            "Invalid API key",
        );
        assert!(matches!(error, ProviderError::AuthFailed { .. }));
    }

    #[test]
    fn test_error_mapping_rate_limited() {
        let error =
            ElevenLabsProvider::map_status(reqwest::StatusCode::TOO_MANY_REQUESTS, "Rate limited");
        assert!(matches!(error, ProviderError::RateLimited { .. }));
    }

    #[test]
    fn test_error_mapping_quota_exceeded() {
        let error = ElevenLabsProvider::map_status(
            reqwest::StatusCode::PAYMENT_REQUIRED,
            "Quota exceeded",
        );
        assert!(matches!(error, ProviderError::QuotaExceeded { .. }));
    }
}
