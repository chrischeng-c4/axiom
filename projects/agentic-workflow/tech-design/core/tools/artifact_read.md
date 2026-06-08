---
id: projects-sdd-src-tools-artifact-read-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# Standardized projects/agentic-workflow/src/tools/artifact_read.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/artifact_read.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `definition` | projects/agentic-workflow/src/tools/artifact_read.rs | function | pub | 14 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/artifact_read.rs | function | pub | 67 | execute(args: &Value, project_root: &Path) -> Result<String> |
## Source
<!-- type: source lang: rust -->

````rust
//! sdd_read_artifact — Unified Artifact Reader
//!
//! Adapter that dispatches to existing read/task handlers based on scope.
//! See spec: .aw/tech-design/sdd/tools/read-artifact.md

use super::{get_required_string, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_read_artifact".to_string(),
        description: "Read any SDD artifact: change artifacts, main specs, listings, or task instructions. Use scope to select what to read.".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "scope"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path"
                },
                "change_id": {
                    "type": "string",
                    "description": "Required for change artifact reads and task scope."
                },
                "scope": {
                    "type": "string",
                    "description": "What to read.\n\nChange artifacts (require change_id): context_clarifications, spec_clarifications, codebase_context, spec_context, knowledge_context, gap_*, proposal, tasks, requirements, review_{artifact}, or {spec_id}.\n\nMain spec: main_spec:{group/id}\nListings: list:main_specs, list:specs\nTask instructions: task (requires task_type)"
                },
                "task_type": {
                    "type": "string",
                    "enum": [
                        "create_spec",
                        "review_spec",
                        "revise_spec",
                        "implement", "code_review", "resolve", "review_archive"
                    ],
                    "description": "Task type (required when scope=task)"
                },
                "spec_id": {
                    "type": "string",
                    "description": "Spec ID (for spec-related task_types)"
                },
                "description": {
                    "type": "string",
                    "description": "Change description (for create_proposal task)"
                },
                "iteration": {
                    "type": "integer",
                    "description": "Current iteration (for review task_types)"
                },
                "dependencies": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Spec dependency IDs (for create_spec task)"
                }
            }
        }),
    }
}

pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let scope = get_required_string(args, "scope")?;

    if scope == "task" {
        // Dispatch to task handler — pass through all relevant args
        return super::task::execute(args, project_root);
    }

    // All other scopes delegate to the read handler with scope mapped to `file` param
    let mut read_args = serde_json::Map::new();

    // Copy project_path
    if let Some(pp) = args.get("project_path") {
        read_args.insert("project_path".to_string(), pp.clone());
    }

    // Copy change_id if present
    if let Some(cid) = args.get("change_id") {
        read_args.insert("change_id".to_string(), cid.clone());
    }

    // Map scope → file param
    read_args.insert("file".to_string(), json!(scope));

    super::read::execute(&Value::Object(read_args), project_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_read_artifact_change_scope() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        std::fs::write(
            change_dir.join("proposal.md"),
            "# Test Proposal\n\nContent.",
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change",
            "scope": "proposal"
        });

        let result = execute(&args, project_root).unwrap();
        assert!(result.contains("# Test Proposal"));
    }

    #[test]
    fn test_read_artifact_task_scope() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let args = json!({
            "change_id": "my-change",
            "scope": "task",
            "task_type": "create_spec",
            "spec_id": "test-spec"
        });

        let result = execute(&args, project_root).unwrap();
        assert!(result.contains("my-change"));
    }

    #[test]
    fn test_read_artifact_missing_change_id() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let args = json!({
            "scope": "proposal"
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
  - path: projects/agentic-workflow/src/tools/artifact_read.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:claim-code>"
    description: "Unified artifact reader adapter and regression tests."
```
