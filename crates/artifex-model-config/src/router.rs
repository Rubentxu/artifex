//! Model router for resolving model profiles to providers.

use std::sync::Arc;
use async_trait::async_trait;
use thiserror::Error;

use super::audio_provider::AudioProvider;
use super::image_provider::ImageProvider;
use super::model_profile::ModelProfile;
use super::registry::ProviderRegistry;
use super::routing_rule::{RoutingRule, MAX_FALLBACK_DEPTH};
use super::tts_provider::TtsProvider;
use super::credential_store::CredentialStore;

/// Errors that can occur during routing.
#[derive(Debug, Clone, Error)]
pub enum RoutingError {
    #[error("No routing rule for operation: {0}")]
    NoRuleForOperation(String),

    #[error("No enabled profile available for operation: {0}")]
    NoAvailableProfile(String),

    #[error("Provider not registered: {0}")]
    ProviderNotRegistered(String),

    #[error("Credential not found for provider: {0}")]
    CredentialNotFound(String),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Max fallback depth exceeded")]
    MaxFallbackDepthExceeded,
}

/// A resolved model profile with its associated provider.
pub struct ResolvedModelProfile {
    /// The model profile.
    pub profile: Arc<ModelProfile>,
    /// The provider instance.
    pub provider: Arc<dyn ImageProvider>,
}

/// A resolved audio profile with its associated provider.
pub struct ResolvedAudioProfile {
    /// The model profile.
    pub profile: Arc<ModelProfile>,
    /// The provider instance.
    pub provider: Arc<dyn AudioProvider>,
}

/// A resolved TTS profile with its associated provider.
pub struct ResolvedTtsProfile {
    /// The model profile.
    pub profile: Arc<ModelProfile>,
    /// The provider instance.
    pub provider: Arc<dyn TtsProvider>,
}

/// Repository trait for model configuration storage.
#[async_trait]
pub trait ModelConfigRepository: Send + Sync {
    /// Finds a profile by ID.
    async fn find_profile(&self, id: &uuid::Uuid) -> Result<Option<ModelProfile>, String>;

    /// Finds a routing rule by operation type.
    async fn find_rule(&self, operation_type: &str) -> Result<Option<RoutingRule>, String>;

    /// Lists all enabled profiles for a given capability.
    async fn list_enabled_profiles(&self, _capability: super::provider::ModelCapability) -> Result<Vec<ModelProfile>, String> {
        Ok(vec![])
    }
}

/// Model router for resolving operations to model profiles and providers.
pub struct ModelRouter {
    registry: Arc<ProviderRegistry>,
    repo: Arc<dyn ModelConfigRepository>,
    credential_store: Arc<dyn CredentialStore>,
}

impl ModelRouter {
    /// Creates a new model router.
    pub fn new(
        registry: Arc<ProviderRegistry>,
        repo: Arc<dyn ModelConfigRepository>,
        credential_store: Arc<dyn CredentialStore>,
    ) -> Self {
        Self {
            registry,
            repo,
            credential_store,
        }
    }

