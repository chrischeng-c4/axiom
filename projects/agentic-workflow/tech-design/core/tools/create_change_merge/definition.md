---
id: sdd-tools-create-change-merge-definition
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create change merge definition

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_change_merge.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `execute_workflow` | projects/agentic-workflow/src/tools/create_change_merge.rs | function | pub | 69 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_change_merge.rs | function | pub | 29 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
//! Programmatic merge tool for change-merge.
//!
//! `sdd_workflow_create_change_merge` — single tool that:
//! 1. Reads all change specs from `changes/{id}/specs/`
//! 2. Extracts `main_spec_ref` from each spec's frontmatter
//! 3. Strips change-spec-only fields
//! 4. Writes cleaned specs to `.aw/tech-design/{main_spec_ref}`
//! 5. Updates phase to `ChangeArchived`
//!
//! No agent needed. No CRR loop. Single programmatic operation.

use crate::models::state::StatePhase;
use crate::models::SddConfig;
use crate::models::WorkflowArtifact;
use crate::tools::common_change_spec as common;
use crate::tools::merge_git_ops::{find_git_binary, post_archive_git_ops, resolve_worktree_dir};
use crate::tools::workflow_common;
use crate::tools::{get_required_string, ToolDefinition};
use crate::workflow::helpers;
use crate::Result;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

// ─── Tool Definition ──────────────────────────────────────────────────────────

pub fn workflow_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_create_change_merge".to_string(),
        description:
            "Programmatic merge: copy all change specs to .aw/tech-design/ and archive the change"
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/create_change_merge.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
      - "workflow_definition"
    description: "Module preamble and workflow tool definition for programmatic change merge."
```
