//! Provider registry for managing AI providers.

use std::sync::Arc;

use dashmap::DashMap;
use thiserror::Error;

use super::audio_provider::AudioProvider;
use super::image_provider::ImageProvider;
use super::provider::ProviderMetadata;
use super::text_provider::TextProvider;
use super::tts_provider::TtsProvider;

/// Errors that can occur when registering or accessing providers.
#[derive(Debug, Clone, Error)]
pub enum RegistryError {
    #[error("Provider not registered: {0}")]
    NotRegistered(String),

    #[error("Provider already registered: {0}")]
    AlreadyRegistered(String),
}

/// Thread-safe registry for image providers, indexed by canonical provider_id (lowercase).
pub struct ProviderRegistry {
    image_providers: DashMap<String, Arc<dyn ImageProvider>>,
    audio_providers: DashMap<String, Arc<dyn AudioProvider>>,
    tts_providers: DashMap<String, Arc<dyn TtsProvider>>,
    text_providers: DashMap<String, Arc<dyn TextProvider>>,
}

impl ProviderRegistry {
    /// Creates a new empty provider registry.
    pub fn new() -> Self {
        Self {
            image_providers: DashMap::new(),
            audio_providers: DashMap::new(),
            tts_providers: DashMap::new(),
            text_providers: DashMap::new(),
        }
    }

    /// Registers an image provider by its canonical id.
    ///
    /// # Errors
    /// Returns `RegistryError::AlreadyRegistered` if a provider with the same id exists.
    pub fn register_image(
        &self,
        id: &str,
        provider: Arc<dyn ImageProvider>,
    ) -> Result<(), RegistryError> {
        if self.image_providers.contains_key(id) {
            return Err(RegistryError::AlreadyRegistered(id.to_string()));
        }
        self.image_providers.insert(id.to_string(), provider);
        Ok(())
    }

    /// Gets an image provider by its canonical id.
    pub fn get_image(&self, id: &str) -> Option<Arc<dyn ImageProvider>> {
        self.image_providers.get(id).map(|r| r.value().clone())
    }

    /// Lists all registered image providers.
    pub fn list_image_providers(&self) -> Vec<ProviderMetadata> {
        self.image_providers
            .iter()
            .map(|r| r.value().metadata().clone())
            .collect()
    }

    /// Registers an audio provider by its canonical id.
    ///
    /// # Errors
    /// Returns `RegistryError::AlreadyRegistered` if a provider with the same id exists.
    pub fn register_audio(
        &self,
        id: &str,
        provider: Arc<dyn AudioProvider>,
    ) -> Result<(), RegistryError> {
        if self.audio_providers.contains_key(id) {
            return Err(RegistryError::AlreadyRegistered(id.to_string()));
        }
        self.audio_providers.insert(id.to_string(), provider);
        Ok(())
    }

    /// Gets an audio provider by its canonical id.
    pub fn get_audio(&self, id: &str) -> Option<Arc<dyn AudioProvider>> {
        self.audio_providers.get(id).map(|r| r.value().clone())
    }

    /// Lists all registered audio providers.
    pub fn list_audio_providers(&self) -> Vec<ProviderMetadata> {
        self.audio_providers
            .iter()
            .map(|r| r.value().metadata().clone())
            .collect()
    }

    /// Registers a TTS provider by its canonical id.
    ///
    /// # Errors
    /// Returns `RegistryError::AlreadyRegistered` if a provider with the same id exists.
    pub fn register_tts(
        &self,
        id: &str,
        provider: Arc<dyn TtsProvider>,
    ) -> Result<(), RegistryError> {
        if self.tts_providers.contains_key(id) {
            return Err(RegistryError::AlreadyRegistered(id.to_string()));
        }
        self.tts_providers.insert(id.to_string(), provider);
        Ok(())
    }

    /// Gets a TTS provider by its canonical id.
    pub fn get_tts(&self, id: &str) -> Option<Arc<dyn TtsProvider>> {
        self.tts_providers.get(id).map(|r| r.value().clone())
    }

    /// Lists all registered TTS providers.
    pub fn list_tts_providers(&self) -> Vec<ProviderMetadata> {
        self.tts_providers
            .iter()
            .map(|r| r.value().metadata().clone())
            .collect()
    }

