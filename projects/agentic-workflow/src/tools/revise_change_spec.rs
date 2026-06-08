// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/revise_change_spec/preamble-definitions.md#source
// CODEGEN-BEGIN
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
// CODEGEN-END
// ─── Workflow ─────────────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/revise_change_spec/workflow.md#source
// CODEGEN-BEGIN
/// Execute sdd_workflow_revise_change_spec.
///
/// Resolves which spec needs revision via `resolve_next_spec()`, then
/// re-enters the fill loop for flagged sections.
/// @spec projects/agentic-workflow/tech-design/core/tools/revise_change_spec/workflow.md#source
pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;
    let interface = workflow_common::load_interface(project_root);

    let group_id = workflow_common::resolve_single_group_id(&change_dir);

    match common::resolve_next_spec(&change_dir, &change_id)? {
        common::SpecSubState::Revise { spec_id } => {
            handle_revise_sub_state(
                &change_id,
                &spec_id,
                &change_dir,
                group_id.as_deref(),
                project_root,
            )
            .await
        }
        _ => {
            // Not in revise sub-state — redirect back to create router
            let result = json!({
                "status": "ok",
                "message": "Spec is not in Revise sub-state. Redirecting to router.",
                "next_actions": [
                    workflow_common::next_action(interface, "sdd_workflow_create_change_spec", json!({"change_id": change_id}))
                ]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
    }
}
// CODEGEN-END
// ─── Artifact ─────────────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/revise_change_spec/artifact.md#source
// CODEGEN-BEGIN
/// Execute sdd_artifact_revise_change_spec.
///
/// Delegates to `create::execute_artifact()` — same write behavior.
/// @spec projects/agentic-workflow/tech-design/core/tools/revise_change_spec/artifact.md#source
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    let result = create::execute_artifact(args, project_root)?;

    // Increment revision count so auto-approve (threshold >= 1) triggers on next review.
    let change_id = get_required_string(args, "change_id")?;
    let spec_id = get_required_string(args, "spec_id")?;
    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    let rev_key = format!("spec:{}", spec_id);
    if let Ok(mut sm) = crate::state::StateManager::load(&change_dir) {
        sm.increment_revision_count(&rev_key);
        let _ = sm.save();
    }

    Ok(result)
}
// CODEGEN-END
// ─── Internal ─────────────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/revise_change_spec/revision-loop.md#source
// CODEGEN-BEGIN
/// Handle the revise sub-state: re-enter fill loop for flagged sections.
async fn handle_revise_sub_state(
    change_id: &str,
    spec_id: &str,
    change_dir: &Path,
    group_id: Option<&str>,
    project_root: &Path,
) -> Result<String> {
    // Use group-aware path lookup (checks groups/*/specs/ first, then specs/)
    let spec_path = common::find_spec_path(change_dir, spec_id);
    let content = std::fs::read_to_string(&spec_path)?;

    // Read problem_sections from frontmatter (set by review artifact tool)
    let problem_sections = read_problem_sections(&content);

    if problem_sections.is_empty() {
        // No specific sections flagged — revise overview + requirements by default
        let prompt = build_revise_prompt(
            change_id,
            spec_id,
            &["overview", "requirements"],
            group_id,
            project_root,
        )
        .await?;
        return Ok(prompt);
    }

    // Check which flagged sections are not yet re-filled
    let filled_sections = common::read_filled_sections(&content);
    let next_section = problem_sections
        .iter()
        .find(|s| !filled_sections.contains(s));

    if let Some(section) = next_section {
        create::build_fill_prompt(change_id, spec_id, section, group_id, project_root).await
    } else {
        // All problem sections re-filled — strip old review, mark complete
        let stripped = review_helpers::strip_review_section(&content);
        let marked = review_helpers::upsert_frontmatter_field(&stripped, "create_complete", "true");
        // Remove problem_sections and filled_sections (clean state)
        let cleaned = review_helpers::remove_frontmatter_field(&marked, "problem_sections");
        let cleaned = review_helpers::remove_frontmatter_field(&cleaned, "filled_sections");
        // Append fresh Reviews section
        let final_content = format!("{}\n\n# Reviews\n", cleaned.trim_end());
        std::fs::write(&spec_path, &final_content)?;

        // Update phase to revised
        workflow_common::update_phase(change_dir, StatePhase::ChangeSpecRevised)?;

        // Redirect back to workflow router (will go to review)
        let interface = workflow_common::load_interface(project_root);
        let result = json!({
            "status": "ok",
            "spec_id": spec_id,
            "message": "Revision complete. Flagged sections re-filled.",
            "next_actions": [
                workflow_common::next_action(interface, "sdd_workflow_create_change_spec", json!({"change_id": change_id}))
            ]
        });
        Ok(serde_json::to_string_pretty(&result)?)
    }
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/revise_change_spec/frontmatter-and-prompt.md#source
// CODEGEN-BEGIN
/// Read `problem_sections` from spec frontmatter.
fn read_problem_sections(content: &str) -> Vec<String> {
    if !content.starts_with("---\n") {
        return vec![];
    }
    let closing = match content[4..].find("\n---") {
        Some(pos) => 4 + pos,
        None => return vec![],
    };
    let fm = &content[4..closing];

    let mut in_field = false;
    let mut sections = Vec::new();
    for line in fm.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("problem_sections:") {
            in_field = true;
            let after = trimmed.trim_start_matches("problem_sections:").trim();
            if after.starts_with('[') && after.ends_with(']') {
                let inner = &after[1..after.len() - 1];
                for item in inner.split(',') {
                    let s = item.trim().trim_matches('"').trim_matches('\'');
                    if !s.is_empty() {
                        sections.push(s.to_string());
                    }
                }
                return sections;
            }
            continue;
        }
        if in_field {
            if trimmed.starts_with("- ") {
                let item = trimmed
                    .trim_start_matches("- ")
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                sections.push(item.to_string());
            } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                break;
            }
        }
    }
    sections
}

/// Build REVISE prompt for multiple flagged sections.
async fn build_revise_prompt(
    change_id: &str,
    spec_id: &str,
    problem_sections: &[&str],
    group_id: Option<&str>,
    project_root: &Path,
) -> Result<String> {
    let _pp = project_root.display();
    let sections_list = problem_sections.join(", ");

    let prompt = format!(
        r#"# Task: Revise Spec '{spec_id}' for Change '{change_id}'

## Instructions

1. Read the spec and its review:
   `.aw/changes/{change_id}/specs/{spec_id}.md`
2. Address review issues in these sections: {sections_list}
3. Run `score artifact revise-change-spec` for each section that needs revision

## CLI Commands

```
# Read spec
Read file: .aw/changes/{change_id}/specs/{spec_id}.md

# Write revised section (write payload JSON first, then run)
score artifact revise-change-spec {change_id} .aw/changes/{change_id}/payloads/revise-change-spec.json
```"#,
    );

    let change_dir = super::workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor =
        workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviseChangeSpec);

    let mut extra = json!({ "spec_id": spec_id });
    if let Some(gid) = group_id {
        extra["group_id"] = json!(gid);
    }

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        &format!("revise_spec_{}", spec_id),
        prompt,
        executor,
        extra,
        interface,
        project_root,
    )
    .await
}
// CODEGEN-END
