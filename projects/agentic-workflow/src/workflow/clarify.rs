// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/workflow/clarify.md#source
// CODEGEN-BEGIN
//! Clarify flow: Clarify + clarification review cycles.
//!
//! Handles phases: None (new change), ChangeInited.

use super::{helpers, scope};
use crate::models::state::StatePhase;
use crate::state::StateManager;
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

/// Handle the clarify flow. Called when phase is None or Clarified.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/clarify.md#source
pub fn handle(
    change_dir: &Path,
    change_id: &str,
    description: Option<&str>,
    project_path: &str,
) -> Result<Value> {
    let has_clarifications = change_dir.join("context_clarifications.md").exists();

    let state_path = change_dir.join("STATE.yaml");
    let current_phase = if state_path.exists() {
        let sm = StateManager::load(change_dir)?;
        Some(sm.phase().clone())
    } else {
        None
    };

    let complexity = if state_path.exists() {
        helpers::read_complexity_from_state(change_dir)
    } else {
        None
    };

    // Determine action
    let action = match &current_phase {
        Some(StatePhase::ChangeInited) => Action::RouteToReferenceContext,
        None => {
            if description.is_none() && !change_dir.exists() {
                anyhow::bail!(
                    "description is required for new changes. Usage: sdd_run_change with change_id='{}' and description='...'",
                    change_id
                );
            }
            if !has_clarifications {
                Action::Clarify
            } else {
                Action::RouteToReferenceContext
            }
        }
        _ => {
            if !has_clarifications {
                Action::Clarify
            } else {
                Action::RouteToReferenceContext
            }
        }
    };

    let phase_str = current_phase.as_ref().map(helpers::phase_to_string);
    let mut base = json!({
        "change_id": change_id,
        "workflow_version": 2,
        "current_phase": phase_str,
        "has_clarifications": has_clarifications,
        "complexity": complexity,
    });

    match action {
        Action::Clarify => {
            base["action"] = json!("create_pre_clarifications");
            base["message"] = json!("Collect user clarifications using adaptive multi-round Q&A.");
            base["next_phase"] = json!("change_inited");
            // Build scope hints from issue labels for the clarify prompt
            let scope_hints = scope::extract_scope_hints_from_issues(change_dir);
            let scope_hint_text = if scope_hints.is_empty() {
                String::new()
            } else {
                format!("\n     (Issue labels suggest: {})", scope_hints.join(", "))
            };
            // Read description from user_input.md if not provided as param
            let effective_desc = description
                .map(|s| s.to_string())
                .or_else(|| std::fs::read_to_string(change_dir.join("user_input.md")).ok());
            base["prompt"] = json!(format!(
                "# Task: Clarify Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Analyze the description and project context for ambiguities\n\
                 1b. If description references issues (#NNN) or label patterns (\"all P1\", \"P1 issues\"):\n\
                    First fetch issues via `score artifact write-artifact`\n\
                    Use payload with issue_refs for specific issues, or labels for label-based listing.\n\
                    This creates issues/issue_*.md files and builds DAG in STATE.yaml.\n\
                 2. Use AskUserQuestion for targeted questions (no fixed count)\n\
                 3. After answers, evaluate: Are answers ambiguous? Did they raise new questions?\n\
                 4. If more clarification needed: ask follow-up questions\n\
                 5. MANDATORY: Ask about affected modules/scope:\n\
                    - Which crates or paths will this change affect?{scope_hint}\n\
                    - Options: specific crates (e.g. sdd), specific paths, unknown/unsure, whole project\n\
                    Record scope in clarifications.\n\
                 6. When sufficient: call `score artifact write-artifact` with artifact='context_clarifications', action='create' (auto-updates state to 'clarified')\n\
                 7. Call `score run-change` again for next action\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 score run-change --change-id {cid}\n\
                 ```",
                scope_hint = scope_hint_text,
                cid = change_id,
            ));
            if let Some(desc) = effective_desc.as_deref() {
                base["description"] = json!(desc);
                base["suggested_topics"] = json!(scope::suggest_topics(desc));
            }
            if !scope_hints.is_empty() {
                base["scope_hints"] = json!(scope_hints);
            }
        }
        // Route to unified reference context exploration
        Action::RouteToReferenceContext => {
            return super::reference_context::handle(change_dir, change_id, project_path);
        }
    }

    Ok(base)
}

