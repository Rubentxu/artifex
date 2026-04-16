//! Model configuration service.
//!
//! Provides business logic for managing model profiles, routing rules,
//! prompt templates, and provider credentials.

use std::sync::Arc;

use dashmap::DashSet;
use uuid::Uuid;

use artifex_model_config::{
    ModelProfile, ProviderMetadata, PromptTemplate,
    ProviderRegistry, RoutingRule,
};
use artifex_model_config::credential_store::CredentialStore;
use artifex_shared_kernel::ArtifexError;

use super::repository::{
    self, SqliteModelConfigRepository,
};
use super::commands::ProviderDto;
use crate::model_config::providers::{
    ElevenLabsProvider, FalImageProvider,
    HuggingFaceImageProvider, KieImageProvider, ReplicateImageProvider,
};

/// Credential status for a provider.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CredentialStatus {
    /// Canonical lowercase provider ID.
    pub provider_id: String,
    /// Whether a credential exists.
    pub has_credential: bool,
}

/// Service for managing model configurations.
#[derive(Clone)]
pub struct ModelConfigService {
    repo: Arc<SqliteModelConfigRepository>,
    registry: Arc<ProviderRegistry>,
    credential_store: Arc<dyn CredentialStore>,
    /// Set of enabled provider names.
    enabled_providers: DashSet<String>,
}

impl ModelConfigService {
    /// Creates a new ModelConfigService.
    pub fn new(
        repo: Arc<SqliteModelConfigRepository>,
        registry: Arc<ProviderRegistry>,
        credential_store: Arc<dyn CredentialStore>,
    ) -> Self {
        let enabled_providers = DashSet::new();
        // All registered providers start as enabled, keyed by canonical provider_id (lowercase)
        for meta in registry.list_image_providers() {
            enabled_providers.insert(meta.id.clone());
        }
        Self { repo, registry, credential_store, enabled_providers }
    }

    /// Returns the repository.
    pub fn repo(&self) -> &SqliteModelConfigRepository {
        &self.repo
    }

    /// Returns the credential store.
    pub fn credential_store(&self) -> &dyn CredentialStore {
        &*self.credential_store
    }

    /// Returns the provider registry.
    pub fn registry(&self) -> &ProviderRegistry {
        &self.registry
    }

    // === Provider Management ===

    /// Lists all registered providers as DTOs with enabled state.
    pub fn list_providers(&self) -> Vec<ProviderDto> {
        self.registry
            .list_image_providers()
            .iter()
            .map(|m| ProviderDto {
                id: m.id.clone(),
                name: m.name.clone(),
                kind: format!("{:?}", m.kind).to_lowercase(),
                base_url: m.base_url.clone(),
                supported_capabilities: m
                    .supported_capabilities
                    .iter()
                    .map(|c| format!("{:?}", c).to_lowercase())
                    .collect(),
                auth_type: format!("{:?}", m.auth_type).to_lowercase(),
                enabled: self.enabled_providers.contains(&m.id),
            })
            .collect()
    }

    /// Gets a provider by name.
    pub fn get_provider(&self, name: &str) -> Option<ProviderMetadata> {
        self.registry.get_image(name).map(|p| p.metadata().clone())
    }

    /// Sets whether a provider is enabled.
    /// Updates the in-memory DashSet cache AND persists to DB by updating
    /// all model profiles for this provider.
    pub async fn set_provider_enabled(&self, name: &str, enabled: bool) -> Result<(), ArtifexError> {
        if !self.registry.is_registered(name) {
            return Err(ArtifexError::NotFound(format!("Provider {} not found", name)));
        }

        // Update in-memory DashSet cache
        if enabled {
            self.enabled_providers.insert(name.to_string());
        } else {
            self.enabled_providers.remove(name);
        }

        // Persist to DB: update all model profiles for this provider
        repository::set_provider_profiles_enabled(self.repo.pool(), name, enabled)
            .await
            .map_err(|e| ArtifexError::validation(format!("db error: {}", e)))?;

        Ok(())
    }

    /// Checks if a provider is enabled.
    pub fn is_provider_enabled(&self, name: &str) -> bool {
        self.enabled_providers.contains(name)
    }

