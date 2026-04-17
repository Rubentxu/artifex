//! Code template listing IPC command.

use crate::model_config::built_in_templates::{templates_for_engine, BuiltInTemplate};

/// DTO for a code template.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeTemplateDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub engine: String,
    pub variables: Vec<String>,
}

impl From<BuiltInTemplate> for CodeTemplateDto {
    fn from(t: BuiltInTemplate) -> Self {
        Self {
            id: t.id,
            name: t.name,
            description: t.description,
            engine: t.engine,
            variables: t.variables,
        }
    }
}

/// Lists all code templates for a given engine.
#[tauri::command]
pub fn list_code_templates(engine: String) -> Result<Vec<CodeTemplateDto>, String> {
    let templates = templates_for_engine(&engine.to_lowercase());
    Ok(templates.into_iter().map(CodeTemplateDto::from).collect())
}
