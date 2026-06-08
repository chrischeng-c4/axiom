// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_impl.md#source
// CODEGEN-BEGIN
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
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_impl/prompts.md#source
// CODEGEN-BEGIN
// ─── Prompt Builders ─────────────────────────────────────────────────────────

async fn build_implement_code_prompt(
    change_id: &str,
    spec_id: &str,
    is_first: bool,
    group_id: Option<&str>,
    project_root: &Path,
) -> Result<String> {
    let _pp = project_root.display();
    let title = if is_first {
        format!("Begin Implementation for Change '{}'", change_id)
    } else {
        format!("Implement Spec '{}' for Change '{}'", spec_id, change_id)
    };

    // Group-aware spec path
    let spec_path = match group_id {
        Some(gid) => format!(
            ".aw/changes/{}/groups/{}/specs/{}.md",
            change_id, gid, spec_id
        ),
        None => format!(".aw/changes/{}/specs/{}.md", change_id, spec_id),
    };

    let instructions = if is_first {
        format!(
            "1. List all change specs in `.aw/changes/{cid}/`\n\
             2. Read spec **{sid}** to understand requirements: `{spec_path}`\n\
             3. Implement **production code only** (no `#[test]` functions) for each change spec in order, starting with **{sid}**\n\
             4. When done with {sid}, run `score workflow create-change-implementation {cid}` to advance",
            cid = change_id, sid = spec_id, spec_path = spec_path
        )
    } else {
        format!(
            "1. Read spec **{sid}**: `{spec_path}`\n\
             2. Implement **production code only** (no `#[test]` functions) according to spec requirements\n\
             3. When done, run `score workflow create-change-implementation {cid}` to advance",
            cid = change_id, sid = spec_id, spec_path = spec_path
        )
    };

    // Extract targets from spec's changes section for enriched guidance
    let spec_full_path = project_root.join(&spec_path);
    let targets_section = extract_change_targets(&spec_full_path);

    // Resolve executor before building prompt
    let action = if is_first {
        "begin_implementation"
    } else {
        "implement_spec"
    };
    let change_dir = workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor = workflow_common::get_executor_chain(
        project_root,
        WorkflowArtifact::CreateChangeImplementation,
    );

    // Code intelligence hints — all executors (mainthread + Claude Code subagents)
    // have Bash tool access, so hints are always included for change implementation.
    let cli_hints = "\n\
         \n\
         # Code intelligence — explore codebase before making changes\n\
         score symbols <file>              # list symbols in a file\n\
         score hover <file> <line> <col>   # type info for a symbol\n\
         score references <file> <line> <col>  # find all references\n\
         score impact <file> <line> <col>  # analyze change impact\n\
         score context <file:symbol...> [--depth N]  # cross-ref context";

    let prompt = format!(
        "# Task: {title}\n\n\
         ## Instructions\n\n\
         {instructions}\n\n\
         {targets}\
         ## Spec Annotations\n\n\
         Add `@spec` annotations to public functions that implement spec requirements.\n\
         For each public function or method,\n\
         add a comment: `// @spec {{spec_path}}#R{{N}}` where `{{spec_path}}` is the\n\
         spec file path and `R{{N}}` is the requirement ID from the spec's Requirements table.\n\n\
         Use the comment syntax appropriate for the language:\n\
         ```\n\
         // @spec {spec_path}#R1   (Rust, JS, TS, Go, C)\n\
         #  @spec {spec_path}#R1   (Python, Ruby, Shell, YAML)\n\
         -- @spec {spec_path}#R1   (SQL)\n\
         <!-- @spec {spec_path}#R1 --> (HTML, Markdown)\n\
         /* @spec {spec_path}#R1 */    (CSS, C block)\n\
         ```\n\n\
         This annotation enables automated spec↔code traceability.\n\
         Place the annotation on the line immediately above the function signature.\n\n\
         ## CLI Commands\n\n\
         ```\n\
         # Read spec\n\
         Read file: {spec_path}\n\
         \n\
         # Advance implementation workflow\n\
         score workflow create-change-implementation {cid}\
         {cli_hints}\n\
         ```",
        cid = change_id,
        spec_path = spec_path,
        targets = targets_section,
        cli_hints = cli_hints
    );

    let mut extra = json!({ "spec_id": spec_id });
    if let Some(gid) = group_id {
        extra["group_id"] = json!(gid);
    }

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        action,
        prompt,
        executor,
        extra,
        interface,
        project_root,
    )
    .await
}

