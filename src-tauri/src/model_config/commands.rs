//! IPC commands for model configuration.
//!
//! These commands are exposed to the frontend via Tauri IPC.

use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use artifex_model_config::{ModelProfile, ProviderMetadata, PromptTemplate, RoutingRule};

use super::service::CredentialStatus;

/// DTO for provider information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderDto {
    /// Canonical lowercase provider ID (e.g. "replicate", "fal").
    pub id: String,
    /// Human-readable display name (e.g. "Replicate", "Fal").
    pub name: String,
    pub kind: String,
    pub base_url: String,
    pub supported_capabilities: Vec<String>,
    pub auth_type: String,
    pub enabled: bool,
}

impl From<&ProviderMetadata> for ProviderDto {
    fn from(m: &ProviderMetadata) -> Self {
        Self {
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
            enabled: false, // Default; actual state comes from service.is_provider_enabled
        }
    }
}

/// DTO for model profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ModelProfileDto {
    pub id: String,
    pub provider_name: String,
    pub model_id: String,
    pub display_name: String,
    pub capabilities: Vec<String>,
    pub enabled: bool,
    pub pricing_tier: String,
    pub config: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&ModelProfile> for ModelProfileDto {
    fn from(p: &ModelProfile) -> Self {
        Self {
            id: p.id.to_string(),
            provider_name: p.provider_name.clone(),
            model_id: p.model_id.clone(),
            display_name: p.display_name.clone(),
            capabilities: p
                .capabilities
                .iter()
                .map(|c| format!("{:?}", c).to_lowercase())
                .collect(),
            enabled: p.enabled,
            pricing_tier: format!("{:?}", p.pricing_tier).to_lowercase(),
            config: p.config.clone(),
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
        }
    }
}

/// DTO for routing rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RoutingRuleDto {
    pub id: String,
    pub operation_type: String,
    pub default_profile_id: String,
    pub fallback_profile_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&RoutingRule> for RoutingRuleDto {
    fn from(r: &RoutingRule) -> Self {
        Self {
            id: r.id.to_string(),
            operation_type: r.operation_type.clone(),
            default_profile_id: r.default_profile_id.to_string(),
            fallback_profile_ids: r
                .fallback_profile_ids
                .iter()
                .map(|id| id.to_string())
                .collect(),
            created_at: r.created_at.to_rfc3339(),
            updated_at: r.updated_at.to_rfc3339(),
        }
    }
}

/// DTO for prompt template.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PromptTemplateDto {
    pub id: String,
    pub name: String,
    pub template_text: String,
    pub variables: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&PromptTemplate> for PromptTemplateDto {
    fn from(t: &PromptTemplate) -> Self {
        Self {
            id: t.id.to_string(),
            name: t.name.clone(),
            template_text: t.template_text.clone(),
            variables: t.variables.clone(),
            created_at: t.created_at.to_rfc3339(),
            updated_at: t.updated_at.to_rfc3339(),
        }
    }
}

/// Lists all registered providers.
#[tauri::command]
pub fn list_providers(state: State<'_, crate::AppState>) -> Result<Vec<ProviderDto>, String> {
    let service = &state.model_config_service;
    Ok(service.list_providers())
}

/// Gets a provider by canonical id (lowercase).
#[tauri::command]
pub fn get_provider(state: State<'_, crate::AppState>, provider_id: String) -> Result<Option<ProviderDto>, String> {
    let service = &state.model_config_service;
    Ok(service.get_provider(&provider_id).map(|p| {
        ProviderDto {
            id: p.id.clone(),
            name: p.name.clone(),
            kind: format!("{:?}", p.kind).to_lowercase(),
            base_url: p.base_url.clone(),
            supported_capabilities: p
                .supported_capabilities
                .iter()
                .map(|c| format!("{:?}", c).to_lowercase())
                .collect(),
            auth_type: format!("{:?}", p.auth_type).to_lowercase(),
            enabled: service.is_provider_enabled(&provider_id),
        }
    }))
}

/// Sets whether a provider is enabled.
#[tauri::command]
pub async fn set_provider_enabled(
    state: State<'_, crate::AppState>,
    provider_id: String,
    enabled: bool,
) -> Result<(), String> {
    let service = &state.model_config_service;
    // Normalize to lowercase at command boundary
    let provider_id = provider_id.to_lowercase();
    service.set_provider_enabled(&provider_id, enabled).await.map_err(|e| e.to_string())
}

/// Tests connection to a provider.
#[tauri::command]
pub async fn test_provider_connection(
    state: State<'_, crate::AppState>,
    provider_id: String,
) -> Result<bool, String> {
    let service = &state.model_config_service;
    // Normalize to lowercase at command boundary
    let provider_id = provider_id.to_lowercase();
    service.test_provider_connection(&provider_id).await.map_err(|e| e.to_string())
}

