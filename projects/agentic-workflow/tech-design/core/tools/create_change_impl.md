---
id: projects-sdd-src-tools-create-change-impl-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# Standardized projects/agentic-workflow/src/tools/create_change_impl.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_change_impl.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/create_change_impl.rs | function | pub | 46 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/create_change_impl.rs | function | pub | 348 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/create_change_impl.rs | function | pub | 83 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `parse_test_plan_count` | projects/agentic-workflow/src/tools/create_change_impl.rs | function | pub | 869 | parse_test_plan_count(spec_content: &str) -> Option<usize> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_change_impl.rs | function | pub | 21 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
//! Create tools for change-implementation.
//!
//! - `sdd_workflow_create_change_implementation` — sub-state router
//! - `sdd_artifact_create_change_implementation` — write implementation.md with git diff

use super::common_change_impl::{self as common, ImplSubState, MAX_SPEC_REVISIONS};
use super::common_change_spec;
use crate::models::WorkflowArtifact;
use crate::state::StateManager;
use crate::tools::workflow_common;
use crate::tools::{get_required_string, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

// ─── Tool Definitions ────────────────────────────────────────────────────────

/// @spec projects/agentic-workflow/tech-design/core/tools/create_change_impl.md#source
pub fn workflow_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_create_change_implementation".to_string(),
        description:
            "Sub-state router for implementation: per-spec implement -> write diff -> done"
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

/// @spec projects/agentic-workflow/tech-design/core/tools/create_change_impl.md#source
pub fn artifact_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_artifact_create_change_implementation".to_string(),
        description: "Write implementation.md with git diff snapshot".to_string(),
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

/// Execute sdd_workflow_create_change_implementation.
///
/// Resolves ImplSubState, applies STATE.yaml side effects, and returns
/// a prompt + next_actions for the caller.
/// @spec projects/agentic-workflow/tech-design/core/tools/create_change_impl.md#source
pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;
    let interface = workflow_common::load_interface(project_root);

    let (sub_state, new_spec_id, incr_rev_spec) =
        common::resolve_next_impl(&change_dir, &change_id)?;

    // Apply STATE.yaml side effects
    if new_spec_id.is_some() || incr_rev_spec.is_some() {
        if let Ok(mut sm) = StateManager::load(&change_dir) {
            if let Some(ref spec_id) = new_spec_id {
                sm.state_mut().current_task_id = Some(spec_id.clone());
            }
            if let Some(ref spec_id) = incr_rev_spec {
                sm.state_mut()
                    .task_revisions
                    .entry(spec_id.clone())
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
            }
            let _ = sm.save();
        }
    }

    match sub_state {
        ImplSubState::NoSpecs => {
            let result = json!({
                "status": "error",
                "message": "No change specs found in specs/ directory. Cannot implement.",
                "next_actions": []
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }

        ImplSubState::ImplementSpecCode { spec_id, is_first } => {
            // Update STATE.yaml: impl_spec_phase["spec_id"] = "code"
            if let Ok(mut sm) = crate::state::StateManager::load(&change_dir) {
                sm.state_mut()
                    .impl_spec_phase
                    .insert(spec_id.clone(), "code".to_string());
                let _ = sm.save();
            }
            // Resolve group_id per-spec for group-scoped prompt placement
            let group_id = common_change_spec::resolve_group_id_for_spec(&change_dir, &spec_id)
                .or_else(|| workflow_common::resolve_single_group_id(&change_dir));
            build_implement_code_prompt(
                &change_id,
                &spec_id,
                is_first,
                group_id.as_deref(),
                project_root,
            )
            .await
        }

        ImplSubState::BuildCheck { spec_id } => {
            // Run cargo build --workspace — hard gate before test phase
            let build_result = std::process::Command::new("cargo")
                .args(["build", "--workspace"])
                .current_dir(project_root)
                .output();

            match build_result {
                Ok(output) if output.status.success() => {
                    // Build passed → transition to tests phase
                    if let Ok(mut sm) = crate::state::StateManager::load(&change_dir) {
                        sm.state_mut()
                            .impl_spec_phase
                            .insert(spec_id.clone(), "tests".to_string());
                        let _ = sm.save();
                    }
                    let group_id =
                        common_change_spec::resolve_group_id_for_spec(&change_dir, &spec_id)
                            .or_else(|| workflow_common::resolve_single_group_id(&change_dir));
                    build_implement_tests_prompt(
                        &change_id,
                        &spec_id,
                        group_id.as_deref(),
                        project_root,
                    )
                    .await
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let build_output = format!("{}{}", stdout, stderr).trim().to_string();
                    let result = serde_json::json!({
                        "status": "error",
                        "message": format!("Build failed after implementing production code for spec '{}'. Fix compilation errors before tests can be added.", spec_id),
                        "spec_id": spec_id,
                        "build_output": build_output,
                        "next_actions": []
                    });
                    Ok(serde_json::to_string_pretty(&result)?)
                }
                Err(e) => {
                    let result = serde_json::json!({
                        "status": "error",
                        "message": format!("Failed to run `cargo build --workspace`: {}", e),
                        "spec_id": spec_id,
                        "next_actions": []
                    });
                    Ok(serde_json::to_string_pretty(&result)?)
                }
            }
        }

        ImplSubState::ImplementSpecTests { spec_id } => {
            // Update STATE.yaml: impl_spec_phase["spec_id"] = "tests" (idempotent, already set by BuildCheck)
            if let Ok(mut sm) = crate::state::StateManager::load(&change_dir) {
                sm.state_mut()
                    .impl_spec_phase
                    .insert(spec_id.clone(), "tests".to_string());
                let _ = sm.save();
            }
            let group_id = common_change_spec::resolve_group_id_for_spec(&change_dir, &spec_id)
                .or_else(|| workflow_common::resolve_single_group_id(&change_dir));
            build_implement_tests_prompt(&change_id, &spec_id, group_id.as_deref(), project_root)
                .await
        }

        ImplSubState::TestCountCheck { spec_id } => {
            // Count #[test] in diff added lines and compare vs spec Unit Test design
            let group_id = common_change_spec::resolve_group_id_for_spec(&change_dir, &spec_id)
                .or_else(|| workflow_common::resolve_single_group_id(&change_dir));
            let spec_path = match group_id.as_deref() {
                Some(gid) => change_dir
                    .join("groups")
                    .join(gid)
                    .join("specs")
                    .join(format!("{}.md", spec_id)),
                None => change_dir.join("specs").join(format!("{}.md", spec_id)),
            };

            let actual_count = count_tests_in_diff(project_root);
            let required_count = spec_path
                .exists()
                .then(|| std::fs::read_to_string(&spec_path).ok())
                .flatten()
                .and_then(|c| parse_test_plan_count(&c));

            // Clear impl_spec_phase for this spec (done with both phases)
            if let Ok(mut sm) = crate::state::StateManager::load(&change_dir) {
                sm.state_mut().impl_spec_phase.remove(&spec_id);
                let _ = sm.save();
            }

            let verification = match required_count {
                Some(required) => {
                    let passed = actual_count >= required;
                    serde_json::json!({
                        "passed": passed,
                        "test_count": actual_count,
                        "required": required
                    })
                }
                None => serde_json::json!({
                    "skipped": true,
                    "reason": "No numeric unit-test section found in spec"
                }),
            };

            let result = serde_json::json!({
                "status": "ok",
                "action": "test_count_verified",
                "spec_id": spec_id,
                "verification": verification,
                "next_actions": [
                    workflow_common::next_action(interface, "sdd_workflow_create_change_implementation", serde_json::json!({"change_id": change_id}))
                ]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }

        ImplSubState::ImplementSpecWithCodegen { spec_id } => {
            let group_id = common_change_spec::resolve_group_id_for_spec(&change_dir, &spec_id)
                .or_else(|| workflow_common::resolve_single_group_id(&change_dir));
            build_codegen_prompt(&change_id, &spec_id, group_id.as_deref(), project_root).await
        }

        ImplSubState::WriteDiff => {
            // No spec-specific group_id for diff; use single-group heuristic
            let group_id = workflow_common::resolve_single_group_id(&change_dir);
            build_write_diff_prompt(&change_id, group_id.as_deref(), project_root).await
        }

        ImplSubState::ReviewSpec { spec_id } => {
            // Redirect to review workflow
            let result = json!({
                "status": "ok",
                "prompt": format!("Spec '{}' ready for review. Redirecting to review workflow.", spec_id),
                "next_actions": [
                    workflow_common::next_action(interface, "sdd_workflow_review_change_implementation", json!({"change_id": change_id}))
                ]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }

        ImplSubState::ReviseSpec { spec_id } => {
            let result = json!({
                "status": "ok",
                "spec_id": spec_id,
                "next_actions": [
                    workflow_common::next_action(interface, "sdd_workflow_revise_change_implementation", json!({"change_id": change_id}))
                ]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }

        ImplSubState::TerminalFailure { spec_id, revisions } => {
            // Reset state to allow retry
            let mut sm = StateManager::load(&change_dir)?;
            sm.state_mut().task_revisions.clear();
            sm.set_phase(crate::models::state::StatePhase::ChangeImplementationCreated)?;
            sm.save()?;

            let result = json!({
                "status": "error",
                "message": format!(
                    "Spec '{}' failed review after {} revisions (limit: {}). \
                     State reset to allow retry, or fix manually.",
                    spec_id, revisions, MAX_SPEC_REVISIONS
                ),
                "spec_id": spec_id,
                "next_actions": [
                    workflow_common::next_action(interface, "sdd_run_change", json!({
                        "change_id": change_id,
                    }))
                ]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }

        ImplSubState::AdvanceToMerge => {
            // Advance STATE.yaml to TestCheck → DocsCheck → ChangeMergeCreated
            // (TestCheck and DocsCheck are transient phases resolved inline by route())
            if let Ok(mut sm) = StateManager::load(&change_dir) {
                sm.state_mut().phase = crate::models::state::StatePhase::TestCheck;
                let _ = sm.save();
            }

            let result = json!({
                "status": "phase_complete",
                "change_id": change_id,
                "message": "All specs implemented and approved! Phase advanced to test_check.",
                "next_actions": [{
                    "args": { "change_id": change_id },
                    "cli": format!("score run-change --change-id {}", change_id)
                }]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
    }
}

// ─── Artifact ─────────────────────────────────────────────────────────────────

/// Execute sdd_artifact_create_change_implementation.
///
/// Writes implementation.md with the provided git diff.
/// @spec projects/agentic-workflow/tech-design/core/tools/create_change_impl.md#source
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let diff = get_required_string(args, "diff")?;
    let summary = get_required_string(args, "summary")?;
    workflow_common::validate_change_id(&change_id)?;
    let interface = workflow_common::load_interface(project_root);

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;

    let impl_path = change_dir.join("implementation.md");

    let content = format!(
        "---\nid: implementation\ntype: change_implementation\nchange_id: {change_id}\n---\n\n\
         # Implementation\n\n\
         ## Summary\n\n{summary}\n\n\
         ## Diff\n\n```diff\n{diff}\n```\n"
    );

    std::fs::write(&impl_path, &content)?;

    let result = json!({
        "status": "ok",
        "artifacts_written": ["implementation.md"],
        "next_actions": [
            workflow_common::next_action(interface, "sdd_workflow_create_change_implementation", json!({"change_id": change_id}))
        ]
    });
    Ok(serde_json::to_string_pretty(&result)?)
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/create_change_impl.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:projects-sdd-src-tools-create-change-impl-rs>"
    description: "Change implementation workflow and artifact entrypoints."
```
