---
id: sdd-tools-review-reference-context-definitions
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools review reference context definitions

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/review_reference_context.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/review_reference_context.rs | function | pub | 47 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/review_reference_context.rs | function | pub | 160 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/review_reference_context.rs | function | pub | 121 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/review_reference_context.rs | function | pub | 23 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
//! Review tools for reference context.
//!
//! - `sdd_workflow_review_reference_context` — returns review prompt for a group
//! - `sdd_artifact_review_reference_context` — writes inline review into `reference_context.md`

use super::common_reference_context as common;
use crate::models::state::StatePhase;
use crate::models::WorkflowArtifact;
use crate::state::StateManager;
use crate::tools::review_helpers;
use crate::tools::workflow_common;
use crate::tools::{get_optional_string, get_required_string, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

// ─── Tool Definitions ────────────────────────────────────────────────────────

/// MCP tool definition for sdd_workflow_review_reference_context
pub fn workflow_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_review_reference_context".to_string(),
        description: "Return review prompt for a group's reference context".to_string(),
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

/// MCP tool definition for sdd_artifact_review_reference_context
pub fn artifact_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_artifact_review_reference_context".to_string(),
        description: "Write inline review for a group's reference context".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "group_id", "verdict", "summary"],
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
                    "description": "Group ID to review"
                },
                "verdict": {
                    "type": "string",
                    "enum": ["APPROVED", "REVIEWED"],
                    "description": "Review verdict"
                },
                "summary": {
                    "type": "string",
                    "description": "Review summary"
                },
                "checklist_results": {
                    "type": "array",
                    "description": "Checklist results",
                    "items": {
                        "type": "object",
                        "required": ["item", "passed"],
                        "properties": {
                            "item": { "type": "string" },
                            "passed": { "type": "boolean" },
                            "note": { "type": "string" }
                        }
                    }
                },
                "issues": {
                    "type": "array",
                    "description": "Issues found during review",
                    "items": {
                        "type": "object",
                        "required": ["severity", "description"],
                        "properties": {
                            "severity": {
                                "type": "string",
                                "enum": ["HIGH", "MEDIUM", "LOW"]
                            },
                            "description": { "type": "string" },
                            "recommendation": { "type": "string" }
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
  - path: projects/agentic-workflow/src/tools/review_reference_context.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
      - "workflow_definition"
      - "artifact_definition"
    description: "Module preamble and MCP tool definitions for reference-context review workflow and artifact tools."
```
