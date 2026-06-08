// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/workflow/mod.md#source
// CODEGEN-BEGIN
//! sdd_run_change MCP Tool — Pure Bridge
//!
//! Reads STATE.yaml, determines the next action, and returns the appropriate
//! `sdd_workflow_*` tool for mainthread to call. No internal agent dispatch.
//!
//! Each call reads state once and returns immediately with `next_actions` pointing to
//! the workflow tool that handles that phase. The loop lives in the skill
//! template — mainthread calls `sdd_run_change` repeatedly to advance.
//!
//! ## File organization
//!
//! - post_clarifications.rs: PostClarificationsCreated routing
//! - helpers.rs: Shared helpers (verdicts, spec analysis, etc.)

pub mod helpers;
mod post_clarifications;
pub mod scope;
pub mod task_graph;
pub mod test_gate;

use crate::models::change::SddInterface;
use crate::models::state::StatePhase;
use crate::state::StateManager;
use crate::tools::workflow_common::{self, validate_change_id};
use crate::tools::{get_required_string, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

/// Get the tool definition for sdd_run_change
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/mod.md#source
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_run_change".to_string(),
        description: "Unified workflow bridge. Reads STATE.yaml and returns the next \
            `sdd_workflow_*` tool for mainthread to call."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path (use $PWD for current directory)"
                },
                "change_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Change ID (lowercase, hyphens allowed)"
                },
                "description": {
                    "type": "string",
                    "description": "User's description of the change (required for new changes)"
                },
                "issues": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Issue references (e.g. [\"#188\", \"#189\"]). Must be explicitly provided — not auto-detected from description."
                },
                "git_workflow": {
                    "type": "string",
                    "enum": ["new_branch", "in_place"],
                    "description": "Git workflow chosen by user: 'new_branch' or 'in_place'"
                },
                "last_action": {
                    "type": "string",
                    "description": "Action label for telemetry"
                },
                "skip_tests": {
                    "type": "boolean",
                    "description": "Skip TestCheck gate (logs warning, sets tests_skipped in STATE.yaml)"
                },
            }
        }),
    }
}

/// Execute the sdd_run_change tool
///
/// Pure bridge: reads STATE.yaml, determines next action, returns `next_actions`
/// pointing to the appropriate `sdd_workflow_*` tool. No agent dispatch.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/mod.md#source
pub async fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    validate_change_id(&change_id)?;

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);

    // Load interface mode from config
    let interface = workflow_common::load_interface(project_root);

    let description: Option<String> = args
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let issues: Option<Vec<String>> = args.get("issues").and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    });

    let git_workflow: Option<String> = args
        .get("git_workflow")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // @spec projects/agentic-workflow/tech-design/core/logic/tdd-gate.md#R9
    let skip_tests = args
        .get("skip_tests")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // --- Route to init_change if change dir doesn't exist ---
    if !change_dir.exists() {
        // REQ: issue-centric-workflow#R1 — --issue alone is sufficient.
        // When description is absent, synthesize from issue title + slug.
        let effective_description = match description {
            Some(ref d) => d.clone(),
            None => {
                // Try to derive from --issue refs
                let slug = issues.as_deref().and_then(|refs| {
                    crate::services::issue_parser::resolve_issue_slug(project_root, "", Some(refs))
                });
                match slug {
                    Some(ref s) => {
                        let title =
                            crate::services::issue_parser::load_issue_title(project_root, s)
                                .unwrap_or_else(|| s.clone());
                        format!("{} issue:{}", title, s)
                    }
                    None => anyhow::bail!(
                        "Either --description or --issue is required for new changes. \
                         Usage: score run-change --change-id '{}' --issue \"<slug>\"",
                        change_id
                    ),
                }
            }
        };

        let mut init_args = json!({
            "change_id": &change_id,
            "description": &effective_description,
        });
        if let Some(ref refs) = issues {
            if !refs.is_empty() {
                init_args["issues"] = json!(refs);
            }
        }
        if let Some(ref wf) = git_workflow {
            init_args["git_workflow"] = json!(wf);
        }

        return Ok(serde_json::to_string_pretty(&json!({
            "change_id": &change_id,
            "current_phase": Value::Null,
            "action": "init_change",
            "executor": ["mainthread"],
            "message": "Initialize change directory, user_input.md, and STATE.yaml.",
            "next_actions": [helpers::next_action(interface, "sdd_workflow_init_change", init_args)]
        }))?);
    }

    // --- Route: determine the next workflow tool ---
    let response = route(&change_dir, &change_id, interface, skip_tests)?;
    Ok(serde_json::to_string_pretty(&response)?)
}

