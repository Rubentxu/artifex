//! Chat provider trait and types for conversation-based completions.

use serde::{Deserialize, Serialize};

use super::provider::{ProviderError, ProviderMetadata};
use super::text_provider::{TextParams, TextResult, TextProvider};

/// Role of the message sender in a chat conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    /// System message providing instructions or context.
    System,
    /// User message from the human participant.
    User,
    /// Assistant message from the AI model.
    Assistant,
}

impl ChatRole {
    /// Converts the role to a string for API serialization.
    pub fn as_str(&self) -> &'static str {
        match self {
            ChatRole::System => "system",
            ChatRole::User => "user",
            ChatRole::Assistant => "assistant",
        }
    }
}

/// A single message in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    /// The role of the message sender.
    pub role: ChatRole,
    /// The content of the message.
    pub content: String,
}

impl ChatMessage {
    /// Creates a new system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::System,
            content: content.into(),
        }
    }

    /// Creates a new user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
        }
    }

    /// Creates a new assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content.into(),
        }
    }
}

/// Parameters for chat completion requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatParams {
    /// The messages to send for completion.
    pub messages: Vec<ChatMessage>,
    /// Maximum number of tokens to generate.
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    /// Temperature for generation (0.0 to 2.0).
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// Optional stop sequences.
    #[serde(default)]
    pub stop_sequences: Option<Vec<String>>,
}

fn default_max_tokens() -> u32 {
    1024
}

fn default_temperature() -> f32 {
    0.7
}

impl ChatParams {
    /// Validates the chat parameters.
    pub fn validate(&self) -> Result<(), String> {
        if self.messages.is_empty() {
            return Err("Messages cannot be empty".to_string());
        }
        if self.max_tokens == 0 {
            return Err("Max tokens must be greater than 0".to_string());
        }
        if self.temperature < 0.0 || self.temperature > 2.0 {
            return Err("Temperature must be between 0.0 and 2.0".to_string());
        }
        Ok(())
    }
}

/// Provider trait for chat-based completion services.
///
/// This extends TextProvider with conversation history support.
#[async_trait::async_trait]
pub trait ChatProvider: Send + Sync {
    /// Generates a chat completion based on the given messages.
    async fn complete_chat(
        &self,
        params: &ChatParams,
        api_key: &str,
    ) -> Result<ChatResult, ProviderError>;

    /// Returns the provider metadata.
    fn metadata(&self) -> &ProviderMetadata;
}

/// Result of chat completion.
#[derive(Debug, Clone)]
pub struct ChatResult {
    /// Generated content from the model.
    pub content: String,
    /// Number of tokens in the completion.
    pub tokens_used: u32,
    /// Whether the response was truncated.
    pub truncated: bool,
}

impl ChatResult {
    /// Creates a new ChatResult.
    pub fn new(content: String, tokens_used: u32, truncated: bool) -> Self {
        Self {
            content,
            tokens_used,
            truncated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_role_as_str() {
        assert_eq!(ChatRole::System.as_str(), "system");
        assert_eq!(ChatRole::User.as_str(), "user");
        assert_eq!(ChatRole::Assistant.as_str(), "assistant");
    }

    #[test]
    fn test_chat_message_system() {
        let msg = ChatMessage::system("You are helpful");
        assert_eq!(msg.role, ChatRole::System);
        assert_eq!(msg.content, "You are helpful");
    }

    #[test]
    fn test_chat_message_user() {
        let msg = ChatMessage::user("Hello, world!");
        assert_eq!(msg.role, ChatRole::User);
        assert_eq!(msg.content, "Hello, world!");
    }

    #[test]
    fn test_chat_message_assistant() {
        let msg = ChatMessage::assistant("I can help with that");
        assert_eq!(msg.role, ChatRole::Assistant);
        assert_eq!(msg.content, "I can help with that");
    }

    #[test]
    fn test_chat_params_validate_valid() {
        let params = ChatParams {
            messages: vec![
                ChatMessage::system("You are a helpful assistant"),
                ChatMessage::user("Hello"),
            ],
            max_tokens: 100,
            temperature: 0.7,
            stop_sequences: None,
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_chat_params_validate_empty_messages() {
        let params = ChatParams {
            messages: vec![],
            max_tokens: 100,
            temperature: 0.7,
            stop_sequences: None,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn test_chat_params_validate_invalid_temperature() {
        let params = ChatParams {
            messages: vec![ChatMessage::user("Hello")],
            max_tokens: 100,
            temperature: -0.1,
            stop_sequences: None,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn test_chat_result_new() {
        let result = ChatResult::new("Hello, world!".to_string(), 50, false);
        assert_eq!(result.content, "Hello, world!");
        assert_eq!(result.tokens_used, 50);
        assert!(!result.truncated);
    }

    #[test]
    fn test_chat_message_serialization() {
        let msg = ChatMessage::user("Hello");
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("user"));
        assert!(json.contains("Hello"));

        let deserialized: ChatMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.role, ChatRole::User);
        assert_eq!(deserialized.content, "Hello");
    }
}
