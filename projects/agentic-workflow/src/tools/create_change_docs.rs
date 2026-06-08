// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_docs/preamble-definitions.md#source
// CODEGEN-BEGIN
//! Create tools for change-docs.
//!
//! - `sdd_workflow_create_change_docs` — resolve doc targets, build doc-writer prompt, dispatch agent
//! - `sdd_artifact_create_change_docs` — write guide sections to output file

use crate::models::change::SddConfig;
use crate::models::state::StatePhase;
use crate::models::WorkflowArtifact;
use crate::tools::workflow_common;
use crate::tools::{get_required_string, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

// ─── Tool Definitions ────────────────────────────────────────────────────────

/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn workflow_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_create_change_docs".to_string(),
        description:
            "Orchestrate docs creation: resolve target guides, build doc-writer prompt, dispatch agent"
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
        name: "sdd_artifact_create_change_docs".to_string(),
        description: "Write updated guide sections to output_dir for matched doc targets"
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

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_docs/workflow.md#source
// CODEGEN-BEGIN
/// Execute sdd_workflow_create_change_docs.
///
/// Resolves matched doc targets from `[agentic_workflow.docs]` config, builds doc-writer
/// prompt, and dispatches sdd-doc-writer agent.
/// @spec projects/agentic-workflow/tech-design/core/tools/create_change_docs/workflow.md#source
pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;
    let interface = workflow_common::load_interface(project_root);

    // Load docs config — if absent, skip docs phase
    let docs_config = match SddConfig::load_validated(project_root) {
        Ok(config) => config.docs,
        Err(_) => None,
    };

    let docs_config = match docs_config {
        Some(dc) => dc,
        None => {
            // No [agentic_workflow.docs] config — skip docs phase, advance to merge
            let na = workflow_common::next_action(
                interface,
                "sdd_workflow_create_change_merge",
                json!({"change_id": change_id}),
            );
            let result = json!({
                "status": "skip",
                "skip_reason": "No [agentic_workflow.docs] config section — docs phase skipped.",
                "next_actions": [na]
            });
            // Advance phase directly to ChangeMergeCreated
            workflow_common::update_phase(&change_dir, StatePhase::ChangeMergeCreated)?;
            return Ok(serde_json::to_string_pretty(&result)?);
        }
    };

    // Resolve matched targets: intersect config targets with change-affected crates
    let affected_crates = resolve_affected_crates(&change_dir);
    let matched_targets: Vec<Value> = docs_config
        .targets
        .iter()
        .filter(|t| affected_crates.contains(&t.crate_name))
        .map(|t| {
            json!({
                "crate": t.crate_name,
                "guide": t.guide,
                "sections": t.sections,
                "audience": t.audience,
            })
        })
        .collect();

    if matched_targets.is_empty() {
        // No crate intersection — skip docs phase
        let na = workflow_common::next_action(
            interface,
            "sdd_workflow_create_change_merge",
            json!({"change_id": change_id}),
        );
        let result = json!({
            "status": "skip",
            "skip_reason": "No matching crates between [agentic_workflow.docs] targets and change-affected crates.",
            "next_actions": [na]
        });
        workflow_common::update_phase(&change_dir, StatePhase::ChangeMergeCreated)?;
        return Ok(serde_json::to_string_pretty(&result)?);
    }

    // Build doc-writer prompt
    let group_id = workflow_common::resolve_single_group_id(&change_dir);
    let prompt = build_create_docs_prompt(
        &change_id,
        &matched_targets,
        group_id.as_deref(),
        project_root,
    );

    let executor =
        workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateChangeDocs);

    let extra = json!({
        "targets": matched_targets,
    });

    workflow_common::build_workflow_response(
        &change_dir,
        &change_id,
        "create_change_docs",
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

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_docs/artifact.md#source
// CODEGEN-BEGIN
/// Execute sdd_artifact_create_change_docs.
///
/// Writes sections_content map to guide_path file. Merges new sections into
/// existing guide (preserves unchanged sections). Updates STATE.yaml phase
/// to DocsCreated.
/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let target_crate = get_required_string(args, "target_crate")?;
    let guide_path = get_required_string(args, "guide_path")?;
    let summary = get_required_string(args, "summary")?;
    let sections_content = args
        .get("sections_content")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("Missing required object field: sections_content"))?;

    workflow_common::validate_change_id(&change_id)?;
    let interface = workflow_common::load_interface(project_root);

    let _change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    let abs_guide_path = project_root.join(&guide_path);

    // Ensure parent directory exists
    if let Some(parent) = abs_guide_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Read existing guide content (if any)
    let existing_content = if abs_guide_path.exists() {
        std::fs::read_to_string(&abs_guide_path)?
    } else {
        String::new()
    };

    // Merge sections: replace existing sections, append new ones
    let merged = merge_guide_sections(&existing_content, sections_content);
    std::fs::write(&abs_guide_path, &merged)?;

    let sections_updated: Vec<String> = sections_content.keys().cloned().collect();

    // Phase advance moved to `score workflow validate` (three-role-contract R8).

    let na = workflow_common::next_action(
        interface,
        "sdd_workflow_review_change_docs",
        json!({"change_id": change_id}),
    );

    let result = json!({
        "status": "ok",
        "artifacts_written": [guide_path],
        "guide_path": guide_path,
        "target_crate": target_crate,
        "sections_updated": sections_updated,
        "summary": summary,
        "next_actions": [na]
    });
    Ok(serde_json::to_string_pretty(&result)?)
}
// CODEGEN-END
// ─── Helpers ─────────────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_docs/helpers.md#source
// CODEGEN-BEGIN
/// Resolve crates affected by this change from spec files.
///
/// Reads spec files to determine which crates are referenced in the change.
/// Falls back to inspecting the `changes` YAML blocks in spec files.
fn resolve_affected_crates(change_dir: &Path) -> Vec<String> {
    let mut crates = Vec::new();

    // Check groups/*/specs/ and specs/ for spec files
    let spec_dirs: Vec<std::path::PathBuf> = {
        let mut dirs = vec![change_dir.join("specs")];
        let groups_dir = change_dir.join("groups");
        if groups_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&groups_dir) {
                for entry in entries.flatten() {
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        dirs.push(entry.path().join("specs"));
                    }
                }
            }
        }
        dirs
    };

    for spec_dir in spec_dirs {
        if !spec_dir.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&spec_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "md").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        // Extract crate names from change paths like "crates/cclab-xxx/..."
                        for line in content.lines() {
                            if let Some(rest) = line.trim().strip_prefix("- path: crates/") {
                                if let Some(crate_name) = rest.split('/').next() {
                                    let name = crate_name.to_string();
                                    if !crates.contains(&name) {
                                        crates.push(name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    crates
}

/// Merge new sections into existing guide content.
///
/// For each section in `sections_content`, if a matching `## section_name`
/// heading exists in the guide, replace that section's content. Otherwise
/// append the new section at the end.
fn merge_guide_sections(existing: &str, sections: &serde_json::Map<String, Value>) -> String {
    let mut result = existing.to_string();

    for (section_name, content) in sections {
        let content_str = content.as_str().unwrap_or("");
        let heading = format!("## {}", section_name);

        if let Some(start) = result.find(&heading) {
            // Find the end of this section (next ## heading or end of file)
            let after_heading = start + heading.len();
            let section_end = result[after_heading..]
                .find("\n## ")
                .map(|pos| after_heading + pos)
                .unwrap_or(result.len());

            // Replace section content
            let new_section = format!("{}\n\n{}\n", heading, content_str.trim());
            result = format!(
                "{}{}{}",
                &result[..start],
                new_section,
                &result[section_end..]
            );
        } else {
            // Append new section
            if !result.ends_with('\n') {
                result.push('\n');
            }
            result.push_str(&format!("\n{}\n\n{}\n", heading, content_str.trim()));
        }
    }

    result
}

/// Build doc-writer prompt for creating docs.
fn build_create_docs_prompt(
    change_id: &str,
    targets: &[Value],
    group_id: Option<&str>,
    _project_root: &Path,
) -> String {
    let targets_summary: Vec<String> = targets
        .iter()
        .map(|t| {
            let crate_name = t["crate"].as_str().unwrap_or("unknown");
            let guide = t["guide"].as_str().unwrap_or("unknown");
            let audience = t["audience"].as_str().unwrap_or("developer");
            let sections: Vec<&str> = t["sections"]
                .as_array()
                .map(|arr| arr.iter().filter_map(|s| s.as_str()).collect())
                .unwrap_or_default();
            format!(
                "- Crate: `{}`, Guide: `{}`, Audience: {}, Sections: [{}]",
                crate_name,
                guide,
                audience,
                sections.join(", ")
            )
        })
        .collect();

    let spec_path_prefix = match group_id {
        Some(gid) => format!(".aw/changes/{}/groups/{}/specs", change_id, gid),
        None => format!(".aw/changes/{}/specs", change_id),
    };

    format!(
        r#"# Task: Create Docs for Change '{change_id}'

## Instructions

1. Read all change specs in `{spec_path_prefix}/`
2. Read existing guide files (if they exist)
3. For each matched doc target below, generate/update the specified sections
4. Write each target's sections via the artifact CLI command

## Matched Doc Targets

{targets_list}

## Guidelines

- Write clear, accurate documentation based on the change specs
- Match the audience level (developer = technical detail, end-user = usage-focused, admin = deployment/config)
- Preserve existing guide content for sections not being updated
- Include CLI examples where relevant
- Reference actual command names and parameters from the specs

## CLI Commands

```
# Read change specs
Glob pattern: {spec_path_prefix}/*.md

# Write docs artifact (write payload JSON first, then run)
score artifact create-change-docs {change_id} .aw/changes/{change_id}/payloads/create-change-docs.json
```"#,
        targets_list = targets_summary.join("\n"),
    )
}
// CODEGEN-END
