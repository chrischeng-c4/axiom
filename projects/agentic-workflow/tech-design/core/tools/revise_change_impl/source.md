---
id: sdd-tools-revise-change-impl-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools revise change impl source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/revise_change_impl.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/revise_change_impl.rs | function | pub | 44 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/revise_change_impl.rs | function | pub | 118 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/revise_change_impl.rs | function | pub | 82 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/revise_change_impl.rs | function | pub | 20 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
//! Revise tools for change-implementation.
//!
//! - `sdd_workflow_revise_change_implementation` — returns revise prompt for a spec
//! - `sdd_artifact_revise_change_implementation` — delegates to `create::execute_artifact()`

use super::common_change_impl as common;
use super::create_change_impl as create;
use crate::models::WorkflowArtifact;
use crate::tools::workflow_common;
use crate::tools::{get_required_string, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

// ─── Tool Definitions ────────────────────────────────────────────────────────

pub fn workflow_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_revise_change_implementation".to_string(),
        description: "Orchestrate revision of change-implementation: fix review issues for a spec"
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
        name: "sdd_artifact_revise_change_implementation".to_string(),
        description: "Write implementation.md with revised git diff. Delegates to create artifact."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "diff", "summary"],
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
                "diff": {
                    "type": "string",
                    "description": "Full git diff content"
                },
                "summary": {
                    "type": "string",
                    "description": "Brief description of all changes"
                }
            }
        }),
    }
}

// ─── Workflow ─────────────────────────────────────────────────────────────────

/// Execute sdd_workflow_revise_change_implementation.
///
/// Resolves which spec needs revision via `resolve_next_impl()`, then
/// returns a revise prompt.
pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;
    let interface = workflow_common::load_interface(project_root);

    let group_id = workflow_common::resolve_single_group_id(&change_dir);

    let (sub_state, _, _) = common::resolve_next_impl(&change_dir, &change_id)?;

    match sub_state {
        common::ImplSubState::ReviseSpec { spec_id } => {
            build_revise_prompt(&change_id, &spec_id, group_id.as_deref(), project_root).await
        }
        _ => {
            // Not in revise sub-state — redirect back to create router
            let result = json!({
                "status": "ok",
                "message": "Implementation is not in Revise sub-state. Redirecting to router.",
                "next_actions": [
                    workflow_common::next_action(interface, "sdd_workflow_create_change_implementation", json!({"change_id": change_id}))
                ]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
    }
}

// ─── Artifact ─────────────────────────────────────────────────────────────────

/// Execute sdd_artifact_revise_change_implementation.
///
/// Delegates to `create::execute_artifact()` — same write behavior.
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    let result = create::execute_artifact(args, project_root)?;

    // Increment revision count so auto-approve (threshold >= 1) triggers on next review.
    let change_id = get_required_string(args, "change_id")?;
    let spec_id = get_required_string(args, "spec_id")?;
    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    let rev_key = format!("impl:{}", spec_id);
    if let Ok(mut sm) = crate::state::StateManager::load(&change_dir) {
        sm.increment_revision_count(&rev_key);
        let _ = sm.save();
    }

    Ok(result)
}

// ─── Prompt Builder ──────────────────────────────────────────────────────────

async fn build_revise_prompt(
    change_id: &str,
    spec_id: &str,
    group_id: Option<&str>,
    project_root: &Path,
) -> Result<String> {
    let _pp = project_root.display();

    let prompt = format!(
        "# Task: Revise Implementation of Spec '{spec_id}' for Change '{change_id}'\n\n\
         ## Instructions\n\n\
         1. Read `implementation.md` for the inline `## Review: {spec_id}` section\n\
         2. Fix all identified issues in the code\n\
         3. Re-run tests to verify\n\
         4. When done, run `score run-change --change-id {change_id}` to continue the workflow\n\n\
         ## CLI Commands\n\n\
         ```\n\
         # Read implementation and spec\n\
         Read file: .aw/changes/{change_id}/implementation.md\n\
         Read file: .aw/changes/{change_id}/specs/{spec_id}.md\n\
         \n\
         # Continue workflow\n\
         score run-change --change-id {change_id}\n\
         ```"
    );

    let change_dir = super::workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor = workflow_common::get_executor_chain(
        project_root,
        WorkflowArtifact::ReviseChangeImplementation,
    );

    let mut extra = json!({ "spec_id": spec_id });
    if let Some(gid) = group_id {
        extra["group_id"] = json!(gid);
    }

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        "revise_change_implementation",
        prompt,
        executor,
        extra,
        interface,
        project_root,
    )
    .await
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/revise_change_impl.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:claim-code>"
    description: "Revise-change-implementation tool definitions, workflow, artifact delegation, and prompt construction."
```