/// Handle clarifications review cycle.
/// Called when phase is ChangeInited or PostClarificationsCreated.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/clarify.md#source
pub fn handle_clarifications_review(
    change_dir: &Path,
    change_id: &str,
    _project_path: &str,
) -> Result<Value> {
    let sm = StateManager::load(change_dir)?;
    let phase = sm.phase().clone();

    let action = match &phase {
        StatePhase::ChangeInited => ReviewAction::Review,
        _ => ReviewAction::Review,
    };

    let phase_str = helpers::phase_to_string(&phase);
    let mut base = json!({
        "change_id": change_id,
        "workflow_version": 2,
        "current_phase": phase_str,
    });

    match action {
        ReviewAction::Review => {
            base["action"] = json!("review_pre_clarifications");
            base["message"] = json!("Review context_clarifications.md against checklist.");
            base["stage"] = json!("clarifications");
            base["artifact"] = json!("context_clarifications.md");
            base["review_checklist"] = json!([
                "User's intent is clearly captured",
                "All ambiguities resolved with specific answers",
                "Git workflow decision recorded",
                "Affected modules/scope identified",
                "No contradictions between answers",
            ]);
            base["prompt"] = json!(format!(
                "# Task: Review Clarifications for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read context_clarifications.md via `score workflow read-artifact`\n\
                 2. Check against review checklist:\n\
                    - User's intent is clearly captured\n\
                    - All ambiguities resolved with specific answers\n\
                    - Git workflow decision recorded\n\
                    - Affected modules/scope identified\n\
                    - No contradictions between answers\n\
                 3. Call `score artifact write-artifact` with artifact='clarifications', verdict, summary, checklist_results, issues\n\
                 4. Verdict: APPROVED -> phase='clarifications_approved', REVIEWED -> phase='clarifications_reviewed', \
                    REJECTED -> phase='clarifications_rejected'\n\
                 5. Call `score run-change` to continue the workflow\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"clarifications\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 score run-change --change-id {cid}\n\
                 ```",
                cid = change_id
            ));
        }
        ReviewAction::Revise => {
            base["action"] = json!("revise_pre_clarifications");
            base["message"] = json!("Revise context_clarifications.md based on review feedback.");
            base["stage"] = json!("clarifications");
            base["artifact"] = json!("context_clarifications.md");
            base["prompt"] = json!(format!(
                "# Task: Revise Clarifications for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read context_clarifications.md and review_clarifications.md via `score workflow read-artifact`\n\
                 2. Address all issues from the review\n\
                 3. Rewrite via `score artifact write-artifact` with artifact='context_clarifications', action='revise' (auto-updates state)\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"clarifications\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"review_clarifications\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                cid = change_id
            ));
        }
    }

    Ok(base)
}

#[derive(Debug)]
enum ReviewAction {
    Review,
    Revise,
}

#[derive(Debug)]
enum Action {
    Clarify,
    RouteToReferenceContext,
}

/// Handle post-clarify flow: reference context approved → decide.
/// Stub: routes to decided phase (TODO: implement full post-clarify logic).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/clarify.md#source
pub fn handle_post_clarify(
    change_dir: &Path,
    change_id: &str,
    _project_path: &str,
) -> Result<Value> {
    let phase_str = helpers::phase_to_string(&StateManager::load(change_dir)?.phase().clone());
    Ok(json!({
        "change_id": change_id,
        "current_phase": phase_str,
        "action": "post_clarify",
        "executor": ["mainthread"],
        "message": "Reference context approved. Proceed to post-clarifications or decide.",
        "prompt": format!(
            "# Post-Clarify for Change '{}'\n\n\
             Reference context is approved. Create post-clarifications or proceed to decided.\n\n\
             ## CLI Commands\n\n\
             ```\n\
             score run-change --change-id {}\n\
             ```",
            change_id, change_id
        ),
        "next": [],
    }))
}

/// Handle post-clarifications review cycle.
/// Stub: routes review/revise for post-clarifications.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/clarify.md#source
pub fn handle_post_clarifications_review(
    change_dir: &Path,
    change_id: &str,
    _project_path: &str,
) -> Result<Value> {
    let sm = StateManager::load(change_dir)?;
    let phase = sm.phase().clone();
    let phase_str = helpers::phase_to_string(&phase);

    Ok(json!({
        "change_id": change_id,
        "current_phase": phase_str,
        "action": "review_post_clarifications",
        "executor": ["mainthread"],
        "message": "Review or revise post-clarifications.",
        "prompt": format!(
            "# Post-Clarifications Review for Change '{}'\n\n\
             Review the post-clarifications and approve or request revisions.\n\n\
             ## CLI Commands\n\n\
             ```\n\
             score run-change --change-id {}\n\
             ```",
            change_id, change_id
        ),
        "next": [],
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_change_requires_description() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().join("new-change");
        let result = handle(&change_dir, "new-change", None, "/tmp");
        assert!(result.is_err());
    }

    #[test]
    fn test_new_change_returns_clarify() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().join("new-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        let result = handle(&change_dir, "new-change", Some("Add feature"), "/tmp").unwrap();
        assert_eq!(result["action"], "create_pre_clarifications");
        assert!(result["prompt"]
            .as_str()
            .unwrap()
            .contains("Clarify Change"));
        // Git workflow should NOT appear in clarify prompt (moved to skill entry point)
        let prompt = result["prompt"].as_str().unwrap();
        assert!(!prompt.contains("git workflow"));
    }
}

// CODEGEN-END