/// Lists all model profiles.
#[tauri::command]
pub async fn list_model_profiles(state: State<'_, crate::AppState>) -> Result<Vec<ModelProfileDto>, String> {
    let service = &state.model_config_service;
    let profiles = service.list_model_profiles().await.map_err(|e| e.to_string())?;
    Ok(profiles.iter().map(ModelProfileDto::from).collect())
}

/// Creates a new model profile.
#[tauri::command]
pub async fn create_model_profile(
    state: State<'_, crate::AppState>,
    provider_name: String,
    model_id: String,
    display_name: String,
    capabilities: Vec<String>,
) -> Result<ModelProfileDto, String> {
    let service = &state.model_config_service;

    // Parse capabilities
    let caps: Result<Vec<artifex_model_config::ModelCapability>, String> = capabilities
        .iter()
        .map(|c| {
            match c.to_lowercase().as_str() {
                "imagegen" => Ok(artifex_model_config::ModelCapability::ImageGen),
                "audiogen" => Ok(artifex_model_config::ModelCapability::AudioGen),
                "tts" => Ok(artifex_model_config::ModelCapability::Tts),
                "textcomplete" | "text_complete" => Ok(artifex_model_config::ModelCapability::TextComplete),
                "codecomplete" | "code_complete" => Ok(artifex_model_config::ModelCapability::CodeComplete),
                "imageedit" | "image_edit" => Ok(artifex_model_config::ModelCapability::ImageEdit),
                "videogen" | "video_gen" => Ok(artifex_model_config::ModelCapability::VideoGen),
                _ => Err(format!("Unknown capability: {}", c)),
            }
        })
        .collect();

    let profile = ModelProfile::new(provider_name, model_id, display_name, caps.map_err(|e| e.to_string())?);

    let created = service.create_model_profile(profile).await.map_err(|e| e.to_string())?;
    Ok(ModelProfileDto::from(&created))
}

/// Updates an existing model profile.
#[tauri::command]
pub async fn update_model_profile(
    state: State<'_, crate::AppState>,
    id: String,
    provider_name: String,
    model_id: String,
    display_name: String,
    capabilities: Vec<String>,
    enabled: bool,
    pricing_tier: String,
    config: serde_json::Value,
) -> Result<ModelProfileDto, String> {
    let service = &state.model_config_service;
    let uuid = Uuid::parse_str(&id).map_err(|e| format!("Invalid UUID: {}", e))?;

    // Find existing profile
    let existing = service.get_model_profile(uuid).await.map_err(|e| e.to_string())?;
    let existing = existing.ok_or_else(|| "Profile not found".to_string())?;

    // Parse capabilities
    let caps: Result<Vec<artifex_model_config::ModelCapability>, String> = capabilities
        .iter()
        .map(|c| {
            match c.to_lowercase().as_str() {
                "imagegen" => Ok(artifex_model_config::ModelCapability::ImageGen),
                "audiogen" => Ok(artifex_model_config::ModelCapability::AudioGen),
                "tts" => Ok(artifex_model_config::ModelCapability::Tts),
                "textcomplete" | "text_complete" => Ok(artifex_model_config::ModelCapability::TextComplete),
                "codecomplete" | "code_complete" => Ok(artifex_model_config::ModelCapability::CodeComplete),
                "imageedit" | "image_edit" => Ok(artifex_model_config::ModelCapability::ImageEdit),
                "videogen" | "video_gen" => Ok(artifex_model_config::ModelCapability::VideoGen),
                _ => Err(format!("Unknown capability: {}", c)),
            }
        })
        .collect();

    let pricing = match pricing_tier.to_lowercase().as_str() {
        "free" => artifex_model_config::PricingTier::Free,
        "standard" => artifex_model_config::PricingTier::Standard,
        "premium" => artifex_model_config::PricingTier::Premium,
        _ => return Err(format!("Invalid pricing tier: {}", pricing_tier)),
    };

    let profile = ModelProfile {
        id: existing.id,
        provider_name,
        model_id,
        display_name,
        capabilities: caps.map_err(|e| e.to_string())?,
        enabled,
        pricing_tier: pricing,
        config,
        created_at: existing.created_at,
        updated_at: chrono::Utc::now(),
    };

    let updated = service.update_model_profile(profile).await.map_err(|e| e.to_string())?;
    Ok(ModelProfileDto::from(&updated))
}

/// Deletes a model profile.
#[tauri::command]
pub async fn delete_model_profile(state: State<'_, crate::AppState>, id: String) -> Result<(), String> {
    let service = &state.model_config_service;
    let uuid = Uuid::parse_str(&id).map_err(|e| format!("Invalid UUID: {}", e))?;
    service.delete_model_profile(uuid).await.map_err(|e| e.to_string())
}

/// Lists all routing rules.
#[tauri::command]
pub async fn list_routing_rules(state: State<'_, crate::AppState>) -> Result<Vec<RoutingRuleDto>, String> {
    let service = &state.model_config_service;
    let rules = service.list_routing_rules().await.map_err(|e| e.to_string())?;
    Ok(rules.iter().map(RoutingRuleDto::from).collect())
}

