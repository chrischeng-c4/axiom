---
id: sdd-tools-create-change-spec-preamble-definitions
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create change spec preamble definitions

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_change_spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/create_change_spec.rs | function | pub | 55 | artifact_definition() -> ToolDefinition |
| `build_fill_prompt` | projects/agentic-workflow/src/tools/create_change_spec.rs | function | pub | 736 | build_fill_prompt(     change_id: &str,     spec_id: &str,     section: &str,     group_id: Option<&str>,     project_root: &Path, ) -> Result<String> |
| `execute_artifact` | projects/agentic-workflow/src/tools/create_change_spec.rs | function | pub | 338 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/create_change_spec.rs | function | pub | 120 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_change_spec.rs | function | pub | 29 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
//! Create tools for change-spec.
//!
//! - `sdd_workflow_create_change_spec` — sub-state router: skeleton → analyze → fill → prune
//! - `sdd_artifact_create_change_spec` — writes one section at a time into the spec file
//!
//! Revise is handled by `revise.rs`. When `resolve_next_spec()` returns
//! `SpecSubState::Revise`, this module redirects to `sdd_workflow_revise_change_spec`.

use super::common_change_spec::{self as common, SpecSubState};
use crate::models::change::SddInterface;
use crate::models::spec_rules::SectionType;
use crate::models::state::StatePhase;
use crate::models::WorkflowArtifact;
use crate::tools::review_helpers;
use crate::tools::workflow_common;
use crate::tools::{get_optional_string, get_required_string, ToolDefinition};
use crate::workflow::helpers;
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;
use std::str::FromStr;

// ─── Tool Definitions ────────────────────────────────────────────────────────

/// MCP tool definition for sdd_workflow_create_change_spec
/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn workflow_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_create_change_spec".to_string(),
        description: "Orchestrate per-spec change-spec lifecycle \
            (skeleton → analyze → fill sections → prune → review → revise)"
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

/// MCP tool definition for sdd_artifact_create_change_spec
/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn artifact_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_artifact_create_change_spec".to_string(),
        description: "Write one section of a change spec. Used for both create and revise."
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
                    "description": "Which section to fill/replace"
                },
                "content": {
                    "type": "string",
                    "description": "Markdown content for this section (everything after the H2 heading)"
                },
                "fill_sections": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Sections to fill (set during analyze). Persisted to frontmatter."
                },
                "main_spec_ref": {
                    "type": "string",
                    "description": "Target path in .aw/tech-design/ for merge (e.g. sdd/tools/foo.md)"
                },
                "group_id": {
                    "type": "string",
                    "description": "Group ID for group-scoped spec path (optional; uses groups/{group_id}/specs/)"
                },
                "section_type": {
                    "type": "string",
                    "description": "Section type for annotation injection (e.g. overview, changes). Uses SectionType enum."
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
  - path: projects/agentic-workflow/src/tools/create_change_spec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
      - "<handwrite-gap:missing-generator:sdd-tool-definition-json-schema>"
    description: "Module preamble, imports, and create-change-spec tool definitions."
```
