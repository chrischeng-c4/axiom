//! Create tools for reference context.
//!
//! - `sdd_workflow_create_reference_context` — central router for per-group lifecycle
//! - `sdd_artifact_create_reference_context` — writes `groups/{group_id}/reference_context.md`

use super::common_reference_context::{self as common, GroupSubState};
use super::review_helpers;
use crate::models::reference_context_sections::REFERENCE_CONTEXT_SECTIONS;
use crate::models::state::StatePhase;
use crate::models::WorkflowArtifact;
use crate::state::StateManager;
use crate::tools::workflow_common;
use crate::tools::{get_optional_string, get_required_string, ToolDefinition};
use crate::workflow::scope;
use crate::Result;
use chrono::Local;
use serde_json::{json, Value};
use std::path::Path;

// ─── Tool Definitions ────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_reference_context/definitions.md#source
// CODEGEN-BEGIN
/// MCP tool definition for sdd_workflow_create_reference_context
/// @spec projects/agentic-workflow/tech-design/core/logic/remaining-fixes.md#changes
pub fn workflow_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_create_reference_context".to_string(),
        description: "Orchestrate per-group reference context lifecycle (create/review/revise)"
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

/// MCP tool definition for sdd_artifact_create_reference_context
/// @spec projects/agentic-workflow/tech-design/core/logic/remaining-fixes.md#changes
pub fn artifact_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_artifact_create_reference_context".to_string(),
        description: "Write reference context for a group — supports both legacy (full specs array) and section-loop (section + content) modes".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "group_id"],
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
                    "description": "Group ID to write reference context for"
                },
                "section": {
                    "type": "string",
                    "description": "Section name to fill (section-loop mode). One of: source_refs, related_specs, reproductions, related_issues, first_fix"
                },
                "content": {
                    "type": "string",
                    "description": "Section content (section-loop mode). Used with 'section' parameter."
                },
                "specs": {
                    "type": "array",
                    "description": "Legacy mode: full specs array for one-shot write",
                    "items": {
                        "type": "object",
                        "required": ["spec_id", "spec_group", "relevance"],
                        "properties": {
                            "spec_id": {
                                "type": "string",
                                "description": "Spec ID (e.g. 'create-pre-clarifications')"
                            },
                            "spec_group": {
                                "type": "string",
                                "description": "Spec group path (e.g. 'sdd/tools/workflows')"
                            },
                            "relevance": {
                                "type": "string",
                                "enum": ["high", "medium", "low"],
                                "description": "How relevant this spec is to the group"
                            },
                            "key_requirements": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Key requirement IDs from the spec (e.g. ['R1', 'R3'])"
                            }
                        }
                    }
                },
                "spec_plan": {
                    "type": "array",
                    "description": "Optional spec plan entries for this group",
                    "items": {
                        "type": "object",
                        "required": ["spec_id", "action", "main_spec_ref", "sections"],
                        "properties": {
                            "spec_id": {
                                "type": "string",
                                "description": "Spec ID for the change-spec"
                            },
                            "action": {
                                "type": "string",
                                "enum": ["modify", "create"],
                                "description": "Whether to modify existing spec or create new"
                            },
                            "main_spec_ref": {
                                "type": "string",
                                "description": "Target path in .aw/tech-design/ (REQUIRED — must reside in a named subfolder, min 4 path components: {category}/{crate}/{subdir}/{file}.md)"
                            },
                            "source": {
                                "type": "string",
                                "description": "Source path for modify action (relative to .aw/tech-design/)"
                            },
                            "sections": {
                                "type": "array",
                                "items": {
                                    "type": "string",
                                    "enum": [
                                        "overview", "changes",
                                        "rest-api", "rpc-api", "async-api", "cli",
                                        "schema", "logic", "interaction",
                                        "state-machine", "db-model",
                                        "unit-test", "e2e-test", "dependency",
                                        "wireframe", "component", "design-token",
                                        "config", "runtime-image", "deployment",
                                        "e2e-scenario", "test-fixture", "perf-test",
                                        "threat-model", "auth-matrix", "security-test",
                                        "container", "deploy", "cloud-resource",
                                        "pipeline", "observability",
                                        "grpc", "graphql",
                                        "model", "prompt"
                                    ]
                                },
                                "description": "Section types this spec needs. Determined by rule engine + agent input."
                            }
                        }
                    }
                }
            }
        }),
    }
}
// CODEGEN-END
// ─── Workflow Orchestration ──────────────────────────────────────────────────