    /// Resolves the best model profile for an image generation operation.
    ///
    /// Tries the default profile first, then falls back to alternatives in order.
    pub async fn resolve_image(
        &self,
        operation_type: &str,
    ) -> Result<ResolvedModelProfile, RoutingError> {
        let rule = self
            .repo
            .find_rule(operation_type)
            .await
            .map_err(|e| RoutingError::NoRuleForOperation(e.to_string()))?
            .ok_or_else(|| RoutingError::NoRuleForOperation(operation_type.to_string()))?;

        // Try each profile in order
        let mut fallbacks_exhausted = 0;
        for profile_id in rule.profile_ids() {
            if fallbacks_exhausted >= MAX_FALLBACK_DEPTH {
                return Err(RoutingError::MaxFallbackDepthExceeded);
            }

            let profile = self
                .repo
                .find_profile(profile_id)
                .await
                .map_err(|e| RoutingError::ProfileNotFound(e.to_string()))?
                .ok_or_else(|| RoutingError::ProfileNotFound(profile_id.to_string()))?;

            if !profile.enabled {
                fallbacks_exhausted += 1;
                continue;
            }

            let provider = self
                .registry
                .get_image(&profile.provider_name)
                .ok_or_else(|| {
                    RoutingError::ProviderNotRegistered(profile.provider_name.clone())
                })?;

            // Verify credential exists
            let credential_id = format!("{}::api_key", profile.provider_name);
            if self.credential_store.get(&credential_id).is_err() {
                fallbacks_exhausted += 1;
                continue;
            }

            return Ok(ResolvedModelProfile {
                profile: Arc::new(profile),
                provider,
            });
        }

        Err(RoutingError::NoAvailableProfile(operation_type.to_string()))
    }

    /// Resolves the best model profile for an audio generation operation.
    ///
    /// Tries the default profile first, then falls back to alternatives in order.
    pub async fn resolve_audio(
        &self,
        operation_type: &str,
    ) -> Result<ResolvedAudioProfile, RoutingError> {
        let rule = self
            .repo
            .find_rule(operation_type)
            .await
            .map_err(|e| RoutingError::NoRuleForOperation(e.to_string()))?
            .ok_or_else(|| RoutingError::NoRuleForOperation(operation_type.to_string()))?;

        // Try each profile in order
        let mut fallbacks_exhausted = 0;
        for profile_id in rule.profile_ids() {
            if fallbacks_exhausted >= MAX_FALLBACK_DEPTH {
                return Err(RoutingError::MaxFallbackDepthExceeded);
            }

            let profile = self
                .repo
                .find_profile(profile_id)
                .await
                .map_err(|e| RoutingError::ProfileNotFound(e.to_string()))?
                .ok_or_else(|| RoutingError::ProfileNotFound(profile_id.to_string()))?;

            if !profile.enabled {
                fallbacks_exhausted += 1;
                continue;
            }

            let provider = self
                .registry
                .get_audio(&profile.provider_name)
                .ok_or_else(|| {
                    RoutingError::ProviderNotRegistered(profile.provider_name.clone())
                })?;

            // Verify credential exists
            let credential_id = format!("{}::api_key", profile.provider_name);
            if self.credential_store.get(&credential_id).is_err() {
                fallbacks_exhausted += 1;
                continue;
            }

            return Ok(ResolvedAudioProfile {
                profile: Arc::new(profile),
                provider,
            });
        }

        Err(RoutingError::NoAvailableProfile(operation_type.to_string()))
    }

