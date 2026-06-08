// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/knowledge_service.md#source
// CODEGEN-BEGIN
//! Spec service - Business logic for writing main specs
//!
//! Previously "knowledge_service" — the knowledge concept has been merged into specs.
//! All project documentation now lives in .aw/tech-design/.

use crate::shared::workspace;
use crate::Result;
use std::path::Path;

/// Write or update a spec in the main .aw/tech-design/ directory (for archive merge)
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/knowledge_service.md#source
pub fn write_main_spec(path: &str, content: &str, project_root: &Path) -> Result<String> {
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

        let content = "---\ntitle: Test Spec\n---\n\n# Test Spec\n\nContent here.";
        let result = write_main_spec("test-spec.md", content, project_root).unwrap();
        assert!(result.contains("✓ Spec created: .aw/tech-design/test-spec.md"));

        let file_path = project_root.join(".aw/tech-design/test-spec.md");
        assert!(file_path.exists());
        let file_content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(file_content, content);
    }
}
// CODEGEN-END