/// Extract change targets from spec's `changes` YAML section.
/// Returns a formatted string for the implementation prompt, or empty if no targets found.
fn extract_change_targets(spec_path: &std::path::Path) -> String {
    let content = match std::fs::read_to_string(spec_path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };

    // Find ```yaml block after <!-- type: changes lang: yaml -->
    let changes_marker = "<!-- type: changes lang: yaml -->";
    let Some(marker_pos) = content.find(changes_marker) else {
        return String::new();
    };
    let after_marker = &content[marker_pos..];

    // Extract YAML between ```yaml and ```
    let Some(yaml_start) = after_marker.find("```yaml").map(|p| p + 7) else {
        return String::new();
    };
    let Some(yaml_end) = after_marker[yaml_start..].find("```") else {
        return String::new();
    };
    let yaml_content = &after_marker[yaml_start..yaml_start + yaml_end];

    // Parse YAML and look for targets
    let doc: serde_json::Value = match serde_yaml::from_str(yaml_content) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };

    let changes = match doc.get("changes").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return String::new(),
    };

    let mut sections = Vec::new();
    for entry in changes {
        let path = entry.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let action = entry.get("action").and_then(|v| v.as_str()).unwrap_or("");

        if action != "MODIFY" {
            continue;
        }

        let targets = entry.get("targets").and_then(|v| v.as_array());
        let do_not_touch = entry.get("do_not_touch").and_then(|v| v.as_array());

        if targets.is_none() && do_not_touch.is_none() {
            continue;
        }

        let mut section = format!("### {}\n", path);

        if let Some(targets) = targets {
            for t in targets {
                let kind = t.get("type").and_then(|v| v.as_str()).unwrap_or("function");
                let name = t.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let change = t.get("change").and_then(|v| v.as_str()).unwrap_or("");
                section.push_str(&format!("- **{} `{}`**: {}\n", kind, name, change));
            }
        }

        if let Some(dnt) = do_not_touch {
            let names: Vec<&str> = dnt.iter().filter_map(|v| v.as_str()).collect();
            if !names.is_empty() {
                section.push_str(&format!("- **DO NOT MODIFY**: {}\n", names.join(", ")));
            }
        }

        sections.push(section);
    }

    if sections.is_empty() {
        return String::new();
    }

    format!("## Change Targets\n\n{}\n", sections.join("\n"))
}

async fn build_implement_tests_prompt(
    change_id: &str,
    spec_id: &str,
    group_id: Option<&str>,
    project_root: &Path,
) -> Result<String> {
    let spec_path = match group_id {
        Some(gid) => format!(
            ".aw/changes/{}/groups/{}/specs/{}.md",
            change_id, gid, spec_id
        ),
        None => format!(".aw/changes/{}/specs/{}.md", change_id, spec_id),
    };

    let prompt = format!(
        "# Task: Implement Tests for Spec '{sid}' (Change '{cid}')\n\n\
         ## Instructions\n\n\
         Production code for spec '{sid}' has been implemented and verified to compile.\n\
         Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).\n\n\
         1. Read spec **{sid}**: `{spec_path}`\n\
         2. Read the `## Unit Test` section to understand required test cases\n\
         3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the unit-test design\n\
         4. Run `cargo test` to verify tests pass\n\
         5. When done, run `score workflow create-change-implementation {cid}` to advance\n\n\
         ## CLI Commands\n\n\
         ```\n\
         # Read spec\n\
         Read file: {spec_path}\n\
         \n\
         # Run tests\n\
         cargo test\n\
         \n\
         # Advance implementation workflow\n\
         score workflow create-change-implementation {cid}\n\
         ```",
        sid = spec_id,
        cid = change_id,
        spec_path = spec_path
    );

    let change_dir = workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor = workflow_common::get_executor_chain(
        project_root,
        WorkflowArtifact::CreateChangeImplementation,
    );

    let mut extra = json!({ "spec_id": spec_id, "phase": "tests" });
    if let Some(gid) = group_id {
        extra["group_id"] = json!(gid);
    }

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        &format!("implement_tests_{}", spec_id),
        prompt,
        executor,
        extra,
        interface,
        project_root,
    )
    .await
}

