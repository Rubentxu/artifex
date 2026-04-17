//! SQLite implementation of the model config repository.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use artifex_model_config::{
    ModelCapability, ModelProfile, PricingTier, PromptTemplate, RoutingRule,
};
use artifex_model_config::router::ModelConfigRepository;
use artifex_shared_kernel::{is_unique_violation, ArtifexError};

/// SQLite row representation of a model profile.
#[derive(FromRow)]
struct ModelProfileRow {
    id: String,
    provider_name: String,
    model_id: String,
    display_name: String,
    capabilities: String,
    enabled: i32,
    pricing_tier: String,
    config: String,
    created_at: String,
    updated_at: String,
}

/// SQLite row representation of a routing rule.
#[derive(FromRow)]
struct RoutingRuleRow {
    id: String,
    operation_type: String,
    default_profile_id: String,
    fallback_profile_ids: String,
    created_at: String,
    updated_at: String,
}

/// SQLite row representation of a provider credential reference.
#[derive(FromRow)]
#[allow(dead_code)]
struct ProviderCredentialRow {
    id: String,
    provider_name: String,
    key_type: String,
    created_at: String,
    updated_at: String,
}

/// SQLite row representation of a prompt template.
#[derive(FromRow)]
struct PromptTemplateRow {
    id: String,
    name: String,
    template_text: String,
    variables: String,
    created_at: String,
    updated_at: String,
}

/// SQLite-backed model config repository.
pub struct SqliteModelConfigRepository {
    pool: SqlitePool,
}

impl SqliteModelConfigRepository {
    /// Creates a new SqliteModelConfigRepository.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Returns a reference to the database pool.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[async_trait]
impl ModelConfigRepository for SqliteModelConfigRepository {
    async fn find_profile(&self, id: &Uuid) -> Result<Option<ModelProfile>, String> {
        find_profile(&self.pool, id)
            .await
            .map_err(|e| e.to_string())
    }

    async fn find_rule(&self, operation_type: &str) -> Result<Option<RoutingRule>, String> {
        find_rule(&self.pool, operation_type)
            .await
            .map_err(|e| e.to_string())
    }

