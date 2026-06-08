// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/knowledge.md#source
// CODEGEN-BEGIN
//! Main Spec MCP Tools
//!
//! Tool for writing specs to .aw/tech-design/.
//! Previously included knowledge read/write/list tools — those have been
//! removed as the knowledge concept was merged into specs.

use super::{get_required_string, ToolDefinition};
use crate::shared::workspace;
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

/// Get the tool definition for write_main_spec
/// @spec projects/agentic-workflow/tech-design/core/tools/knowledge.md#source
pub fn write_main_spec_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_write_main_spec".to_string(),
        description:
            "Write or update a spec in the main .aw/tech-design/ directory (for archive merge)"
                .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "path", "content"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path (use $PWD for current directory)"
                },
                "path": {
                    "type": "string",
                    "description": "Path relative to .aw/tech-design/, e.g. 'math-utility.md'"
                },
                "content": {
                    "type": "string",
                    "description": "Full markdown content including frontmatter"
                }
            }
        }),
    }
}

/// Execute the write_main_spec tool
/// @spec projects/agentic-workflow/tech-design/core/tools/knowledge.md#source
pub fn execute_write_main_spec(args: &Value, project_root: &Path) -> Result<String> {
    let path = get_required_string(args, "path")?;
    let content = get_required_string(args, "content")?;

    let specs_dir = workspace::tech_design_path(project_root);
    if !specs_dir.exists() {
        std::fs::create_dir_all(&specs_dir)?;
    }

    // Normalize path and prevent directory traversal
    let normalized_path = path.trim_start_matches('/').trim_start_matches("./");
    if normalized_path.contains("..") {
        anyhow::bail!("Invalid path: directory traversal not allowed");
    }

    let file_path = specs_dir.join(normalized_path);

    // Security: ensure path is within specs directory
    if !file_path.starts_with(&specs_dir) {
        anyhow::bail!("Invalid path: must be within .aw/tech-design/");
    }

    // Create parent directories if needed
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let is_update = file_path.exists();
    std::fs::write(&file_path, content)?;

    let action = if is_update { "updated" } else { "created" };
    Ok(format!(
        "✓ Spec {}: .aw/tech-design/{}",
        action, normalized_path
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_main_spec() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let args = json!({
            "path": "test-group/test-spec.md",
            "content": "---\ntitle: Test Spec\n---\n\n# Test Spec\n\nContent."
        });

        let result = execute_write_main_spec(&args, project_root).unwrap();
        assert!(result.contains("✓ Spec created"));

        let file_path = project_root.join(".aw/tech-design/test-group/test-spec.md");
        assert!(file_path.exists());
    }

    #[test]
    fn test_directory_traversal_blocked() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let args = json!({
            "path": "../secrets.md",
            "content": "evil"
        });

        let result = execute_write_main_spec(&args, project_root);
        assert!(result.is_err());
    }
}

// CODEGEN-END
