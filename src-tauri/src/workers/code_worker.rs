//! Code generation worker.
//!
//! Handles code_generate jobs by invoking text providers.

use std::path::PathBuf;
use std::sync::Arc;

use artifex_asset_management::CodeEngine;
use artifex_job_queue::Job;
use artifex_model_config::credential_store::CredentialStore;
use artifex_model_config::{ModelRouter, ResolvedTextProfile, TextParams, TextResult};
use artifex_shared_kernel::AppError;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::model_config::built_in_templates::built_in_templates;
use crate::workers::traits::{JobFuture, JobResult, JobWorker};

/// Payload for code generation jobs.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeGenOperation {
    /// The project ID.
    pub project_id: String,
    /// Target engine ("godot" or "unity").
    pub engine: String,
    /// User's prompt describing what code to generate.
    pub prompt: String,
    /// Optional template ID to use.
    #[serde(default)]
    pub template_id: Option<String>,
    /// Optional specific model ID to use.
    #[serde(default)]
    pub model_id: Option<String>,
    /// Temperature for generation (0.0 to 1.0).
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// Maximum tokens to generate.
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

fn default_temperature() -> f32 {
    0.25
}

fn default_max_tokens() -> u32 {
    4096
}

/// Response structure from code generation model.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct CodeGenResponse {
    /// The engine used (godot or unity).
    engine: String,
    /// Summary of what was generated.
    summary: String,
    /// List of generated files.
    files: Vec<CodeFile>,
}

/// A single generated code file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeFile {
    /// Relative path for the file.
    pub path: String,
    /// Programming language.
    pub language: String,
    /// Description of what the file contains.
    pub description: String,
    /// The actual code content.
    pub content: String,
}

/// Worker for code generation jobs.
pub struct CodeWorker {
    /// Base directory for saving output assets.
    assets_dir: String,
    /// Model router for resolving text providers.
    model_router: Arc<ModelRouter>,
    /// Credential store for API keys.
    credential_store: Arc<dyn CredentialStore>,
}

impl CodeWorker {
    /// Creates a new CodeWorker.
    pub fn new(
        assets_dir: String,
        model_router: Arc<ModelRouter>,
        credential_store: Arc<dyn CredentialStore>,
    ) -> Self {
        Self {
            assets_dir,
            model_router,
            credential_store,
        }
    }

