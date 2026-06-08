// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/revise_reference_context/definitions.md#source
// CODEGEN-BEGIN
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
/// @spec projects/agentic-workflow/tech-design/core/tools/revise_reference_context/definitions.md#source
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
/// @spec projects/agentic-workflow/tech-design/core/tools/revise_reference_context/definitions.md#source
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
// CODEGEN-END
// ─── Workflow ─────────────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/revise_reference_context/workflow.md#source
// CODEGEN-BEGIN
// ─── Workflow ─────────────────────────────────────────────────────────────────

/// Execute sdd_workflow_revise_reference_context.
///
/// Returns revise prompt for the current group in Revise sub-state.
/// @spec projects/agentic-workflow/tech-design/core/tools/revise_reference_context/workflow.md#source
pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;
    let interface = workflow_common::load_interface(project_root);

    // Resolve current group — should be in Revise sub-state
    match common::resolve_next_group(&change_dir)? {
        common::GroupSubState::Revise { group_id } => {
            build_revise_prompt(&change_id, &group_id, project_root).await
        }
        _ => {
            // Not in revise sub-state — redirect back to router
            let result = json!({
                "status": "ok",
                "prompt": "Group is not in Revise sub-state. Redirecting to router.",
                "group_id": null,
                "next_actions": [
                    workflow_common::next_action(interface, "sdd_workflow_create_reference_context", json!({"change_id": change_id}))
                ]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
    }
}
// CODEGEN-END
// ─── Artifact Revise ─────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/revise_reference_context/artifact.md#source
// CODEGEN-BEGIN
// ─── Artifact Revise ─────────────────────────────────────────────────────────

/// Execute sdd_artifact_revise_reference_context.
///
/// Delegates to `create::execute_artifact()` for writing, then increments revision count.
/// This ensures auto-approve triggers regardless of whether the revise was done by an agent
/// or by mainthread (the workflow_common agent-dispatch post-hook only covers the agent path).
/// @spec projects/agentic-workflow/tech-design/core/tools/revise_reference_context/artifact.md#source
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    let result = create::execute_artifact(args, project_root)?;

    // Increment revision count so auto-approve (threshold >= 1) can trigger on next review.
    let change_id = get_required_string(args, "change_id")?;
    let group_id = get_required_string(args, "group_id")?;
    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    let rev_key = format!("ref_ctx:{}", group_id);
    if let Ok(mut sm) = crate::state::StateManager::load(&change_dir) {
        sm.increment_revision_count(&rev_key);
        let _ = sm.save();
    }

    Ok(result)
}
// CODEGEN-END
// ─── Prompt Builder ──────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/revise_reference_context/prompt.md#source
// CODEGEN-BEGIN
// ─── Prompt Builder ──────────────────────────────────────────────────────────

/// Build REVISE prompt for a group's reference context.
async fn build_revise_prompt(
    change_id: &str,
    group_id: &str,
    project_root: &Path,
) -> Result<String> {
    let project_path = project_root.display();

    let prompt = format!(
        r#"# Task: Revise Reference Context for Group '{group_id}' (Change '{change_id}')

## Instructions

1. **Read artifact + review feedback**:
   `{project_path}/.aw/changes/{change_id}/groups/{group_id}/reference_context.md`
   Focus on the `# Reviews` section — list each issue to address.
2. **Read pre-clarifications** (confirm scope):
   `{project_path}/.aw/changes/{change_id}/groups/{group_id}/pre_clarifications.md`
3. **Address each issue one by one**: For each review issue:
   - Identify what needs to change (add spec? fix relevance? update key requirements?)
   - If a missing spec is mentioned, read it from `{project_path}/.aw/tech-design/`
   - Apply the fix to your specs array
4. **Self-verify**: Walk through each original review issue — is it resolved in the new specs array?
5. **Scope re-check**: Do the revised specs still cover all crates/areas from pre-clarifications?
6. Rewrite via artifact tool:

## CLI Commands

```
# Write revised artifact (write payload JSON first, then run)
score artifact revise-reference-context {change_id} .aw/changes/{change_id}/payloads/revise-reference-context.json
```"#,
    );

    let change_dir = super::workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor =
        workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviseReferenceContext);

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        "revise_reference_context",
        prompt,
        executor,
        json!({ "group_id": group_id }),
        interface,
        project_root,
    )
    .await
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/revise_reference_context/tests.md#source
// CODEGEN-BEGIN
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::state::StatePhase;
    use crate::state::StateManager;
    use tempfile::TempDir;

    fn setup_revise_change(change_id: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes").join(change_id);
        let group_dir = change_dir.join("groups").join("my-group");
        std::fs::create_dir_all(&group_dir).unwrap();

        crate::test_util::write_minimal_issue(tmp.path(), change_id);
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = StatePhase::ChangeInited;
        sm.save().unwrap();

        std::fs::write(change_dir.join("user_input.md"), "Test").unwrap();

        // Write artifact with REVIEWED verdict (Revise sub-state)
        std::fs::write(
            group_dir.join("reference_context.md"),
            "---\nchange: test\ngroup: my-group\ndate: 2026-03-04\nreview_verdict: REVIEWED\n---\n\n# Reference Context\n\n| Spec | Group | Relevance | Key Requirements |\n|------|-------|-----------|------------------|\n| some-spec | test | high | R1 |\n\n# Reviews\n\n## Review: reviewer (Iteration 1)\n\n**Verdict**: REVIEWED\n\nMissing specs.\n",
        ).unwrap();

        tmp
    }

    /// Read prompt content from either inline response or prompt file.
    fn read_prompt(parsed: &Value, project_root: &std::path::Path) -> String {
        if let Some(p) = parsed["prompt"].as_str() {
            return p.to_string();
        }
        let rel = parsed["prompt_path"]
            .as_str()
            .expect("Expected prompt_path in response");
        let prompt_path = project_root.join(rel);
        std::fs::read_to_string(&prompt_path)
            .unwrap_or_else(|_| panic!("No prompt at {:?}", prompt_path))
    }

    #[tokio::test]
    async fn test_revise_workflow_returns_prompt() {
        let tmp = setup_revise_change("rev-wf");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-wf"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["group_id"], "my-group");
        let prompt = read_prompt(&parsed, tmp.path());
        assert!(prompt.contains("Revise Reference Context"));
    }

    #[test]
    fn test_revise_artifact_delegates_to_create() {
        let tmp = setup_revise_change("rev-art");

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-art",
            "group_id": "my-group",
            "specs": [
                {
                    "spec_id": "some-spec",
                    "spec_group": "test",
                    "relevance": "high",
                    "key_requirements": ["R1"]
                },
                {
                    "spec_id": "missing-spec",
                    "spec_group": "test",
                    "relevance": "medium",
                    "key_requirements": ["R2"]
                }
            ]
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        // Verify file was rewritten
        let artifact_path = tmp
            .path()
            .join(".aw/changes/rev-art/groups/my-group/reference_context.md");
        let content = std::fs::read_to_string(&artifact_path).unwrap();
        assert!(content.contains("missing-spec"));
        // Review section should be gone (overwritten)
        assert!(!content.contains("# Reviews"));

        // Revise artifact increments revision count so auto-approve triggers.
        let change_dir = tmp.path().join(".aw/changes/rev-art");
        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(sm.revision_count("ref_ctx:my-group"), 1);
    }

    #[tokio::test]
    async fn test_revise_workflow_redirects_when_not_in_revise_state() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/not-revise");
        let group_dir = change_dir.join("groups/my-group");
        std::fs::create_dir_all(&group_dir).unwrap();
        std::fs::write(change_dir.join("user_input.md"), "Test").unwrap();

        crate::test_util::write_minimal_issue(tmp.path(), "not-revise");
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = StatePhase::ChangeInited;
        sm.save().unwrap();

        // No artifact = Create sub-state, not Revise
        std::fs::write(
            group_dir.join("pre_clarifications.md"),
            "---\nchange: test\ngroup: my-group\n---\n",
        )
        .unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "not-revise"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        // Should redirect to router
        let next = &parsed["next_actions"][0];
        assert_eq!(next["args"]["change_id"], "not-revise");
    }
}
// CODEGEN-END
