//! Provider types and errors for AI model providers.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Errors that can occur when interacting with AI providers.
#[derive(Debug, Clone)]
pub enum ProviderError {
    AuthFailed {
        provider: String,
        message: String,
    },
    RateLimited {
        provider: String,
        retry_after_secs: Option<u64>,
    },
    QuotaExceeded {
        provider: String,
        message: String,
    },
    ModelNotFound {
        model_id: String,
    },
    Timeout {
        provider: String,
        message: String,
    },
    NetworkError(String),
    ProviderSpecific(String, String),
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderError::AuthFailed { provider, message } => {
                write!(f, "Authentication failed with {}: {}", provider, message)
            }
            ProviderError::RateLimited {
                provider,
                retry_after_secs,
            } => {
                if let Some(secs) = retry_after_secs {
                    write!(f, "Rate limited by {}, retry after {}s", provider, secs)
                } else {
                    write!(f, "Rate limited by {}", provider)
                }
            }
            ProviderError::QuotaExceeded { provider, message } => {
                write!(f, "Quota exceeded for {}: {}", provider, message)
            }
            ProviderError::ModelNotFound { model_id } => {
                write!(f, "Model not found: {}", model_id)
            }
            ProviderError::Timeout { provider, message } => {
                write!(f, "Timeout calling {}: {}", provider, message)
            }
            ProviderError::NetworkError(msg) => {
                write!(f, "Network error: {}", msg)
            }
            ProviderError::ProviderSpecific(provider, detail) => {
                write!(f, "Provider {} error: {}", provider, detail)
            }
        }
    }
}

impl std::error::Error for ProviderError {}

impl ProviderError {
    /// Returns the provider name if available.
    pub fn provider_name(&self) -> Option<&str> {
        match self {
            ProviderError::AuthFailed { provider, .. } => Some(provider),
            ProviderError::RateLimited { provider, .. } => Some(provider),
            ProviderError::QuotaExceeded { provider, .. } => Some(provider),
            ProviderError::ModelNotFound { .. } => None,
            ProviderError::Timeout { provider, .. } => Some(provider),
            ProviderError::NetworkError(_) => None,
            ProviderError::ProviderSpecific(p, _) => Some(p),
        }
    }
}

/// Supported AI provider kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Replicate,
    Fal,
    HuggingFace,
    OpenRouter,
    Together,
    ElevenLabs,
    Suno,
    OpenAI,
    Ollama,
    Kie,
    Custom,
}

impl ProviderKind {
    /// Returns the provider kind from a string name.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "replicate" => Some(Self::Replicate),
            "fal" => Some(Self::Fal),
            "huggingface" | "hugging_face" => Some(Self::HuggingFace),
            "openrouter" | "open_router" => Some(Self::OpenRouter),
            "together" | "togetherai" => Some(Self::Together),
            "elevenlabs" | "eleven_labs" => Some(Self::ElevenLabs),
            "suno" => Some(Self::Suno),
            "openai" | "open_ai" => Some(Self::OpenAI),
            "ollama" => Some(Self::Ollama),
            "kie" | "kieai" => Some(Self::Kie),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }

    /// Returns the provider kind as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderKind::Replicate => "replicate",
            ProviderKind::Fal => "fal",
            ProviderKind::HuggingFace => "huggingface",
            ProviderKind::OpenRouter => "openrouter",
            ProviderKind::Together => "together",
            ProviderKind::ElevenLabs => "elevenlabs",
            ProviderKind::Suno => "suno",
            ProviderKind::OpenAI => "openai",
            ProviderKind::Ollama => "ollama",
            ProviderKind::Kie => "kie",
            ProviderKind::Custom => "custom",
        }
    }
}

impl fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Capabilities that AI models can support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelCapability {
    ImageGen,
    AudioGen,
    Tts,
    TextComplete,
    CodeComplete,
    ImageEdit,
    VideoGen,
    BackgroundRemoval,
    TileGen,
}

impl ModelCapability {
    /// Returns the capability as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelCapability::ImageGen => "image_gen",
            ModelCapability::AudioGen => "audio_gen",
            ModelCapability::Tts => "tts",
            ModelCapability::TextComplete => "text_complete",
            ModelCapability::CodeComplete => "code_complete",
            ModelCapability::ImageEdit => "image_edit",
            ModelCapability::VideoGen => "video_gen",
            ModelCapability::BackgroundRemoval => "background_removal",
            ModelCapability::TileGen => "tile_gen",
        }
    }
}

/// Authentication types supported by providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    ApiKey,
    None,
    OAuth,
    BearerToken,
}

/// Metadata about a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    /// Canonical lowercase provider ID (e.g. "replicate", "fal", "huggingface").
    pub id: String,
    /// Human-readable display name of the provider (e.g. "Replicate", "Fal").
    pub name: String,
    /// The kind of provider.
    pub kind: ProviderKind,
    /// Base URL for API calls.
    pub base_url: String,
    /// Capabilities supported by this provider.
    pub supported_capabilities: Vec<ModelCapability>,
    /// Authentication type required.
    pub auth_type: AuthType,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_error_display() {
        let err = ProviderError::AuthFailed {
            provider: "replicate".to_string(),
            message: "Invalid API key".to_string(),
        };
        assert!(err.to_string().contains("Authentication failed"));
        assert!(err.to_string().contains("replicate"));
        assert!(err.to_string().contains("Invalid API key"));
    }

    #[test]
    fn test_provider_error_rate_limited() {
        let err = ProviderError::RateLimited {
            provider: "openai".to_string(),
            retry_after_secs: Some(60),
        };
        assert!(err.to_string().contains("Rate limited"));
        assert!(err.to_string().contains("60s"));

        let err_no_retry = ProviderError::RateLimited {
            provider: "openai".to_string(),
            retry_after_secs: None,
        };
        assert!(err_no_retry.to_string().contains("Rate limited"));
    }

    #[test]
    fn test_provider_kind_from_str() {
        assert_eq!(
            ProviderKind::from_str("replicate"),
            Some(ProviderKind::Replicate)
        );
        assert_eq!(ProviderKind::from_str("fal"), Some(ProviderKind::Fal));
        assert_eq!(ProviderKind::from_str("unknown"), None);
    }

    #[test]
    fn test_provider_kind_as_str() {
        assert_eq!(ProviderKind::Replicate.as_str(), "replicate");
        assert_eq!(ProviderKind::Fal.as_str(), "fal");
    }

    #[test]
    fn test_model_capability_as_str() {
        assert_eq!(ModelCapability::ImageGen.as_str(), "image_gen");
        assert_eq!(ModelCapability::TextComplete.as_str(), "text_complete");
    }
}
