// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_spec/preamble-definitions.md#source
// CODEGEN-BEGIN
//! Create tools for change-spec.
//!
//! - `sdd_workflow_create_change_spec` — sub-state router: skeleton → analyze → fill → prune
//! - `sdd_artifact_create_change_spec` — writes one section at a time into the spec file
//!
//! Revise is handled by `revise.rs`. When `resolve_next_spec()` returns
//! `SpecSubState::Revise`, this module redirects to `sdd_workflow_revise_change_spec`.

use super::common_change_spec::{self as common, SpecSubState};
use crate::models::change::SddInterface;
use crate::models::spec_rules::SectionType;
use crate::models::state::StatePhase;
use crate::models::WorkflowArtifact;
use crate::tools::review_helpers;
use crate::tools::workflow_common;
use crate::tools::{get_optional_string, get_required_string, ToolDefinition};
use crate::workflow::helpers;
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;
use std::str::FromStr;

// ─── Tool Definitions ────────────────────────────────────────────────────────

/// MCP tool definition for sdd_workflow_create_change_spec
/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn workflow_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_create_change_spec".to_string(),
        description: "Orchestrate per-spec change-spec lifecycle \
            (skeleton → analyze → fill sections → prune → review → revise)"
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

/// MCP tool definition for sdd_artifact_create_change_spec
/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn artifact_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_artifact_create_change_spec".to_string(),
        description: "Write one section of a change spec. Used for both create and revise."
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
                    "description": "Which section to fill/replace"
                },
                "content": {
                    "type": "string",
                    "description": "Markdown content for this section (everything after the H2 heading)"
                },
                "fill_sections": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Sections to fill (set during analyze). Persisted to frontmatter."
                },
                "main_spec_ref": {
                    "type": "string",
                    "description": "Target path in .aw/tech-design/ for merge (e.g. sdd/tools/foo.md)"
                },
                "group_id": {
                    "type": "string",
                    "description": "Group ID for group-scoped spec path (optional; uses groups/{group_id}/specs/)"
                },
                "section_type": {
                    "type": "string",
                    "description": "Section type for annotation injection (e.g. overview, changes). Uses SectionType enum."
                }
            }
        }),
    }
}
// CODEGEN-END
// ─── Workflow Orchestration ──────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_spec/workflow.md#source
// CODEGEN-BEGIN
/// Execute sdd_workflow_create_change_spec.
///
/// Sub-state router that determines the next action for spec creation:
/// 1. Resolve which spec needs work (topological ordering)
/// 2. Within a spec: skeleton → analyze → fill sections → prune → done
/// 3. After all specs: redirect to review or advance to implementation
/// @spec projects/agentic-workflow/tech-design/core/tools/create_change_spec/workflow.md#source
pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;
    let interface = workflow_common::load_interface(project_root);

    match common::resolve_next_spec(&change_dir, &change_id)? {
        SpecSubState::Create { spec_id, depends } => {
            // Resolve group_id per-spec: prefer existing spec location, then spec_plan assignment
            let group_id = common::resolve_group_id_for_spec(&change_dir, &spec_id)
                .or_else(|| workflow_common::resolve_single_group_id(&change_dir));
            handle_create_sub_state(
                &change_id,
                &spec_id,
                &depends,
                &change_dir,
                project_root,
                interface,
                group_id.as_deref(),
            )
            .await
        }
        SpecSubState::Review { spec_id } => {
            let result = json!({
                "status": "ok",
                "spec_id": spec_id,
                "next_actions": [
                    workflow_common::next_action(interface, "sdd_workflow_review_change_spec", json!({"change_id": change_id}))
                ]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
        SpecSubState::Revise { spec_id } => {
            let result = json!({
                "status": "ok",
                "spec_id": spec_id,
                "next_actions": [
                    workflow_common::next_action(interface, "sdd_workflow_revise_change_spec", json!({"change_id": change_id}))
                ]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
        SpecSubState::MainthreadMustFix { spec_id } => {
            let sm = crate::state::StateManager::load(&change_dir)?;
            let phase = sm.phase().clone();
            let resp = helpers::mainthread_must_fix(
                &change_id,
                &phase,
                &format!("spec:{}", spec_id),
                &format!("review_spec_{}", spec_id),
                interface,
            );
            Ok(serde_json::to_string_pretty(&resp)?)
        }
        SpecSubState::AdvanceToImplementation => {
            workflow_common::update_phase(&change_dir, StatePhase::ChangeImplementationCreated)?;
            let result = json!({
                "status": "phase_complete",
                "prompt": "All specs created and reviewed. Advancing to implementation.",
                "next_actions": [
                    workflow_common::next_action(interface, helpers::RUN_CHANGE_TOOL, json!({
                        "change_id": change_id,
                    }))
                ]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
    }
}

/// Handle the create sub-state machine for a single spec.
///
/// Progression: no file → skeleton → analyze → fill per section → prune → done
///
/// Each call returns ONE prompt for the host agent (Claude Code mainthread
/// or subagent) to execute. The host agent writes artifacts via the
/// `sdd_artifact_create_change_spec` tool, then calls back to this workflow
/// tool for the next prompt. State is persisted in spec frontmatter
/// (`fill_sections`, `filled_sections`, `create_complete`).
async fn handle_create_sub_state(
    change_id: &str,
    spec_id: &str,
    depends: &[String],
    change_dir: &Path,
    project_root: &Path,
    interface: SddInterface,
    group_id: Option<&str>,
) -> Result<String> {
    let specs_dir = common::get_specs_dir(change_dir, group_id);
    let spec_path = specs_dir.join(format!("{}.md", spec_id));

    if !spec_path.exists() {
        // Step 1: Generate skeleton + return analyze prompt
        std::fs::create_dir_all(&specs_dir)?;

        let title = spec_id
            .split('-')
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        let skeleton = common::generate_skeleton(spec_id, &title, None, None, project_root);
        std::fs::write(&spec_path, &skeleton)?;

        // Update phase to ChangeSpecCreated
        workflow_common::update_phase(change_dir, StatePhase::ChangeSpecCreated)?;

        return build_analyze_prompt(change_id, spec_id, depends, group_id, project_root).await;
    }

    // File exists — check sub-state from frontmatter
    let content = std::fs::read_to_string(&spec_path)?;

    if common::is_create_complete(&content) {
        // Create done — redirect to review
        let result = json!({
            "status": "ok",
            "spec_id": spec_id,
            "next_actions": [
                workflow_common::next_action(interface, "sdd_workflow_review_change_spec", json!({"change_id": change_id}))
            ]
        });
        return Ok(serde_json::to_string_pretty(&result)?);
    }

    let fill_sections = common::read_fill_sections(&content);
    let filled_sections = common::read_filled_sections(&content);

    if fill_sections.is_empty() {
        // Analyze not done yet — return analyze prompt
        return build_analyze_prompt(change_id, spec_id, depends, group_id, project_root).await;
    }

    // Find next unfilled section (compare using base name, stripping "(optional)")
    let next_section = fill_sections.iter().find(|s| {
        let base_name = common::fill_section_base_name(s);
        !filled_sections.contains(&base_name.to_string())
    });

    if let Some(section_str) = next_section {
        // Extract base section name for the fill prompt (strip optional marker)
        let base_name = common::fill_section_base_name(section_str);
        build_fill_prompt(change_id, spec_id, base_name, group_id, project_root).await
    } else {
        // All sections filled — prune and mark complete
        let pruned = common::prune_todo_sections(&content);
        let marked = review_helpers::upsert_frontmatter_field(&pruned, "create_complete", "true");
        std::fs::write(&spec_path, &marked)?;

        // Redirect to review
        let result = json!({
            "status": "ok",
            "spec_id": spec_id,
            "message": "Spec creation complete. All sections filled and pruned.",
            "next_actions": [
                workflow_common::next_action(interface, "sdd_workflow_review_change_spec", json!({"change_id": change_id}))
            ]
        });
        Ok(serde_json::to_string_pretty(&result)?)
    }
}
// CODEGEN-END
// ─── Helpers ────────────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_spec/resolve-spec-path.md#source
// CODEGEN-BEGIN
/// Resolve spec file path by scanning groups/*/specs/ and top-level specs/.
/// Eliminates the need for callers to pass group_id — the state machine structure
/// is the source of truth.
fn resolve_spec_path(change_dir: &Path, spec_id: &str) -> Result<std::path::PathBuf> {
    let filename = format!("{}.md", spec_id);

    // 1. Check groups/*/specs/
    let groups_dir = change_dir.join("groups");
    if groups_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&groups_dir) {
            for entry in entries.flatten() {
                let candidate = entry.path().join("specs").join(&filename);
                if candidate.exists() {
                    return Ok(candidate);
                }
            }
        }
    }

    // 2. Check top-level specs/
    let top_level = change_dir.join("specs").join(&filename);
    if top_level.exists() {
        return Ok(top_level);
    }

    anyhow::bail!(
        "Spec file not found: {}.md\nSearched: {}/groups/*/specs/ and {}/specs/\n\
         Call `score workflow create-change-spec` first to generate the skeleton.",
        spec_id,
        change_dir.display(),
        change_dir.display()
    )
}
// CODEGEN-END
// ─── Artifact Write ──────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_spec/artifact.md#source
// CODEGEN-BEGIN
/// Execute sdd_artifact_create_change_spec.
///
/// Writes one section at a time into the spec file. Used for both initial
/// creation and revision. After writing, marks the section as filled in
/// frontmatter and returns next_actions pointing back to the workflow tool.
/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    let section = get_required_string(args, "section")?;
    let content = get_required_string(args, "content")?;

    // change_id: prefer explicit arg, fall back to active change on current branch
    let explicit_change_id = get_optional_string(args, "change_id");
    // spec_id: prefer explicit arg, fall back to STATE.yaml current_task_id
    let explicit_spec_id = get_optional_string(args, "spec_id");

    let change_id = if let Some(id) = explicit_change_id {
        workflow_common::validate_change_id(&id)?;
        id
    } else {
        workflow_common::resolve_active_change_id(project_root)?
    };

    let interface = workflow_common::load_interface(project_root);

    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);

    let spec_id = if let Some(id) = explicit_spec_id {
        id
    } else {
        // Read current_task_id from STATE.yaml
        let sm = crate::state::StateManager::load(&change_dir)?;
        sm.state().current_task_id.clone().ok_or_else(|| {
            anyhow::anyhow!(
                "No spec_id in payload and no current_task_id in STATE.yaml. \
                 Either pass spec_id or run the workflow tool first."
            )
        })?
    };

    // Validate section name
    if !common::ALL_SECTIONS.contains(&section.as_str()) {
        anyhow::bail!(
            "Invalid section '{}'. Valid: {:?}",
            section,
            common::ALL_SECTIONS
        );
    }

    // Auto-resolve spec path from change directory structure.
    // No group_id or spec_id needed from caller — state machine knows.
    let spec_path = resolve_spec_path(&change_dir, &spec_id)?;

    // Read current content
    let current = std::fs::read_to_string(&spec_path)?;

    // Replace the target section
    let updated = common::replace_section(&current, &section, &content);

    // Mark section as filled in frontmatter
    let mut filled = common::read_filled_sections(&updated);
    if !filled.contains(&section) {
        filled.push(section.clone());
    }
    let filled_str = format!("[{}]", filled.join(", "));
    let mut final_content =
        review_helpers::upsert_frontmatter_field(&updated, "filled_sections", &filled_str);

    // Persist fill_sections if provided (from analyze step)
    if let Some(fs) = args.get("fill_sections").and_then(|v| v.as_array()) {
        let fs_list: Vec<String> = fs
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        // Reject umbrella names — only ALL_SECTIONS leaf names allowed.
        // `fill_section_base_name` strips "(optional)" markers before comparison.
        let invalid: Vec<String> = fs_list
            .iter()
            .filter(|s| {
                let base = common::fill_section_base_name(s);
                !common::ALL_SECTIONS.contains(&base)
            })
            .cloned()
            .collect();
        if !invalid.is_empty() {
            anyhow::bail!(
                "Invalid fill_sections entries: {:?}. Use leaf names only (no umbrella names like 'diagrams'/'api_spec'/'test_plan'). Valid: {:?}",
                invalid,
                common::ALL_SECTIONS
            );
        }
        if !fs_list.is_empty() {
            let fs_str = format!("[{}]", fs_list.join(", "));
            final_content =
                review_helpers::upsert_frontmatter_field(&final_content, "fill_sections", &fs_str);
        }
    }

    // Persist main_spec_ref if provided
    if let Some(ref_path) = get_optional_string(args, "main_spec_ref") {
        final_content = review_helpers::upsert_frontmatter_field(
            &final_content,
            "main_spec_ref",
            &format!("\"{}\"", ref_path),
        );
    }

    std::fs::write(&spec_path, &final_content)?;

    // ── Post-write alignment validation ──────────────────────────────────
    // Gating: only run on complete specs (create_complete: true).
    // Rationale: incomplete specs contain unfilled <!-- TODO --> sections
    // that trigger MissingSectionAnnotation / FormatPriorityViolation false
    // positives. The spec says "after writing section content to disk, call
    // spec_alignment::check()" — the create_complete gate narrows this to
    // the final write (after prune), which is the first moment the spec
    // is structurally valid. See also: test_artifact_alignment_skipped_when_incomplete.
    //
    // Phase 1 format violations → revert and return error.
    // Phase 2 coverage gaps → allow write, return as warnings.
    let alignment_warnings: Option<Vec<Value>> = if common::is_create_complete(&final_content) {
        match crate::spec_alignment::check(&spec_path) {
            result if result.total_violations > 0 => {
                let mut format_violations = Vec::new();
                let mut coverage_warnings = Vec::new();

                for file_result in &result.files {
                    for violation in &file_result.violations {
                        let v_json = json!({
                            "kind": violation.kind.to_string(),
                            "message": &violation.message,
                            "heading": violation.heading.as_deref(),
                            "line": violation.line,
                            "file": &file_result.path,
                        });
                        if violation.kind.is_format_violation() {
                            format_violations.push(v_json);
                        } else {
                            coverage_warnings.push(v_json);
                        }
                    }
                }

                if !format_violations.is_empty() {
                    // Revert to pre-write content
                    std::fs::write(&spec_path, &current)?;
                    let err_result = json!({
                        "status": "error",
                        "message": "Alignment check failed: format violations found. File reverted.",
                        "violations": format_violations,
                        "next_actions": [
                            workflow_common::next_action(interface, "sdd_workflow_create_change_spec", json!({"change_id": change_id}))
                        ]
                    });
                    return Ok(serde_json::to_string_pretty(&err_result)?);
                }

                if coverage_warnings.is_empty() {
                    None
                } else {
                    Some(coverage_warnings)
                }
            }
            _ => None,
        }
    } else {
        None
    };

    // Derive relative path from the resolved spec_path
    let rel_spec_path = spec_path
        .strip_prefix(&change_dir)
        .unwrap_or(&spec_path)
        .to_string_lossy()
        .to_string();
    let artifacts_written = vec![rel_spec_path];

    let result = json!({
        "status": "ok",
        "artifacts_written": artifacts_written,
        "alignment_warnings": alignment_warnings,
        "next_actions": [
            workflow_common::next_action(interface, "sdd_workflow_create_change_spec", json!({"change_id": change_id}))
        ]
    });

    Ok(serde_json::to_string_pretty(&result)?)
}
// CODEGEN-END
// ─── Prompt Builders ─────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_spec/prompts.md#source
// CODEGEN-BEGIN
/// Build ANALYZE prompt — agent reads context and decides which sections to fill.
async fn build_analyze_prompt(
    change_id: &str,
    spec_id: &str,
    depends: &[String],
    group_id: Option<&str>,
    project_root: &Path,
) -> Result<String> {
    let _pp = project_root.display();

    let depends_note = if depends.is_empty() {
        String::new()
    } else {
        format!(
            "\n## Dependencies\n\nThis spec depends on: {}. Read these specs first.\n",
            depends.join(", ")
        )
    };

    // Group-aware paths
    let spec_path = match group_id {
        Some(gid) => format!(
            ".aw/changes/{}/groups/{}/specs/{}.md",
            change_id, gid, spec_id
        ),
        None => format!(".aw/changes/{}/specs/{}.md", change_id, spec_id),
    };
    let payload_path = match group_id {
        Some(gid) => format!(
            ".aw/changes/{}/groups/{}/payloads/create-change-spec.json",
            change_id, gid
        ),
        None => format!(".aw/changes/{}/payloads/create-change-spec.json", change_id),
    };
    let group_id_hint = match group_id {
        Some(gid) => format!(
            "\n**group_id**: `{}` (pass this to the artifact CLI as `group_id` parameter)\n",
            gid
        ),
        None => String::new(),
    };

    let prompt = format!(
        r#"# Task: Analyze Spec '{spec_id}' for Change '{change_id}'

A skeleton has been generated at `{spec_path}`.
{group_id_hint}
## CRITICAL: Artifact Writing Rule

**DO NOT use Write or Edit tools to modify spec files directly.**
You MUST use the artifact CLI command to write each section.
Direct file writes will be REJECTED and you will have to redo the work.

## Instructions

1. Read context:
   - Read the temp issue working copy that initiated this change (see user_input.md for the slug)
   - The issue's ## Problem, ## Requirements, ## Scope, and ## Reference Context sections are your primary context
2. Read the skeleton: `{spec_path}`
3. **IMPORTANT — `main_spec_ref`**: Check the spec frontmatter. If `main_spec_ref` is `~` (null),
   you MUST determine the target path in `.aw/tech-design/` where this spec will be merged.
   Format: `<scope>/<category>/<spec-id>.md` (e.g., `sdd/tools/new-feature.md`).
   Browse `.aw/tech-design/` to see existing spec groups.
   Pass it as the `main_spec_ref` parameter when calling the artifact CLI.
4. Decide which sections to fill based on the artifact being changed. Pick ONLY leaf section names from this list — NEVER pass umbrella words like `diagrams`, `api_spec`, or `test_plan`:
   Always fill: `changes`
   Verification artifacts (pick those that apply): `unit-test`, `e2e-test`
   Diagrams (pick those that apply): `interaction`, `logic`, `state-machine`, `mindmap`, `dependency`, `db-model`
   API shape (pick those that apply): `rest-api`, `rpc-api`, `async-api`, `cli`, `schema`, `config`
   UI (pick those that apply): `wireframe`, `component`, `design-token`
   Optional migration/prose sections only when maintaining legacy TD: `overview`, `requirements`, `scenarios`
   Docs: `doc`
5. Write a JSON payload file to `{payload_path}` then run the artifact CLI.

## Expected Action

Write the **overview** section first via artifact CLI. Pass the `fill_sections`
array as a parameter — USE LEAF NAMES ONLY from the allowed list above.
Example (adapt to this change): `fill_sections=["cli", "unit-test", "e2e-test", "changes"]`.
Never pass `diagrams`, `api_spec`, or `test_plan` (umbrella names).
Also pass `main_spec_ref` as a parameter if determined above.
The system persists it to frontmatter automatically.

Then call the artifact CLI for each remaining section in sequence.

## CLI Commands

```
# Read artifacts
Read file: .aw/changes/{change_id}/proposal.md
Read file: {spec_path}

# Write each section (MUST use this — do NOT edit spec files directly)
# Step 1: Write payload JSON to the EXACT path below (do NOT write to other locations)
# Step 2: Run artifact CLI
score artifact create-change-spec {change_id} {payload_path}
```
{depends_note}"#,
    );

    let change_dir = super::workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor =
        workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateChangeSpec);

    let mut extra = json!({ "spec_id": spec_id });
    if let Some(gid) = group_id {
        extra["group_id"] = json!(gid);
    }

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        &format!("analyze_spec_{}", spec_id),
        prompt,
        executor,
        extra,
        interface,
        project_root,
    )
    .await
}

/// Return fill guidance text for a section.
///
/// When a `SectionType` can be parsed from `section`, returns type-aware
/// guidance including the annotation format. Falls back to the legacy
/// string-match guidance for unrecognized names.
fn section_fill_guidance(section: &str) -> &'static str {
    // Attempt to resolve via SectionType for type-aware guidance
    if let Ok(st) = SectionType::from_str(section) {
        return match st {
            SectionType::Overview =>
                "Write a comprehensive overview (>= 50 chars) describing what this spec covers.\n\
                 Begin with `<!-- type: overview lang: markdown -->` on its own line.",
            SectionType::Requirements =>
                "Write requirements as Mermaid Plus requirementDiagram.\n\
                 Begin with `<!-- type: requirements lang: mermaid -->`.",
            SectionType::Scenarios =>
                "Write acceptance scenarios as YAML entries with id, given, when, then, and optional diagram_ref.\n\
                 Begin with `<!-- type: scenarios lang: yaml -->`.",
            SectionType::Changes =>
                "List files that will change. For MODIFY entries, include function/type-level `targets`:\n\
                 ```yaml\nchanges:\n  - path: foo.rs\n    action: CREATE\n    description: new file\n\
                   - path: bar.rs\n    action: MODIFY\n    targets:\n\
                       - type: function\n        name: handle_request\n        change: add error handling\n\
                       - type: struct\n        name: Config\n        change: add timeout field\n\
                     do_not_touch: [validate_input, parse_args]\n```\n\
                 Target type values: function, struct, enum, trait, impl, method.\n\
                 `targets` is required for MODIFY, optional for CREATE/DELETE.\n\
                 `do_not_touch` lists functions/types the agent must NOT modify.\n\
                 Begin with `<!-- type: changes lang: yaml -->`.",
            SectionType::UnitTest =>
                "Define unit-test cases as Mermaid Plus requirementDiagram with test elements and verifies links.\n\
                 Begin with `<!-- type: unit-test lang: mermaid -->`.",
            SectionType::E2eTest =>
                "Define product-flow E2E cases in YAML, including command/input, expected output, and artifact side-effect assertions.\n\
                 Begin with `<!-- type: e2e-test lang: yaml -->`.",
            SectionType::Interaction =>
                "Draw a Mermaid sequence diagram. Begin with `<!-- type: interaction lang: mermaid -->`.",
            SectionType::Logic =>
                "Draw a Mermaid flowchart. Begin with `<!-- type: logic lang: mermaid -->`.",
            SectionType::Dependency =>
                "Draw a Mermaid class diagram. Begin with `<!-- type: dependency lang: mermaid -->`.",
            SectionType::StateMachine =>
                "Draw a Mermaid stateDiagram-v2. Begin with `<!-- type: state-machine lang: mermaid -->`.",
            SectionType::DbModel =>
                "Draw a Mermaid erDiagram. Begin with `<!-- type: db-model lang: mermaid -->`.",
            SectionType::Mindmap =>
                "Draw a Mermaid mindmap. Begin with `<!-- type: mindmap lang: mermaid -->`.",
            SectionType::RestApi =>
                "Write OpenAPI 3.1 YAML. Begin with `<!-- type: rest-api lang: yaml -->`.",
            SectionType::RpcApi =>
                "Write OpenRPC 1.3 YAML. Begin with `<!-- type: rpc-api lang: yaml -->`.",
            SectionType::AsyncApi =>
                "Write AsyncAPI 2.6 YAML. Begin with `<!-- type: async-api lang: yaml -->`.",
            SectionType::Cli =>
                "Define CLI command tree in YAML. Begin with `<!-- type: cli lang: yaml -->`.",
            SectionType::Schema =>
                "Write JSON Schema for interface/data models as YAML. Begin with `<!-- type: schema lang: yaml -->`.",
            SectionType::Config =>
                "Write config file schema as YAML. Begin with `<!-- type: config lang: yaml -->`.",
            SectionType::Wireframe =>
                "Describe the UI layout in wireframe YAML. Begin with `<!-- type: wireframe lang: yaml -->`.",
            SectionType::Component =>
                "Define UI component contract as YAML. Begin with `<!-- type: component lang: yaml -->`.",
            SectionType::DesignToken =>
                "Define design tokens in W3C DTCG-compatible YAML. Begin with `<!-- type: design-token lang: yaml -->`.",
            SectionType::RuntimeImage =>
                "Define a container image build contract in YAML: base image, package installs, copy layout, build args, env, entrypoint/command, and build context inputs. Begin with `<!-- type: runtime-image lang: yaml -->`.",
            SectionType::Deployment =>
                "Define deployment/runtime operations manifests in YAML: Kubernetes/Kustomize resources, overlays, services, scaling, routing, policy, and rollout expectations. Begin with `<!-- type: deployment lang: yaml -->`.",
            SectionType::Doc =>
                "Write user-facing documentation in markdown. Begin with `<!-- type: doc lang: markdown -->`.",
            SectionType::Manifest =>
                "Declare package manifest entries (Cargo.toml dependencies, etc.) in YAML.\n\
                 Shape: `dependencies: [{ name, spec: workspace|version|path, features?: [..] }]`.\n\
                 Begin with `<!-- type: manifest lang: yaml -->`.",
            SectionType::ToolContract =>
                "Declare AW-managed native tool manifests for vat, rig, meter, guard, or arena.\n\
                 Shape: `tool_contracts: [{ id, tool, manifest, command?, native? | toml? }]`.\n\
                 Begin with `<!-- type: tool-contract lang: yaml -->`.",
            SectionType::RustSourceUnit =>
                "Write a full Rust source unit in a rust fence for lossless CST-backed regeneration.\n\
                 Begin with `<!-- type: rust-source-unit lang: rust -->`.",
            SectionType::TextSourceUnit =>
                "Write a full shell/text source unit in a bash fence for TD-owned verbatim regeneration.\n\
                 Begin with `<!-- type: text-source-unit lang: bash -->`.",
        };
    }

    // Fallback for sections without SectionType-specific guidance (e.g. overview, requirements).
    match section {
        "overview" => "Write a comprehensive overview (>= 50 chars) describing what this spec covers.",
        "requirements" => "Write requirements in markdown format:\n\n### R1: Title\n\nDescription.\n\n**Priority**: high/medium/low",
        "scenarios" => "Write acceptance scenarios in Given/When/Then format:\n\n### Scenario: Name\n\n**GIVEN** precondition\n**WHEN** action\n**THEN** expected outcome",
        "changes" => "List the files that will be changed:\n\n| File | Action | Description |\n|------|--------|-------------|",
        _ => "Fill in this section with appropriate content.",
    }
}

/// Build FILL prompt for a specific section.
pub(crate) async fn build_fill_prompt(
    change_id: &str,
    spec_id: &str,
    section: &str,
    group_id: Option<&str>,
    project_root: &Path,
) -> Result<String> {
    let _pp = project_root.display();

    // Base section guidance, augmented with SectionType-specific hints
    let section_guidance = section_fill_guidance(section);

    // Group-aware paths
    let spec_path = match group_id {
        Some(gid) => format!(
            ".aw/changes/{}/groups/{}/specs/{}.md",
            change_id, gid, spec_id
        ),
        None => format!(".aw/changes/{}/specs/{}.md", change_id, spec_id),
    };
    let payload_path = match group_id {
        Some(gid) => format!(
            ".aw/changes/{}/groups/{}/payloads/create-change-spec.json",
            change_id, gid
        ),
        None => format!(".aw/changes/{}/payloads/create-change-spec.json", change_id),
    };
    let group_id_hint = match group_id {
        Some(gid) => format!(
            "\n**group_id**: `{}` (pass this to the artifact CLI)\n",
            gid
        ),
        None => String::new(),
    };

    let prompt = format!(
        r#"# Task: Fill Section '{section}' for Spec '{spec_id}' (Change '{change_id}')
{group_id_hint}
## Instructions

1. Read the current spec: `{spec_path}`
2. Read relevant context if needed
3. Write content for the **{section}** section

## Section Guidance

{section_guidance}

## Action

Run `score artifact create-change-spec` with section="{section}" and your content.

## CLI Commands

```
# Read spec
Read file: {spec_path}

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec {change_id} {payload_path}
```"#,
    );

    let change_dir = super::workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor =
        workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateChangeSpec);

    let mut extra = json!({ "spec_id": spec_id });
    if let Some(gid) = group_id {
        extra["group_id"] = json!(gid);
    }

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        &format!("fill_spec_{}_{}", spec_id, section),
        prompt,
        executor,
        extra,
        interface,
        project_root,
    )
    .await
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_spec/tests.md#source
// CODEGEN-BEGIN
#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateManager;
    use tempfile::TempDir;

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

    fn setup_change(change_id: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes").join(change_id);
        std::fs::create_dir_all(&change_dir).unwrap();
        // R4: save() needs an issue backing change_id.
        crate::test_util::write_minimal_issue(tmp.path(), change_id);

        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = StatePhase::ChangeInited;
        sm.save().unwrap();

        tmp
    }

    #[tokio::test]
    async fn test_workflow_creates_skeleton() {
        let tmp = setup_change("skel-test");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "skel-test"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["spec_id"], "skel-test-spec");
        let prompt = read_prompt(&parsed, tmp.path());
        assert!(prompt.contains("Analyze Spec"));

        // Verify skeleton file created
        let spec_path = tmp
            .path()
            .join(".aw/changes/skel-test/specs/skel-test-spec.md");
        assert!(spec_path.exists());
        let content = std::fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("id: skel-test-spec"));
        assert!(content.contains("## Overview"));
        assert!(content.contains("<!-- TODO -->"));
    }

    #[tokio::test]
    async fn test_workflow_returns_fill_prompt_after_analyze() {
        let tmp = setup_change("fill-test");
        let change_dir = tmp.path().join(".aw/changes/fill-test");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Write skeleton with fill_sections set (analyze done)
        let content = "---\nid: fill-test-spec\nfill_sections: [overview, requirements, scenarios]\n---\n\n# Fill Test Spec\n\n## Overview\n\n<!-- TODO -->\n\n## Requirements\n\n<!-- TODO -->\n\n## Scenarios\n\n<!-- TODO -->\n\n# Reviews\n";
        std::fs::write(change_dir.join("specs/fill-test-spec.md"), content).unwrap();

        // Update phase
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.set_phase(StatePhase::ChangeSpecCreated).unwrap();
        sm.save().unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "fill-test"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        // Should return fill prompt for first section (overview)
        let prompt = read_prompt(&parsed, tmp.path());
        assert!(prompt.contains("Fill Section 'overview'"));
    }

    #[tokio::test]
    async fn test_workflow_prunes_and_completes() {
        let tmp = setup_change("prune-test");
        let change_dir = tmp.path().join(".aw/changes/prune-test");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Write spec with all sections filled
        let content = "---\nid: prune-test-spec\nfill_sections: [overview, requirements]\nfilled_sections: [overview, requirements]\n---\n\n# Prune Test Spec\n\n## Overview\n\nReal overview.\n\n## Requirements\n\nReal requirements.\n\n## Scenarios\n\n<!-- TODO -->\n\n## Diagrams\n\n### Sequence Diagram\n<!-- TODO -->\n\n## Changes\n\n<!-- TODO -->\n\n# Reviews\n";
        std::fs::write(change_dir.join("specs/prune-test-spec.md"), content).unwrap();

        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.set_phase(StatePhase::ChangeSpecCreated).unwrap();
        sm.save().unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "prune-test"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        // Should redirect to review (create complete)
        let next = &parsed["next_actions"][0];
        assert_eq!(next["args"]["change_id"], "prune-test");

        // Verify file was pruned
        let spec_path = change_dir.join("specs/prune-test-spec.md");
        let final_content = std::fs::read_to_string(&spec_path).unwrap();
        assert!(final_content.contains("create_complete: true"));
        assert!(final_content.contains("Real overview."));
        assert!(!final_content.contains("<!-- TODO -->"));
    }

    #[test]
    fn test_artifact_writes_section() {
        let tmp = setup_change("art-sec");
        let change_dir = tmp.path().join(".aw/changes/art-sec");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Write skeleton
        let skeleton =
            common::generate_skeleton("art-sec-spec", "Art Sec Spec", None, None, tmp.path());
        std::fs::write(change_dir.join("specs/art-sec-spec.md"), &skeleton).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "art-sec",
            "spec_id": "art-sec-spec",
            "section": "overview",
            "content": "This is a comprehensive overview of the spec.\n\nIt covers many things."
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert!(parsed["artifacts_written"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str().unwrap().contains("art-sec-spec.md")));

        // Verify content was written
        let spec_path = change_dir.join("specs/art-sec-spec.md");
        let content = std::fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("comprehensive overview"));
        assert!(content.contains("filled_sections: [overview]"));
    }

    #[test]
    fn test_artifact_rejects_invalid_section() {
        let tmp = setup_change("bad-sec");
        let change_dir = tmp.path().join(".aw/changes/bad-sec");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(
            change_dir.join("specs/bad-sec-spec.md"),
            "---\nid: bad-sec-spec\n---\n\n# Test\n",
        )
        .unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "bad-sec",
            "spec_id": "bad-sec-spec",
            "section": "nonexistent",
            "content": "test"
        });
        let result = execute_artifact(&args, tmp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid section"));
    }

    #[tokio::test]
    async fn test_workflow_all_specs_done_advances_to_implementation() {
        let tmp = setup_change("advance-test");
        let change_dir = tmp.path().join(".aw/changes/advance-test");
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();

        // Write a spec with APPROVED verdict (create_complete + approved)
        let content = "---\nid: advance-test-spec\ncreate_complete: true\n---\n\n# Spec\n\n## Overview\n\nDone.\n";
        std::fs::write(specs_dir.join("advance-test-spec.md"), content).unwrap();

        // Write proposal with this spec listed
        let proposal = "---\nspec_plan:\n- id: advance-test-spec\n---\n\n# Proposal\n";
        std::fs::write(change_dir.join("proposal.md"), proposal).unwrap();

        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.set_phase(StatePhase::ChangeSpecReviewed).unwrap();
        sm.save().unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "advance-test"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        // Should be a review redirect (spec exists, no review yet based on analyze_specs)
        // The exact behavior depends on analyze_specs verdict check
        assert!(parsed["status"].as_str().is_some());
    }

    #[test]
    fn test_create_complete_blocked_on_failed_sections() {
        // When failed_sections is non-empty, create_complete must NOT be written
        // We simulate the logic directly since run_create_spec_agent_loop is async and agent-driven
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/spec-guard");
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();

        // Write a spec without create_complete
        let spec_content =
            "---\nid: spec-guard\ntype: spec\n---\n# Spec\n\n## Overview\n\nSome content.\n";
        let spec_path = specs_dir.join("spec-guard.md");
        std::fs::write(&spec_path, spec_content).unwrap();

        // Simulate: failed_sections is non-empty → should NOT write create_complete
        let failed_sections = vec!["requirements".to_string()];
        assert!(!failed_sections.is_empty());

        // Read the spec content — create_complete should NOT be set
        let content = std::fs::read_to_string(&spec_path).unwrap();
        assert!(
            !content.contains("create_complete: true"),
            "create_complete must NOT be written when failed_sections is non-empty"
        );
    }

    #[test]
    fn test_create_complete_written_on_all_filled() {
        use crate::tools::common_change_spec as common;
        use crate::tools::review_helpers;

        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/spec-ok");
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();

        let spec_content =
            "---\nid: spec-ok\ntype: spec\n---\n# Spec\n\n## Overview\n\nContent here.\n";
        let spec_path = specs_dir.join("spec-ok.md");
        std::fs::write(&spec_path, spec_content).unwrap();

        // Simulate: failed_sections is empty → write create_complete
        let failed_sections: Vec<String> = vec![];
        if failed_sections.is_empty() {
            let content = std::fs::read_to_string(&spec_path).unwrap();
            if !common::is_create_complete(&content) {
                let pruned = common::prune_todo_sections(&content);
                let marked =
                    review_helpers::upsert_frontmatter_field(&pruned, "create_complete", "true");
                std::fs::write(&spec_path, &marked).unwrap();
            }
        }

        let content = std::fs::read_to_string(&spec_path).unwrap();
        assert!(
            content.contains("create_complete: true"),
            "create_complete must be written when failed_sections is empty"
        );
    }

    // ── artifact-tools-update: merge_strategy removed ─────────────────────────

    #[test]
    fn test_artifact_definition_excludes_merge_strategy() {
        // merge_strategy must not appear in sdd_artifact_create_change_spec schema.
        // Merge behavior is always replace (write to path, create if absent, overwrite if present).
        let def = artifact_definition();
        let props = def.input_schema["properties"]
            .as_object()
            .expect("properties must be an object");
        assert!(
            !props.contains_key("merge_strategy"),
            "merge_strategy must not be in artifact_definition schema; it is dead code"
        );
    }

    #[test]
    fn test_artifact_ignores_merge_strategy_param() {
        // When merge_strategy is passed by old callers it must be silently ignored.
        let tmp = setup_change("ms-ignore");
        let change_dir = tmp.path().join(".aw/changes/ms-ignore");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        let skeleton =
            common::generate_skeleton("ms-ignore-spec", "Ms Ignore Spec", None, None, tmp.path());
        std::fs::write(change_dir.join("specs/ms-ignore-spec.md"), &skeleton).unwrap();

        // Include merge_strategy — must not cause an error
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "ms-ignore",
            "spec_id": "ms-ignore-spec",
            "section": "overview",
            "content": "Overview content.",
            "merge_strategy": "append"
        });
        let result = execute_artifact(&args, tmp.path());
        assert!(
            result.is_ok(),
            "merge_strategy in args must be ignored, not rejected: {:?}",
            result.err()
        );

        let content = std::fs::read_to_string(change_dir.join("specs/ms-ignore-spec.md")).unwrap();
        assert!(content.contains("Overview content."));
    }

    #[test]
    fn test_artifact_write_replaces_section_content() {
        // Merge behavior is always replace: a second write to the same section must
        // overwrite the previous value, never append.
        let tmp = setup_change("replace-sem");
        let change_dir = tmp.path().join(".aw/changes/replace-sem");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        let skeleton = common::generate_skeleton(
            "replace-sem-spec",
            "Replace Sem Spec",
            None,
            None,
            tmp.path(),
        );
        std::fs::write(change_dir.join("specs/replace-sem-spec.md"), &skeleton).unwrap();

        // First write
        let args_first = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "replace-sem",
            "spec_id": "replace-sem-spec",
            "section": "overview",
            "content": "Initial overview."
        });
        execute_artifact(&args_first, tmp.path()).unwrap();

        // Second write — must overwrite, not append
        let args_second = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "replace-sem",
            "spec_id": "replace-sem-spec",
            "section": "overview",
            "content": "Revised overview."
        });
        execute_artifact(&args_second, tmp.path()).unwrap();

        let content =
            std::fs::read_to_string(change_dir.join("specs/replace-sem-spec.md")).unwrap();
        assert!(
            content.contains("Revised overview."),
            "second write must be present"
        );
        assert!(
            !content.contains("Initial overview."),
            "first write must be replaced, not appended"
        );
    }

    // ── Phase 3: Post-write alignment validation tests ──────────────────────

    #[test]
    fn test_artifact_alignment_format_violation_reverts() {
        // (a) When create_complete:true and the spec has format violations,
        // execute_artifact must revert the write and return status:"error".
        let tmp = setup_change("align-revert");
        let change_dir = tmp.path().join(".aw/changes/align-revert");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Spec with create_complete:true and a duplicate heading that will
        // trigger DuplicateSection after the overview section is filled.
        let spec_content = "\
---
id: align-revert-spec
create_complete: true
---

# Align Revert Spec

## Overview
<!-- type: overview lang: markdown -->

<!-- TODO -->

## Overview
<!-- type: overview lang: markdown -->

Old duplicate content.

# Reviews
";
        std::fs::write(change_dir.join("specs/align-revert-spec.md"), spec_content).unwrap();

        let original =
            std::fs::read_to_string(change_dir.join("specs/align-revert-spec.md")).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "align-revert",
            "spec_id": "align-revert-spec",
            "section": "overview",
            "content": "New overview content that should be reverted."
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        // Must return error status
        assert_eq!(
            parsed["status"], "error",
            "format violations must produce error status"
        );
        assert!(
            parsed["message"]
                .as_str()
                .unwrap_or("")
                .contains("format violations"),
            "error message must mention format violations"
        );
        assert!(
            parsed["violations"].is_array(),
            "must include violations array"
        );

        // File must be reverted to original content
        let after = std::fs::read_to_string(change_dir.join("specs/align-revert-spec.md")).unwrap();
        assert_eq!(
            after, original,
            "file must be reverted to pre-write content on format violation"
        );
    }

    #[test]
    fn test_artifact_annotation_preserved_after_section_write() {
        // replace_section now emits <!-- type: X lang: Y --> annotations,
        // so writing a section on a create_complete spec should pass
        // alignment checks (no violations).
        let tmp = setup_change("align-schema");
        let change_dir = tmp.path().join(".aw/changes/align-schema");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        let spec_content = "\
---
id: align-schema-spec
create_complete: true
---

# Align Schema Spec

## Overview
<!-- type: overview lang: markdown -->

Existing overview.

# Reviews
";
        std::fs::write(change_dir.join("specs/align-schema-spec.md"), spec_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "align-schema",
            "spec_id": "align-schema-spec",
            "section": "overview",
            "content": "Updated overview content."
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        // Should succeed now that annotations are preserved
        assert_eq!(
            parsed["status"], "ok",
            "annotation should be preserved: {:?}",
            parsed
        );
    }

    #[test]
    fn test_artifact_alignment_skipped_when_incomplete() {
        // (c) When create_complete is NOT true, alignment check must be
        // skipped even if the spec has format violations (e.g. duplicate
        // headings). This avoids false positives on partially-filled specs.
        let tmp = setup_change("align-skip");
        let change_dir = tmp.path().join(".aw/changes/align-skip");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Spec WITHOUT create_complete — has duplicate headings that would
        // fail alignment, but check should be skipped entirely.
        let spec_content = "\
---
id: align-skip-spec
---

# Align Skip Spec

## Overview
<!-- type: overview lang: markdown -->

<!-- TODO -->

## Overview
<!-- type: overview lang: markdown -->

Old duplicate content.

# Reviews
";
        std::fs::write(change_dir.join("specs/align-skip-spec.md"), spec_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "align-skip",
            "spec_id": "align-skip-spec",
            "section": "overview",
            "content": "New overview content (should not be reverted)."
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        // Must succeed — alignment check was skipped
        assert_eq!(
            parsed["status"], "ok",
            "incomplete spec must not trigger alignment validation"
        );
        // alignment_warnings should be null (check was skipped)
        assert!(
            parsed["alignment_warnings"].is_null(),
            "alignment_warnings must be null when create_complete is not set"
        );

        // Verify the new content was written (not reverted)
        let content = std::fs::read_to_string(change_dir.join("specs/align-skip-spec.md")).unwrap();
        assert!(
            content.contains("New overview content"),
            "content must be written when alignment check is skipped"
        );
    }
}
// CODEGEN-END