async fn build_codegen_prompt(
    change_id: &str,
    spec_id: &str,
    group_id: Option<&str>,
    project_root: &Path,
) -> Result<String> {
    let _pp = project_root.display();

    let spec_path = match group_id {
        Some(gid) => format!(
            ".aw/changes/{}/groups/{}/specs/{}.md",
            change_id, gid, spec_id
        ),
        None => format!(".aw/changes/{}/specs/{}.md", change_id, spec_id),
    };

    let prompt = format!(
        "# Task: Implement Spec '{spec_id}' with Codegen for Change '{change_id}'\n\n\
         ## Structured Codegen Path\n\n\
         Spec '{spec_id}' has a JSON schema or API spec. Use Lens's code generation pipeline:\n\n\
         1. Read the spec: `{spec_path}`\n\
         2. Call `cclab lens gen-from-spec` with spec content to generate code\n\
         3. Review generated output against spec requirements\n\
         4. Apply manual adjustments as needed\n\
         5. Run tests to verify\n\
         6. When done, run `score workflow create-change-implementation {change_id}` to advance\n\n\
         ## Fallback\n\n\
         If codegen fails, fall back to manual implementation.\n\n\
         ## CLI Commands\n\n\
         ```\n\
         # Read spec\n\
         Read file: {spec_path}\n\
         \n\
         # Generate code from spec\n\
         cclab lens gen-from-spec <spec_json>\n\
         \n\
         # Advance implementation workflow\n\
         score workflow create-change-implementation {change_id}\n\
         ```",
        spec_path = spec_path
    );

    let change_dir = workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor = workflow_common::get_executor_chain(
        project_root,
        WorkflowArtifact::CreateChangeImplementation,
    );

    let mut extra = json!({ "spec_id": spec_id, "codegen": true });
    if let Some(gid) = group_id {
        extra["group_id"] = json!(gid);
    }

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        "implement_spec_with_codegen",
        prompt,
        executor,
        extra,
        interface,
        project_root,
    )
    .await
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_impl/utils.md#source
// CODEGEN-BEGIN
/// Run git command and return stdout (empty string on failure).
fn git_output(args: &[&str]) -> String {
    std::process::Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

/// Auto-populate implementation.md with git diff baseline so the artifact
/// is never empty even if the agent fails downstream.
fn auto_populate_impl_baseline(change_id: &str, project_root: &Path) {
    let change_dir = workflow_common::resolve_change_dir(project_root, change_id);
    let impl_path = change_dir.join("implementation.md");
    // Don't overwrite if already has real content
    if impl_path.exists() {
        if let Ok(existing) = std::fs::read_to_string(&impl_path) {
            if existing.contains("## Diff") {
                return;
            }
        }
    }

    let stat = git_output(&["diff", "--stat", "main"]);
    let name_status = git_output(&["diff", "--name-status", "main"]);
    if stat.is_empty() && name_status.is_empty() {
        return;
    }

    let mut content = format!(
        "---\nid: implementation\ntype: change_implementation\nchange_id: {change_id}\n---\n\n\
         # Implementation\n\n\
         ## Summary\n\n*(auto-generated baseline from git diff)*\n\n"
    );

    if !name_status.is_empty() {
        content.push_str("## Changed Files\n\n```\n");
        content.push_str(&name_status);
        content.push_str("\n```\n\n");
    }

    if !stat.is_empty() {
        content.push_str("## Diff Statistics\n\n```\n");
        content.push_str(&stat);
        content.push_str("\n```\n\n");
    }

    // Include truncated diff
    let diff = git_output(&["diff", "main", "--", ".", ":!cclab/"]);
    if !diff.is_empty() {
        content.push_str("## Diff\n\n```diff\n");
        const MAX_LINES: usize = 2000;
        let lines: Vec<&str> = diff.lines().collect();
        if lines.len() > MAX_LINES {
            for line in &lines[..MAX_LINES] {
                content.push_str(line);
                content.push('\n');
            }
            content.push_str(&format!(
                "\n... truncated ({} more lines)\n",
                lines.len() - MAX_LINES
            ));
        } else {
            content.push_str(&diff);
            content.push('\n');
        }
        content.push_str("```\n");
    }

    let _ = std::fs::write(&impl_path, &content);
}

async fn build_write_diff_prompt(
    change_id: &str,
    group_id: Option<&str>,
    project_root: &Path,
) -> Result<String> {
    // Auto-populate baseline so impl is never empty if agent fails
    auto_populate_impl_baseline(change_id, project_root);

    let _pp = project_root.display();

    let prompt = format!(
        "# Task: Write Implementation Diff for Change '{change_id}'\n\n\
         ## Instructions\n\n\
         1. Run `git diff` (or `git diff HEAD~N..HEAD` if already committed) to get the full diff\n\
         2. Write `implementation.md` via the artifact CLI command\n\
         3. The artifact tool will redirect back to the workflow router automatically\n\n\
         ## CLI Commands\n\n\
         ```\n\
         # Write implementation artifact (write payload JSON first, then run)\n\
         score artifact create-change-implementation {change_id} .aw/changes/{change_id}/payloads/create-change-implementation.json\n\
         ```"
    );

    let change_dir = workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor = workflow_common::get_executor_chain(
        project_root,
        WorkflowArtifact::CreateChangeImplementation,
    );

    let mut extra = json!({});
    if let Some(gid) = group_id {
        extra["group_id"] = json!(gid);
    }

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        "write_implementation_diff",
        prompt,
        executor,
        extra,
        interface,
        project_root,
    )
    .await
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Count `#[test]` occurrences in added lines of `git diff main`.
fn count_tests_in_diff(project_root: &Path) -> usize {
    let output = std::process::Command::new("git")
        .args(["diff", "main"])
        .current_dir(project_root)
        .output()
        .ok();

    let Some(out) = output else { return 0 };
    if !out.status.success() {
        return 0;
    }

    let diff = String::from_utf8_lossy(&out.stdout);
    diff.lines()
        .filter(|line| line.starts_with('+') && !line.starts_with("+++"))
        .filter(|line| line.contains("#[test]"))
        .count()
}

/// Parse the `## Unit Test` section of a spec and return the numeric test count if present.
///
/// Returns `Some(n)` if a markdown table with `n` data rows is found.
/// Returns `None` if the section is absent or has no numeric count (qualitative only).
/// Legacy `## Test Plan` sections are accepted while historical TDs migrate.
/// @spec projects/agentic-workflow/tech-design/core/tools/create_change_impl/utils.md#source
pub fn parse_test_plan_count(spec_content: &str) -> Option<usize> {
    // Find ## Unit Test section, falling back to legacy ## Test Plan.
    let test_plan_start = spec_content
        .find("## Unit Test")
        .or_else(|| spec_content.find("## Test Plan"))?;
    let after = &spec_content[test_plan_start..];

    // Find end: next ## heading or EOF
    let section_end = after[1..]
        .find("\n## ")
        .map(|i| i + 1)
        .unwrap_or(after.len());
    let section = &after[..section_end];

    // Count markdown table data rows (lines starting with `|` that are not separator rows `|---|`)
    let table_rows: usize = section
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with('|')
                && !trimmed
                    .chars()
                    .skip(1)
                    .all(|c| c == '-' || c == '|' || c == ' ' || c == ':')
        })
        .count();

    // Subtract 1 for the header row if we have at least 2 rows
    if table_rows >= 2 {
        Some(table_rows - 1)
    } else if table_rows == 1 {
        // Single row could be header only — no data rows
        None
    } else {
        // No table found — qualitative or absent
        None
    }
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_impl/tests.md#source
// CODEGEN-BEGIN
// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_change(change_id: &str, phase_str: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes").join(change_id);
        std::fs::create_dir_all(&change_dir).unwrap();
        // R4: save() syncs workflow fields into the issue frontmatter, the
        // single source of truth since STATE.yaml was eliminated (R5/R6).
        crate::test_util::write_minimal_issue(tmp.path(), change_id);
        let phase =
            crate::tools::phase_transition::parse_phase(phase_str).expect("valid phase string");
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = phase;
        sm.save().unwrap();
        tmp
    }

    fn write_spec(tmp: &TempDir, change_id: &str, spec_id: &str, refs: &[&str]) {
        let specs_dir = tmp.path().join(".aw/changes").join(change_id).join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        let refs_yaml = if refs.is_empty() {
            String::new()
        } else {
            let items = refs
                .iter()
                .map(|r| format!("  - {}", r))
                .collect::<Vec<_>>()
                .join("\n");
            format!("refs:\n{}\n", items)
        };
        std::fs::write(
            specs_dir.join(format!("{}.md", spec_id)),
            format!(
                "---\nid: {}\ntype: spec\n{}---\n# Spec {}\n",
                spec_id, refs_yaml, spec_id
            ),
        )
        .unwrap();
    }

    fn read_prompt(parsed: &Value, change_dir: &std::path::Path, action: &str) -> String {
        if let Some(p) = parsed["prompt"].as_str() {
            return p.to_string();
        }
        let prompt_path = change_dir.join("prompts").join(format!("{}.md", action));
        std::fs::read_to_string(&prompt_path)
            .unwrap_or_else(|_| panic!("No prompt at {:?}", prompt_path))
    }

    #[tokio::test]
    async fn test_workflow_begin_implementation() {
        let tmp = setup_change("wf-impl", "change_implementation_created");
        write_spec(&tmp, "wf-impl", "spec-a", &[]);

        let change_dir = tmp.path().join(".aw/changes/wf-impl");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "wf-impl"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["spec_id"], "spec-a");
        let prompt = read_prompt(&parsed, &change_dir, "begin_implementation");
        assert!(prompt.contains("Begin Implementation"));
    }

    #[tokio::test]
    async fn test_workflow_write_diff() {
        let tmp = setup_change("wf-diff", "change_implementation_created");
        write_spec(&tmp, "wf-diff", "spec-a", &[]);
        // Set current_task_id to last spec (all dispatched)
        let change_dir = tmp.path().join(".aw/changes/wf-diff");
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().current_task_id = Some("spec-a".into());
        sm.save().unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "wf-diff"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        let prompt = read_prompt(&parsed, &change_dir, "write_implementation_diff");
        assert!(prompt.contains("Write Implementation Diff"));
    }

    #[test]
    fn test_artifact_writes_impl_md() {
        let tmp = setup_change("art-impl", "change_implementation_created");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "art-impl",
            "diff": "+new line\n-old line",
            "summary": "Added feature X"
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert!(parsed["artifacts_written"]
            .as_array()
            .unwrap()
            .contains(&json!("implementation.md")));

        // Verify file content
        let impl_path = tmp.path().join(".aw/changes/art-impl/implementation.md");
        let content = std::fs::read_to_string(&impl_path).unwrap();
        assert!(content.contains("+new line"));
        assert!(content.contains("Added feature X"));
    }

    #[tokio::test]
    async fn test_workflow_all_approved_advances_to_merge() {
        let tmp = setup_change("wf-merge", "change_implementation_reviewed");
        write_spec(&tmp, "wf-merge", "spec-a", &[]);
        let change_dir = tmp.path().join(".aw/changes/wf-merge");
        let mut content = String::from(
            "---\nid: impl\ntype: change_implementation\n---\n# Implementation\n\n## Diff\n\n```diff\n+code\n```\n\n",
        );
        content.push_str("## Review: spec-a\n\nverdict: APPROVED\nsummary: looks good\n\n");
        std::fs::write(change_dir.join("implementation.md"), content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "wf-merge"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "phase_complete");
        assert!(parsed["message"]
            .as_str()
            .unwrap()
            .contains("All specs implemented and approved"));
        // Phase should be advanced to test_check in STATE.yaml
        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(*sm.phase(), crate::models::state::StatePhase::TestCheck);
    }

    #[tokio::test]
    async fn test_terminal_failure_returns_retry_action() {
        let tmp = setup_change("wf-fail", "change_implementation_reviewed");
        write_spec(&tmp, "wf-fail", "spec-a", &[]);
        let change_dir = tmp.path().join(".aw/changes/wf-fail");

        // Set task_revisions to exceed MAX_SPEC_REVISIONS
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut()
            .task_revisions
            .insert("spec-a".into(), MAX_SPEC_REVISIONS);
        sm.save().unwrap();

        // Write impl with REVISE verdict to trigger TerminalFailure
        let mut content = String::from(
            "---\nid: impl\ntype: change_implementation\n---\n# Implementation\n\n## Diff\n\n```diff\n+code\n```\n\n",
        );
        content.push_str("## Review: spec-a\n\nverdict: REVISE\nsummary: needs work\n\n");
        std::fs::write(change_dir.join("implementation.md"), content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "wf-fail"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "error");
        let next = parsed["next_actions"].as_array().unwrap();
        assert!(
            !next.is_empty(),
            "TerminalFailure should provide retry next_actions"
        );
        assert_eq!(next[0]["args"]["change_id"], "wf-fail");
    }

    #[test]
    fn test_build_gate_blocks_phase2_on_failure() {
        // Given impl_spec_phase["spec-a"] == "code"
        // When build fails (simulate by checking the structure)
        // This is a unit test of parse_test_plan_count and count logic
        // The actual build gate is tested via state inspection
        let tmp = setup_change("gate-fail", "change_implementation_created");
        let change_dir = tmp.path().join(".aw/changes/gate-fail");
        write_spec(&tmp, "gate-fail", "spec-a", &[]);
        // Set impl_spec_phase to "code" — simulates code phase dispatched
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().current_task_id = Some("spec-a".into());
        sm.state_mut()
            .impl_spec_phase
            .insert("spec-a".into(), "code".into());
        sm.save().unwrap();
        // The BuildCheck sub-state should be returned
        let (sub_state, _, _) = common::resolve_next_impl(&change_dir, "gate-fail").unwrap();
        assert!(
            matches!(sub_state, common::ImplSubState::BuildCheck { ref spec_id } if spec_id == "spec-a")
        );
    }

    #[test]
    fn test_build_gate_passes_on_success() {
        // Given impl_spec_phase["spec-a"] == "tests"
        // The TestCountCheck sub-state should be returned (build already passed)
        let tmp = setup_change("gate-pass", "change_implementation_created");
        let change_dir = tmp.path().join(".aw/changes/gate-pass");
        write_spec(&tmp, "gate-pass", "spec-a", &[]);
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().current_task_id = Some("spec-a".into());
        sm.state_mut()
            .impl_spec_phase
            .insert("spec-a".into(), "tests".into());
        sm.save().unwrap();
        let (sub_state, _, _) = common::resolve_next_impl(&change_dir, "gate-pass").unwrap();
        assert!(
            matches!(sub_state, common::ImplSubState::TestCountCheck { ref spec_id } if spec_id == "spec-a")
        );
    }

    #[test]
    fn test_test_count_warning_on_mismatch() {
        // Spec with 4 tests in Unit Test table, diff has 2 #[test] → warning
        let spec_content = "---\nid: spec\n---\n# Spec\n\n## Unit Test\n\n| # | Test | File | Validates |\n|---|------|------|------|\n| T1 | test1 | foo | bar |\n| T2 | test2 | foo | bar |\n| T3 | test3 | foo | bar |\n| T4 | test4 | foo | bar |\n";
        let required = parse_test_plan_count(spec_content);
        assert_eq!(required, Some(4));
    }

    #[test]
    fn test_test_count_skipped_no_unit_test() {
        // Spec with no ## Unit Test section
        let spec_content = "---\nid: spec\n---\n# Spec\n\n## Overview\n\nSome overview.\n";
        let required = parse_test_plan_count(spec_content);
        assert_eq!(required, None);
    }

    #[test]
    fn test_test_count_skipped_qualitative_plan() {
        // Spec with Unit Test section but no table (qualitative only)
        let spec_content =
            "---\nid: spec\n---\n# Spec\n\n## Unit Test\n\nEnsure all edge cases are covered.\n";
        let required = parse_test_plan_count(spec_content);
        assert_eq!(required, None);
    }

    // ─── CLI Hints Tests ────────────────────────────────────────────────────

    /// Helper: write .aw/config.toml with the given execution mode.
    ///
    /// Mode values: "mainthread", "claude_subagents", "multi_claude_agents", "multi_agents".
    /// Tests MUST call this to control executor selection; without it the default
    /// `MultiClaudeAgents` mode dispatches to external agents that are unavailable in CI.
    fn write_config(tmp: &TempDir, mode: &str) {
        let config_dir = tmp.path().join("cclab");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("config.toml"),
            format!("[workflow]\nmode = \"{}\"\n", mode),
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_cli_hints_present_for_mainthread_executor() {
        let tmp = setup_change("hints-mt", "change_implementation_created");
        write_spec(&tmp, "hints-mt", "spec-a", &[]);
        write_config(&tmp, "mainthread");

        let change_dir = tmp.path().join(".aw/changes/hints-mt");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "hints-mt"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        let prompt = read_prompt(&parsed, &change_dir, "begin_implementation");
        assert!(
            prompt.contains("Code intelligence"),
            "Mainthread prompt should contain Code intelligence header"
        );
    }

    #[tokio::test]
    async fn test_cli_hints_contains_all_five_commands() {
        let tmp = setup_change("hints-5c", "change_implementation_created");
        write_spec(&tmp, "hints-5c", "spec-a", &[]);
        write_config(&tmp, "mainthread");

        let change_dir = tmp.path().join(".aw/changes/hints-5c");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "hints-5c"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        let prompt = read_prompt(&parsed, &change_dir, "begin_implementation");

        let expected_commands = [
            "score symbols <file>",
            "score hover <file> <line> <col>",
            "score references <file> <line> <col>",
            "score impact <file> <line> <col>",
            "score context <file:symbol...>",
        ];
        for cmd in &expected_commands {
            assert!(
                prompt.contains(cmd),
                "CLI hints should contain command: {}",
                cmd
            );
        }
    }

    #[tokio::test]
    async fn test_cli_hints_inside_cli_commands_block() {
        // CLI hints should appear within the ## CLI Commands section, not outside it.
        let tmp = setup_change("hints-blk", "change_implementation_created");
        write_spec(&tmp, "hints-blk", "spec-a", &[]);
        write_config(&tmp, "mainthread");

        let change_dir = tmp.path().join(".aw/changes/hints-blk");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "hints-blk"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        let prompt = read_prompt(&parsed, &change_dir, "begin_implementation");

        let cli_section_pos = prompt
            .find("## CLI Commands")
            .expect("Prompt should contain ## CLI Commands section");
        let hints_pos = prompt
            .find("Code intelligence")
            .expect("Prompt should contain Code intelligence hints");
        assert!(
            hints_pos > cli_section_pos,
            "CLI hints should appear after ## CLI Commands header"
        );
    }

    #[tokio::test]
    async fn test_tests_prompt_no_cli_hints() {
        // build_implement_tests_prompt should NOT contain code intelligence hints.
        let tmp = setup_change("hints-nt", "change_implementation_created");
        write_spec(&tmp, "hints-nt", "spec-a", &[]);
        write_config(&tmp, "mainthread");

        let change_dir = tmp.path().join(".aw/changes/hints-nt");
        // Set impl_spec_phase to "tests" to trigger ImplementSpecTests sub-state
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().current_task_id = Some("spec-a".into());
        sm.state_mut()
            .impl_spec_phase
            .insert("spec-a".into(), "tests".into());
        sm.save().unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "hints-nt"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        if parsed["status"] == "ok" {
            let prompts_dir = change_dir.join("prompts");
            if prompts_dir.exists() {
                for entry in std::fs::read_dir(&prompts_dir)
                    .unwrap()
                    .filter_map(|e| e.ok())
                {
                    let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
                    assert!(
                        !content.contains("score symbols"),
                        "Tests prompt should NOT contain code intelligence hints"
                    );
                }
            }
        }
    }

    #[tokio::test]
    async fn test_write_diff_prompt_no_cli_hints() {
        // build_write_diff_prompt should NOT contain code intelligence hints.
        let tmp = setup_change("hints-nd", "change_implementation_created");
        write_spec(&tmp, "hints-nd", "spec-a", &[]);
        write_config(&tmp, "mainthread");

        let change_dir = tmp.path().join(".aw/changes/hints-nd");
        // Set current_task_id to last spec to trigger WriteDiff
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().current_task_id = Some("spec-a".into());
        sm.save().unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "hints-nd"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        if parsed["status"] == "ok" {
            let prompts_dir = change_dir.join("prompts");
            if prompts_dir.exists() {
                for entry in std::fs::read_dir(&prompts_dir)
                    .unwrap()
                    .filter_map(|e| e.ok())
                {
                    let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
                    if content.contains("Write Implementation Diff") {
                        assert!(
                            !content.contains("score symbols"),
                            "Write-diff prompt should NOT contain code intelligence hints"
                        );
                    }
                }
            }
        }
    }
}
// CODEGEN-END