// ---------------------------------------------------------------------------
// Direct phase → workflow tool routing
// ---------------------------------------------------------------------------

/// Route to the appropriate workflow tool based on current StatePhase.
///
/// Each phase maps to a single `sdd_workflow_*` tool. The per-action tools
/// handle sub-state routing internally (CRR cycles, group progress, verdicts).
///
/// For alignment-eligible phases (ChangeSpecCreated, ChangeSpecReviewed,
/// ChangeImplementationCreated, ChangeImplementationReviewed), the response
/// includes `alignment_warnings` from the current group's spec files.
/// For all other phases, `alignment_warnings` is `null`.
fn route(
    change_dir: &Path,
    change_id: &str,
    interface: SddInterface,
    skip_tests: bool,
) -> Result<Value> {
    let sm_result = StateManager::load(change_dir);
    let phase = match &sm_result {
        Ok(sm) => sm.phase().clone(),
        Err(_) => StatePhase::ChangeInited,
    };

    let phase_str = workflow_common::phase_to_string(&phase);

    // Compute alignment warnings for eligible phases (R25)
    let alignment_warnings: Option<Vec<serde_json::Value>> = match phase {
        StatePhase::ChangeSpecCreated
        | StatePhase::ChangeSpecReviewed
        | StatePhase::ChangeImplementationCreated
        | StatePhase::ChangeImplementationReviewed => {
            helpers::collect_alignment_warnings(change_dir)
        }
        _ => None,
    };

    let mut response = match phase {
        // ChangeInited is the entry point after init_change.
        // Issue body provides all context — route directly to spec creation.
        StatePhase::ChangeInited => {
            let na = helpers::next_action(
                interface,
                "sdd_workflow_create_change_spec",
                json!({"change_id": change_id}),
            );
            Ok::<Value, anyhow::Error>(json!({
                "change_id": change_id,
                "current_phase": phase_str,
                "action": "delegate_to_per_action_tools",
                "message": "Issue provides context. Spec lifecycle managed by per-action tools.",
                "executor": ["mainthread"],
                "next_actions": [na]
            }))
        }

        StatePhase::ChangeSpecCreated
        | StatePhase::ChangeSpecReviewed
        | StatePhase::ChangeSpecRevised => {
            let na = helpers::next_action(
                interface,
                "sdd_workflow_create_change_spec",
                json!({"change_id": change_id}),
            );
            Ok(json!({
                "change_id": change_id,
                "current_phase": phase_str,
                "action": "delegate_to_per_action_tools",
                "message": "Spec lifecycle managed by per-action tools.",
                "executor": ["mainthread"],
                "next_actions": [na]
            }))
        }

        StatePhase::ChangeImplementationCreated
        | StatePhase::ChangeImplementationReviewed
        | StatePhase::ChangeImplementationRevised => {
            let na = helpers::next_action(
                interface,
                "sdd_workflow_create_change_implementation",
                json!({"change_id": change_id}),
            );
            Ok(json!({
                "change_id": change_id,
                "current_phase": phase_str,
                "action": "delegate_to_per_action_tools",
                "message": "Implementation lifecycle managed by per-action tools.",
                "executor": ["mainthread"],
                "next_actions": [na]
            }))
        }

        // TestCheck: transient phase resolved inline (same pattern as DocsCheck)
        // @spec projects/agentic-workflow/tech-design/core/logic/tdd-gate.md#R4
        StatePhase::TestCheck => {
            // --skip-tests escape hatch (R9)
            if skip_tests {
                tracing::warn!("TestCheck: skipped via --skip-tests flag");
                // Set tests_skipped in STATE.yaml
                if let Ok(mut sm) = StateManager::load(change_dir) {
                    // We don't have a dedicated field, so we record it via last_action
                    sm.state_mut().last_action = Some("tests_skipped".to_string());
                    let _ = sm.save();
                }
                let na = helpers::next_action(
                    interface,
                    "sdd_workflow_create_change_docs",
                    json!({"change_id": change_id}),
                );
                return Ok(json!({
                    "change_id": change_id,
                    "current_phase": "test_check",
                    "action": "test_check_skipped",
                    "message": "WARNING: TestCheck skipped via --skip-tests flag. Tests were NOT run.",
                    "test_skipped": true,
                    "tests_skipped": true,
                    "executor": ["mainthread"],
                    "next_actions": [na]
                }));
            }

            // TestCheck is never persisted — it's resolved inline here.
            // The implementation workflow tool advances to TestCheck after APPROVED review.
            // We run the test gate and either advance to DocsCheck or revert to re-implement.
            let project_root = change_dir
                .parent() // .aw/changes
                .and_then(|p| p.parent()) // .aw
                .and_then(|p| p.parent()) // project root
                .unwrap_or(Path::new("."));

            let gate_result = test_gate::run_full_test_gate(change_dir, project_root);

            match gate_result {
                Ok(result) if result.passed => {
                    // Pass or skip → advance to DocsCheck
                    let na = helpers::next_action(
                        interface,
                        "sdd_workflow_create_change_docs",
                        json!({"change_id": change_id}),
                    );
                    Ok(json!({
                        "change_id": change_id,
                        "current_phase": "test_check",
                        "action": "test_check_passed",
                        "message": result.messages.join("\n"),
                        "test_skipped": result.skipped,
                        "executor": ["mainthread"],
                        "next_actions": [na]
                    }))
                }
                Ok(result) => {
                    // Gate failed → revert to ChangeImplementationCreated for re-implementation
                    let na = helpers::next_action(
                        interface,
                        "sdd_workflow_create_change_implementation",
                        json!({"change_id": change_id}),
                    );
                    Ok(json!({
                        "change_id": change_id,
                        "current_phase": "test_check",
                        "action": "test_check_failed",
                        "message": result.messages.join("\n"),
                        "test_skipped": false,
                        "executor": ["mainthread"],
                        "next_actions": [na]
                    }))
                }
                Err(e) => {
                    // Error running gate → treat as failure
                    let na = helpers::next_action(
                        interface,
                        "sdd_workflow_create_change_implementation",
                        json!({"change_id": change_id}),
                    );
                    Ok(json!({
                        "change_id": change_id,
                        "current_phase": "test_check",
                        "action": "test_check_failed",
                        "message": format!("TestCheck error: {}", e),
                        "test_skipped": false,
                        "executor": ["mainthread"],
                        "next_actions": [na]
                    }))
                }
            }
        }

        StatePhase::DocsCheck
        | StatePhase::DocsCreated
        | StatePhase::DocsReviewed
        | StatePhase::DocsRevised => {
            let na = helpers::next_action(
                interface,
                "sdd_workflow_create_change_docs",
                json!({"change_id": change_id}),
            );
            Ok(json!({
                "change_id": change_id,
                "current_phase": phase_str,
                "action": "delegate_to_per_action_tools",
                "message": "Docs lifecycle managed by per-action tools.",
                "executor": ["mainthread"],
                "next_actions": [na]
            }))
        }

        StatePhase::ChangeMergeCreated
        | StatePhase::ChangeMergeReviewed
        | StatePhase::ChangeMergeRevised => {
            let na = helpers::next_action(
                interface,
                "sdd_workflow_create_change_merge",
                json!({"change_id": change_id}),
            );
            Ok(json!({
                "change_id": change_id,
                "current_phase": phase_str,
                "action": "begin_merge",
                "message": "Merge is programmatic — specs will be copied to .aw/tech-design/ automatically.",
                "executor": ["mainthread"],
                "next_actions": [na]
            }))
        }

        StatePhase::ChangeArchived | StatePhase::ChangeRejected => Ok(json!({
            "action": "complete",
            "change_id": change_id,
            "current_phase": phase_str,
            "executor": ["mainthread"],
            "message": "Change workflow is complete (archived or rejected).",
            "next_actions": [],
        })),
    }?;

    // Inject alignment_warnings into every response (R25)
    if let Some(obj) = response.as_object_mut() {
        match alignment_warnings {
            Some(warnings) => {
                obj.insert("alignment_warnings".to_string(), json!(warnings));
            }
            None => {
                obj.insert("alignment_warnings".to_string(), Value::Null);
            }
        };

        // Inject dispatch hints for mainthread continuous loop
        obj.insert("is_terminal".to_string(), json!(phase.is_terminal()));
        obj.insert("user_action_required".to_string(), json!(false));

        // Tell mainthread which agent type to dispatch.
        // Model is NOT specified here — it lives in the agent definition
        // (`.claude/agents/<type>.md` frontmatter `model:` field).
        let default_agent = match phase {
            // ChangeInited → spec creation (issue body provides context)
            StatePhase::ChangeInited
            | StatePhase::ChangeSpecCreated
            | StatePhase::ChangeSpecRevised => "score-change-spec",

            // Spec review → quality gate
            StatePhase::ChangeSpecReviewed => "score-review",

            // Implementation create/revise
            StatePhase::ChangeImplementationCreated
            | StatePhase::ChangeImplementationRevised
            | StatePhase::TestCheck => "score-change-implementation",

            // Implementation review → quality gate
            StatePhase::ChangeImplementationReviewed => "score-review",

            // Docs phase
            StatePhase::DocsCheck | StatePhase::DocsCreated | StatePhase::DocsRevised => {
                "score-change-spec"
            }

            StatePhase::DocsReviewed => "score-review",

            // Merge
            StatePhase::ChangeMergeCreated
            | StatePhase::ChangeMergeReviewed
            | StatePhase::ChangeMergeRevised => "score-change-implementation",

            // Terminal
            StatePhase::ChangeArchived | StatePhase::ChangeRejected => "mainthread",
        };
        obj.insert("default_agent".to_string(), json!(default_agent));
    }

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Set up a change directory backed by an issue at a given phase.
    /// R4 (refactor-eliminate-state-yaml-user-input-md-groups-nesting): save()
    /// needs an issue file to sync workflow fields into.
    fn setup_change(phase: StatePhase) -> (TempDir, String) {
        let tmp = TempDir::new().unwrap();
        let change_id = "test-change";
        let change_dir = tmp.path().join(".aw/changes").join(change_id);
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(tmp.path(), change_id);

        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = phase;
        sm.save().unwrap();

        (tmp, change_id.to_string())
    }

    fn call_route(tmp: &TempDir, change_id: &str) -> Value {
        let change_dir = tmp.path().join(".aw/changes").join(change_id);
        let interface = SddInterface::Cli;
        route(&change_dir, change_id, interface, false).unwrap()
    }

    // ── ChangeInited: should bail (hard gate) ───────────────────────────

    // ── ChangeInited → create-change-spec (entry point) ─────────────────

    #[test]
    fn test_route_change_inited_to_spec() {
        let (tmp, cid) = setup_change(StatePhase::ChangeInited);
        let resp = call_route(&tmp, &cid);
        assert_eq!(resp["current_phase"], "change_inited");
        assert_eq!(resp["default_agent"], "score-change-spec");
        assert!(
            resp.get("model").is_none(),
            "model should not be in response — lives in agent definition"
        );
        assert!(!resp["is_terminal"].as_bool().unwrap());
    }

    // ── ChangeSpecCreated → score-change-spec:sonnet ────────────────────

    #[test]
    fn test_route_change_spec_created() {
        let (tmp, cid) = setup_change(StatePhase::ChangeSpecCreated);
        let resp = call_route(&tmp, &cid);
        assert_eq!(resp["current_phase"], "change_spec_created");
        assert_eq!(resp["default_agent"], "score-change-spec");
        assert!(
            resp.get("model").is_none(),
            "model should not be in response — lives in agent definition"
        );
        let cli = resp["next_actions"][0]["cli"].as_str().unwrap();
        assert!(cli.contains("create-change-spec"));
    }

    // ── ChangeSpecReviewed → score-review:opus ──────────────────────────

    #[test]
    fn test_route_change_spec_reviewed_uses_review_agent() {
        let (tmp, cid) = setup_change(StatePhase::ChangeSpecReviewed);
        let resp = call_route(&tmp, &cid);
        assert_eq!(resp["default_agent"], "score-review");
        assert!(
            resp.get("model").is_none(),
            "model should not be in response — lives in agent definition"
        );
    }

    // ── ChangeImplCreated → score-change-implementation:sonnet ──────────

    #[test]
    fn test_route_change_impl_created() {
        let (tmp, cid) = setup_change(StatePhase::ChangeImplementationCreated);
        let resp = call_route(&tmp, &cid);
        assert_eq!(resp["default_agent"], "score-change-implementation");
        assert!(
            resp.get("model").is_none(),
            "model should not be in response — lives in agent definition"
        );
        let cli = resp["next_actions"][0]["cli"].as_str().unwrap();
        assert!(cli.contains("create-change-implementation"));
    }

    // ── ChangeImplReviewed → score-review:opus ──────────────────────────

    #[test]
    fn test_route_change_impl_reviewed_uses_review_agent() {
        let (tmp, cid) = setup_change(StatePhase::ChangeImplementationReviewed);
        let resp = call_route(&tmp, &cid);
        assert_eq!(resp["default_agent"], "score-review");
        assert!(
            resp.get("model").is_none(),
            "model should not be in response — lives in agent definition"
        );
    }

    // ── DocsCheck → score-change-spec ─────────────────────────────────

    #[test]
    fn test_route_docs_check() {
        let (tmp, cid) = setup_change(StatePhase::DocsCheck);
        let resp = call_route(&tmp, &cid);
        assert_eq!(resp["default_agent"], "score-change-spec");
        assert!(
            resp.get("model").is_none(),
            "model should not be in response — lives in agent definition"
        );
        let cli = resp["next_actions"][0]["cli"].as_str().unwrap();
        assert!(cli.contains("create-change-docs"));
    }

    // ── DocsReviewed → score-review:opus ────────────────────────────────

    #[test]
    fn test_route_docs_reviewed_uses_review_agent() {
        let (tmp, cid) = setup_change(StatePhase::DocsReviewed);
        let resp = call_route(&tmp, &cid);
        assert_eq!(resp["default_agent"], "score-review");
        assert!(
            resp.get("model").is_none(),
            "model should not be in response — lives in agent definition"
        );
    }

    // ── ChangeMergeCreated → create-change-merge ────────────────────────

    #[test]
    fn test_route_change_merge_created() {
        let (tmp, cid) = setup_change(StatePhase::ChangeMergeCreated);
        let resp = call_route(&tmp, &cid);
        let cli = resp["next_actions"][0]["cli"].as_str().unwrap();
        assert!(cli.contains("create-change-merge"));
    }

    // ── Terminal phases ─────────────────────────────────────────────────

    #[test]
    fn test_route_archived_is_terminal() {
        let (tmp, cid) = setup_change(StatePhase::ChangeArchived);
        let resp = call_route(&tmp, &cid);
        assert_eq!(resp["action"], "complete");
        assert!(resp["is_terminal"].as_bool().unwrap());
        assert!(resp["next_actions"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_route_rejected_is_terminal() {
        let (tmp, cid) = setup_change(StatePhase::ChangeRejected);
        let resp = call_route(&tmp, &cid);
        assert_eq!(resp["action"], "complete");
        assert!(resp["is_terminal"].as_bool().unwrap());
    }

    // ── Alignment warnings injected for spec/impl phases ────────────────

    #[test]
    fn test_route_spec_phase_has_alignment_warnings_field() {
        let (tmp, cid) = setup_change(StatePhase::ChangeSpecCreated);
        let resp = call_route(&tmp, &cid);
        // Field exists (may be null if no specs yet, but key must be present)
        assert!(resp.get("alignment_warnings").is_some());
    }

    #[test]
    fn test_route_non_spec_phase_has_null_alignment_warnings() {
        let (tmp, cid) = setup_change(StatePhase::ChangeMergeCreated);
        let resp = call_route(&tmp, &cid);
        assert!(resp["alignment_warnings"].is_null());
    }

    // ── execute() routes to init_change when no change dir ──────────────

    #[tokio::test]
    async fn test_execute_no_change_dir_no_description_no_issue() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".aw/changes")).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "new-change",
        });
        let result = execute(&args, tmp.path()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("--description or --issue is required"));
    }

    #[tokio::test]
    async fn test_execute_issue_only_synthesizes_description() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".aw/changes")).unwrap();
        // Create an issue file so resolve_issue_slug can find it
        let issues_dir = crate::shared::workspace::issues_path(tmp.path()).join("open");
        std::fs::create_dir_all(&issues_dir).unwrap();
        std::fs::write(
            issues_dir.join("my-test-issue.md"),
            "---\ntitle: Fix the widget\ntype: bug\nstate: open\n---\n\n## Problem\nBroken\n\n## Requirements\nFix it\n\n## Scope\nWidget",
        ).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "fix-widget",
            "issues": ["my-test-issue"],
        });
        let result = execute(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["action"], "init_change");
        // Description should be synthesized from issue title
        let cli = parsed["next_actions"][0]["cli"].as_str().unwrap();
        assert!(
            cli.contains("Fix the widget"),
            "cli should contain issue title: {}",
            cli
        );
        assert!(
            cli.contains("issue:my-test-issue"),
            "cli should contain issue slug: {}",
            cli
        );
    }

    #[tokio::test]
    async fn test_execute_no_change_dir_routes_to_init() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".aw/changes")).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "new-change",
            "description": "test",
        });
        let result = execute(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["action"], "init_change");
        let cli = parsed["next_actions"][0]["cli"].as_str().unwrap();
        assert!(cli.contains("init-change"));
    }
}

// CODEGEN-END