/// Sets a routing rule (creates or updates).
#[tauri::command]
pub async fn set_routing_rule(
    state: State<'_, crate::AppState>,
    operation_type: String,
    default_profile_id: String,
    fallback_profile_ids: Vec<String>,
) -> Result<RoutingRuleDto, String> {
    let service = &state.model_config_service;

    let default_id = Uuid::parse_str(&default_profile_id)
        .map_err(|e| format!("Invalid default profile ID: {}", e))?;

    let fallback_ids: Result<Vec<Uuid>, String> = fallback_profile_ids
        .iter()
        .map(|id| Uuid::parse_str(id).map_err(|e| format!("Invalid fallback ID {}: {}", id, e)))
        .collect();

    // Try to find existing rule
    let existing = service.get_routing_rule(&operation_type).await.map_err(|e| e.to_string())?;

    let rule = if let Some(existing_rule) = existing {
        RoutingRule {
            id: existing_rule.id,
            operation_type,
            default_profile_id: default_id,
            fallback_profile_ids: fallback_ids.map_err(|e| e.to_string())?,
            created_at: existing_rule.created_at,
            updated_at: chrono::Utc::now(),
        }
    } else {
        RoutingRule::new(operation_type, default_id, fallback_ids.map_err(|e| e.to_string())?)
    };

    let saved = service.set_routing_rule(rule).await.map_err(|e| e.to_string())?;
    Ok(RoutingRuleDto::from(&saved))
}

/// Lists all prompt templates.
#[tauri::command]
pub async fn list_prompt_templates(state: State<'_, crate::AppState>) -> Result<Vec<PromptTemplateDto>, String> {
    let service = &state.model_config_service;
    let templates = service.list_prompt_templates().await.map_err(|e| e.to_string())?;
    Ok(templates.iter().map(PromptTemplateDto::from).collect())
}

/// Creates a new prompt template.
#[tauri::command]
pub async fn create_prompt_template(
    state: State<'_, crate::AppState>,
    name: String,
    template_text: String,
) -> Result<PromptTemplateDto, String> {
    let service = &state.model_config_service;

    let template = PromptTemplate::new(name, template_text);
    let created = service.create_prompt_template(template).await.map_err(|e| e.to_string())?;
    Ok(PromptTemplateDto::from(&created))
}

/// Deletes a prompt template.
#[tauri::command]
pub async fn delete_prompt_template(state: State<'_, crate::AppState>, id: String) -> Result<(), String> {
    let service = &state.model_config_service;
    let uuid = Uuid::parse_str(&id).map_err(|e| format!("Invalid UUID: {}", e))?;
    service.delete_prompt_template(uuid).await.map_err(|e| e.to_string())
}

/// Gets the credential status for a provider.
#[tauri::command]
pub async fn get_credential_status(
    state: State<'_, crate::AppState>,
    provider_id: String,
) -> Result<CredentialStatus, String> {
    let service = &state.model_config_service;
    // Normalize to lowercase at command boundary
    let provider_id = provider_id.to_lowercase();
    service.get_credential_status(&provider_id).await.map_err(|e| e.to_string())
}

/// Sets a credential for a provider.
#[tauri::command]
pub async fn set_credential(
    state: State<'_, crate::AppState>,
    provider_id: String,
    api_key: String,
) -> Result<(), String> {
    let service = &state.model_config_service;
    // Normalize to lowercase at command boundary
    let provider_id = provider_id.to_lowercase();
    service.set_credential(&provider_id, &api_key).await.map_err(|e| e.to_string())
}

/// Deletes a credential for a provider.
#[tauri::command]
pub async fn delete_credential(
    state: State<'_, crate::AppState>,
    provider_id: String,
) -> Result<(), String> {
    let service = &state.model_config_service;
    // Normalize to lowercase at command boundary
    let provider_id = provider_id.to_lowercase();
    service.delete_credential(&provider_id).await.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_dto_serde() {
        let dto = ProviderDto {
            id: "replicate".to_string(),
            name: "Replicate".to_string(),
            kind: "replicate".to_string(),
            base_url: "https://api.replicate.com".to_string(),
            supported_capabilities: vec!["imagegen".to_string()],
            auth_type: "apikey".to_string(),
            enabled: true,
        };

        let json = serde_json::to_string(& dto).unwrap();
        assert!(json.contains("replicate"));
        assert!(json.contains("Replicate"));
    }

    #[test]
    fn test_model_profile_dto_serde() {
        let dto = ModelProfileDto {
            id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            provider_name: "replicate".to_string(),
            model_id: "black-forest-labs/flux-1.1-pro".to_string(),
            display_name: "FLUX 1.1 Pro".to_string(),
            capabilities: vec!["imagegen".to_string()],
            enabled: true,
            pricing_tier: "standard".to_string(),
            config: serde_json::json!({}),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("FLUX 1.1 Pro"));
        assert!(json.contains("replicate"));
    }
}