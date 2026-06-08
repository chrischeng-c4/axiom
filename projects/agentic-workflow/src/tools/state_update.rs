// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/state_update.md#source
// CODEGEN-BEGIN
//! sdd_update_state MCP Tool
//!
//! Safely updates STATE.yaml phase with transition validation.
//! Prevents invalid state transitions and ensures consistent state management.

use super::{get_optional_string, get_required_string, ToolDefinition};
use crate::models::state::StatePhase;
use crate::state::StateManager;
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

/// Get the tool definition for sdd_update_state
/// @spec projects/agentic-workflow/tech-design/core/tools/state_update.md#source
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_update_state".to_string(),
        description: "Update STATE.yaml phase with transition validation. Use this instead of manually editing STATE.yaml.".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "phase"],
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
                "phase": {
                    "type": "string",
                    "enum": [
                        "change_inited", "input_restructured",
                        "pre_clarifications_created",
                        "post_clarifications_created",
                        "change_spec_created", "change_spec_reviewed", "change_spec_revised",
                        "change_implementation_created", "change_implementation_reviewed", "change_implementation_revised",
                        "change_merge_created", "change_merge_reviewed", "change_merge_revised",
                        "change_archived", "change_rejected"
                    ],
                    "description": "Target phase to transition to"
                },
                "last_action": {
                    "type": "string",
                    "description": "Optional action description (e.g., 'begin_implementation', 'review_approved')"
                },
                "git_workflow": {
                    "type": "string",
                    "enum": ["new_branch", "in_place", "worktree"],
                    "description": "Git workflow to use for this change"
                },
                "current_task_id": {
                    "type": "string",
                    "description": "Current task being implemented (for per-task workflow)"
                },
                "increment_task_revision": {
                    "type": "string",
                    "description": "Task ID to increment revision count for (per-task workflow)"
                }
            }
        }),
    }
}

/// Execute the sdd_update_state tool
/// @spec projects/agentic-workflow/tech-design/core/tools/state_update.md#source
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let phase_str = get_required_string(args, "phase")?;
    let last_action = get_optional_string(args, "last_action");
    let git_workflow = get_optional_string(args, "git_workflow");
    let current_task_id = get_optional_string(args, "current_task_id");
    let increment_task_revision = get_optional_string(args, "increment_task_revision");

    // Validate change_id format (security: prevent directory traversal)
    if change_id.is_empty() {
        anyhow::bail!("Invalid change_id: cannot be empty");
    }
    if !change_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!("Invalid change_id: must be lowercase alphanumeric with hyphens only");
    }

    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);

    if !change_dir.exists() {
        anyhow::bail!("Change directory not found: {}", change_dir.display());
    }

    // Security: Verify change_dir is not a symlink pointing outside the project
    let canonical_change_dir = change_dir
        .canonicalize()
        .map_err(|e| anyhow::anyhow!("Failed to resolve change directory: {}", e))?;
    let canonical_changes_parent = project_root
        .join(".aw/changes")
        .canonicalize()
        .map_err(|e| anyhow::anyhow!("Failed to resolve changes directory: {}", e))?;
    if !canonical_change_dir.starts_with(&canonical_changes_parent) {
        anyhow::bail!(
            "Security error: change directory escapes project boundary (possible symlink attack)"
        );
    }

    // Parse target phase
    let target_phase = parse_phase(&phase_str)?;

    // Load current state
    let mut state_manager = StateManager::load(&change_dir)?;
    let current_phase = state_manager.phase().clone();

    // Validate transition
    validate_transition(&current_phase, &target_phase)?;

    // Update phase
    state_manager.set_phase(target_phase.clone())?;

    // Auto-increment revision counter when transitioning to a Revised phase
    match &target_phase {
        StatePhase::ChangeSpecRevised => {
            // Use last_action as spec_id hint, or generic "spec"
            let key = if let Some(action) = &last_action {
                if action.starts_with("spec:") {
                    format!("spec:{}", action.trim_start_matches("spec:"))
                } else {
                    "spec".to_string()
                }
            } else {
                "spec".to_string()
            };
            state_manager.increment_revision_count(&key);
        }
        StatePhase::ChangeImplementationRevised => {
            state_manager.increment_revision_count("implementation");
        }
        StatePhase::ChangeMergeRevised => {
            state_manager.increment_revision_count("merge");
        }
        _ => {}
    }

    // Update last_action if provided
    if let Some(action) = &last_action {
        state_manager.set_last_action(action);

        // Handle DAG index advancement
        match action.as_str() {
            "dag_clarify_next" | "dag_context_next" => {
                if let Some(dag) = &mut state_manager.state_mut().dag {
                    if dag.current_index < dag.issues.len() {
                        dag.current_index += 1;
                    }
                }
            }
            _ => {}
        }
    }

    // Update git_workflow if provided
    if let Some(workflow) = &git_workflow {
        state_manager.set_git_workflow(workflow.clone());
    }

    // Update current_task_id if provided
    if let Some(task_id) = &current_task_id {
        state_manager.state_mut().current_task_id = Some(task_id.clone());
    }

    // Increment task revision if requested
    if let Some(task_id) = &increment_task_revision {
        let count = state_manager
            .state_mut()
            .task_revisions
            .entry(task_id.clone())
            .or_insert(0);
        *count += 1;
    }

    // Save
    state_manager.save()?;

    let response = json!({
        "success": true,
        "change_id": change_id,
        "previous_phase": phase_to_string(&current_phase),
        "current_phase": phase_to_string(&target_phase),
        "last_action": last_action,
        "message": format!("State updated: {} → {}", phase_to_string(&current_phase), phase_to_string(&target_phase))
    });

    Ok(serde_json::to_string_pretty(&response)?)
}

