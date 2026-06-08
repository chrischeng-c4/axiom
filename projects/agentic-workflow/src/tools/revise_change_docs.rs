// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/revise_change_docs/preamble-definitions.md#source
// CODEGEN-BEGIN
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

/// @spec projects/agentic-workflow/tech-design/core/tools/revise_change_docs/preamble-definitions.md#source
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

/// @spec projects/agentic-workflow/tech-design/core/tools/revise_change_docs/preamble-definitions.md#source
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
// CODEGEN-END
// ─── Workflow ─────────────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/revise_change_docs/workflow.md#source
// CODEGEN-BEGIN
/// Execute sdd_workflow_revise_change_docs.
///
/// Builds doc-writer prompt with review feedback included and dispatches
/// sdd-doc-writer agent for revision.
/// @spec projects/agentic-workflow/tech-design/core/tools/revise_change_docs/workflow.md#source
pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;
    let interface = workflow_common::load_interface(project_root);

    let group_id = workflow_common::resolve_single_group_id(&change_dir);

    // Read review feedback
    let review_path = change_dir.join("docs_review/review.md");
    let review_content = if review_path.exists() {
        std::fs::read_to_string(&review_path)?
    } else {
        "No review feedback found.".to_string()
    };

    // Get current revision count
    let sm = StateManager::load(&change_dir)?;
    let rev_count = sm.revision_count("docs") as u64;
    drop(sm);

    let prompt = build_revise_docs_prompt(
        &change_id,
        &review_content,
        rev_count,
        group_id.as_deref(),
        project_root,
    );

    let executor =
        workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviseChangeDocs);

    let extra = json!({
        "revision_count": rev_count + 1,
    });

    workflow_common::build_workflow_response(
        &change_dir,
        &change_id,
        "revise_change_docs",
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

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/revise_change_docs/artifact.md#source
// CODEGEN-BEGIN
/// Execute sdd_artifact_revise_change_docs.
///
/// Delegates to create_change_docs::execute_artifact() for the actual write.
/// Increments revision count in STATE.yaml task_revisions.
/// Updates phase to DocsRevised → next_action points to review.
/// @spec projects/agentic-workflow/tech-design/core/tools/revise_change_docs/artifact.md#source
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    // Delegate write to create artifact
    let result = create_change_docs::execute_artifact(args, project_root)?;

    // Increment docs revision count
    let change_id = get_required_string(args, "change_id")?;
    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);

    if let Ok(mut sm) = StateManager::load(&change_dir) {
        sm.increment_revision_count("docs");
        let _ = sm.save();
    }

    // Override phase to DocsRevised (create artifact set it to DocsCreated)
    workflow_common::update_phase(&change_dir, StatePhase::DocsRevised)?;

    // Parse result and override next_actions to point to review
    let mut parsed: Value = serde_json::from_str(&result)?;
    let interface = workflow_common::load_interface(project_root);
    let na = workflow_common::next_action(
        interface,
        "sdd_workflow_review_change_docs",
        json!({"change_id": change_id}),
    );

    if let Some(obj) = parsed.as_object_mut() {
        obj.insert("next_actions".to_string(), json!([na]));
        // Add revision count to response
        if let Ok(sm) = StateManager::load(&change_dir) {
            let rev_count = sm.revision_count("docs");
            obj.insert("revision_count".to_string(), json!(rev_count));
        }
    }

    Ok(serde_json::to_string_pretty(&parsed)?)
}
// CODEGEN-END
// ─── Prompt Builder ──────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/revise_change_docs/prompt.md#source
// CODEGEN-BEGIN
fn build_revise_docs_prompt(
    change_id: &str,
    review_content: &str,
    revision_count: u64,
    group_id: Option<&str>,
    _project_root: &Path,
) -> String {
    let spec_path_prefix = match group_id {
        Some(gid) => format!(".aw/changes/{}/groups/{}/specs", change_id, gid),
        None => format!(".aw/changes/{}/specs", change_id),
    };

    format!(
        r#"# Task: Revise Docs for Change '{change_id}' (Revision {rev_num})

## Instructions

1. Read the review feedback below
2. Read the current guide files and change specs
3. Address all issues identified in the review
4. Write revised sections via the artifact CLI command

## Review Feedback

{review_content}

## Guidelines

- Fix all accuracy issues identified in the review
- Address completeness gaps
- Improve audience fit where noted
- Re-verify CLI command accuracy after changes

## CLI Commands

```
# Read specs
Glob pattern: {spec_path_prefix}/*.md

# Read review
Read file: .aw/changes/{change_id}/docs_review/review.md

# Write revised docs (write payload JSON first, then run)
score artifact revise-change-docs {change_id} .aw/changes/{change_id}/payloads/revise-change-docs.json
```"#,
        rev_num = revision_count + 1,
    )
}
// CODEGEN-END