    async fn list_enabled_profiles(&self, _capability: ModelCapability) -> Result<Vec<ModelProfile>, String> {
        list_profiles(&self.pool)
            .await
            .map(|ps| ps.into_iter().filter(|p| p.enabled).collect())
            .map_err(|e| e.to_string())
    }
}

// === Model Profile Operations ===

/// Creates a new model profile.
pub async fn create_profile(
    pool: &SqlitePool,
    profile: &ModelProfile,
) -> Result<(), ArtifexError> {
    let capabilities_json =
        serde_json::to_string(&profile.capabilities).unwrap_or_else(|_| "[]".to_string());
    let config_json = serde_json::to_string(&profile.config).unwrap_or_else(|_| "{}".to_string());

    let result = sqlx::query(
        r#"INSERT INTO model_profiles
           (id, provider_name, model_id, display_name, capabilities, enabled, pricing_tier, config, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(profile.id.to_string())
    .bind(&profile.provider_name)
    .bind(&profile.model_id)
    .bind(&profile.display_name)
    .bind(&capabilities_json)
    .bind(if profile.enabled { 1 } else { 0 })
    .bind(profile.pricing_tier.as_str())
    .bind(&config_json)
    .bind(profile.created_at.to_rfc3339())
    .bind(profile.updated_at.to_rfc3339())
    .execute(pool)
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(e) if is_unique_violation(&e) => {
            Err(ArtifexError::duplicate_name(&profile.display_name))
        }
        Err(e) => Err(ArtifexError::IoError(e.to_string())),
    }
}

/// Finds a model profile by ID.
pub async fn find_profile(pool: &SqlitePool, id: &Uuid) -> Result<Option<ModelProfile>, ArtifexError> {
    let row: Option<ModelProfileRow> = sqlx::query_as(
        "SELECT id, provider_name, model_id, display_name, capabilities, enabled, pricing_tier, config, created_at, updated_at FROM model_profiles WHERE id = ?",
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(|e| ArtifexError::IoError(e.to_string()))?;

    match row {
        Some(r) => row_to_profile(&r).map(Some),
        None => Ok(None),
    }
}

/// Lists all model profiles.
pub async fn list_profiles(pool: &SqlitePool) -> Result<Vec<ModelProfile>, ArtifexError> {
    let rows: Vec<ModelProfileRow> = sqlx::query_as(
        "SELECT id, provider_name, model_id, display_name, capabilities, enabled, pricing_tier, config, created_at, updated_at FROM model_profiles ORDER BY display_name",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ArtifexError::IoError(e.to_string()))?;

    let mut profiles = Vec::with_capacity(rows.len());
    for row in rows {
        profiles.push(row_to_profile(&row)?);
    }
    Ok(profiles)
}

/// Updates a model profile.
pub async fn update_profile(pool: &SqlitePool, profile: &ModelProfile) -> Result<(), ArtifexError> {
    let capabilities_json =
        serde_json::to_string(&profile.capabilities).unwrap_or_else(|_| "[]".to_string());
    let config_json = serde_json::to_string(&profile.config).unwrap_or_else(|_| "{}".to_string());

    let result = sqlx::query(
        r#"UPDATE model_profiles
           SET provider_name = ?, model_id = ?, display_name = ?, capabilities = ?,
               enabled = ?, pricing_tier = ?, config = ?, updated_at = ?
           WHERE id = ?"#,
    )
    .bind(&profile.provider_name)
    .bind(&profile.model_id)
    .bind(&profile.display_name)
    .bind(&capabilities_json)
    .bind(if profile.enabled { 1 } else { 0 })
    .bind(profile.pricing_tier.as_str())
    .bind(&config_json)
    .bind(profile.updated_at.to_rfc3339())
    .bind(profile.id.to_string())
    .execute(pool)
    .await;

    match result {
        Ok(affected) if affected.rows_affected() == 0 => {
            Err(ArtifexError::NotFound(format!(
                "ModelProfile {} not found",
                profile.id
            )))
        }
        Ok(_) => Ok(()),
        Err(e) if is_unique_violation(&e) => {
            Err(ArtifexError::duplicate_name(&profile.display_name))
        }
        Err(e) => Err(ArtifexError::IoError(e.to_string())),
    }
}

/// Deletes a model profile.
pub async fn delete_profile(pool: &SqlitePool, id: &Uuid) -> Result<(), ArtifexError> {
    let result = sqlx::query("DELETE FROM model_profiles WHERE id = ?")
        .bind(id.to_string())
        .execute(pool)
        .await;

    match result {
        Ok(affected) if affected.rows_affected() == 0 => {
            Err(ArtifexError::NotFound(format!("ModelProfile {} not found", id)))
        }
        Ok(_) => Ok(()),
        Err(e) => Err(ArtifexError::IoError(e.to_string())),
    }
}

/// Sets enabled state for all model profiles of a provider.
pub async fn set_provider_profiles_enabled(
    pool: &SqlitePool,
    provider_name: &str,
    enabled: bool,
) -> Result<(), ArtifexError> {
    let now = chrono::Utc::now().to_rfc3339();

    let result = sqlx::query(
        r#"UPDATE model_profiles
           SET enabled = ?, updated_at = ?
           WHERE provider_name = ?"#,
    )
    .bind(if enabled { 1 } else { 0 })
    .bind(&now)
    .bind(provider_name)
    .execute(pool)
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(ArtifexError::IoError(e.to_string())),
    }
}

// === Routing Rule Operations ===

/// Creates a new routing rule.
pub async fn create_rule(pool: &SqlitePool, rule: &RoutingRule) -> Result<(), ArtifexError> {
    let fallback_ids_json =
        serde_json::to_string(&rule.fallback_profile_ids).unwrap_or_else(|_| "[]".to_string());

    let result = sqlx::query(
        r#"INSERT INTO routing_rules
           (id, operation_type, default_profile_id, fallback_profile_ids, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?)"#,
    )
    .bind(rule.id.to_string())
    .bind(&rule.operation_type)
    .bind(rule.default_profile_id.to_string())
    .bind(&fallback_ids_json)
    .bind(rule.created_at.to_rfc3339())
    .bind(rule.updated_at.to_rfc3339())
    .execute(pool)
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(e) if is_unique_violation(&e) => {
            Err(ArtifexError::duplicate_name(&rule.operation_type))
        }
        Err(e) => Err(ArtifexError::IoError(e.to_string())),
    }
}

/// Finds a routing rule by operation type.
pub async fn find_rule(
    pool: &SqlitePool,
    operation_type: &str,
) -> Result<Option<RoutingRule>, ArtifexError> {
    let row: Option<RoutingRuleRow> = sqlx::query_as(
        "SELECT id, operation_type, default_profile_id, fallback_profile_ids, created_at, updated_at FROM routing_rules WHERE operation_type = ?",
    )
    .bind(operation_type)
    .fetch_optional(pool)
    .await
    .map_err(|e| ArtifexError::IoError(e.to_string()))?;

    match row {
        Some(r) => row_to_rule(&r).map(Some),
        None => Ok(None),
    }
}

/// Lists all routing rules.
pub async fn list_rules(pool: &SqlitePool) -> Result<Vec<RoutingRule>, ArtifexError> {
    let rows: Vec<RoutingRuleRow> = sqlx::query_as(
        "SELECT id, operation_type, default_profile_id, fallback_profile_ids, created_at, updated_at FROM routing_rules ORDER BY operation_type",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ArtifexError::IoError(e.to_string()))?;

    let mut rules = Vec::with_capacity(rows.len());
    for row in rows {
        rules.push(row_to_rule(&row)?);
    }
    Ok(rules)
}

/// Updates a routing rule.
pub async fn update_rule(pool: &SqlitePool, rule: &RoutingRule) -> Result<(), ArtifexError> {
    let fallback_ids_json =
        serde_json::to_string(&rule.fallback_profile_ids).unwrap_or_else(|_| "[]".to_string());

    let result = sqlx::query(
        r#"UPDATE routing_rules
           SET default_profile_id = ?, fallback_profile_ids = ?, updated_at = ?
           WHERE id = ?"#,
    )
    .bind(rule.default_profile_id.to_string())
    .bind(&fallback_ids_json)
    .bind(rule.updated_at.to_rfc3339())
    .bind(rule.id.to_string())
    .execute(pool)
    .await;

    match result {
        Ok(affected) if affected.rows_affected() == 0 => {
            Err(ArtifexError::NotFound(format!("RoutingRule {} not found", rule.id)))
        }
        Ok(_) => Ok(()),
        Err(e) if is_unique_violation(&e) => {
            Err(ArtifexError::duplicate_name(&rule.operation_type))
        }
        Err(e) => Err(ArtifexError::IoError(e.to_string())),
    }
}

/// Deletes a routing rule.
pub async fn delete_rule(pool: &SqlitePool, id: &Uuid) -> Result<(), ArtifexError> {
    let result = sqlx::query("DELETE FROM routing_rules WHERE id = ?")
        .bind(id.to_string())
        .execute(pool)
        .await;

    match result {
        Ok(affected) if affected.rows_affected() == 0 => {
            Err(ArtifexError::NotFound(format!("RoutingRule {} not found", id)))
        }
        Ok(_) => Ok(()),
        Err(e) => Err(ArtifexError::IoError(e.to_string())),
    }
}

// === Provider Credential Operations ===

/// Creates a provider credential reference.
pub async fn create_credential_ref(
    pool: &SqlitePool,
    id: &Uuid,
    provider_name: &str,
    key_type: &str,
) -> Result<(), ArtifexError> {
    let now = Utc::now();

    let result = sqlx::query(
        r#"INSERT INTO provider_credentials
           (id, provider_name, key_type, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?)"#,
    )
    .bind(id.to_string())
    .bind(provider_name)
    .bind(key_type)
    .bind(now.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(pool)
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(e) if is_unique_violation(&e) => {
            Err(ArtifexError::duplicate_name(provider_name))
        }
        Err(e) => Err(ArtifexError::IoError(e.to_string())),
    }
}

/// Lists all provider credential references.
pub async fn list_credential_refs(pool: &SqlitePool) -> Result<Vec<(String, String)>, ArtifexError> {
    let rows: Vec<ProviderCredentialRow> = sqlx::query_as(
        "SELECT id, provider_name, key_type, created_at, updated_at FROM provider_credentials ORDER BY provider_name",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ArtifexError::IoError(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|r| (r.provider_name, r.key_type))
        .collect())
}

/// Deletes a provider credential reference.
pub async fn delete_credential_ref(pool: &SqlitePool, provider_name: &str) -> Result<(), ArtifexError> {
    let result = sqlx::query("DELETE FROM provider_credentials WHERE provider_name = ?")
        .bind(provider_name)
        .execute(pool)
        .await;

    match result {
        Ok(affected) if affected.rows_affected() == 0 => {
            Err(ArtifexError::NotFound(format!(
                "Credential for provider {} not found",
                provider_name
            )))
        }
        Ok(_) => Ok(()),
        Err(e) => Err(ArtifexError::IoError(e.to_string())),
    }
}

// === Prompt Template Operations ===

/// Creates a new prompt template.
pub async fn create_template(pool: &SqlitePool, template: &PromptTemplate) -> Result<(), ArtifexError> {
    let variables_json =
        serde_json::to_string(&template.variables).unwrap_or_else(|_| "[]".to_string());

    let result = sqlx::query(
        r#"INSERT INTO prompt_templates
           (id, name, template_text, variables, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?)"#,
    )
    .bind(template.id.to_string())
    .bind(&template.name)
    .bind(&template.template_text)
    .bind(&variables_json)
    .bind(template.created_at.to_rfc3339())
    .bind(template.updated_at.to_rfc3339())
    .execute(pool)
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(e) if is_unique_violation(&e) => {
            Err(ArtifexError::duplicate_name(&template.name))
        }
        Err(e) => Err(ArtifexError::IoError(e.to_string())),
    }
}