    /// Registers a text provider by its canonical id.
    ///
    /// # Errors
    /// Returns `RegistryError::AlreadyRegistered` if a provider with the same id exists.
    pub fn register_text(
        &self,
        id: &str,
        provider: Arc<dyn TextProvider>,
    ) -> Result<(), RegistryError> {
        if self.text_providers.contains_key(id) {
            return Err(RegistryError::AlreadyRegistered(id.to_string()));
        }
        self.text_providers.insert(id.to_string(), provider);
        Ok(())
    }

    /// Gets a text provider by its canonical id.
    pub fn get_text(&self, id: &str) -> Option<Arc<dyn TextProvider>> {
        self.text_providers.get(id).map(|r| r.value().clone())
    }

    /// Lists all registered text providers.
    pub fn list_text_providers(&self) -> Vec<ProviderMetadata> {
        self.text_providers
            .iter()
            .map(|r| r.value().metadata().clone())
            .collect()
    }

    /// Checks if a provider is registered by its canonical id.
    pub fn is_registered(&self, id: &str) -> bool {
        self.image_providers.contains_key(id)
            || self.audio_providers.contains_key(id)
            || self.tts_providers.contains_key(id)
            || self.text_providers.contains_key(id)
    }

    /// Returns the total number of registered providers.
    pub fn len(&self) -> usize {
        self.image_providers.len()
            + self.audio_providers.len()
            + self.tts_providers.len()
            + self.text_providers.len()
    }

