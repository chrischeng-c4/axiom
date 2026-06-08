---
id: sdd-tools-revise-change-spec-preamble-definitions
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools revise change spec preamble definitions

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/revise_change_spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/revise_change_spec.rs | function | pub | 46 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/revise_change_spec.rs | function | pub | 135 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/revise_change_spec.rs | function | pub | 92 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/revise_change_spec.rs | function | pub | 22 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
//! Revise tools for change-spec.
//!
//! - `sdd_workflow_revise_change_spec` — re-fill flagged sections after review
//! - `sdd_artifact_revise_change_spec` — delegates to `create::execute_artifact()`

use super::common_change_spec as common;
use super::create_change_spec as create;
use crate::models::state::StatePhase;
use crate::models::WorkflowArtifact;
use crate::tools::review_helpers;
use crate::tools::workflow_common;
use crate::tools::{get_required_string, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

// ─── Tool Definitions ────────────────────────────────────────────────────────

/// @spec projects/agentic-workflow/tech-design/core/tools/revise_change_spec/preamble-definitions.md#source
pub fn workflow_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_revise_change_spec".to_string(),
        description: "Orchestrate revision of change-spec: re-fill flagged sections from review"
            .to_string(),
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

/// @spec projects/agentic-workflow/tech-design/core/tools/revise_change_spec/preamble-definitions.md#source
pub fn artifact_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_artifact_revise_change_spec".to_string(),
        description: "Write one section of a change spec (revision). Delegates to create artifact."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "spec_id", "section", "content"],
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
                "spec_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Spec ID"
                },
                "section": {
                    "type": "string",
                    "enum": ["overview", "requirements", "scenarios", "db-model", "dependency", "state-machine", "logic", "interaction", "mindmap", "rest-api", "rpc-api", "async-api", "cli", "schema", "config", "wireframe", "component", "design-token", "unit-test", "e2e-test", "changes", "doc"],
                    "description": "Which section to revise"
                },
                "content": {
                    "type": "string",
                    "description": "Revised content for this section"
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
  - path: projects/agentic-workflow/src/tools/revise_change_spec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
      - "workflow_definition"
      - "artifact_definition"
    description: "Module preamble and revise-change-spec tool definitions."
```