/// Finds a prompt template by ID.
pub async fn find_template(pool: &SqlitePool, id: &Uuid) -> Result<Option<PromptTemplate>, ArtifexError> {
    let row: Option<PromptTemplateRow> = sqlx::query_as(
        "SELECT id, name, template_text, variables, created_at, updated_at FROM prompt_templates WHERE id = ?",
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(|e| ArtifexError::IoError(e.to_string()))?;

    match row {
        Some(r) => row_to_template(&r).map(Some),
        None => Ok(None),
    }
}

/// Lists all prompt templates.
pub async fn list_templates(pool: &SqlitePool) -> Result<Vec<PromptTemplate>, ArtifexError> {
    let rows: Vec<PromptTemplateRow> = sqlx::query_as(
        "SELECT id, name, template_text, variables, created_at, updated_at FROM prompt_templates ORDER BY name",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ArtifexError::IoError(e.to_string()))?;

    let mut templates = Vec::with_capacity(rows.len());
    for row in rows {
        templates.push(row_to_template(&row)?);
    }
    Ok(templates)
}

/// Updates a prompt template.
pub async fn update_template(pool: &SqlitePool, template: &PromptTemplate) -> Result<(), ArtifexError> {
    let variables_json =
        serde_json::to_string(&template.variables).unwrap_or_else(|_| "[]".to_string());

    let result = sqlx::query(
        r#"UPDATE prompt_templates
           SET name = ?, template_text = ?, variables = ?, updated_at = ?
           WHERE id = ?"#,
    )
    .bind(&template.name)
    .bind(&template.template_text)
    .bind(&variables_json)
    .bind(template.updated_at.to_rfc3339())
    .bind(template.id.to_string())
    .execute(pool)
    .await;

    match result {
        Ok(affected) if affected.rows_affected() == 0 => {
            Err(ArtifexError::NotFound(format!(
                "PromptTemplate {} not found",
                template.id
            )))
        }
        Ok(_) => Ok(()),
        Err(e) if is_unique_violation(&e) => {
            Err(ArtifexError::duplicate_name(&template.name))
        }
        Err(e) => Err(ArtifexError::IoError(e.to_string())),
    }
}

/// Deletes a prompt template.
pub async fn delete_template(pool: &SqlitePool, id: &Uuid) -> Result<(), ArtifexError> {
    let result = sqlx::query("DELETE FROM prompt_templates WHERE id = ?")
        .bind(id.to_string())
        .execute(pool)
        .await;

    match result {
        Ok(affected) if affected.rows_affected() == 0 => {
            Err(ArtifexError::NotFound(format!("PromptTemplate {} not found", id)))
        }
        Ok(_) => Ok(()),
        Err(e) => Err(ArtifexError::IoError(e.to_string())),
    }
}

// === Helper Functions ===

fn row_to_profile(row: &ModelProfileRow) -> Result<ModelProfile, ArtifexError> {
    let id = Uuid::parse_str(&row.id)
        .map_err(|e| ArtifexError::ValidationError(format!("Invalid profile id: {}", e)))?;

    let capabilities: Vec<ModelCapability> =
        serde_json::from_str(&row.capabilities).unwrap_or_else(|_| vec![]);

    let pricing_tier = PricingTier::from_str(&row.pricing_tier).unwrap_or(PricingTier::Standard);

    let config: JsonValue = serde_json::from_str(&row.config).unwrap_or(serde_json::json!({}));

    let created_at = parse_rfc3339(&row.created_at)?;
    let updated_at = parse_rfc3339(&row.updated_at)?;

    Ok(ModelProfile {
        id,
        provider_name: row.provider_name.clone(),
        model_id: row.model_id.clone(),
        display_name: row.display_name.clone(),
        capabilities,
        enabled: row.enabled != 0,
        pricing_tier,
        config,
        created_at,
        updated_at,
    })
}

fn row_to_rule(row: &RoutingRuleRow) -> Result<RoutingRule, ArtifexError> {
    let id = Uuid::parse_str(&row.id)
        .map_err(|e| ArtifexError::ValidationError(format!("Invalid rule id: {}", e)))?;

    let default_profile_id = Uuid::parse_str(&row.default_profile_id).map_err(|e| {
        ArtifexError::ValidationError(format!("Invalid default profile id: {}", e))
    })?;

    let fallback_profile_ids: Vec<Uuid> =
        serde_json::from_str(&row.fallback_profile_ids).unwrap_or_else(|_| vec![]);

    let created_at = parse_rfc3339(&row.created_at)?;
    let updated_at = parse_rfc3339(&row.updated_at)?;

    Ok(RoutingRule {
        id,
        operation_type: row.operation_type.clone(),
        default_profile_id,
        fallback_profile_ids,
        created_at,
        updated_at,
    })
}

fn row_to_template(row: &PromptTemplateRow) -> Result<PromptTemplate, ArtifexError> {
    let id = Uuid::parse_str(&row.id)
        .map_err(|e| ArtifexError::ValidationError(format!("Invalid template id: {}", e)))?;

    let variables: Vec<String> = serde_json::from_str(&row.variables).unwrap_or_else(|_| vec![]);

    let created_at = parse_rfc3339(&row.created_at)?;
    let updated_at = parse_rfc3339(&row.updated_at)?;

    Ok(PromptTemplate {
        id,
        name: row.name.clone(),
        template_text: row.template_text.clone(),
        variables,
        created_at,
        updated_at,
    })
}

fn parse_rfc3339(s: &str) -> Result<DateTime<Utc>, ArtifexError> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| ArtifexError::ValidationError(format!("Invalid timestamp: {}", e)))
}

