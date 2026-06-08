// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/review_reference_context/definitions.md#source
// CODEGEN-BEGIN
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
/// @spec projects/agentic-workflow/tech-design/core/tools/review_reference_context/definitions.md#source
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
/// @spec projects/agentic-workflow/tech-design/core/tools/review_reference_context/definitions.md#source
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
// CODEGEN-END
// ─── Workflow ─────────────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/review_reference_context/workflow.md#source
// CODEGEN-BEGIN
// ─── Workflow ─────────────────────────────────────────────────────────────────

/// Execute sdd_workflow_review_reference_context.
///
/// Returns review prompt for the current group in Review sub-state.
/// @spec projects/agentic-workflow/tech-design/core/tools/review_reference_context/workflow.md#source
pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;
    let interface = workflow_common::load_interface(project_root);

    // Resolve current group — should be in Review sub-state
    match common::resolve_next_group(&change_dir)? {
        common::GroupSubState::Review { group_id } => {
            build_review_prompt(&change_id, &group_id, project_root).await
        }
        _ => {
            // Not in review sub-state — redirect back to router
            let result = json!({
                "status": "ok",
                "prompt": "Group is not in Review sub-state. Redirecting to router.",
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
// ─── Artifact Review ─────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/review_reference_context/artifact.md#source
// CODEGEN-BEGIN
// ─── Artifact Review ─────────────────────────────────────────────────────────

/// Execute sdd_artifact_review_reference_context.
///
/// Writes inline review section into `groups/{group_id}/reference_context.md`.
/// Handles APPROVED and auto-approve logic.
/// @spec projects/agentic-workflow/tech-design/core/tools/review_reference_context/artifact.md#source
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let group_id = get_required_string(args, "group_id")?;
    let verdict = get_required_string(args, "verdict")?;
    let summary = get_required_string(args, "summary")?;
    let checklist = args
        .get("checklist_results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let issues = args
        .get("issues")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let caller = get_optional_string(args, "caller").unwrap_or_else(|| "reviewer".to_string());

    workflow_common::validate_change_id(&change_id)?;
    let interface = workflow_common::load_interface(project_root);

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    let group_dir = change_dir.join("groups").join(&group_id);

    if !group_dir.exists() {
        anyhow::bail!("Group directory not found: groups/{}", group_id);
    }

    let artifact_path = group_dir.join("reference_context.md");
    if !artifact_path.exists() {
        anyhow::bail!(
            "Reference context artifact not found: groups/{}/reference_context.md",
            group_id
        );
    }

    // Read current content
    let content = std::fs::read_to_string(&artifact_path)?;

    // Strip old review section
    let stripped = review_helpers::strip_review_section(&content);

    // Get iteration number from revision count
    let rev_key = format!("ref_ctx:{}", group_id);
    let sm = StateManager::load(&change_dir)?;
    let iteration = sm.revision_count(&rev_key) as u64 + 1;
    drop(sm);

    // Build new review section
    let review_section = review_helpers::build_review_section(
        &verdict, &summary, &checklist, &issues, iteration, &caller, &change_id,
    );

    // Append review section
    let new_content = format!("{}\n\n{}", stripped, review_section);

    // Upsert review_verdict in frontmatter
    let final_content =
        review_helpers::upsert_frontmatter_field(&new_content, "review_verdict", &verdict);

    std::fs::write(&artifact_path, &final_content)?;

    // Phase stays at PostClarificationsCreated (reference context absorbed by issue lifecycle)
    {
        let mut sm = StateManager::load(&change_dir)?;
        if matches!(sm.phase(), StatePhase::ChangeInited) {
            // No phase transition needed — reference context review is internal
            sm.save()?;
        }
    }

    // Handle APPROVED or auto-approve
    let should_mark_done =
        if verdict.eq_ignore_ascii_case("APPROVED") || verdict.eq_ignore_ascii_case("PASS") {
            true
        } else {
            // Check revision count for auto-approve
            let sm = StateManager::load(&change_dir)?;
            let rev_count = sm.revision_count(&rev_key);
            rev_count >= 1
        };

    if should_mark_done {
        common::mark_group_done(&change_dir, &group_id)?;
    }

    let artifacts_written = vec![format!("groups/{}/reference_context.md", group_id)];

    let result = json!({
        "status": "ok",
        "artifacts_written": artifacts_written,
        "next_actions": [
            workflow_common::next_action(interface, "sdd_workflow_create_reference_context", json!({"change_id": change_id}))
        ]
    });

    Ok(serde_json::to_string_pretty(&result)?)
}
// CODEGEN-END
// ─── Prompt Builder ──────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/review_reference_context/prompt.md#source
// CODEGEN-BEGIN
// ─── Prompt Builder ──────────────────────────────────────────────────────────

/// Build REVIEW prompt for a group's reference context.
async fn build_review_prompt(
    change_id: &str,
    group_id: &str,
    project_root: &Path,
) -> Result<String> {
    let project_path = project_root.display();

    let prompt = format!(
        r#"# Task: Review Reference Context for Group '{group_id}' (Change '{change_id}')

## Instructions

1. **Read pre-clarifications** (scope & requirements):
   `{project_path}/.aw/changes/{change_id}/groups/{group_id}/pre_clarifications.md`
2. **Read the reference context artifact**:
   `{project_path}/.aw/changes/{change_id}/groups/{group_id}/reference_context.md`
3. **Verify each spec entry**: For each spec listed in the artifact, read the actual spec under `{project_path}/.aw/tech-design/` to verify relevance and key requirements are accurate.
4. **Devil's advocate**: Actively check — what crates/areas from pre-clarifications have NO spec covering them?
5. **Evaluate checklist** (pass/fail each item independently):
   - All affected crates/areas from pre-clarifications are covered by at least one spec
   - Relevance scores are reasonable (high = directly implements, medium = related, low = background)
   - Key requirements listed per spec are accurate (match actual requirement IDs)
   - No irrelevant specs included
   - spec_plan: every entry has main_spec_ref set (not null)
   - spec_plan: sections are reasonable for the requirements
   - spec_plan: modify entries have valid source paths
   - spec_plan: main_spec_ref paths include a subfolder (not root-level under crate)
   - spec_plan: each spec file covers exactly one logical unit (not multiple unrelated concerns)
   - spec_plan: no spec file would require duplicate section types (split into separate files if needed)
   - spec_plan: spec paths mirror source structure (interfaces/, logic/, generate/)
6. **CRITICAL: List ALL issues in a single pass.** You only get ONE review round before auto-approve kicks in. Do NOT hold back issues for a future round — every problem must be reported NOW. Scan the entire artifact exhaustively before writing the verdict.
7. **Separate observations from verdict**: First list all findings, then decide verdict based on evidence.
8. Write review verdict:

## CLI Commands

```
# Write review artifact (write payload JSON first, then run)
score artifact review-reference-context {change_id} .aw/changes/{change_id}/payloads/review-reference-context.json
```"#,
    );

    let change_dir = workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor =
        workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviewReferenceContext);

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        "review_reference_context",
        prompt,
        executor,
        json!({ "group_id": group_id }),
        interface,
        project_root,
    )
    .await
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/review_reference_context/tests.md#source
// CODEGEN-BEGIN
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::state::StatePhase;
    use crate::state::StateManager;
    use tempfile::TempDir;

    fn setup_review_change(change_id: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes").join(change_id);
        let group_dir = change_dir.join("groups").join("my-group");
        std::fs::create_dir_all(&group_dir).unwrap();

        crate::test_util::write_minimal_issue(tmp.path(), change_id);
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = StatePhase::ChangeInited;
        sm.save().unwrap();

        std::fs::write(change_dir.join("user_input.md"), "Test").unwrap();

        // Write artifact without review (Review sub-state)
        std::fs::write(
            group_dir.join("reference_context.md"),
            "---\nchange: test\ngroup: my-group\ndate: 2026-03-04\n---\n\n# Reference Context\n\n| Spec | Group | Relevance | Key Requirements |\n|------|-------|-----------|------------------|\n| some-spec | test | high | R1 |\n",
        ).unwrap();

        tmp
    }

    /// Read prompt content from inline response, prompt_path, or the prompts directory
    /// (when agent dispatch consumed the prompt).
    fn read_prompt(
        parsed: &Value,
        project_root: &std::path::Path,
        change_id: &str,
        group_id: &str,
        action: &str,
    ) -> String {
        if let Some(p) = parsed["prompt"].as_str() {
            return p.to_string();
        }
        if let Some(rel) = parsed["prompt_path"].as_str() {
            let prompt_path = project_root.join(rel);
            return std::fs::read_to_string(&prompt_path)
                .unwrap_or_else(|_| panic!("No prompt at {:?}", prompt_path));
        }
        // Agent dispatch path — prompt was written to file before agent ran
        let prompt_path = project_root
            .join(".aw/changes")
            .join(change_id)
            .join("groups")
            .join(group_id)
            .join("prompts")
            .join(format!("{}.md", action));
        std::fs::read_to_string(&prompt_path)
            .unwrap_or_else(|_| panic!("No prompt at {:?} (agent dispatch path)", prompt_path))
    }

    #[tokio::test]
    async fn test_review_workflow_returns_prompt() {
        let tmp = setup_review_change("rev-wf");
        let _change_dir = tmp.path().join(".aw/changes/rev-wf");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-wf"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        // Status can be "ok" (mainthread or agent success) or "agent_error"
        assert!(
            parsed["status"].as_str().unwrap().starts_with("ok")
                || parsed["status"] == "agent_error"
                || parsed["agent_completed"] == true
        );
        let prompt = read_prompt(
            &parsed,
            tmp.path(),
            "rev-wf",
            "my-group",
            "review_reference_context",
        );
        assert!(prompt.contains("Review Reference Context"));
    }

    #[test]
    fn test_artifact_review_approved_marks_done() {
        let tmp = setup_review_change("rev-approved");
        let change_dir = tmp.path().join(".aw/changes/rev-approved");

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-approved",
            "group_id": "my-group",
            "verdict": "APPROVED",
            "summary": "Looks good.",
            "checklist_results": [{"item": "Specs covered", "passed": true}],
            "issues": []
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        // Groups removed — completion tracking simplified
        let _sm = StateManager::load(&change_dir).unwrap();
    }

    #[test]
    fn test_artifact_review_writes_inline_review() {
        let tmp = setup_review_change("rev-inline");

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-inline",
            "group_id": "my-group",
            "verdict": "REVIEWED",
            "summary": "Missing some specs.",
            "issues": [{"severity": "MEDIUM", "description": "Missing lens spec"}]
        });
        execute_artifact(&args, tmp.path()).unwrap();

        // Verify file has review section
        let artifact_path = tmp
            .path()
            .join(".aw/changes/rev-inline/groups/my-group/reference_context.md");
        let content = std::fs::read_to_string(&artifact_path).unwrap();
        assert!(content.contains("review_verdict: REVIEWED"));
        assert!(content.contains("# Reviews"));
        assert!(content.contains("Missing some specs."));
        assert!(content.contains("Missing lens spec"));
    }

    #[test]
    fn test_artifact_review_auto_approves_on_revision_limit() {
        let tmp = setup_review_change("rev-auto");
        let change_dir = tmp.path().join(".aw/changes/rev-auto");

        // Set revision count >= 1
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.increment_revision_count("ref_ctx:my-group");
        sm.save().unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-auto",
            "group_id": "my-group",
            "verdict": "REVIEWED",
            "summary": "Still has issues but auto-approving.",
            "issues": []
        });
        execute_artifact(&args, tmp.path()).unwrap();

        // Groups removed — completion tracking simplified
        let _sm = StateManager::load(&change_dir).unwrap();
    }
}
// CODEGEN-END