    /// Main processing function for code generation.
    async fn process_code_job(
        &self,
        job_id: artifex_shared_kernel::JobId,
        payload: CodeGenOperation,
    ) -> Result<JobResult, AppError> {
        let engine = CodeEngine::from_str(&payload.engine)
            .ok_or_else(|| AppError::validation(format!("Unsupported engine: {}", payload.engine)))?;

        // Build the full prompt
        let full_prompt = self.build_prompt(&engine, &payload)?;

        // Resolve text provider - use model_id if provided, otherwise use routing rules
        let resolved = if let Some(model_id) = &payload.model_id {
            let profile_uuid = uuid::Uuid::parse_str(model_id)
                .map_err(|_| AppError::validation(format!("Invalid model_id: {}", model_id)))?;
            self.model_router
                .resolve_text_by_profile_id(&profile_uuid)
                .await
                .map_err(|e| match e {
                    artifex_model_config::RoutingError::ProviderNotRegistered(_) => {
                        AppError::validation("No text provider registered for the specified model")
                    }
                    artifex_model_config::RoutingError::CredentialNotFound(_) => {
                        AppError::validation("No credential found for the specified model provider")
                    }
                    artifex_model_config::RoutingError::ProfileNotFound(_) => {
                        AppError::validation(format!("Model profile not found: {}", model_id))
                    }
                    _ => AppError::internal(format!("Routing error: {}", e)),
                })?
        } else {
            let operation_type = format!("codegen.{}", payload.engine);
            self.model_router
                .resolve_text(&operation_type)
                .await
                .map_err(|e| match e {
                    artifex_model_config::RoutingError::ProviderNotRegistered(_) => {
                        AppError::validation("No text provider registered for code generation")
                    }
                    artifex_model_config::RoutingError::CredentialNotFound(_) => {
                        AppError::validation("No credential found for text provider")
                    }
                    artifex_model_config::RoutingError::NoRuleForOperation(_) => {
                        AppError::validation(format!(
                            "No routing rule configured for operation: {}",
                            operation_type
                        ))
                    }
                    _ => AppError::internal(format!("Routing error: {}", e)),
                })?
        };

        // Call the text provider
        let text_result = self
            .call_text_provider(&resolved, &full_prompt, payload.temperature, payload.max_tokens)
            .await?;

        // Parse the response
        let code_files = self.parse_response(&text_result.text, &engine)?;

        // Save files and create job result
        let output_dir = PathBuf::from(&self.assets_dir)
            .join(&payload.project_id)
            .join("code")
            .join(job_id.into_uuid().to_string());

        fs::create_dir_all(&output_dir)
            .await
            .map_err(|e| AppError::io_error(format!("Failed to create output directory: {}", e)))?;

        let mut output_files = Vec::new();
        let mut metadata_files = Vec::new();

        for file in &code_files {
            let file_path = output_dir.join(&file.path);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| AppError::io_error(format!("Failed to create directory: {}", e)))?;
            }
            fs::write(&file_path, &file.content)
                .await
                .map_err(|e| AppError::io_error(format!("Failed to write file: {}", e)))?;
            output_files.push(file_path);
            metadata_files.push(serde_json::json!({
                "path": file.path,
                "language": file.language,
                "description": file.description,
            }));
        }

        let metadata = serde_json::json!({
            "operation": "code_generate",
            "engine": payload.engine,
            "template_id": payload.template_id,
            "model_id": payload.model_id,
            "tokens_used": text_result.tokens_used,
            "truncated": text_result.truncated,
            "summary": format!("Generated {} file(s)", code_files.len()),
            "files": metadata_files,
        });

        Ok(JobResult::with_metadata(output_files, metadata))
    }

    /// Builds the full prompt from system prompt, template, and user prompt.
    fn build_prompt(&self, engine: &CodeEngine, payload: &CodeGenOperation) -> Result<String, AppError> {
        let mut full_prompt = engine.system_prompt().to_string();
        full_prompt.push_str("\n\n");

        // If a template is specified, expand it
        if let Some(template_id) = &payload.template_id {
            let templates = built_in_templates();
            if let Some(template) = templates.iter().find(|t| t.id == *template_id) {
                // For now, just append the template prompt template
                // In a full implementation, we'd render the template with variables
                full_prompt.push_str(&format!("\nUse the following template as a guide:\n{}\n\n", template.prompt_template));
            }
        }

        // Append the user's prompt
        full_prompt.push_str(&format!("User request: {}", payload.prompt));

        Ok(full_prompt)
    }

    /// Calls the text provider to generate code.
    async fn call_text_provider(
        &self,
        resolved: &ResolvedTextProfile,
        prompt: &str,
        temperature: f32,
        max_tokens: u32,
    ) -> Result<TextResult, AppError> {
        let params = TextParams {
            prompt: prompt.to_string(),
            max_tokens,
            temperature,
            stop_sequences: None,
            stream: false,
        };

        let credential_id = format!("{}::api_key", resolved.profile.provider_name);
        let api_key = self.credential_store
            .get(&credential_id)
            .map_err(|_| AppError::internal(format!(
                "Credential not found for provider: {}",
                resolved.profile.provider_name
            )))?;

        resolved
            .provider
            .complete(&params, &api_key)
            .await
            .map_err(|e| AppError::internal(format!("Text provider error: {}", e)))
    }

    /// Parses the model's JSON response into code files.
    fn parse_response(
        &self,
        response: &str,
        _engine: &CodeEngine,
    ) -> Result<Vec<CodeFile>, AppError> {
        // Try to parse as JSON first
        if let Ok(parsed) = serde_json::from_str::<CodeGenResponse>(response) {
            // Check for empty files array
            if parsed.files.is_empty() {
                return Err(AppError::validation("Empty files array in response"));
            }
            return Ok(parsed.files.into_iter().map(|f| CodeFile {
                path: f.path,
                language: f.language,
                description: f.description,
                content: f.content,
            }).collect());
        }

        // Invalid JSON - return error per spec
        Err(AppError::validation("Model returned invalid JSON output"))
    }
}

impl JobWorker for CodeWorker {
    fn can_handle(&self, job_type: &str) -> bool {
        job_type == "code_generate"
    }

