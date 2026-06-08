// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/engine/error_preamble.md#source
// CODEGEN-BEGIN
//! Template engine error types

use std::path::PathBuf;
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/engine/error_types.md#schema
// CODEGEN-BEGIN
/// Template engine errors.
/// @spec projects/agentic-workflow/tech-design/core/generate/engine/error_types.md#schema
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Template not found: {0}")]
    NotFound(String),
    #[error("Template parse error in '{template}': {message}")]
    ParseError { template: String, message: String },
    #[error("Template render error in '{template}': {message}")]
    RenderError { template: String, message: String },
    #[error("Context type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
    #[error("Filter error in '{filter}': {message}")]
    FilterError { filter: String, message: String },
    #[error("Template directory not found: {0}")]
    DirectoryNotFound(PathBuf),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/engine/error_tera_from.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/generate/engine/error_tera_from.md#source
impl From<tera::Error> for TemplateError {
    fn from(e: tera::Error) -> Self {
        let msg = e.to_string();
        if msg.contains("not found") {
            TemplateError::NotFound(msg)
        } else if msg.contains("Failed to parse") {
            TemplateError::ParseError {
                template: "unknown".to_string(),
                message: msg,
            }
        } else {
            TemplateError::RenderError {
                template: "unknown".to_string(),
                message: msg,
            }
        }
    }
}
// CODEGEN-END
