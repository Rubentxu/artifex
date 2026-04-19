//! Code generation DTOs.

use serde::{Deserialize, Serialize};

/// Request type for generating code/scripts.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateCodeRequest {
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
    #[serde(default = "default_code_temperature")]
    pub temperature: f32,
    /// Maximum tokens to generate.
    #[serde(default = "default_code_max_tokens")]
    pub max_tokens: u32,
}

fn default_code_temperature() -> f32 {
    0.25
}

fn default_code_max_tokens() -> u32 {
    4096
}

/// Request type for the multi-step code agent.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeAgentRequest {
    pub project_id: String,
    /// Target engine ("godot" or "unity").
    pub engine: String,
    /// User's prompt describing what code to generate.
    pub prompt: String,
    /// Optional specific model ID to use.
    #[serde(default)]
    pub model_id: Option<String>,
    /// Maximum duration in seconds.
    #[serde(default)]
    pub max_duration_secs: Option<u64>,
}

/// Progress update from the code agent.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStepProgress {
    /// Current phase of the agent.
    pub phase: String,
    /// Human-readable step name.
    pub step_name: String,
    /// Progress percentage (0-100).
    pub percent: u8,
}