    /// Sets a credential for a provider.
    /// Stores in-memory and persists a flag in the database.
    pub async fn set_credential(&self, provider: &str, api_key: &str) -> Result<(), ArtifexError> {
        // Store in-memory
        let credential_id = format!("{}::api_key", provider);
        self.credential_store
            .set(&credential_id, api_key)
            .map_err(|e| ArtifexError::validation(format!("credential store error: {}", e)))?;

        // Persist flag in DB
        let id = Uuid::new_v4();
        repository::create_credential_ref(self.repo.pool(), &id, provider, "api_key")
            .await
            .map_err(|e| ArtifexError::validation(format!("db error: {}", e)))?;

        Ok(())
    }

    /// Deletes a credential for a provider.
    /// Removes from memory and deletes the row from database.
    pub async fn delete_credential(&self, provider: &str) -> Result<(), ArtifexError> {
        // Remove from in-memory store
        let credential_id = format!("{}::api_key", provider);
        let _ = self.credential_store.delete(&credential_id);

        // Delete from DB
        repository::delete_credential_ref(self.repo.pool(), provider)
            .await
            .map_err(|e| ArtifexError::validation(format!("db error: {}", e)))?;

        Ok(())
    }

    /// Gets the credential status for a provider.
    /// Checks both DB reference AND credential store secret existence.
    pub async fn get_credential_status(&self, provider_id: &str) -> Result<CredentialStatus, ArtifexError> {
        // Check if provider is registered
        let is_registered = self.registry.is_registered(provider_id);

        // Check if credential exists in DB
        let refs = repository::list_credential_refs(self.repo.pool()).await
            .map_err(|e| ArtifexError::validation(format!("db error: {}", e)))?;
        let has_db_ref = refs.iter().any(|(name, _)| name == provider_id);

        // Also verify the secret actually exists in the credential store
        let credential_id = format!("{}::api_key", provider_id);
        let store_has_credential = self.credential_store.get(&credential_id).is_ok();

        // Both DB ref AND store secret must exist
        let has_credential = has_db_ref && store_has_credential;

        Ok(CredentialStatus {
            provider_id: provider_id.to_string(),
            has_credential: is_registered && has_credential,
        })
    }

    /// Tests connection to a provider by attempting a minimal API call.
    /// Returns true if the credential is valid and provider responds.
    pub async fn test_provider_connection(&self, provider_id: &str) -> Result<bool, ArtifexError> {
        // Check if provider is registered
        if !self.registry.is_registered(provider_id) {
            return Ok(false);
        }

        // Get credential from store
        let credential_id = format!("{}::api_key", provider_id);
        let api_key = match self.credential_store.get(&credential_id) {
            Ok(key) => key,
            Err(_) => return Ok(false), // No credential stored
        };

        // Get the provider to test connection
        let provider = match self.registry.get_image(provider_id) {
            Some(p) => p,
            None => return Ok(false),
        };

        // Create minimal test params for image provider
        let test_params = artifex_model_config::image_provider::ImageGenParams::test_params();

        // Attempt a minimal API call
        match provider.generate(&test_params, &api_key).await {
            Ok(_) => Ok(true),
            Err(e) => {
                // For auth failures, credential is valid but API issue
                // For network errors, provider might be temporarily unavailable
                tracing::debug!("Provider {} connection test failed: {}", provider_id, e);
                Ok(false)
            }
        }
    }

    // === Model Profile CRUD ===

    /// Lists all model profiles.
    pub async fn list_model_profiles(&self) -> Result<Vec<ModelProfile>, ArtifexError> {
        repository::list_profiles(self.repo.pool()).await
    }

    /// Gets a model profile by ID.
    pub async fn get_model_profile(&self, id: Uuid) -> Result<Option<ModelProfile>, ArtifexError> {
        repository::find_profile(self.repo.pool(), &id).await
    }

    /// Creates a new model profile.
    pub async fn create_model_profile(&self, profile: ModelProfile) -> Result<ModelProfile, ArtifexError> {
        repository::create_profile(self.repo.pool(), &profile).await?;
        Ok(profile)
    }

