---
id: sdd-tools-revise-change-docs-preamble-definitions
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools revise change docs preamble definitions

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/revise_change_docs.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/revise_change_docs.rs | function | pub | 46 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/revise_change_docs.rs | function | pub | 157 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/revise_change_docs.rs | function | pub | 96 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/revise_change_docs.rs | function | pub | 21 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
//! Revise tools for change-docs.
//!
//! - `sdd_workflow_revise_change_docs` — build doc-writer prompt with review feedback
//! - `sdd_artifact_revise_change_docs` — delegates to create_change_docs::execute_artifact()

use super::create_change_docs;
use crate::models::state::StatePhase;
use crate::models::WorkflowArtifact;
use crate::state::StateManager;
use crate::tools::workflow_common;
use crate::tools::{get_required_string, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

// ─── Tool Definitions ────────────────────────────────────────────────────────

pub fn workflow_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_revise_change_docs".to_string(),
        description:
            "Orchestrate docs revision: build doc-writer prompt with review feedback, dispatch agent"
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

pub fn artifact_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_artifact_revise_change_docs".to_string(),
        description:
            "Write revised guide sections based on review feedback. Delegates to create artifact logic."
                .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "target_crate", "guide_path", "sections_content", "summary"],
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
                "target_crate": {
                    "type": "string",
                    "description": "Crate name from docs target config"
                },
                "guide_path": {
                    "type": "string",
                    "description": "Output guide file path (relative to project root)"
                },
                "sections_content": {
                    "type": "object",
                    "additionalProperties": { "type": "string" },
                    "description": "Map of section_name -> markdown content"
                },
                "summary": {
                    "type": "string",
                    "description": "Brief description of doc changes"
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
  - path: projects/agentic-workflow/src/tools/revise_change_docs.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
      - "workflow_definition"
      - "artifact_definition"
    description: "Module preamble and revise-change-docs tool definitions."
```