    /// Resolves the best model profile for a TTS operation.
    ///
    /// Tries the default profile first, then falls back to alternatives in order.
    pub async fn resolve_tts(
        &self,
        operation_type: &str,
    ) -> Result<ResolvedTtsProfile, RoutingError> {
        let rule = self
            .repo
            .find_rule(operation_type)
            .await
            .map_err(|e| RoutingError::NoRuleForOperation(e.to_string()))?
            .ok_or_else(|| RoutingError::NoRuleForOperation(operation_type.to_string()))?;

        // Try each profile in order
        let mut fallbacks_exhausted = 0;
        for profile_id in rule.profile_ids() {
            if fallbacks_exhausted >= MAX_FALLBACK_DEPTH {
                return Err(RoutingError::MaxFallbackDepthExceeded);
            }

            let profile = self
                .repo
                .find_profile(profile_id)
                .await
                .map_err(|e| RoutingError::ProfileNotFound(e.to_string()))?
                .ok_or_else(|| RoutingError::ProfileNotFound(profile_id.to_string()))?;

            if !profile.enabled {
                fallbacks_exhausted += 1;
                continue;
            }

            let provider = self
                .registry
                .get_tts(&profile.provider_name)
                .ok_or_else(|| {
                    RoutingError::ProviderNotRegistered(profile.provider_name.clone())
                })?;

            // Verify credential exists
            let credential_id = format!("{}::api_key", profile.provider_name);
            if self.credential_store.get(&credential_id).is_err() {
                fallbacks_exhausted += 1;
                continue;
            }

            return Ok(ResolvedTtsProfile {
                profile: Arc::new(profile),
                provider,
            });
        }

        Err(RoutingError::NoAvailableProfile(operation_type.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{AuthType, ModelCapability, ProviderError, ProviderKind, ProviderMetadata};
    use async_trait::async_trait;

    /// Mock repository for testing.
    struct MockRepository {
        profiles: Vec<ModelProfile>,
        rules: Vec<RoutingRule>,
    }

    impl MockRepository {
        fn new(profiles: Vec<ModelProfile>, rules: Vec<RoutingRule>) -> Self {
            Self { profiles, rules }
        }
    }

    #[async_trait]
    impl ModelConfigRepository for MockRepository {
        async fn find_profile(&self, id: &uuid::Uuid) -> Result<Option<ModelProfile>, String> {
            Ok(self.profiles.iter().find(|p| &p.id == id).cloned())
        }

        async fn find_rule(&self, operation_type: &str) -> Result<Option<RoutingRule>, String> {
            Ok(self.rules.iter().find(|r| r.operation_type == operation_type).cloned())
        }
    }

    /// Mock image provider for testing.
    struct MockImageProvider {
        metadata: ProviderMetadata,
    }

    impl MockImageProvider {
        fn new(name: &str) -> Self {
            Self {
                metadata: ProviderMetadata {
                    id: name.to_string(),
                    name: name.to_string(),
                    kind: ProviderKind::Replicate,
                    base_url: format!("https://api.{}.com", name),
                    supported_capabilities: vec![ModelCapability::ImageGen],
                    auth_type: AuthType::ApiKey,
                },
            }
        }
    }

    #[async_trait]
    impl ImageProvider for MockImageProvider {
        async fn generate(
            &self,
            _params: &crate::image_provider::ImageGenParams,
            _api_key: &str,
        ) -> Result<crate::image_provider::ImageGenResult, ProviderError> {
            Ok(crate::image_provider::ImageGenResult::new(
                vec![0, 1, 2],
                512,
                512,
                "png",
            ))
        }

        async fn remove_background(
            &self,
            _image_data: &[u8],
            _api_key: &str,
        ) -> Result<crate::image_provider::ImageGenResult, ProviderError> {
            Ok(crate::image_provider::ImageGenResult::new(
                vec![0, 1, 2],
                512,
                512,
                "png",
            ))
        }

        fn metadata(&self) -> &ProviderMetadata {
            &self.metadata
        }
    }

    /// Mock audio provider for testing.
    struct MockAudioProvider {
        metadata: ProviderMetadata,
    }

    impl MockAudioProvider {
        fn new(name: &str) -> Self {
            Self {
                metadata: ProviderMetadata {
                    id: name.to_string(),
                    name: name.to_string(),
                    kind: ProviderKind::ElevenLabs,
                    base_url: format!("https://api.{}.com", name),
                    supported_capabilities: vec![ModelCapability::AudioGen],
                    auth_type: AuthType::ApiKey,
                },
            }
        }
    }

    #[async_trait]
    impl AudioProvider for MockAudioProvider {
        async fn generate(
            &self,
            _params: &crate::audio_provider::AudioGenParams,
            _api_key: &str,
        ) -> Result<crate::audio_provider::AudioGenResult, ProviderError> {
            Ok(crate::audio_provider::AudioGenResult::new(
                vec![0, 1, 2],
                30.0,
                "mp3",
            ))
        }

        fn metadata(&self) -> &ProviderMetadata {
            &self.metadata
        }
    }

    /// Mock TTS provider for testing.
    struct MockTtsProvider {
        metadata: ProviderMetadata,
    }

    impl MockTtsProvider {
        fn new(name: &str) -> Self {
            Self {
                metadata: ProviderMetadata {
                    id: name.to_string(),
                    name: name.to_string(),
                    kind: ProviderKind::ElevenLabs,
                    base_url: format!("https://api.{}.com", name),
                    supported_capabilities: vec![ModelCapability::Tts],
                    auth_type: AuthType::ApiKey,
                },
            }
        }
    }

    #[async_trait]
    impl TtsProvider for MockTtsProvider {
        async fn synthesize(
            &self,
            _params: &crate::tts_provider::TtsParams,
            _api_key: &str,
        ) -> Result<crate::tts_provider::TtsResult, ProviderError> {
            Ok(crate::tts_provider::TtsResult::new(
                vec![0, 1, 2],
                5.0,
                "mp3",
            ))
        }

        fn metadata(&self) -> &ProviderMetadata {
            &self.metadata
        }
    }

    #[test]
    fn test_resolve_image_success() {
        let profile = ModelProfile::new(
            "replicate".to_string(),
            "test-model".to_string(),
            "Test Model".to_string(),
            vec![ModelCapability::ImageGen],
        );

        let rule = RoutingRule::new(
            "imagegen.txt2img".to_string(),
            profile.id,
            vec![],
        );

        let repo = Arc::new(MockRepository::new(vec![profile.clone()], vec![rule]));
        let registry = Arc::new(ProviderRegistry::new());
        let cred_store = Arc::new(crate::credential_store::InMemoryCredentialStore::new());

        // Register provider
        let provider = Arc::new(MockImageProvider::new("replicate"));
        registry.register_image("replicate", provider).unwrap();

        // Set credential
        cred_store.set("replicate::api_key", "test-key").unwrap();

        let router = ModelRouter::new(registry.clone(), repo.clone(), cred_store);

        // Note: resolve_image is async, we need to use a runtime
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let resolved = router.resolve_image("imagegen.txt2img").await.unwrap();
                assert_eq!(resolved.profile.model_id, "test-model");
            });
    }

    #[test]
    fn test_resolve_image_no_rule() {
        let repo = Arc::new(MockRepository::new(vec![], vec![]));
        let registry = Arc::new(ProviderRegistry::new());
        let cred_store = Arc::new(crate::credential_store::InMemoryCredentialStore::new());

        let router = ModelRouter::new(registry.clone(), repo.clone(), cred_store);

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let result = router.resolve_image("nonexistent").await;
                assert!(matches!(result, Err(RoutingError::NoRuleForOperation(_))));
            });
    }

    #[test]
    fn test_resolve_image_fallback_chain() {
        // Create default profile (disabled) and fallback profile (enabled)
        let default_profile = ModelProfile::new(
            "replicate".to_string(),
            "default-model".to_string(),
            "Default".to_string(),
            vec![ModelCapability::ImageGen],
        );
        let mut default_profile = default_profile;
        default_profile.enabled = false; // Disable default

        let fallback_profile = ModelProfile::new(
            "fal".to_string(),
            "fallback-model".to_string(),
            "Fallback".to_string(),
            vec![ModelCapability::ImageGen],
        );

        let rule = RoutingRule::new(
            "imagegen.txt2img".to_string(),
            default_profile.id,
            vec![fallback_profile.id],
        );

        let repo = Arc::new(MockRepository::new(
            vec![default_profile.clone(), fallback_profile.clone()],
            vec![rule],
        ));

        let registry = Arc::new(ProviderRegistry::new());
        let cred_store = Arc::new(crate::credential_store::InMemoryCredentialStore::new());

        // Register both providers
        let replicate_provider = Arc::new(MockImageProvider::new("replicate"));
        let fal_provider = Arc::new(MockImageProvider::new("fal"));
        registry.register_image("replicate", replicate_provider).unwrap();
        registry.register_image("fal", fal_provider).unwrap();

        // Only set credential for fal (not replicate)
        cred_store.set("fal::api_key", "test-key").unwrap();

        let router = ModelRouter::new(registry.clone(), repo.clone(), cred_store);

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                // Should succeed by falling back to fal
                let resolved = router.resolve_image("imagegen.txt2img").await.unwrap();
                assert_eq!(resolved.profile.model_id, "fallback-model");
            });
    }

    #[test]
    fn test_resolve_audio_success() {
        let profile = ModelProfile::new(
            "elevenlabs".to_string(),
            "eleven_turbo_v2".to_string(),
            "ElevenLabs SFX".to_string(),
            vec![ModelCapability::AudioGen],
        );

        let rule = RoutingRule::new(
            "audiogen.sfx".to_string(),
            profile.id,
            vec![],
        );

        let repo = Arc::new(MockRepository::new(vec![profile.clone()], vec![rule]));
        let registry = Arc::new(ProviderRegistry::new());
        let cred_store = Arc::new(crate::credential_store::InMemoryCredentialStore::new());

        // Register provider
        let provider = Arc::new(MockAudioProvider::new("elevenlabs"));
        registry.register_audio("elevenlabs", provider).unwrap();

        // Set credential
        cred_store.set("elevenlabs::api_key", "test-key").unwrap();

        let router = ModelRouter::new(registry.clone(), repo.clone(), cred_store);

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let resolved = router.resolve_audio("audiogen.sfx").await.unwrap();
                assert_eq!(resolved.profile.model_id, "eleven_turbo_v2");
            });
    }

    #[test]
    fn test_resolve_audio_no_rule() {
        let repo = Arc::new(MockRepository::new(vec![], vec![]));
        let registry = Arc::new(ProviderRegistry::new());
        let cred_store = Arc::new(crate::credential_store::InMemoryCredentialStore::new());

        let router = ModelRouter::new(registry.clone(), repo.clone(), cred_store);

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let result = router.resolve_audio("unknown.op").await;
                assert!(matches!(result, Err(RoutingError::NoRuleForOperation(_))));
            });
    }

    #[test]
    fn test_resolve_tts_success() {
        let profile = ModelProfile::new(
            "elevenlabs".to_string(),
            "eleven_multilingual_v2".to_string(),
            "ElevenLabs TTS".to_string(),
            vec![ModelCapability::Tts],
        );

        let rule = RoutingRule::new(
            "tts.npc_line".to_string(),
            profile.id,
            vec![],
        );

        let repo = Arc::new(MockRepository::new(vec![profile.clone()], vec![rule]));
        let registry = Arc::new(ProviderRegistry::new());
        let cred_store = Arc::new(crate::credential_store::InMemoryCredentialStore::new());

        // Register provider
        let provider = Arc::new(MockTtsProvider::new("elevenlabs"));
        registry.register_tts("elevenlabs", provider).unwrap();

        // Set credential
        cred_store.set("elevenlabs::api_key", "test-key").unwrap();

        let router = ModelRouter::new(registry.clone(), repo.clone(), cred_store);

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let resolved = router.resolve_tts("tts.npc_line").await.unwrap();
                assert_eq!(resolved.profile.model_id, "eleven_multilingual_v2");
            });
    }

    #[test]
    fn test_resolve_tts_no_rule() {
        let repo = Arc::new(MockRepository::new(vec![], vec![]));
        let registry = Arc::new(ProviderRegistry::new());
        let cred_store = Arc::new(crate::credential_store::InMemoryCredentialStore::new());

        let router = ModelRouter::new(registry.clone(), repo.clone(), cred_store);

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let result = router.resolve_tts("unknown.op").await;
                assert!(matches!(result, Err(RoutingError::NoRuleForOperation(_))));
            });
    }
}