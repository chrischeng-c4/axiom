---
id: projects-sdd-src-tools-read-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# Standardized projects/agentic-workflow/src/tools/read.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/read.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `definition` | projects/agentic-workflow/src/tools/read.rs | function | pub | 16 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/read.rs | function | pub | 44 | execute(args: &Value, project_root: &Path) -> Result<String> |
## Source
<!-- type: source lang: rust -->

````rust
//! read_file MCP Tool (Consolidated)
//!
//! Unified file reader for change artifacts, knowledge docs, main specs, and listings.
//! Supports scope prefixes in the `file` parameter.

use super::{get_optional_string, ToolDefinition};
use crate::services::file_service::read_file;
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

/// Get the tool definition for read_file
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_read_file".to_string(),
        description: "Read SDD files: change artifacts, main specs, or listings. Use scope prefixes in the file parameter.".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path (use $PWD for current directory)"
                },
                "change_id": {
                    "type": "string",
                    "description": "Change ID (required for change-scoped reads, optional for knowledge/main_spec/list scopes)"
                },
                "file": {
                    "type": "string",
                    "description": "File to read with optional scope prefix.\n\nScope prefixes:\n- main_spec:group/id — read from .aw/tech-design/ (e.g. 'main_spec:sdd/run-change')\n- list:main_specs — list all main specs (or list:main_specs:group to filter)\n- list:specs — list change specs (or list:specs:spec_id for dependencies)\n- requirements — read all requirements (proposal+tasks+specs)\n\nUnprefixed (requires change_id):\n- proposal, tasks, clarifications, spec_context, knowledge_context, codebase_context, gap_*, or spec name",
                    "default": "proposal"
                }
            }
        }),
    }
}

/// Execute the read_file tool
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_optional_string(args, "change_id").unwrap_or_default();
    let file = get_optional_string(args, "file").unwrap_or_else(|| "proposal".to_string());

    // Validate change_id is provided for change-scoped reads
    let needs_change_id = !file.starts_with("main_spec:") && !file.starts_with("list:main_specs");

    if needs_change_id && change_id.is_empty() {
        anyhow::bail!("change_id is required for file '{}'. Only main_spec: and list:main_specs scopes can omit change_id.", file);
    }

    read_file(&change_id, &file, project_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_read_proposal() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        std::fs::write(
            change_dir.join("proposal.md"),
            "# Test Proposal\n\nThis is a test.",
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change"
        });

        let result = execute(&args, project_root).unwrap();
        assert!(result.contains("# Test Proposal"));
        assert!(result.contains("This is a test"));
    }

    #[test]
    fn test_read_spec() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let specs_dir = project_root.join(".aw/changes/test-change/specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(
            specs_dir.join("my-feature.md"),
            "# My Feature Spec\n\nRequirements here.",
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "my-feature"
        });

        let result = execute(&args, project_root).unwrap();
        assert!(result.contains("# My Feature Spec"));
    }

    #[test]
    fn test_read_main_spec_scoped() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let group_dir = project_root.join(".aw/tech-design/test-group");
        std::fs::create_dir_all(&group_dir).unwrap();
        std::fs::write(
            group_dir.join("test-spec.md"),
            "---\ntitle: Test Spec\n---\n\n# Test Spec\n\nContent.",
        )
        .unwrap();

        let args = json!({
            "file": "main_spec:test-group/test-spec"
        });

        let result = execute(&args, project_root).unwrap();
        assert!(result.contains("# Main Spec: test-group/test-spec"));
        assert!(result.contains("Content."));
    }

    #[test]
    fn test_list_main_specs_scoped() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let group_dir = project_root.join(".aw/tech-design/test-group");
        std::fs::create_dir_all(&group_dir).unwrap();
        std::fs::write(
            group_dir.join("auth.md"),
            "---\ntitle: Auth Spec\n---\n\n# Auth",
        )
        .unwrap();

        let args = json!({
            "file": "list:main_specs"
        });

        let result = execute(&args, project_root).unwrap();
        assert!(result.contains("# Main Specs"));
        assert!(result.contains("test-group"));
    }

    #[test]
    fn test_list_specs_scoped() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let specs_dir = project_root.join(".aw/changes/test-change/specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(specs_dir.join("spec-a.md"), "# Spec A").unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "list:specs"
        });

        let result = execute(&args, project_root).unwrap();
        assert!(result.contains("spec-a"));
    }

    #[test]
    fn test_requirements_scoped() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        std::fs::write(change_dir.join("proposal.md"), "# Proposal").unwrap();
        std::fs::write(change_dir.join("tasks.md"), "# Tasks").unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "requirements"
        });

        let result = execute(&args, project_root).unwrap();
        assert!(result.contains("# Requirements for Change: test-change"));
        assert!(result.contains("## Proposal"));
        assert!(result.contains("## Tasks"));
    }

    #[test]
    fn test_missing_change_id_for_change_scoped() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let args = json!({
            "file": "proposal"
        });

        let result = execute(&args, project_root);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("change_id is required"));
    }

    #[test]
    fn test_path_traversal_blocked() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let knowledge_dir = project_root.join(".aw/tech-design/knowledge");
        std::fs::create_dir_all(&knowledge_dir).unwrap();

        let args = json!({
            "file": "knowledge:../../etc/passwd"
        });

        let result = execute(&args, project_root);
        assert!(result.is_err());
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/read.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:projects-sdd-src-tools-read-rs>"
    description: "Consolidated MCP file reader tool."
```