/// Execute sdd_workflow_create_reference_context.
///
/// Central router: determines next group sub-state and either returns a create
/// prompt or redirects to review/revise workflow tools via `next_actions`.
pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;
    let interface = workflow_common::load_interface(project_root);

    match common::resolve_next_group(&change_dir)? {
        GroupSubState::Create { group_id } => {
            // Initialize section-loop skeleton, then return prompt for first section
            let group_dir = change_dir.join("groups").join(&group_id);
            let artifact_path = group_dir.join("reference_context.md");
            if !artifact_path.exists() {
                let skeleton = common::generate_section_loop_skeleton(
                    &change_id,
                    &group_id,
                    REFERENCE_CONTEXT_SECTIONS,
                );
                std::fs::write(&artifact_path, &skeleton)?;
            }
            // Now find the first section and build its prompt
            let content = std::fs::read_to_string(&artifact_path)?;
            if let Some(section) = common::resolve_next_section(&content) {
                build_section_prompt(&change_id, &group_id, &section, &change_dir, project_root)
                    .await
            } else {
                // All filled — mark complete and redirect
                mark_complete_and_redirect(
                    &change_id,
                    &change_dir,
                    &group_id,
                    project_root,
                    interface,
                )
                .await
            }
        }
        GroupSubState::CreateSection { group_id, section } => {
            build_section_prompt(&change_id, &group_id, &section, &change_dir, project_root).await
        }
        GroupSubState::Review { group_id } => {
            let result = json!({
                "status": "ok",
                "group_id": group_id,
                "next_actions": [
                    workflow_common::next_action(interface, "sdd_workflow_review_reference_context", json!({"change_id": change_id}))
                ]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
        GroupSubState::Revise { group_id } => {
            let result = json!({
                "status": "ok",
                "group_id": group_id,
                "next_actions": [
                    workflow_common::next_action(interface, "sdd_workflow_revise_reference_context", json!({"change_id": change_id}))
                ]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
        GroupSubState::AllDone => {
            // Deduplicate + prepare specs from spec_plan before advancing phase
            let prepared = super::spec_plan::prepare_specs_from_plan(&change_dir, project_root)?;
            if !prepared.is_empty() {
                eprintln!(
                    "[create_reference_context] Prepared {} spec(s) from spec_plan.",
                    prepared.len()
                );
            }

            let mut sm = StateManager::load(&change_dir)?;
            sm.set_phase(StatePhase::ChangeInited)?;
            sm.save()?;

            let result = json!({
                "status": "phase_complete",
                "prompt": "All groups have reference context approved. Phase advanced to PostClarificationsCreated.",
                "group_id": null,
                "next_actions": [
                    workflow_common::next_action(interface, "sdd_run_change", json!({"change_id": change_id}))
                ]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
    }
}

// ─── Artifact Write ──────────────────────────────────────────────────────────

/// Execute sdd_artifact_create_reference_context.
///
/// Supports two modes:
/// 1. **Section-loop** (new): `section` + `content` params — writes one section,
///    updates `filled_sections` frontmatter, returns next_actions to workflow.
/// 2. **Legacy**: `specs` array — writes full reference_context.md in one shot.

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_reference_context/artifact.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/logic/remaining-fixes.md#changes
// @spec projects/agentic-workflow/tech-design/core/logic/reference-context.md#R1
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let group_id = get_required_string(args, "group_id")?;
    workflow_common::validate_change_id(&change_id)?;
    let interface = workflow_common::load_interface(project_root);

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    let group_dir = change_dir.join("groups").join(&group_id);

    if !group_dir.exists() {
        anyhow::bail!("Group directory not found: groups/{}", group_id);
    }

    let artifact_path = group_dir.join("reference_context.md");

    // Detect mode: section-loop vs legacy
    let section = get_optional_string(args, "section");
    let content_arg = get_optional_string(args, "content");

    if let (Some(section), Some(content)) = (section, content_arg) {
        // Section-loop mode
        return execute_artifact_section_loop(
            &change_id,
            &group_id,
            &section,
            &content,
            &artifact_path,
            &change_dir,
            interface,
        );
    }

    // Legacy mode: full specs array
    let specs = args
        .get("specs")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Missing required array field: specs (or use section+content for section-loop mode)"))?;

    if specs.is_empty() {
        anyhow::bail!("specs array must not be empty");
    }

    let spec_plan = args.get("spec_plan").and_then(|v| v.as_array());

    let today = Local::now().format("%Y-%m-%d").to_string();
    let content = common::render_specs_markdown(&change_id, &group_id, &today, specs, spec_plan);
    std::fs::write(&artifact_path, &content)?;

    if let Some(plan_entries) = spec_plan {
        if !plan_entries.is_empty() {
            common::write_spec_plan_yaml(&group_dir, plan_entries)?;
        }
    }

    // Phase stays at PostClarificationsCreated (reference context absorbed by issue lifecycle)
    let mut sm = StateManager::load(&change_dir)?;
    if matches!(sm.phase(), StatePhase::ChangeInited) {
        // Phase stays at PostClarificationsCreated - reference context is now
        // an internal artifact within the issue lifecycle, not a separate phase.
        sm.save()?;
    }

    let artifacts_written = vec![format!("groups/{}/reference_context.md", group_id)];
    let next_actions = json!([workflow_common::next_action(
        interface,
        "sdd_workflow_create_reference_context",
        json!({"change_id": change_id})
    )]);

    let result = json!({
        "status": "ok",
        "artifacts_written": artifacts_written,
        "next_actions": next_actions
    });

    Ok(serde_json::to_string_pretty(&result)?)
}
// CODEGEN-END
/// Section-loop artifact write: replaces one section, updates filled_sections.
// @spec projects/agentic-workflow/tech-design/core/logic/reference-context.md#R5
fn execute_artifact_section_loop(
    change_id: &str,
    group_id: &str,
    section: &str,
    content: &str,
    artifact_path: &Path,
    change_dir: &Path,
    interface: crate::models::SddInterface,
) -> Result<String> {
    use crate::models::reference_context_sections::is_valid_section;

    if !is_valid_section(section) {
        anyhow::bail!(
            "Invalid reference context section '{}'. Valid: {:?}",
            section,
            REFERENCE_CONTEXT_SECTIONS
        );
    }

    // Read existing content
    let current = std::fs::read_to_string(artifact_path).unwrap_or_else(|_| {
        // If file doesn't exist, generate skeleton
        common::generate_section_loop_skeleton(change_id, group_id, REFERENCE_CONTEXT_SECTIONS)
    });

    // Replace the target section (overwrites partial content from crashed agents)
    let updated = common::replace_ref_ctx_section(&current, section, content);

    // Mark section as filled in frontmatter
    let mut filled = common::read_filled_sections(&updated);
    if !filled.contains(&section.to_string()) {
        filled.push(section.to_string());
    }
    let filled_str = format!("[{}]", filled.join(", "));
    let final_content =
        review_helpers::upsert_frontmatter_field(&updated, "filled_sections", &filled_str);

    std::fs::write(artifact_path, &final_content)?;

    // Advance phase
    let mut sm = StateManager::load(change_dir)?;
    if matches!(sm.phase(), StatePhase::ChangeInited) {
        // Phase stays at PostClarificationsCreated — reference context is now
        // an internal artifact within the issue lifecycle, not a separate phase.
        sm.save()?;
    }

    let artifacts_written = vec![format!("groups/{}/reference_context.md", group_id)];
    let next_actions = json!([workflow_common::next_action(
        interface,
        "sdd_workflow_create_reference_context",
        json!({"change_id": change_id})
    )]);

    let result = json!({
        "status": "ok",
        "section_filled": section,
        "filled_sections": filled,
        "artifacts_written": artifacts_written,
        "next_actions": next_actions
    });

    Ok(serde_json::to_string_pretty(&result)?)
}

// ���── Section-Loop Completion ────────────────────────────────────────────────

/// All sections filled: prune TODOs, set create_complete, redirect to review.
// @spec projects/agentic-workflow/tech-design/core/logic/reference-context.md#R1
async fn mark_complete_and_redirect(
    change_id: &str,
    change_dir: &Path,
    group_id: &str,
    _project_root: &Path,
    interface: crate::models::SddInterface,
) -> Result<String> {
    let artifact_path = change_dir
        .join("groups")
        .join(group_id)
        .join("reference_context.md");
    let content = std::fs::read_to_string(&artifact_path)?;

    // Mark create_complete: true
    let marked = review_helpers::upsert_frontmatter_field(&content, "create_complete", "true");
    // Add written_by marker for verification
    let marked = review_helpers::upsert_frontmatter_field(&marked, "written_by", "artifact_cli");
    // Add date
    let today = Local::now().format("%Y-%m-%d").to_string();
    let marked = review_helpers::upsert_frontmatter_field(&marked, "date", &today);
    std::fs::write(&artifact_path, &marked)?;

    // Advance phase
    let mut sm = StateManager::load(change_dir)?;
    if matches!(sm.phase(), StatePhase::ChangeInited) {
        // Phase stays at PostClarificationsCreated — reference context is now
        // an internal artifact within the issue lifecycle, not a separate phase.
        sm.save()?;
    }

    let result = json!({
        "status": "ok",
        "group_id": group_id,
        "message": "All reference context sections filled. Ready for review.",
        "next_actions": [
            workflow_common::next_action(interface, "sdd_workflow_review_reference_context", json!({"change_id": change_id}))
        ]
    });
    Ok(serde_json::to_string_pretty(&result)?)
}

// ─── Prompt Builder ──────────────────────────────────────────────────────────

/// Build per-section prompt for the reference context section-loop.
///
/// Returns a prompt for filling exactly ONE section with cross-section context.
// @spec projects/agentic-workflow/tech-design/core/logic/reference-context.md#R4
async fn build_section_prompt(
    change_id: &str,
    group_id: &str,
    section: &str,
    change_dir: &Path,
    project_root: &Path,
) -> Result<String> {
    let project_path = project_root.display();
    let issues_hint = workflow_common::build_group_issues_hint(change_dir, group_id);

    // Read existing artifact content for cross-section context
    let artifact_path = change_dir
        .join("groups")
        .join(group_id)
        .join("reference_context.md");
    let artifact_content = std::fs::read_to_string(&artifact_path).unwrap_or_default();
    let filled_sections = common::read_filled_sections(&artifact_content);
    let already_filled_summary =
        common::build_already_filled_summary(&artifact_content, &filled_sections);

    // Extract scope for spec filtering
    let scope_info = scope::extract_scope(change_dir);
    let sdd_config = crate::models::SddConfig::load(project_root).ok();
    let filtered_specs =
        scope::pre_filter_specs(&scope_info.spec_groups, project_root, sdd_config.as_ref());
    let has_scoped_specs = !filtered_specs.is_empty();

    let spec_instructions = if has_scoped_specs {
        format!(
            "## In-Scope Specs\n\n\
             {filtered_specs}\n\
             Read these specs using the Read tool (file paths under `{project_path}/.aw/tech-design/`).\n\
             Do NOT explore specs outside the scope above.\n"
        )
    } else {
        format!(
            "## Specs\n\n\
             - List specs under `{project_path}/.aw/tech-design/` using Glob\n\
             - Read at most 5 specs. Focus on the most relevant ones.\n"
        )
    };

    let spec_dir_tree =
        scope::build_spec_dir_tree(&scope_info.spec_groups, project_root, sdd_config.as_ref());
    let spec_structure_block = if !spec_dir_tree.is_empty() {
        format!(
            "## Existing Spec Structure\n\n\
             ```\n{}```\n",
            spec_dir_tree
        )
    } else {
        String::new()
    };

    let section_heading = crate::models::reference_context_sections::section_to_heading(section);
    let section_guidance = match section {
        "source_refs" => "Identify source code files and functions relevant to this change. List file paths, key functions, and why each is relevant.",
        "related_specs" => "Find specs under .aw/tech-design/ that relate to this change. For each spec, note its path, relevance (high/medium/low), and key requirements.",
        "reproductions" => "Document how to reproduce the problem (for bugs) or demonstrate the current behavior gap (for enhancements). Include steps, expected vs actual behavior.",
        "related_issues" => "List related issues from the temp issue working copy that touch the same area. Note how they relate (blocker, duplicate, related, superseded).",
        "first_fix" => "Analyze the most likely approach to implement the change. Identify key files to modify, potential risks, and rough implementation strategy.",
        _ => "Fill this section with relevant context.",
    };

    // Read requirements for context
    let requirements_text = {
        let user_input_path = change_dir.join("user_input.md");
        let group_reqs_path = change_dir
            .join("groups")
            .join(group_id)
            .join("requirements.md");
        if let Ok(s) = std::fs::read_to_string(&group_reqs_path) {
            s
        } else {
            std::fs::read_to_string(&user_input_path).unwrap_or_default()
        }
    };

    // For section-loop: also include spec_plan guidance in related_specs section
    let spec_plan_block = if section == "related_specs" {
        let suggested = super::spec_plan::suggest_sections(&requirements_text);
        let suggested_list = suggested.join(", ");
        format!(
            "\n## Spec Plan Guidance\n\n\
             Include a spec_plan in your content: for each change spec that will be created,\n\
             note spec_id, action (modify/create), main_spec_ref, and sections.\n\
             Suggested sections from keyword analysis: [{suggested_list}]\n\n\
             **Action preference**: Use `action: modify` for files visible in the spec structure above.\n\
             Reserve `action: create` for genuinely new subsystems.\n"
        )
    } else {
        String::new()
    };

    let prompt = format!(
        r#"# Task: Fill Section '{section}' — Reference Context for Group '{group_id}' (Change '{change_id}')

{issues_section}
## CRITICAL: Artifact Writing Rule

**DO NOT use Write or Edit tools to create/modify artifact files directly.**
You MUST use the CLI command below to write section content.

## Your Task

Fill ONLY the **{section_heading}** section. Do NOT fill other sections.

## Section Guidance: {section}

{section_guidance}

{already_filled_block}
{spec_structure_block}
{spec_instructions}
{spec_plan_block}
## Context

1. Read group pre-clarifications: `{project_path}/.aw/changes/{change_id}/groups/{group_id}/pre_clarifications.md`
2. Read the current artifact: `{project_path}/.aw/changes/{change_id}/groups/{group_id}/reference_context.md`
3. Explore relevant specs and source code as needed

## CLI Commands

```
# Write payload JSON file with section + content
Write file: .aw/changes/{change_id}/payloads/create-reference-context.json

# Run artifact CLI with section-loop params
score artifact create-reference-context {change_id} .aw/changes/{change_id}/payloads/create-reference-context.json
```

Payload format:
```json
{{
  "group_id": "{group_id}",
  "section": "{section}",
  "content": "... your section content here ..."
}}
```"#,
        issues_section = if issues_hint.is_empty() {
            String::new()
        } else {
            format!("Issues: {}\n", issues_hint)
        },
        already_filled_block = if already_filled_summary.is_empty() {
            String::new()
        } else {
            already_filled_summary
        },
    );

    let interface = workflow_common::load_interface(project_root);
    let executor =
        workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateReferenceContext);

    workflow_common::build_workflow_response(
        change_dir,
        change_id,
        &format!("create_reference_context_{}", section),
        prompt,
        executor,
        json!({ "group_id": group_id }),
        interface,
        project_root,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    fn setup_group_change(change_id: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes").join(change_id);
        let group_dir = change_dir.join("groups").join("my-group");
        std::fs::create_dir_all(&group_dir).unwrap();

        // R4: save() syncs into the backing issue.
        crate::test_util::write_minimal_issue(tmp.path(), change_id);
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = StatePhase::ChangeInited;
        sm.save().unwrap();

        // Pre-clarifications for group
        std::fs::write(
            group_dir.join("pre_clarifications.md"),
            "---\nchange: test\ngroup: my-group\n---\n\n# Pre-Clarifications\n",
        )
        .unwrap();

        // user_input.md
        std::fs::write(change_dir.join("user_input.md"), "Test change").unwrap();

        tmp
    }

    /// Read prompt content from either inline response or prompt file.
    fn read_prompt(parsed: &Value, project_root: &Path) -> String {
        if let Some(p) = parsed["prompt"].as_str() {
            return p.to_string();
        }
        let rel = parsed["prompt_path"]
            .as_str()
            .expect("Expected prompt_path in response");
        let prompt_path = project_root.join(rel);
        std::fs::read_to_string(&prompt_path)
            .unwrap_or_else(|_| panic!("No inline prompt and no prompt file at {:?}", prompt_path))
    }

    #[tokio::test]
    async fn test_workflow_returns_create_prompt() {
        let tmp = setup_group_change("ref-test");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "ref-test"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["group_id"], "my-group");
        let prompt = read_prompt(&parsed, tmp.path());
        // Section-loop: prompt is now per-section, starting with source_refs
        assert!(prompt.contains("Fill Section"));
        assert!(prompt.contains("source_refs"));
    }

    #[tokio::test]
    async fn test_workflow_redirects_to_review() {
        let tmp = setup_group_change("rev-test");
        let group_dir = tmp.path().join(".aw/changes/rev-test/groups/my-group");

        // Write artifact with create_complete (section-loop done) but no review verdict
        std::fs::write(
            group_dir.join("reference_context.md"),
            "---\nchange: rev-test\ngroup: my-group\ndate: 2026-03-04\ncreate_complete: true\n---\n\n# Reference Context\n\nContent here.\n",
        )
        .unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "rev-test"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        // Should redirect to review workflow
        let next = &parsed["next_actions"][0];
        assert_eq!(next["args"]["change_id"], "rev-test");
    }

    #[tokio::test]
    async fn test_workflow_redirects_to_revise() {
        let tmp = setup_group_change("revise-test");
        let group_dir = tmp.path().join(".aw/changes/revise-test/groups/my-group");

        // Write artifact with REVIEWED verdict and create_complete (section-loop done)
        std::fs::write(
            group_dir.join("reference_context.md"),
            "---\nchange: revise-test\ngroup: my-group\ndate: 2026-03-04\ncreate_complete: true\nreview_verdict: REVIEWED\n---\n\n# Reference Context\n\nContent.\n\n# Reviews\n\nNeeds work.\n",
        )
        .unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "revise-test"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        // Should redirect to revise workflow
        let next = &parsed["next_actions"][0];
        assert_eq!(next["args"]["change_id"], "revise-test");
    }

    #[tokio::test]
    async fn test_workflow_auto_approves_after_revision_limit() {
        let tmp = setup_group_change("auto-approve");
        let change_dir = tmp.path().join(".aw/changes/auto-approve");
        let group_dir = change_dir.join("groups/my-group");

        // Set revision count >= 1
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.increment_revision_count("ref_ctx:my-group");
        sm.save().unwrap();

        // Write artifact with REVIEWED verdict and create_complete (section-loop done)
        std::fs::write(
            group_dir.join("reference_context.md"),
            "---\nchange: auto-approve\ngroup: my-group\ndate: 2026-03-04\ncreate_complete: true\nreview_verdict: REVIEWED\n---\n\n# Content\n",
        )
        .unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "auto-approve"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        // Single group -> auto-approve -> all done -> phase_complete
        assert_eq!(parsed["status"], "phase_complete");
    }

    #[tokio::test]
    #[ignore = "groups removed — test needs rewrite for no-group model"]
    async fn test_workflow_phase_complete_when_all_groups_done() {
        let tmp = setup_group_change("all-done");
        let change_dir = tmp.path().join(".aw/changes/all-done");

        // Groups removed — this test is stubbed out
        let sm = StateManager::load(&change_dir).unwrap();
        drop(sm);

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "all-done"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "phase_complete");

        // Phase should be PostClarificationsCreated
        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(*sm.phase(), StatePhase::ChangeInited);
    }

    #[test]
    fn test_artifact_writes_file() {
        let tmp = setup_group_change("art-write");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "art-write",
            "group_id": "my-group",
            "specs": [
                {
                    "spec_id": "create-pre-clarifications",
                    "spec_group": "sdd/tools/workflows",
                    "relevance": "high",
                    "key_requirements": ["R1", "R2"]
                }
            ]
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert!(parsed["artifacts_written"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str().unwrap().contains("reference_context.md")));

        // Verify file
        let file_path = tmp
            .path()
            .join(".aw/changes/art-write/groups/my-group/reference_context.md");
        assert!(file_path.exists());
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("group: my-group"));
        assert!(content.contains("create-pre-clarifications"));
        assert!(content.contains("high"));
        assert!(content.contains("R1, R2"));
    }

    #[test]
    fn test_artifact_does_not_mutate_state() {
        let tmp = setup_group_change("no-state");
        let change_dir = tmp.path().join(".aw/changes/no-state");
        let group_dir = change_dir.join("groups/my-group");

        // Write initial artifact with review verdict and create_complete
        std::fs::write(
            group_dir.join("reference_context.md"),
            "---\nchange: no-state\ngroup: my-group\ncreate_complete: true\nreview_verdict: REVIEWED\n---\n\n# Old content\n",
        )
        .unwrap();

        // Rewrite via artifact tool
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "no-state",
            "group_id": "my-group",
            "specs": [{"spec_id": "some-spec", "spec_group": "test", "relevance": "high"}]
        });
        execute_artifact(&args, tmp.path()).unwrap();

        // Artifact should NOT mutate state — revision count stays 0
        // Workflow layer is responsible for incrementing revision count
        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(sm.revision_count("ref_ctx:my-group"), 0);
    }

    #[tokio::test]
    #[ignore = "groups removed — multi-group test obsolete"]
    async fn test_multi_group_breadth_first() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/multi-grp");
        std::fs::create_dir_all(change_dir.join("groups/group-a")).unwrap();
        std::fs::create_dir_all(change_dir.join("groups/group-b")).unwrap();
        std::fs::write(change_dir.join("user_input.md"), "Test").unwrap();

        // Create pre-clarifications for both groups
        std::fs::write(
            change_dir.join("groups/group-a/pre_clarifications.md"),
            "---\nchange: multi-grp\ngroup: group-a\n---\n",
        )
        .unwrap();
        std::fs::write(
            change_dir.join("groups/group-b/pre_clarifications.md"),
            "---\nchange: multi-grp\ngroup: group-b\n---\n",
        )
        .unwrap();

        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = StatePhase::ChangeInited;
        sm.save().unwrap();

        // First call -> should return create prompt for group-a
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "multi-grp"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["group_id"], "group-a");

        // Mark group-a as done
        common::mark_group_done(&change_dir, "group-a").unwrap();

        // Second call -> should return create prompt for group-b
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["group_id"], "group-b");

        // Mark group-b as done
        common::mark_group_done(&change_dir, "group-b").unwrap();

        // Third call -> phase_complete
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "phase_complete");

        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(*sm.phase(), StatePhase::ChangeInited);
    }

    #[tokio::test]
    async fn test_workflow_prompt_includes_spec_structure_when_specs_exist() {
        let tmp = setup_group_change("spec-tree-test");
        let change_dir = tmp.path().join(".aw/changes/spec-tree-test");

        // Create issue with crate:agentic-workflow label so scope extraction yields the merged project.
        std::fs::create_dir_all(change_dir.join("issues")).unwrap();
        std::fs::write(
            change_dir.join("issues/issue_1.md"),
            "---\nnumber: 1\ntitle: Test\nstate: OPEN\nlabels: [crate:agentic-workflow]\n---\n",
        )
        .unwrap();

        // Create spec files under projects/agentic-workflow/tech-design/core/
        let specs_dir = tmp
            .path()
            .join("projects/agentic-workflow/tech-design/core");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(specs_dir.join("logic.md"), "# Logic\n").unwrap();
        std::fs::write(specs_dir.join("state.md"), "# State\n").unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "spec-tree-test"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        let prompt = read_prompt(&parsed, tmp.path());
        assert!(
            prompt.contains("Existing Spec Structure"),
            "prompt should include ## Existing Spec Structure block when spec dirs exist"
        );
        assert!(
            prompt.contains("agentic-workflow"),
            "prompt tree should include the group directory name"
        );
    }

    #[tokio::test]
    async fn test_workflow_prompt_no_spec_structure_when_no_specs() {
        let tmp = setup_group_change("no-tree-test");
        // No .aw/tech-design/ directory → spec_dir_tree is empty → no Existing Spec Structure block

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "no-tree-test"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        let prompt = read_prompt(&parsed, tmp.path());
        assert!(
            !prompt.contains("Existing Spec Structure"),
            "prompt should NOT include ## Existing Spec Structure when no spec dirs found"
        );
    }

    #[tokio::test]
    async fn test_workflow_prompt_includes_spec_plan_guidance_for_related_specs() {
        let tmp = setup_group_change("guidance-test");
        let change_dir = tmp.path().join(".aw/changes/guidance-test");
        let group_dir = change_dir.join("groups/my-group");

        // Write skeleton with source_refs already filled to advance to related_specs
        std::fs::write(
            group_dir.join("reference_context.md"),
            "---\nchange: guidance-test\ngroup: my-group\nfill_sections: [source_refs, related_specs, reproductions, related_issues, first_fix]\nfilled_sections: [source_refs]\ncreate_complete: false\n---\n\n# Reference Context\n\n## Source References\n\nSome refs.\n\n## Related Specs\n\n<!-- TODO -->\n",
        )
        .unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "guidance-test"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        let prompt = read_prompt(&parsed, tmp.path());
        // spec_plan guidance only appears in related_specs section prompt
        assert!(
            prompt.contains("Spec Plan Guidance"),
            "related_specs prompt must include spec plan guidance"
        );
        assert!(
            prompt.contains("related_specs"),
            "prompt must target related_specs section"
        );
    }
}