    /// Returns true if no providers are registered.
    pub fn is_empty(&self) -> bool {
        self.image_providers.is_empty()
            && self.audio_providers.is_empty()
            && self.tts_providers.is_empty()
            && self.text_providers.is_empty()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{AuthType, ModelCapability, ProviderKind};
    use async_trait::async_trait;

    /// A mock image provider for testing.
    struct MockImageProvider {
        metadata: ProviderMetadata,
    }

    impl MockImageProvider {
        fn new(name: &str, kind: ProviderKind) -> Self {
            Self {
                metadata: ProviderMetadata {
                    id: name.to_string(),
                    name: name.to_string(),
                    kind,
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
        ) -> Result<crate::image_provider::ImageGenResult, crate::provider::ProviderError> {
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
        ) -> Result<crate::image_provider::ImageGenResult, crate::provider::ProviderError> {
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

    /// A mock audio provider for testing.
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
        ) -> Result<crate::audio_provider::AudioGenResult, crate::provider::ProviderError> {
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

    /// A mock TTS provider for testing.
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
        ) -> Result<crate::tts_provider::TtsResult, crate::provider::ProviderError> {
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

    /// A mock text provider for testing.
    struct MockTextProvider {
        metadata: ProviderMetadata,
    }

    impl MockTextProvider {
        fn new(name: &str) -> Self {
            Self {
                metadata: ProviderMetadata {
                    id: name.to_string(),
                    name: name.to_string(),
                    kind: ProviderKind::Together,
                    base_url: format!("https://api.{}.com", name),
                    supported_capabilities: vec![ModelCapability::TextComplete],
                    auth_type: AuthType::ApiKey,
                },
            }
        }
    }

    #[async_trait]
    impl TextProvider for MockTextProvider {
        async fn complete(
            &self,
            _params: &crate::text_provider::TextParams,
            _api_key: &str,
        ) -> Result<crate::text_provider::TextResult, crate::provider::ProviderError> {
            Ok(crate::text_provider::TextResult::new(
                "Hello, world!".to_string(),
                50,
                false,
            ))
        }

        fn metadata(&self) -> &ProviderMetadata {
            &self.metadata
        }
    }

    #[test]
    fn test_registry_register_and_get_image() {
        let registry = ProviderRegistry::new();

        let provider = Arc::new(MockImageProvider::new("replicate", ProviderKind::Replicate));
        registry.register_image("replicate", provider.clone()).unwrap();

        let retrieved = registry.get_image("replicate").unwrap();
        assert_eq!(retrieved.metadata().id, "replicate");
        assert_eq!(retrieved.metadata().name, "replicate");
    }

    #[test]
    fn test_registry_register_audio_and_get() {
        let registry = ProviderRegistry::new();

        let provider = Arc::new(MockAudioProvider::new("elevenlabs"));
        registry.register_audio("elevenlabs", provider.clone()).unwrap();

        let retrieved = registry.get_audio("elevenlabs").unwrap();
        assert_eq!(retrieved.metadata().id, "elevenlabs");
    }

    #[test]
    fn test_registry_register_tts_and_get() {
        let registry = ProviderRegistry::new();

        let provider = Arc::new(MockTtsProvider::new("elevenlabs"));
        registry.register_tts("elevenlabs", provider.clone()).unwrap();

        let retrieved = registry.get_tts("elevenlabs").unwrap();
        assert_eq!(retrieved.metadata().id, "elevenlabs");
    }

    #[test]
    fn test_registry_register_text_and_get() {
        let registry = ProviderRegistry::new();

        let provider = Arc::new(MockTextProvider::new("together"));
        registry.register_text("together", provider.clone()).unwrap();

        let retrieved = registry.get_text("together").unwrap();
        assert_eq!(retrieved.metadata().id, "together");
    }

    #[test]
    fn test_registry_register_duplicate() {
        let registry = ProviderRegistry::new();

        let provider = Arc::new(MockImageProvider::new("replicate", ProviderKind::Replicate));
        registry.register_image("replicate", provider.clone()).unwrap();

        let result = registry.register_image("replicate", provider);
        assert!(matches!(result, Err(RegistryError::AlreadyRegistered(_))));
    }

    #[test]
    fn test_registry_get_not_found() {
        let registry = ProviderRegistry::new();

        let result = registry.get_image("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_registry_list_image_providers() {
        let registry = ProviderRegistry::new();

        let provider1 = Arc::new(MockImageProvider::new("replicate", ProviderKind::Replicate));
        let provider2 = Arc::new(MockImageProvider::new("fal", ProviderKind::Fal));

        registry.register_image("replicate", provider1).unwrap();
        registry.register_image("fal", provider2).unwrap();

        let list = registry.list_image_providers();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_registry_list_audio_providers() {
        let registry = ProviderRegistry::new();

        let provider = Arc::new(MockAudioProvider::new("elevenlabs"));
        registry.register_audio("elevenlabs", provider).unwrap();

        let list = registry.list_audio_providers();
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_registry_list_tts_providers() {
        let registry = ProviderRegistry::new();

        let provider = Arc::new(MockTtsProvider::new("elevenlabs"));
        registry.register_tts("elevenlabs", provider).unwrap();

        let list = registry.list_tts_providers();
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_registry_list_text_providers() {
        let registry = ProviderRegistry::new();

        let provider = Arc::new(MockTextProvider::new("together"));
        registry.register_text("together", provider).unwrap();

        let list = registry.list_text_providers();
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_registry_is_registered() {
        let registry = ProviderRegistry::new();

        let image_provider = Arc::new(MockImageProvider::new("replicate", ProviderKind::Replicate));
        registry.register_image("replicate", image_provider).unwrap();

        let audio_provider = Arc::new(MockAudioProvider::new("elevenlabs"));
        registry.register_audio("elevenlabs", audio_provider).unwrap();

        let tts_provider = Arc::new(MockTtsProvider::new("elevenlabs_tts"));
        registry.register_tts("elevenlabs_tts", tts_provider).unwrap();

        let text_provider = Arc::new(MockTextProvider::new("together"));
        registry.register_text("together", text_provider).unwrap();

        assert!(registry.is_registered("replicate"));
        assert!(registry.is_registered("elevenlabs"));
        assert!(registry.is_registered("elevenlabs_tts"));
        assert!(registry.is_registered("together"));
        assert!(!registry.is_registered("nonexistent"));
    }

    #[test]
    fn test_registry_len_and_is_empty() {
        let registry = ProviderRegistry::new();

        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        let image_provider = Arc::new(MockImageProvider::new("replicate", ProviderKind::Replicate));
        registry.register_image("replicate", image_provider).unwrap();

        let audio_provider = Arc::new(MockAudioProvider::new("elevenlabs"));
        registry.register_audio("elevenlabs", audio_provider).unwrap();

        let tts_provider = Arc::new(MockTtsProvider::new("elevenlabs_tts"));
        registry.register_tts("elevenlabs_tts", tts_provider).unwrap();

        let text_provider = Arc::new(MockTextProvider::new("together"));
        registry.register_text("together", text_provider).unwrap();

        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 4);
    }
}