/// Seeds the database with default model profiles and routing rules.
///
/// This function is idempotent - it will not create duplicates if called multiple times.
pub async fn seed_defaults(pool: &SqlitePool) -> Result<(), ArtifexError> {
    use artifex_model_config::{ModelCapability, ModelProfile, RoutingRule};

    // Check if we already have image/text profiles
    let existing = list_profiles(pool).await?;
    if existing.is_empty() {
        // Seed image and text profiles only if none exist
        let profiles = vec![
            ModelProfile::new(
                "replicate".to_string(),
                "black-forest-labs/flux-1.1-pro".to_string(),
                "FLUX 1.1 Pro (Replicate)".to_string(),
                vec![ModelCapability::ImageGen],
            ),
            ModelProfile::new(
                "fal".to_string(),
                "fal-ai/flux-1-dev".to_string(),
                "FLUX.1 Dev (Fal)".to_string(),
                vec![ModelCapability::ImageGen],
            ),
            ModelProfile::new(
                "kie".to_string(),
                "flux-kontext".to_string(),
                "Flux Kontext (Kie)".to_string(),
                vec![ModelCapability::ImageGen],
            ),
            ModelProfile::new(
                "together".to_string(),
                "meta-llama/Meta-Llama-3-70B".to_string(),
                "Llama 3 (Together)".to_string(),
                vec![ModelCapability::TextComplete],
            ),
        ];

        // Insert profiles and collect their IDs
        let mut profile_ids = std::collections::HashMap::new();
        for profile in profiles {
            profile_ids.insert(profile.display_name.clone(), profile.id);
            let _ = create_profile(pool, &profile).await;
        }

        // Create default routing rules for image/text
        let rules = vec![
            RoutingRule::new(
                "imagegen.txt2img".to_string(),
                *profile_ids
                    .get("FLUX 1.1 Pro (Replicate)")
                    .expect("profile exists"),
                vec![
                    *profile_ids
                        .get("FLUX.1 Dev (Fal)")
                        .expect("profile exists"),
                    *profile_ids
                        .get("Flux Kontext (Kie)")
                        .expect("profile exists"),
                ],
            ),
            RoutingRule::new(
                "textgen.complete".to_string(),
                *profile_ids
                    .get("Llama 3 (Together)")
                    .expect("profile exists"),
                vec![],
            ),
        ];

        for rule in rules {
            let _ = create_rule(pool, &rule).await;
        }
    }

    // Always seed ElevenLabs audio/TTS profiles if they don't exist
    // This allows adding ElevenLabs to an already-seeded database
    let elevenlabs_profiles = vec![
        ModelProfile::new(
            "elevenlabs".to_string(),
            "eleven_multilingual_v2".to_string(),
            "ElevenLabs TTS — Multilingual v2".to_string(),
            vec![ModelCapability::Tts],
        ),
        ModelProfile::new(
            "elevenlabs".to_string(),
            "eleven_turbo_v2".to_string(),
            "ElevenLabs TTS — Turbo v2".to_string(),
            vec![ModelCapability::Tts],
        ),
        ModelProfile::new(
            "elevenlabs".to_string(),
            "sound_generation".to_string(),
            "ElevenLabs SFX".to_string(),
            vec![ModelCapability::AudioGen],
        ),
        ModelProfile::new(
            "elevenlabs".to_string(),
            "eleven_music_v1".to_string(),
            "ElevenLabs Music".to_string(),
            vec![ModelCapability::AudioGen],
        ),
    ];

    let mut elevenlabs_profile_ids = std::collections::HashMap::new();
    for profile in elevenlabs_profiles {
        elevenlabs_profile_ids.insert(profile.display_name.clone(), profile.id);
        let _ = create_profile(pool, &profile).await;
    }

    // Create ElevenLabs routing rules
    let elevenlabs_rules = vec![
        RoutingRule::new(
            "tts.npc_line".to_string(),
            *elevenlabs_profile_ids
                .get("ElevenLabs TTS — Multilingual v2")
                .expect("profile exists"),
            vec![],
        ),
        RoutingRule::new(
            "audiogen.sfx".to_string(),
            *elevenlabs_profile_ids
                .get("ElevenLabs SFX")
                .expect("profile exists"),
            vec![],
        ),
        RoutingRule::new(
            "audiogen.music".to_string(),
            *elevenlabs_profile_ids
                .get("ElevenLabs Music")
                .expect("profile exists"),
            vec![],
        ),
    ];

    for rule in elevenlabs_rules {
        let _ = create_rule(pool, &rule).await;
    }

    // Seed background removal and tile generation profiles and routing rules
    let imageproc_profiles = vec![
        ModelProfile::new(
            "replicate".to_string(),
            "851-labs/background-remover".to_string(),
            "Background Remover (Replicate)".to_string(),
            vec![ModelCapability::BackgroundRemoval],
        ),
        ModelProfile::new(
            "fal".to_string(),
            "fal-ai/rmbg".to_string(),
            "Fal Background Removal".to_string(),
            vec![ModelCapability::BackgroundRemoval],
        ),
    ];

    let mut imageproc_profile_ids = std::collections::HashMap::new();
    for profile in imageproc_profiles {
        imageproc_profile_ids.insert(profile.display_name.clone(), profile.id);
        let _ = create_profile(pool, &profile).await;
    }

    let imageproc_rules = vec![
        RoutingRule::new(
            "imageproc.remove_bg".to_string(),
            *imageproc_profile_ids
                .get("Background Remover (Replicate)")
                .expect("profile exists"),
            vec![
                *imageproc_profile_ids
                    .get("Fal Background Removal")
                    .expect("profile exists"),
            ],
        ),
    ];

    for rule in imageproc_rules {
        let _ = create_rule(pool, &rule).await;
    }

    // Seed inpainting and outpainting profiles and routing rules
    let imageedit_profiles = vec![
        ModelProfile::new(
            "fal".to_string(),
            "fal-ai/flux-fill-dev".to_string(),
            "Flux Fill (Fal)".to_string(),
            vec![ModelCapability::ImageEdit],
        ),
        ModelProfile::new(
            "replicate".to_string(),
            "stability-ai/sdxl-inpainting:0.1.0".to_string(),
            "SDXL Inpainting (Replicate)".to_string(),
            vec![ModelCapability::ImageEdit],
        ),
    ];

    let mut imageedit_profile_ids = std::collections::HashMap::new();
    for profile in imageedit_profiles {
        imageedit_profile_ids.insert(profile.display_name.clone(), profile.id);
        let _ = create_profile(pool, &profile).await;
    }

    let imageedit_rules = vec![
        RoutingRule::new(
            "imageedit.inpaint".to_string(),
            *imageedit_profile_ids
                .get("Flux Fill (Fal)")
                .expect("profile exists"),
            vec![
                *imageedit_profile_ids
                    .get("SDXL Inpainting (Replicate)")
                    .expect("profile exists"),
            ],
        ),
        RoutingRule::new(
            "imageedit.outpaint".to_string(),
            *imageedit_profile_ids
                .get("Flux Fill (Fal)")
                .expect("profile exists"),
            vec![
                *imageedit_profile_ids
                    .get("SDXL Inpainting (Replicate)")
                    .expect("profile exists"),
            ],
        ),
    ];

    for rule in imageedit_rules {
        let _ = create_rule(pool, &rule).await;
    }

    // Seed tile generation profiles
    let tilegen_profiles = vec![
        ModelProfile::new(
            "fal".to_string(),
            "fal-ai/patina/material".to_string(),
            "Tile Material (Fal)".to_string(),
            vec![ModelCapability::TileGen],
        ),
    ];

    let mut tilegen_profile_ids = std::collections::HashMap::new();
    for profile in tilegen_profiles {
        tilegen_profile_ids.insert(profile.display_name.clone(), profile.id);
        let _ = create_profile(pool, &profile).await;
    }

    let tilegen_rules = vec![
        RoutingRule::new(
            "tilegen.seamless".to_string(),
            *tilegen_profile_ids
                .get("Tile Material (Fal)")
                .expect("profile exists"),
            vec![],
        ),
        RoutingRule::new(
            "tilegen.basic".to_string(),
            *tilegen_profile_ids
                .get("Tile Material (Fal)")
                .expect("profile exists"),
            vec![],
        ),
    ];

    for rule in tilegen_rules {
        let _ = create_rule(pool, &rule).await;
    }

    // Seed code generation routing rules using the existing TextComplete profile
    let all_profiles = list_profiles(pool).await?;
    if let Some(text_profile) = all_profiles.iter().find(|p| {
        p.capabilities
            .contains(&ModelCapability::TextComplete)
    }) {
        let codegen_rules = vec![
            RoutingRule::new(
                "codegen.godot".to_string(),
                text_profile.id,
                vec![],
            ),
            RoutingRule::new(
                "codegen.unity".to_string(),
                text_profile.id,
                vec![],
            ),
        ];
        for rule in codegen_rules {
            let _ = create_rule(pool, &rule).await;
        }
    }

    Ok(())
}
