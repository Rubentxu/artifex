//! Agent code worker for multi-step code generation.
//!
//! Implements a state machine-based code agent that processes jobs through
//! Plan → Execute → Verify → Refine → Done/Failed phases.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use artifex_asset_management::CodeEngine;
use artifex_job_queue::Job;
use artifex_model_config::chat_provider::{ChatMessage, ChatParams, ChatRole, ChatResult};
use artifex_model_config::credential_store::CredentialStore;
use artifex_model_config::{ModelRouter, ResolvedTextProfile, TextParams};
use artifex_shared_kernel::AppError;
use tokio::fs;

use crate::workers::traits::{JobFuture, JobResult, JobWorker};

/// Maximum number of refine iterations before failing.
const MAX_REFINE_ITERATIONS: u8 = 2;

/// Phases of the code agent state machine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum AgentPhase {
    /// Planning phase - analyzing the request and creating a plan.
    Plan,
    /// Execution phase - generating code based on the plan.
    Execute,
    /// Verification phase - checking the generated code.
    Verify,
    /// Refinement phase - improving code based on feedback.
    Refine { iteration: u8 },
    /// Successful completion.
    Done,
    /// Failed completion with reason.
    Failed { reason: String },
}

impl AgentPhase {
    /// Returns true if this is a terminal phase.
    pub fn is_terminal(&self) -> bool {
        matches!(self, AgentPhase::Done | AgentPhase::Failed { .. })
    }

    /// Returns the progress percentage range for this phase.
    pub fn progress_range(&self) -> (u8, u8) {
        match self {
            AgentPhase::Plan => (10, 25),
            AgentPhase::Execute => (25, 75),
            AgentPhase::Verify => (75, 90),
            AgentPhase::Refine { .. } => (90, 99),
            AgentPhase::Done => (100, 100),
            AgentPhase::Failed { .. } => (0, 0),
        }
    }

    /// Returns a human-readable name for this phase.
    pub fn name(&self) -> &'static str {
        match self {
            AgentPhase::Plan => "Planning",
            AgentPhase::Execute => "Executing",
            AgentPhase::Verify => "Verifying",
            AgentPhase::Refine { .. } => "Refining",
            AgentPhase::Done => "Done",
            AgentPhase::Failed { .. } => "Failed",
        }
    }
}

/// State of the code agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentState {
    /// Current phase of the agent.
    pub phase: AgentPhase,
    /// The generated plan (in Plan phase) or execution plan.
    pub plan: Option<String>,
    /// Generated code files.
    pub code_files: Vec<CodeFile>,
    /// Feedback from verification.
    pub verify_feedback: Option<String>,
    /// Number of refine iterations performed.
    pub refine_iterations: u8,
    /// Chat history for context.
    pub messages: Vec<ChatMessage>,
}

impl AgentState {
    /// Creates a new AgentState with the initial prompt.
    pub fn new(initial_prompt: String) -> Self {
        Self {
            phase: AgentPhase::Plan,
            plan: None,
            code_files: Vec::new(),
            verify_feedback: None,
            refine_iterations: 0,
            messages: vec![ChatMessage::user(initial_prompt)],
        }
    }

    /// Transitions to a new phase with validation.
    ///
    /// Returns an error if the transition is not allowed.
    pub fn transition_to(&mut self, next_phase: AgentPhase) -> Result<(), String> {
        // Validate transition
        match (&self.phase, &next_phase) {
            // Plan can go to Execute or Failed
            (AgentPhase::Plan, AgentPhase::Execute) => {}
            (AgentPhase::Plan, AgentPhase::Failed { .. }) => {}

            // Execute can go to Verify or Failed
            (AgentPhase::Execute, AgentPhase::Verify) => {}
            (AgentPhase::Execute, AgentPhase::Failed { .. }) => {}

            // Verify can go to Refine, Done, or Failed
            (AgentPhase::Verify, AgentPhase::Refine { iteration }) => {
                // Check refine cap
                if self.refine_iterations >= MAX_REFINE_ITERATIONS {
                    return Err(format!(
                        "Cannot transition to Refine: already at max iterations ({})",
                        MAX_REFINE_ITERATIONS
                    ));
                }
                if *iteration != self.refine_iterations + 1 {
                    return Err(format!(
                        "Invalid refine iteration: expected {}, got {}",
                        self.refine_iterations + 1,
                        iteration
                    ));
                }
            }
            (AgentPhase::Verify, AgentPhase::Done) => {}
            (AgentPhase::Verify, AgentPhase::Failed { .. }) => {}

            // Refine can go back to Verify or Failed
            (AgentPhase::Refine { .. }, AgentPhase::Verify) => {}
            (AgentPhase::Refine { .. }, AgentPhase::Failed { .. }) => {}

            // Terminal states don't transition
            (AgentPhase::Done, _) => {
                return Err("Cannot transition from Done".to_string());
            }
            (AgentPhase::Failed { .. }, _) => {
                return Err("Cannot transition from Failed".to_string());
            }

            // Invalid transitions
            _ => {
                return Err(format!(
                    "Invalid transition from {:?} to {:?}",
                    self.phase, next_phase
                ));
            }
        }

        self.phase = next_phase;
        Ok(())
    }