    /// Updates an existing model profile.
    pub async fn update_model_profile(&self, profile: ModelProfile) -> Result<ModelProfile, ArtifexError> {
        repository::update_profile(self.repo.pool(), &profile).await?;
        Ok(profile)
    }

    /// Deletes a model profile.
    pub async fn delete_model_profile(&self, id: Uuid) -> Result<(), ArtifexError> {
        repository::delete_profile(self.repo.pool(), &id).await
    }

    // === Routing Rules ===

    /// Gets a routing rule by operation type.
    pub async fn get_routing_rule(&self, operation_type: &str) -> Result<Option<RoutingRule>, ArtifexError> {
        repository::find_rule(self.repo.pool(), operation_type).await
    }

    /// Lists all routing rules.
    pub async fn list_routing_rules(&self) -> Result<Vec<RoutingRule>, ArtifexError> {
        repository::list_rules(self.repo.pool()).await
    }

    /// Sets a routing rule (creates or updates).
    pub async fn set_routing_rule(&self, rule: RoutingRule) -> Result<RoutingRule, ArtifexError> {
        // Try to find existing rule
        let existing = repository::find_rule(self.repo.pool(), &rule.operation_type).await?;

        if existing.is_some() {
            repository::update_rule(self.repo.pool(), &rule).await?;
        } else {
            repository::create_rule(self.repo.pool(), &rule).await?;
        }

        Ok(rule)
    }

    // === Prompt Templates ===

    /// Lists all prompt templates.
    pub async fn list_prompt_templates(&self) -> Result<Vec<PromptTemplate>, ArtifexError> {
        repository::list_templates(self.repo.pool()).await
    }

    /// Creates a new prompt template.
    pub async fn create_prompt_template(&self, template: PromptTemplate) -> Result<PromptTemplate, ArtifexError> {
        repository::create_template(self.repo.pool(), &template).await?;
        Ok(template)
    }

    /// Deletes a prompt template.
    pub async fn delete_prompt_template(&self, id: Uuid) -> Result<(), ArtifexError> {
        repository::delete_template(self.repo.pool(), &id).await
    }

    /// Seeds the database with default profiles and routing rules.
    pub async fn seed_defaults(&self) -> Result<(), ArtifexError> {
        repository::seed_defaults(self.repo.pool()).await
    }
}

/// Creates and registers all built-in providers.
pub fn register_builtin_providers(registry: &ProviderRegistry) {
    // Replicate Image Provider
    let replicate = Arc::new(ReplicateImageProvider::new());
    if let Err(e) = registry.register_image("replicate", replicate) {
        tracing::warn!("Failed to register replicate provider: {}", e);
    }

    // Fal Image Provider
    let fal = Arc::new(FalImageProvider::new());
    if let Err(e) = registry.register_image("fal", fal) {
        tracing::warn!("Failed to register fal provider: {}", e);
    }

    // HuggingFace Image Provider
    let huggingface = Arc::new(HuggingFaceImageProvider::new());
    if let Err(e) = registry.register_image("huggingface", huggingface) {
        tracing::warn!("Failed to register huggingface provider: {}", e);
    }

    // Kie AI Image Provider
    let kie = Arc::new(KieImageProvider::new());
    if let Err(e) = registry.register_image("kie", kie) {
        tracing::warn!("Failed to register kie provider: {}", e);
    }

    // ElevenLabs Audio + TTS Provider
    // Note: ElevenLabsProvider implements both AudioProvider and TtsProvider
    let elevenlabs = Arc::new(ElevenLabsProvider::new());
    if let Err(e) = registry.register_audio("elevenlabs", elevenlabs.clone()) {
        tracing::warn!("Failed to register elevenlabs audio provider: {}", e);
    }
    if let Err(e) = registry.register_tts("elevenlabs", elevenlabs) {
        tracing::warn!("Failed to register elevenlabs tts provider: {}", e);
    }

    // Together Text Provider
    // Note: TogetherTextProvider implements TextProvider, not ImageProvider
    // For now, we only register image providers in the registry
    // Future: add text provider registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_status_serde() {
        let status = CredentialStatus {
            provider_id: "replicate".to_string(),
            has_credential: true,
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("replicate"));
        assert!(json.contains("true"));
    }
}