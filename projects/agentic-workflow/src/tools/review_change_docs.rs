//! Review tools for change-docs.
//!
//! - `sdd_workflow_review_change_docs` — build doc-reviewer prompt with accuracy checklist
//! - `sdd_artifact_review_change_docs` — write verdict + review_notes

use crate::models::WorkflowArtifact;
use crate::state::StateManager;
use crate::tools::workflow_common;
use crate::tools::{get_optional_string, get_required_string, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

// ─── Tool Definitions ────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/review_change_docs/definitions.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn workflow_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_review_change_docs".to_string(),
        description:
            "Orchestrate docs review: build doc-reviewer prompt with accuracy checklist, dispatch agent"
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

/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn artifact_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_artifact_review_change_docs".to_string(),
        description: "Write doc review verdict with inline annotations".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "verdict", "review_notes"],
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
                "verdict": {
                    "type": "string",
                    "enum": ["APPROVED", "REVIEWED", "REJECTED"],
                    "description": "Review verdict"
                },
                "review_notes": {
                    "type": "string",
                    "description": "Structured review with accuracy findings, completeness gaps, audience issues"
                },
                "cli_verification_results": {
                    "type": "array",
                    "description": "Results of CLI command verification against documented behavior",
                    "items": {
                        "type": "object",
                        "properties": {
                            "command": { "type": "string" },
                            "expected": { "type": "string" },
                            "actual": { "type": "string" },
                            "pass": { "type": "boolean" }
                        }
                    }
                }
            }
        }),
    }
}
// CODEGEN-END
// ─── Workflow ─────────────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/review_change_docs/workflow.md#source
// CODEGEN-BEGIN
/// Execute sdd_workflow_review_change_docs.
///
/// Builds doc-reviewer prompt with review checklist and dispatches
/// sdd-doc-reviewer agent.
/// @spec projects/agentic-workflow/tech-design/core/tools/review_change_docs/workflow.md#source
pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;
    let interface = workflow_common::load_interface(project_root);

    let group_id = workflow_common::resolve_single_group_id(&change_dir);
    let prompt = build_review_docs_prompt(&change_id, group_id.as_deref(), project_root);

    let executor =
        workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviewChangeDocs);

    let extra = json!({
        "review_checklist": {
            "hard": [
                "Documented CLI commands produce correct output when executed",
                "All change-spec requirements are reflected in the guide",
                "No regression — existing documented features are still accurate"
            ],
            "soft": [
                "Audience-appropriate tone and detail level",
                "Includes practical examples for key workflows",
                "Logical flow and section organization"
            ]
        }
    });

    workflow_common::build_workflow_response(
        &change_dir,
        &change_id,
        "review_change_docs",
        prompt,
        executor,
        extra,
        interface,
        project_root,
    )
    .await
}
// CODEGEN-END
// ─── Artifact ─────────────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/review_change_docs/artifact.md#source
// CODEGEN-BEGIN
/// Execute sdd_artifact_review_change_docs.
///
/// Writes verdict (APPROVED/REVIEWED/REJECTED) + review_notes.
/// Stores cli_verification_results. Updates STATE.yaml phase to DocsReviewed.
/// On APPROVED → next_action points to sdd_workflow_create_change_merge.
/// On REVIEWED/REJECTED → next_action points to sdd_workflow_revise_change_docs.
/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let verdict = get_required_string(args, "verdict")?;
    let review_notes = get_required_string(args, "review_notes")?;
    let cli_results = args
        .get("cli_verification_results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let _caller = get_optional_string(args, "caller").unwrap_or_else(|| "doc-reviewer".to_string());

    workflow_common::validate_change_id(&change_id)?;
    let interface = workflow_common::load_interface(project_root);

    if !["APPROVED", "REVIEWED", "REJECTED"].contains(&verdict.as_str()) {
        anyhow::bail!("verdict must be 'APPROVED', 'REVIEWED', or 'REJECTED'");
    }

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);

    // Write review file
    let review_dir = change_dir.join("docs_review");
    std::fs::create_dir_all(&review_dir)?;

    let review_content = build_review_file(&verdict, &review_notes, &cli_results);
    let review_path = review_dir.join("review.md");
    std::fs::write(&review_path, &review_content)?;

    // Phase advance moved to `score workflow validate` (three-role-contract R8).

    // Check for auto-approve: if revision count >= 1 and verdict is REVIEWED, auto-approve
    let sm = StateManager::load(&change_dir)?;
    let rev_count = sm.revision_count("docs") as u64;
    drop(sm);

    let auto_approved = verdict != "APPROVED" && rev_count >= 1;

    let (next_tool, effective_verdict) = if verdict == "APPROVED" || auto_approved {
        (
            "sdd_workflow_create_change_merge",
            if auto_approved {
                "AUTO_APPROVED"
            } else {
                "APPROVED"
            },
        )
    } else {
        ("sdd_workflow_revise_change_docs", verdict.as_str())
    };

    let na = workflow_common::next_action(interface, next_tool, json!({"change_id": change_id}));

    let result = json!({
        "status": "ok",
        "verdict": effective_verdict,
        "review_path": format!(".aw/changes/{}/docs_review/review.md", change_id),
        "cli_verification_results": cli_results,
        "auto_approved": auto_approved,
        "revision_count": rev_count,
        "next_actions": [na]
    });
    Ok(serde_json::to_string_pretty(&result)?)
}
// CODEGEN-END
// ─── Helpers ─────────────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/review_change_docs/review-file.md#source
// CODEGEN-BEGIN
/// Build review file content.
fn build_review_file(verdict: &str, review_notes: &str, cli_results: &[Value]) -> String {
    let mut md = String::new();
    md.push_str("# Docs Review\n\n");
    md.push_str(&format!("verdict: {}\n\n", verdict));
    md.push_str("## Review Notes\n\n");
    md.push_str(review_notes);
    md.push_str("\n\n");

    if !cli_results.is_empty() {
        md.push_str("## CLI Verification Results\n\n");
        md.push_str("| Command | Expected | Actual | Pass |\n");
        md.push_str("|---------|----------|--------|------|\n");
        for result in cli_results {
            let cmd = result.get("command").and_then(|v| v.as_str()).unwrap_or("");
            let expected = result
                .get("expected")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let actual = result.get("actual").and_then(|v| v.as_str()).unwrap_or("");
            let pass = result
                .get("pass")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let icon = if pass { "PASS" } else { "FAIL" };
            md.push_str(&format!(
                "| `{}` | {} | {} | {} |\n",
                cmd, expected, actual, icon
            ));
        }
        md.push('\n');
    }

    md
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/review_change_docs/prompt.md#source
// CODEGEN-BEGIN
/// Build doc-reviewer prompt.
fn build_review_docs_prompt(
    change_id: &str,
    group_id: Option<&str>,
    _project_root: &Path,
) -> String {
    let spec_path_prefix = match group_id {
        Some(gid) => format!(".aw/changes/{}/groups/{}/specs", change_id, gid),
        None => format!(".aw/changes/{}/specs", change_id),
    };

    format!(
        r#"# Task: Review Docs for Change '{change_id}'

## Instructions

1. Read all change specs in `{spec_path_prefix}/`
2. Read the generated guide files
3. Verify accuracy by executing documented CLI commands
4. Evaluate ALL checklist items below
5. Write review via the artifact CLI command

## Review Checklist

### Hard Checklist (MUST ALL PASS for APPROVED)

- [HARD] Documented CLI commands produce correct output when executed
- [HARD] All change-spec requirements are reflected in the guide
- [HARD] No regression — existing documented features are still accurate

### Soft Checklist (Issues -> REVIEWED verdict)

- Audience-appropriate tone and detail level
- Includes practical examples for key workflows
- Logical flow and section organization

## Verdict Guidelines

- **APPROVED**: All hard checklist items pass, docs are accurate and complete
- **REVIEWED**: Hard checklist passes but has fixable soft issues
- **REJECTED**: Any hard checklist item fails

## CLI Commands

```
# Read specs
Glob pattern: {spec_path_prefix}/*.md

# Write review (write payload JSON first, then run)
score artifact review-change-docs {change_id} .aw/changes/{change_id}/payloads/review-change-docs.json
```"#,
    )
}
// CODEGEN-END