    /// Adds a message to the chat history.
    pub fn add_message(&mut self, role: ChatRole, content: String) {
        self.messages.push(ChatMessage { role, content });
    }

    /// Increments the refine iteration counter.
    pub fn increment_refine(&mut self) -> Result<(), String> {
        if self.refine_iterations >= MAX_REFINE_ITERATIONS {
            return Err("Already at max refine iterations".to_string());
        }
        self.refine_iterations += 1;
        Ok(())
    }
}

/// A generated code file.
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

/// Payload for code agent jobs.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeAgentOperation {
    /// The project ID.
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

/// Worker for code agent jobs.
pub struct AgentCodeWorker {
    /// Base directory for saving output assets.
    assets_dir: String,
    /// Model router for resolving text providers.
    model_router: Arc<ModelRouter>,
    /// Credential store for API keys.
    credential_store: Arc<dyn CredentialStore>,
}

impl AgentCodeWorker {
    /// Creates a new AgentCodeWorker.
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

    /// Processes a code agent job through its state machine.
    async fn process_agent_job(
        &self,
        job_id: artifex_shared_kernel::JobId,
        payload: CodeAgentOperation,
    ) -> Result<JobResult, AppError> {
        let engine = CodeEngine::from_str(&payload.engine)
            .ok_or_else(|| AppError::validation(format!("Unsupported engine: {}", payload.engine)))?;

        // Initialize agent state
        let mut state = AgentState::new(payload.prompt.clone());

        // Resolve the chat provider
        let resolved = self.resolve_chat_provider(&payload).await?;

        // Execute the state machine
        loop {
            let phase_name = state.phase.name().to_string();
            tracing::info!("Agent phase: {} ({})", phase_name, job_id.into_uuid());

            match &state.phase {
                AgentPhase::Plan => {
                    // Build plan prompt
                    let system_prompt = format!(
                        "You are an expert {} code developer. Analyze the user's request and create a detailed plan for implementing the code. Output your plan as a numbered list of steps.",
                        engine
                    );
                    state.add_message(ChatRole::System, system_prompt);

                    // Call the model
                    let response = self.call_chat_provider(&resolved, &state.messages).await?;
                    state.add_message(ChatRole::Assistant, response.content.clone());
                    state.plan = Some(response.content);

                    // Transition to Execute
                    state.transition_to(AgentPhase::Execute).map_err(|e| {
                        AppError::internal(format!("Phase transition error: {}", e))
                    })?;
                }
                AgentPhase::Execute => {
                    // Build execution prompt with plan context
                    let plan = state.plan.as_deref().unwrap_or("");
                    let system_prompt = format!(
                        "You are an expert {} code developer. Based on the following plan, generate the complete code implementation.\n\nPlan:\n{}\n\nOutput ONLY valid JSON in this exact format:\n{{\"engine\":\"{}\",\"summary\":\"...\",\"files\":[{{\"path\":\"...\",\"language\":\"...\",\"description\":\"...\",\"content\":\"...\"}}]}}",
                        engine, plan, engine
                    );
                    state.add_message(ChatRole::System, system_prompt);

                    // Call the model
                    let response = self.call_chat_provider(&resolved, &state.messages).await?;
                    state.add_message(ChatRole::Assistant, response.content.clone());

                    // Parse the response
                    if let Ok(code_response) = serde_json::from_str::<CodeGenResponse>(&response.content) {
                        state.code_files = code_response.files.into_iter().map(|f| CodeFile {
                            path: f.path,
                            language: f.language,
                            description: f.description,
                            content: f.content,
                        }).collect();
                    }

                    // Transition to Verify
                    state.transition_to(AgentPhase::Verify).map_err(|e| {
                        AppError::internal(format!("Phase transition error: {}", e))
                    })?;
                }
                AgentPhase::Verify => {
                    // Build verification prompt
                    let files_summary = state.code_files
                        .iter()
                        .map(|f| format!("{} ({})", f.path, f.language))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let system_prompt = format!(
                        "You are an expert {} code developer. Review the generated code files [{}] for correctness, best practices, and potential issues. Provide feedback in JSON format: {{\"issues\":[\"...\"],\"score\":0-10}}.",
                        engine, files_summary
                    );
                    state.add_message(ChatRole::System, system_prompt);

                    // Call the model
                    let response = self.call_chat_provider(&resolved, &state.messages).await?;
                    let content_lower = response.content.to_lowercase();

                    // Check if we need refinement (simple heuristic: look for "issues" in response)
                    let needs_refine = content_lower.contains("\"issues\":")
                        && !response.content.contains("\"issues\":[]");

                    state.add_message(ChatRole::Assistant, response.content.clone());
                    state.verify_feedback = Some(response.content);

                    if needs_refine {
                        // Increment refine counter
                        let new_iteration = state.refine_iterations + 1;
                        state.increment_refine().map_err(|e| {
                            AppError::internal(format!("Refine iteration error: {}", e))
                        })?;
                        state.transition_to(AgentPhase::Refine { iteration: new_iteration }).map_err(|e| {
                            AppError::internal(format!("Phase transition error: {}", e))
                        })?;
                    } else {
                        // Transition to Done
                        state.transition_to(AgentPhase::Done).map_err(|e| {
                            AppError::internal(format!("Phase transition error: {}", e))
                        })?;
                    }
                }
                AgentPhase::Refine { .. } => {
                    // Build refinement prompt with verification feedback
                    let feedback = state.verify_feedback.as_deref().unwrap_or("");
                    let system_prompt = format!(
                        "You are an expert {} code developer. Based on the following verification feedback, improve the generated code:\n\nFeedback:\n{}\n\nOutput ONLY valid JSON in the same format as before.",
                        engine, feedback
                    );
                    state.add_message(ChatRole::System, system_prompt);

                    // Call the model
                    let response = self.call_chat_provider(&resolved, &state.messages).await?;
                    state.add_message(ChatRole::Assistant, response.content.clone());

                    // Parse the response
                    if let Ok(code_response) = serde_json::from_str::<CodeGenResponse>(&response.content) {
                        state.code_files = code_response.files.into_iter().map(|f| CodeFile {
                            path: f.path,
                            language: f.language,
                            description: f.description,
                            content: f.content,
                        }).collect();
                    }

                    // Transition back to Verify
                    state.transition_to(AgentPhase::Verify).map_err(|e| {
                        AppError::internal(format!("Phase transition error: {}", e))
                    })?;
                }
                AgentPhase::Done | AgentPhase::Failed { .. } => {
                    // Terminal phase - exit the loop
                    break;
                }
            }
        }

        // Save files and create job result
        let output_dir = std::path::PathBuf::from(&self.assets_dir)
            .join(&payload.project_id)
            .join("code")
            .join(job_id.into_uuid().to_string());

        fs::create_dir_all(&output_dir)
            .await
            .map_err(|e| AppError::io_error(format!("Failed to create output directory: {}", e)))?;

        let mut output_files = Vec::new();
        let mut metadata_files = Vec::new();

        for file in &state.code_files {
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
            "operation": "code_agent",
            "engine": payload.engine,
            "model_id": payload.model_id,
            "phase": state.phase.name(),
            "refine_iterations": state.refine_iterations,
            "summary": format!("Generated {} file(s)", state.code_files.len()),
            "files": metadata_files,
        });

