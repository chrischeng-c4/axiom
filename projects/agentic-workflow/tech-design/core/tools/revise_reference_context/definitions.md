---
id: sdd-tools-revise-reference-context-definitions
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools revise reference context definitions

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/revise_reference_context.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/revise_reference_context.rs | function | pub | 45 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/revise_reference_context.rs | function | pub | 148 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/revise_reference_context.rs | function | pub | 108 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/revise_reference_context.rs | function | pub | 21 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
//! Revise tools for reference context.
//!
//! - `sdd_workflow_revise_reference_context` — returns revise prompt for a group
//! - `sdd_artifact_revise_reference_context` — rewrites `reference_context.md` (delegates to create)

use super::common_reference_context as common;
use super::create_reference_context as create;
use crate::models::WorkflowArtifact;
use crate::tools::workflow_common;
use crate::tools::{get_required_string, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

// ─── Tool Definitions ────────────────────────────────────────────────────────

/// MCP tool definition for sdd_workflow_revise_reference_context
pub fn workflow_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_revise_reference_context".to_string(),
        description: "Return revise prompt for a group's reference context".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path"
                },
                "change_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Change ID"
                }
            }
        }),
    }
}

/// MCP tool definition for sdd_artifact_revise_reference_context
pub fn artifact_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_artifact_revise_reference_context".to_string(),
        description: "Rewrite reference context with corrected specs (revision)".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "group_id", "specs"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path"
                },
                "change_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Change ID"
                },
                "group_id": {
                    "type": "string",
                    "description": "Group ID to revise reference context for"
                },
                "specs": {
                    "type": "array",
                    "minItems": 1,
                    "description": "Corrected specs for this group",
                    "items": {
                        "type": "object",
                        "required": ["spec_id", "spec_group", "relevance"],
                        "properties": {
                            "spec_id": {
                                "type": "string",
                                "description": "Spec ID"
                            },
                            "spec_group": {
                                "type": "string",
                                "description": "Spec group path"
                            },
                            "relevance": {
                                "type": "string",
                                "enum": ["high", "medium", "low"]
                            },
                            "key_requirements": {
                                "type": "array",
                                "items": { "type": "string" }
                            }
                        }
                    }
                }
            }
        }),
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/revise_reference_context.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
      - "workflow_definition"
      - "artifact_definition"
    description: "Module preamble and MCP tool definitions for reference-context revise workflow and artifact tools."
```
