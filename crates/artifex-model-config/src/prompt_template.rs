//! Prompt template entity.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents a reusable prompt template with variable placeholders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    /// Unique identifier.
    pub id: Uuid,
    /// Unique template name.
    pub name: String,
    /// The template text with `{{variable}}` placeholders.
    pub template_text: String,
    /// List of variable names used in this template.
    pub variables: Vec<String>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
}

impl PromptTemplate {
    /// Creates a new prompt template.
    pub fn new(name: String, template_text: String) -> Self {
        let now = Utc::now();
        let variables = Self::extract_variables(&template_text);
        Self {
            id: Uuid::new_v4(),
            name,
            template_text,
            variables,
            created_at: now,
            updated_at: now,
        }
    }

    /// Extracts variable names from the template text.
    /// Variables are in the format `{{variable_name}}`.
    fn extract_variables(template_text: &str) -> Vec<String> {
        let mut variables = Vec::new();
        let mut chars = template_text.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' && chars.peek() == Some(&'{') {
                chars.next(); // consume second '{'
                let mut var_name = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '}' {
                        chars.next(); // consume '}'
                        break;
                    }
                    var_name.push(c);
                    chars.next();
                }
                if !var_name.is_empty() && !variables.contains(&var_name) {
                    variables.push(var_name);
                }
            }
        }

        variables
    }

    /// Renders the template with the given variables.
    ///
    /// # Errors
    /// Returns an error if a required variable is missing.
    pub fn render(&self, vars: &HashMap<String, String>) -> Result<String, String> {
        let mut result = self.template_text.clone();
        for var in &self.variables {
            let placeholder = format!("{{{{{}}}}}", var);
            let value = vars
                .get(var)
                .ok_or_else(|| format!("Missing variable: {}", var))?;
            result = result.replace(&placeholder, value);
        }
        Ok(result)
    }

    /// Validates the template.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.template_text.is_empty() {
            return Err("Template text cannot be empty".to_string());
        }
        // Check for unclosed variables
        let open_count = self.template_text.matches("{{").count();
        let close_count = self.template_text.matches("}}").count();
        if open_count != close_count {
            return Err("Unclosed variable placeholder".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_variables() {
        let text = "Hello, {{name}}! You have {{count}} messages.";
        let vars = PromptTemplate::extract_variables(text);
        assert_eq!(vars, vec!["name", "count"]);
    }

    #[test]
    fn test_extract_variables_no_duplicates() {
        let text = "{{name}} and {{name}} again";
        let vars = PromptTemplate::extract_variables(text);
        assert_eq!(vars, vec!["name"]);
    }

    #[test]
    fn test_extract_variables_empty() {
        let text = "No variables here";
        let vars = PromptTemplate::extract_variables(text);
        assert!(vars.is_empty());
    }

    #[test]
    fn test_prompt_template_render() {
        let template = PromptTemplate::new("greeting".to_string(), "Hello, {{name}}!".to_string());

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());

        let result = template.render(&vars).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_prompt_template_render_multiple() {
        let template = PromptTemplate::new(
            "story".to_string(),
            "{{character}} went to {{place}} and saw {{thing}}".to_string(),
        );

        let mut vars = HashMap::new();
        vars.insert("character".to_string(), "Alice".to_string());
        vars.insert("place".to_string(), "the forest".to_string());
        vars.insert("thing".to_string(), "a rabbit".to_string());

        let result = template.render(&vars).unwrap();
        assert_eq!(result, "Alice went to the forest and saw a rabbit");
    }

    #[test]
    fn test_prompt_template_render_missing_variable() {
        let template = PromptTemplate::new("greeting".to_string(), "Hello, {{name}}!".to_string());

        let vars = HashMap::new();
        let result = template.render(&vars);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing variable"));
    }

    #[test]
    fn test_prompt_template_validate_empty_name() {
        let template = PromptTemplate {
            id: Uuid::new_v4(),
            name: "".to_string(),
            template_text: "Hello".to_string(),
            variables: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(template.validate().is_err());
    }

    #[test]
    fn test_prompt_template_validate_unclosed_placeholder() {
        let template = PromptTemplate::new("test".to_string(), "Hello, {{name".to_string());
        assert!(template.validate().is_err());
        assert!(template.validate().unwrap_err().contains("Unclosed"));
    }

    #[test]
    fn test_prompt_template_new() {
        let template = PromptTemplate::new("test".to_string(), "{{var1}} and {{var2}}".to_string());
        assert!(!template.id.is_nil());
        assert_eq!(template.name, "test");
        assert_eq!(template.variables, vec!["var1", "var2"]);
    }
}
