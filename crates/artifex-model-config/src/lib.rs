//! Artifex Model Config
//!
//! This crate provides model configuration, provider abstractions,
//! and routing for AI model services.

pub mod audio_provider;
pub mod credential_store;
pub mod image_provider;
pub mod model_profile;
pub mod prompt_template;
pub mod provider;
pub mod registry;
pub mod router;
pub mod routing_rule;
pub mod text_provider;
pub mod tts_provider;

// Re-export commonly used types
pub use audio_provider::{AudioGenParams, AudioGenResult, AudioProvider};
pub use credential_store::{CredentialStore, CredentialError, InMemoryCredentialStore};
pub use image_provider::{ImageGenParams, ImageGenResult, ImageProvider};
pub use model_profile::{ModelProfile, PricingTier};
pub use prompt_template::PromptTemplate;
pub use provider::{ProviderError, ProviderKind, ModelCapability, ProviderMetadata, AuthType};
pub use registry::{ProviderRegistry, RegistryError};
pub use router::{ModelRouter, ModelConfigRepository, ResolvedModelProfile, ResolvedAudioProfile, ResolvedTtsProfile, ResolvedTextProfile, RoutingError};
pub use routing_rule::RoutingRule;
pub use text_provider::{TextParams, TextResult, TextProvider};
pub use tts_provider::{TtsParams, TtsResult, TtsProvider};