    fn process(&self, job: &Job) -> JobFuture {
        let assets_dir = self.assets_dir.clone();
        let model_router = self.model_router.clone();
        let credential_store = self.credential_store.clone();
        let job_id = job.id;
        let operation = job.operation.clone();

        Box::pin(async move {
            let payload: CodeGenOperation = serde_json::from_value(operation)
                .map_err(|e| AppError::validation(format!("Invalid code generation payload: {}", e)))?;

            tracing::info!(
                "CodeWorker processing job {} for project {}",
                job_id.into_uuid(),
                payload.project_id
            );

            let worker = CodeWorker::new(assets_dir, model_router, credential_store);
            worker.process_code_job(job_id, payload).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock repository for testing
    struct TestRepo;

    #[async_trait::async_trait]
    impl artifex_model_config::router::ModelConfigRepository for TestRepo {
        async fn find_profile(
            &self,
            _id: &uuid::Uuid,
        ) -> Result<Option<artifex_model_config::ModelProfile>, String> {
            Ok(None)
        }

        async fn find_rule(
            &self,
            _operation_type: &str,
        ) -> Result<Option<artifex_model_config::RoutingRule>, String> {
            Ok(None)
        }

        async fn list_enabled_profiles(
            &self,
            _capability: artifex_model_config::ModelCapability,
        ) -> Result<Vec<artifex_model_config::ModelProfile>, String> {
            Ok(vec![])
        }
    }

    #[test]
    fn test_can_handle() {
        let worker = CodeWorker::new(
            String::new(),
            Arc::new(ModelRouter::new(
                Arc::new(artifex_model_config::ProviderRegistry::new()),
                Arc::new(TestRepo),
                Arc::new(artifex_model_config::credential_store::InMemoryCredentialStore::new()),
            )),
            Arc::new(artifex_model_config::credential_store::InMemoryCredentialStore::new()),
        );
        assert!(worker.can_handle("code_generate"));
        assert!(!worker.can_handle("image_generate"));
        assert!(!worker.can_handle("audio_generate"));
    }

    #[test]
    fn test_code_gen_operation_deserialization() {
        let json = r#"{
            "projectId": "test-project",
            "engine": "godot",
            "prompt": "Create a player controller",
            "templateId": "godot_player_controller",
            "temperature": 0.3,
            "maxTokens": 2048
        }"#;

        let operation: CodeGenOperation = serde_json::from_str(json).unwrap();
        assert_eq!(operation.project_id, "test-project");
        assert_eq!(operation.engine, "godot");
        assert_eq!(operation.prompt, "Create a player controller");
        assert_eq!(operation.template_id, Some("godot_player_controller".to_string()));
        assert_eq!(operation.temperature, 0.3);
        assert_eq!(operation.max_tokens, 2048);
    }

    #[test]
    fn test_code_gen_operation_defaults() {
        let json = r#"{
            "projectId": "test-project",
            "engine": "unity",
            "prompt": "Create enemy AI"
        }"#;

        let operation: CodeGenOperation = serde_json::from_str(json).unwrap();
        assert_eq!(operation.temperature, 0.25);
        assert_eq!(operation.max_tokens, 4096);
        assert_eq!(operation.template_id, None);
    }

    #[test]
    fn test_parse_valid_json_response() {
        let _engine = CodeEngine::Godot;

        let response = r#"{
            "engine": "godot",
            "summary": "Player controller generated",
            "files": [
                {
                    "path": "PlayerController.gd",
                    "language": "gdscript",
                    "description": "Main player controller",
                    "content": "extends CharacterBody2D\n..."
                }
            ]
        }"#;

        // Test parsing directly via serde
        let parsed: CodeGenResponse = serde_json::from_str(response).unwrap();
        assert_eq!(parsed.files.len(), 1);
        assert_eq!(parsed.files[0].path, "PlayerController.gd");
        assert_eq!(parsed.files[0].language, "gdscript");
    }

    #[test]
    fn test_parse_invalid_json_fallback() {
        let _engine = CodeEngine::Unity;

        let response = "This is not JSON, just plain text code";

        // Should fail to parse as CodeGenResponse
        let result: Result<CodeGenResponse, _> = serde_json::from_str(response);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_multi_file_response() {
        let response = r#"{
            "engine": "unity",
            "summary": "Inventory system generated",
            "files": [
                {
                    "path": "Inventory/InventoryItem.cs",
                    "language": "csharp",
                    "description": "Item data class",
                    "content": "using UnityEngine;"
                },
                {
                    "path": "Inventory/InventoryManager.cs",
                    "language": "csharp",
                    "description": "Main inventory controller",
                    "content": "using System.Collections.Generic;"
                }
            ]
        }"#;

        let parsed: CodeGenResponse = serde_json::from_str(response).unwrap();
        assert_eq!(parsed.files.len(), 2);
        assert_eq!(parsed.files[0].path, "Inventory/InventoryItem.cs");
        assert_eq!(parsed.files[1].path, "Inventory/InventoryManager.cs");
    }

    #[test]
    fn test_engine_from_str() {
        assert_eq!(CodeEngine::from_str("godot"), Some(CodeEngine::Godot));
        assert_eq!(CodeEngine::from_str("unity"), Some(CodeEngine::Unity));
        assert_eq!(CodeEngine::from_str("GODOT"), Some(CodeEngine::Godot));
        assert_eq!(CodeEngine::from_str("Unknown"), None);
    }

    #[test]
    fn test_engine_properties() {
        let godot = CodeEngine::Godot;
        assert_eq!(godot.file_extension(), ".gd");
        assert_eq!(godot.language(), "GDScript");

        let unity = CodeEngine::Unity;
        assert_eq!(unity.file_extension(), ".cs");
        assert_eq!(unity.language(), "C#");
    }

    #[test]
    fn test_code_file_serialization() {
        let file = CodeFile {
            path: "PlayerController.gd".to_string(),
            language: "gdscript".to_string(),
            description: "Main player controller".to_string(),
            content: "extends CharacterBody2D".to_string(),
        };

        let json = serde_json::to_string(&file).unwrap();
        assert!(json.contains("PlayerController.gd"));
        assert!(json.contains("gdscript"));

        let deserialized: CodeFile = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.path, "PlayerController.gd");
        assert_eq!(deserialized.language, "gdscript");
    }
}