        Ok(JobResult::with_metadata(output_files, metadata))
    }

    /// Resolves the chat provider for the job.
    async fn resolve_chat_provider(
        &self,
        payload: &CodeAgentOperation,
    ) -> Result<ResolvedTextProfile, AppError> {
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
            })
    }

    /// Calls the chat provider with the given messages.
    async fn call_chat_provider(
        &self,
        resolved: &ResolvedTextProfile,
        messages: &[ChatMessage],
    ) -> Result<ChatResult, AppError> {
        let params = ChatParams {
            messages: messages.to_vec(),
            max_tokens: 4096,
            temperature: 0.25,
            stop_sequences: None,
        };

        let credential_id = format!("{}::api_key", resolved.profile.provider_name);
        let api_key = self.credential_store
            .get(&credential_id)
            .map_err(|_| AppError::internal(format!(
                "Credential not found for provider: {}",
                resolved.profile.provider_name
            )))?;

        // Note: We're using TextProvider here because the resolved profile is for text
        // In a full implementation, we'd have a ResolvedChatProfile
        let text_params = TextParams {
            prompt: messages.last().map(|m| m.content.clone()).unwrap_or_default(),
            max_tokens: params.max_tokens,
            temperature: params.temperature,
            stop_sequences: params.stop_sequences,
            stream: false,
        };

        resolved
            .provider
            .complete(&text_params, &api_key)
            .await
            .map_err(|e| AppError::internal(format!("Text provider error: {}", e)))
            .map(|r| ChatResult::new(r.text, r.tokens_used, r.truncated))
    }
}