// Re-export from phase_transition module
use super::phase_transition::{parse_phase, phase_to_string, validate_transition};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_change_dir(phase: StatePhase) -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        let phase_str = phase_to_string(&phase);
        let state_content = format!(
            "change_id: test-change\nphase: {}\niteration: 1\n",
            phase_str
        );
        std::fs::write(change_dir.join("STATE.yaml"), state_content).unwrap();
        crate::test_util::write_minimal_issue(temp_dir.path(), "test-change");

        (temp_dir, change_dir)
    }

    #[test]
    fn test_update_spec_reviewed_to_implementing() {
        let (temp_dir, _) = setup_change_dir(StatePhase::ChangeSpecReviewed);
        let project_root = temp_dir.path();

        let args = json!({
            "change_id": "test-change",
            "phase": "change_implementation_created",
            "last_action": "begin_implementation"
        });

        let result = execute(&args, project_root).unwrap();
        let response: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(response["success"], true);
        assert_eq!(response["previous_phase"], "change_spec_reviewed");
        assert_eq!(response["current_phase"], "change_implementation_created");
    }

    #[test]
    fn test_update_impl_created_to_merge_created() {
        let (temp_dir, _) = setup_change_dir(StatePhase::ChangeImplementationCreated);
        let project_root = temp_dir.path();

        let args = json!({
            "change_id": "test-change",
            "phase": "change_merge_created",
            "last_action": "review_approved"
        });

        let result = execute(&args, project_root).unwrap();
        let response: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(response["success"], true);
        assert_eq!(response["current_phase"], "change_merge_created");
    }

    #[test]
    fn test_invalid_transition_spec_reviewed_to_archived() {
        let (temp_dir, _) = setup_change_dir(StatePhase::ChangeSpecReviewed);
        let project_root = temp_dir.path();

        let args = json!({
            "change_id": "test-change",
            "phase": "change_archived"
        });

        let result = execute(&args, project_root);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid state transition"));
    }

    #[test]
    fn test_cannot_transition_from_archived() {
        let (temp_dir, _) = setup_change_dir(StatePhase::ChangeArchived);
        let project_root = temp_dir.path();

        let args = json!({
            "change_id": "test-change",
            "phase": "change_implementation_created"
        });

        let result = execute(&args, project_root);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("terminal state"));
    }

    #[test]
    fn test_idempotent_same_phase() {
        let (temp_dir, _) = setup_change_dir(StatePhase::ChangeImplementationCreated);
        let project_root = temp_dir.path();

        let args = json!({
            "change_id": "test-change",
            "phase": "change_implementation_created"
        });

        let result = execute(&args, project_root).unwrap();
        let response: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(response["success"], true);
        assert_eq!(response["previous_phase"], "change_implementation_created");
        assert_eq!(response["current_phase"], "change_implementation_created");
    }

    #[test]
    fn test_empty_change_id() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        std::fs::create_dir_all(project_root.join("cclab")).unwrap();

        let args = json!({
            "change_id": "",
            "phase": "change_implementation_created"
        });

        let result = execute(&args, project_root);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_invalid_change_id() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        std::fs::create_dir_all(project_root.join("cclab")).unwrap();

        let args = json!({
            "change_id": "../etc/passwd",
            "phase": "change_implementation_created"
        });

        let result = execute(&args, project_root);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid change_id"));
    }

    #[test]
    fn test_update_with_current_task_id() {
        let (temp_dir, change_dir) = setup_change_dir(StatePhase::ChangeImplementationCreated);
        let project_root = temp_dir.path();

        let args = json!({
            "change_id": "test-change",
            "phase": "change_implementation_created",
            "current_task_id": "2.1"
        });

        let result = execute(&args, project_root).unwrap();
        let response: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(response["success"], true);

        // Verify STATE.yaml has current_task_id
        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(sm.phase(), &StatePhase::ChangeImplementationCreated);
        assert_eq!(sm.state().current_task_id, Some("2.1".to_string()));
    }

    #[test]
    fn test_update_increment_task_revision() {
        let (temp_dir, change_dir) = setup_change_dir(StatePhase::ChangeImplementationRevised);
        let project_root = temp_dir.path();

        // First increment
        let args = json!({
            "change_id": "test-change",
            "phase": "change_implementation_revised",
            "increment_task_revision": "2.1"
        });
        execute(&args, project_root).unwrap();

        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(*sm.state().task_revisions.get("2.1").unwrap(), 1);

        // Second increment
        let args = json!({
            "change_id": "test-change",
            "phase": "change_implementation_revised",
            "increment_task_revision": "2.1"
        });
        execute(&args, project_root).unwrap();

        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(*sm.state().task_revisions.get("2.1").unwrap(), 2);
    }

    #[test]
    fn test_legacy_state_without_task_fields() {
        // S5: Legacy STATE.yaml without current_task_id or task_revisions
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        // Write a legacy STATE.yaml without the new fields
        // (old "implementing" deserializes to ChangeImplementationCreated via legacy alias)
        let legacy_state = "change_id: test-change\nphase: implementing\niteration: 1\n";
        std::fs::write(change_dir.join("STATE.yaml"), legacy_state).unwrap();

        // Should load successfully with defaults
        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(sm.phase(), &StatePhase::ChangeImplementationCreated);
        assert_eq!(sm.state().current_task_id, None);
        assert!(sm.state().task_revisions.is_empty());

        // Should be able to update with new fields
        let project_root = temp_dir.path();
        let args = json!({
            "change_id": "test-change",
            "phase": "change_implementation_created",
            "current_task_id": "3.1",
            "increment_task_revision": "3.1"
        });
        let result = execute(&args, project_root).unwrap();
        let response: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(response["success"], true);

        // Verify saved correctly
        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(sm.state().current_task_id, Some("3.1".to_string()));
        assert_eq!(*sm.state().task_revisions.get("3.1").unwrap(), 1);
    }

    #[test]
    #[cfg(unix)]
    fn test_symlink_traversal_blocked() {
        use std::os::unix::fs::symlink;

        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        let changes_dir = project_root.join(".aw/changes");
        std::fs::create_dir_all(&changes_dir).unwrap();

        // Create a target directory outside the project
        let outside_dir = TempDir::new().unwrap();
        let outside_path = outside_dir.path();
        std::fs::write(
            outside_path.join("STATE.yaml"),
            "change_id: evil\nphase: implementing\niteration: 1\n",
        )
        .unwrap();

        // Create symlink inside changes/ pointing outside
        symlink(outside_path, changes_dir.join("evil-link")).unwrap();

        let args = json!({
            "change_id": "evil-link",
            "phase": "testing"
        });
        let result = execute(&args, project_root);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Security error") || err_msg.contains("escapes project boundary"),
            "Expected security error, got: {}",
            err_msg
        );
    }
}
// CODEGEN-END
