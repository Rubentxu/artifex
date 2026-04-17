//! Code engine definitions for game code generation.
//!
//! Supports Godot (GDScript) and Unity (C#) game engines.

use serde::{Deserialize, Serialize};

/// Supported game engines for code generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CodeEngine {
    /// Godot 4 game engine (GDScript).
    Godot,
    /// Unity game engine (C#).
    Unity,
}

impl CodeEngine {
    /// Returns the file extension for this engine.
    pub fn file_extension(&self) -> &'static str {
        match self {
            CodeEngine::Godot => ".gd",
            CodeEngine::Unity => ".cs",
        }
    }

    /// Returns the language name for this engine.
    pub fn language(&self) -> &'static str {
        match self {
            CodeEngine::Godot => "GDScript",
            CodeEngine::Unity => "C#",
        }
    }

    /// Returns the system prompt for this engine.
    pub fn system_prompt(&self) -> &'static str {
        match self {
            CodeEngine::Godot => GODOT_SYSTEM_PROMPT,
            CodeEngine::Unity => UNITY_SYSTEM_PROMPT,
        }
    }

    /// Parses a string to CodeEngine.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "godot" => Some(CodeEngine::Godot),
            "unity" => Some(CodeEngine::Unity),
            _ => None,
        }
    }

    /// Returns the engine name as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            CodeEngine::Godot => "godot",
            CodeEngine::Unity => "unity",
        }
    }
}

impl std::fmt::Display for CodeEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeEngine::Godot => write!(f, "Godot"),
            CodeEngine::Unity => write!(f, "Unity"),
        }
    }
}

/// System prompt for Godot 4 GDScript code generation.
const GODOT_SYSTEM_PROMPT: &str = r#"You are an expert Godot 4 game developer specializing in GDScript.

Engine: Godot 4.x
Language: GDScript with GDScript 2.0 syntax

Key Godot 4 features to use:
- @export annotations for exported variables
- @onready for node references
- Signals for communication
- @tool for editor scripts
- super() calls using super keyword
- Pattern matching with match statements

Output format: You MUST respond with valid JSON in this exact structure:
{
  "engine": "godot",
  "summary": "Brief description of what was generated",
  "files": [
    {
      "path": "relative/path/to/file.gd",
      "language": "gdscript",
      "description": "What this file contains",
      "content": "The complete GDScript code"
    }
  ]
}

Rules:
- Output ONLY JSON, no markdown code fences, no explanations outside JSON
- Use Godot 4 best practices (preload, @export, signals)
- Include class_name when appropriate
- Use proper Node path references with $ or % syntax
- Write self-documenting code with clear structure"#;

/// System prompt for Unity C# code generation.
const UNITY_SYSTEM_PROMPT: &str = r#"You are an expert Unity game developer specializing in C#.

Engine: Unity 2023.x
Language: C# 11+

Key Unity features to use:
- [SerializeField] for private serialized fields
- [Header] and [Tooltip] attributes
- UnityEngine for core types
- MonoBehaviour lifecycle (Start, Update, FixedUpdate)
- [RequireComponent] for dependencies
- Coroutines for async operations

Output format: You MUST respond with valid JSON in this exact structure:
{
  "engine": "unity",
  "summary": "Brief description of what was generated",
  "files": [
    {
      "path": "relative/path/to/file.cs",
      "language": "csharp",
      "description": "What this file contains",
      "content": "The complete C# code"
    }
  ]
}

Rules:
- Output ONLY JSON, no markdown code fences, no explanations outside JSON
- Use Unity best practices (Awake, Start, Update pattern)
- Include proper namespaces
- Use [Serializable] for custom data classes
- Write self-documenting code with clear structure"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_engine_godot_properties() {
        let engine = CodeEngine::Godot;
        assert_eq!(engine.file_extension(), ".gd");
        assert_eq!(engine.language(), "GDScript");
        assert_eq!(engine.as_str(), "godot");
        assert_eq!(engine.to_string(), "Godot");
        assert!(engine.system_prompt().contains("Godot"));
    }

    #[test]
    fn test_code_engine_unity_properties() {
        let engine = CodeEngine::Unity;
        assert_eq!(engine.file_extension(), ".cs");
        assert_eq!(engine.language(), "C#");
        assert_eq!(engine.as_str(), "unity");
        assert_eq!(engine.to_string(), "Unity");
        assert!(engine.system_prompt().contains("Unity"));
    }

    #[test]
    fn test_code_engine_from_str() {
        assert_eq!(CodeEngine::from_str("godot"), Some(CodeEngine::Godot));
        assert_eq!(CodeEngine::from_str("GODOT"), Some(CodeEngine::Godot));
        assert_eq!(CodeEngine::from_str("Godot"), Some(CodeEngine::Godot));
        assert_eq!(CodeEngine::from_str("unity"), Some(CodeEngine::Unity));
        assert_eq!(CodeEngine::from_str("UNITY"), Some(CodeEngine::Unity));
        assert_eq!(CodeEngine::from_str("Unity"), Some(CodeEngine::Unity));
        assert_eq!(CodeEngine::from_str("unknown"), None);
    }

    #[test]
    fn test_code_engine_serialization() {
        let godot = CodeEngine::Godot;
        let unity = CodeEngine::Unity;

        let godot_json = serde_json::to_string(&godot).unwrap();
        let unity_json = serde_json::to_string(&unity).unwrap();

        assert_eq!(godot_json, "\"godot\"");
        assert_eq!(unity_json, "\"unity\"");

        let godot_deser: CodeEngine = serde_json::from_str("\"godot\"").unwrap();
        let unity_deser: CodeEngine = serde_json::from_str("\"unity\"").unwrap();

        assert_eq!(godot_deser, CodeEngine::Godot);
        assert_eq!(unity_deser, CodeEngine::Unity);
    }

    #[test]
    fn test_system_prompt_contains_json_format() {
        let godot_prompt = CodeEngine::Godot.system_prompt();
        assert!(godot_prompt.contains("\"engine\":"));
        assert!(godot_prompt.contains("\"files\":"));

        let unity_prompt = CodeEngine::Unity.system_prompt();
        assert!(unity_prompt.contains("\"engine\":"));
        assert!(unity_prompt.contains("\"files\":"));
    }
}