impl JobWorker for AgentCodeWorker {
    fn can_handle(&self, job_type: &str) -> bool {
        job_type == "code_agent"
    }

    fn process(&self, job: &Job) -> JobFuture {
        let assets_dir = self.assets_dir.clone();
        let model_router = self.model_router.clone();
        let credential_store = self.credential_store.clone();
        let job_id = job.id;
        let operation = job.operation.clone();

        Box::pin(async move {
            let payload: CodeAgentOperation = serde_json::from_value(operation)
                .map_err(|e| AppError::validation(format!("Invalid code agent payload: {}", e)))?;

            tracing::info!(
                "AgentCodeWorker processing job {} for project {}",
                job_id.into_uuid(),
                payload.project_id
            );

            let worker = AgentCodeWorker::new(assets_dir, model_router, credential_store);
            worker.process_agent_job(job_id, payload).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_phase_progress_range() {
        assert_eq!(AgentPhase::Plan.progress_range(), (10, 25));
        assert_eq!(AgentPhase::Execute.progress_range(), (25, 75));
        assert_eq!(AgentPhase::Verify.progress_range(), (75, 90));
        assert_eq!(AgentPhase::Refine { iteration: 1 }.progress_range(), (90, 99));
        assert_eq!(AgentPhase::Done.progress_range(), (100, 100));
        assert_eq!(AgentPhase::Failed { reason: "err".to_string() }.progress_range(), (0, 0));
    }

    #[test]
    fn test_agent_phase_name() {
        assert_eq!(AgentPhase::Plan.name(), "Planning");
        assert_eq!(AgentPhase::Execute.name(), "Executing");
        assert_eq!(AgentPhase::Verify.name(), "Verifying");
        assert_eq!(AgentPhase::Refine { iteration: 1 }.name(), "Refining");
        assert_eq!(AgentPhase::Done.name(), "Done");
        assert_eq!(AgentPhase::Failed { reason: "err".to_string() }.name(), "Failed");
    }

    #[test]
    fn test_agent_phase_is_terminal() {
        assert!(!AgentPhase::Plan.is_terminal());
        assert!(!AgentPhase::Execute.is_terminal());
        assert!(!AgentPhase::Verify.is_terminal());
        assert!(!AgentPhase::Refine { iteration: 1 }.is_terminal());
        assert!(AgentPhase::Done.is_terminal());
        assert!(AgentPhase::Failed { reason: "err".to_string() }.is_terminal());
    }

    #[test]
    fn test_agent_state_new() {
        let state = AgentState::new("Create a player controller".to_string());
        assert_eq!(state.phase, AgentPhase::Plan);
        assert!(state.plan.is_none());
        assert!(state.code_files.is_empty());
        assert_eq!(state.refine_iterations, 0);
        assert_eq!(state.messages.len(), 1);
        assert_eq!(state.messages[0].role, ChatRole::User);
        assert_eq!(state.messages[0].content, "Create a player controller");
    }

    #[test]
    fn test_agent_state_transition_valid() {
        let mut state = AgentState::new("Test".to_string());

        // Plan -> Execute
        assert!(state.transition_to(AgentPhase::Execute).is_ok());
        assert_eq!(state.phase, AgentPhase::Execute);

        // Execute -> Verify
        assert!(state.transition_to(AgentPhase::Verify).is_ok());
        assert_eq!(state.phase, AgentPhase::Verify);

        // Verify -> Done
        assert!(state.transition_to(AgentPhase::Done).is_ok());
        assert_eq!(state.phase, AgentPhase::Done);
    }

    #[test]
    fn test_agent_state_transition_invalid() {
        let mut state = AgentState::new("Test".to_string());

        // Plan -> Done is invalid
        assert!(state.transition_to(AgentPhase::Done).is_err());

        // Plan -> Verify is invalid
        assert!(state.transition_to(AgentPhase::Verify).is_err());
    }

    #[test]
    fn test_agent_state_refine_cap() {
        let mut state = AgentState::new("Test".to_string());

        // Progress to Verify
        state.transition_to(AgentPhase::Execute).unwrap();
        state.transition_to(AgentPhase::Verify).unwrap();

        // First refine should work
        assert!(state.transition_to(AgentPhase::Refine { iteration: 1 }).is_ok());
        state.increment_refine().unwrap();
        state.transition_to(AgentPhase::Verify).unwrap();

        // Second refine should work
        assert!(state.transition_to(AgentPhase::Refine { iteration: 2 }).is_ok());
        state.increment_refine().unwrap();
        state.transition_to(AgentPhase::Verify).unwrap();

        // Third refine should fail (cap reached)
        assert!(state.transition_to(AgentPhase::Refine { iteration: 3 }).is_err());
    }

    #[test]
    fn test_agent_state_cannot_transition_from_terminal() {
        let mut state = AgentState::new("Test".to_string());

        // Progress to Done via valid path
        state.transition_to(AgentPhase::Execute).unwrap();
        state.transition_to(AgentPhase::Verify).unwrap();
        state.transition_to(AgentPhase::Done).unwrap();

        // Now verify we can't transition from Done
        assert!(state.transition_to(AgentPhase::Plan).is_err());

        let mut state2 = AgentState::new("Test".to_string());
        state2.transition_to(AgentPhase::Execute).unwrap();
        state2.transition_to(AgentPhase::Verify).unwrap();
        state2.transition_to(AgentPhase::Failed { reason: "test".to_string() }).unwrap();
        assert!(state2.transition_to(AgentPhase::Plan).is_err());
    }

    #[test]
    fn test_agent_state_add_message() {
        let mut state = AgentState::new("Test".to_string());
        state.add_message(ChatRole::System, "You are helpful".to_string());
        assert_eq!(state.messages.len(), 2);
        assert_eq!(state.messages[1].role, ChatRole::System);
        assert_eq!(state.messages[1].content, "You are helpful");
    }

    #[test]
    fn test_agent_state_serialization() {
        let mut state = AgentState::new("Create a player".to_string());
        state.plan = Some("Step 1: Create player node".to_string());

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: AgentState = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.phase, AgentPhase::Plan);
        assert_eq!(deserialized.plan, Some("Step 1: Create player node".to_string()));
        assert_eq!(deserialized.messages.len(), 1);
    }

    #[test]
    fn test_agent_phase_serialization() {
        let phase = AgentPhase::Refine { iteration: 1 };
        let json = serde_json::to_string(&phase).unwrap();
        assert!(json.contains("Refine"));

        let deserialized: AgentPhase = serde_json::from_str(&json).unwrap();
        match deserialized {
            AgentPhase::Refine { iteration } => assert_eq!(iteration, 1),
            _ => panic!("Expected Refine phase"),
        }
    }

    #[test]
    fn test_code_agent_operation_deserialization() {
        let json = r#"{
            "projectId": "test-project",
            "engine": "godot",
            "prompt": "Create a player controller",
            "modelId": "custom-model",
            "maxDurationSecs": 300
        }"#;

        let operation: CodeAgentOperation = serde_json::from_str(json).unwrap();
        assert_eq!(operation.project_id, "test-project");
        assert_eq!(operation.engine, "godot");
        assert_eq!(operation.prompt, "Create a player controller");
        assert_eq!(operation.model_id, Some("custom-model".to_string()));
        assert_eq!(operation.max_duration_secs, Some(300));
    }
}
