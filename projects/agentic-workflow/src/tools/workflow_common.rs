// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/workflow_common/validation-and-paths.md#source
// CODEGEN-BEGIN
//! Shared workflow helpers
//!
//! Common functions used by workflow state machine modules
//! (decide_change, plan_change, impl_change, merge_change, run_change).

use crate::models::change::SddInterface;
use crate::models::state::StatePhase;
use crate::models::{SddConfig, WorkflowArtifact};
use crate::state::StateManager;
use crate::Result;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// Validate change_id format (security: prevent directory traversal)
/// @spec projects/agentic-workflow/tech-design/core/tools/workflow_common/validation-and-paths.md#source
pub fn validate_change_id(change_id: &str) -> Result<()> {
    if change_id.is_empty() {
        anyhow::bail!("Invalid change_id: cannot be empty");
    }
    if !change_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!("Invalid change_id: must be lowercase alphanumeric with hyphens only");
    }
    Ok(())
}

/// Validate change directory exists and is not a symlink escape.
///
/// Accepts both legacy and worktree-first layouts:
/// - Legacy:    `project_root/.aw/changes/<id>/`
/// - Worktree:  `project_root/.aw/worktrees/<slug>/.aw/changes/<id>/`
///
/// REQ: change-merge R9 — worktree-first path resolution.
/// @spec projects/agentic-workflow/tech-design/core/tools/workflow_common/validation-and-paths.md#source
pub fn validate_change_dir(change_dir: &Path, project_root: &Path) -> Result<()> {
    if !change_dir.exists() {
        anyhow::bail!("Change directory not found: {}", change_dir.display());
    }
    let canonical_change_dir = change_dir
        .canonicalize()
        .map_err(|e| anyhow::anyhow!("Failed to resolve change directory: {}", e))?;

    // Legacy parent: project_root/.aw/changes/
    let legacy_parent = project_root.join(".aw/changes");
    if let Ok(canonical) = legacy_parent.canonicalize() {
        if canonical_change_dir.starts_with(&canonical) {
            return Ok(());
        }
    }

    // Worktree parent: project_root/.aw/worktrees/ (any worktree's changes/
    // directory is nested two levels deeper and resolves to a real path under
    // the worktrees root, so `starts_with` on the worktrees root is sufficient
    // as a security containment check).
    let worktrees_parent = project_root.join(".aw/worktrees");
    if let Ok(canonical) = worktrees_parent.canonicalize() {
        if canonical_change_dir.starts_with(&canonical) {
            return Ok(());
        }
    }

    anyhow::bail!(
        "Security error: change directory escapes project boundary (possible symlink attack)"
    );
}

/// Resolve the active change_id on the current branch.
///
/// Scans `.aw/changes/*/STATE.yaml` for changes whose phase is non-terminal
/// (not `archived` or `rejected`). Returns the change_id if exactly one is found.
/// @spec projects/agentic-workflow/tech-design/core/tools/workflow_common/validation-and-paths.md#source
pub fn resolve_active_change_id(project_root: &Path) -> Result<String> {
    let changes_dir = project_root.join(".aw/changes");
    if !changes_dir.exists() {
        anyhow::bail!("No .aw/changes/ directory found. Start a change with `score run-change`.");
    }

    let mut active = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&changes_dir) {
        for entry in entries.flatten() {
            let state_path = entry.path().join("STATE.yaml");
            if !state_path.exists() {
                continue;
            }
            if let Ok(sm) = StateManager::load(&entry.path()) {
                let phase = sm.phase();
                if !matches!(
                    phase,
                    StatePhase::ChangeArchived | StatePhase::ChangeRejected
                ) {
                    active.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
    }

    match active.len() {
        0 => anyhow::bail!("No active change on this branch. Start one with `score run-change`."),
        1 => Ok(active.into_iter().next().unwrap()),
        n => anyhow::bail!(
            "Multiple active changes found ({}): {}. Pass --change-id explicitly.",
            n,
            active.join(", ")
        ),
    }
}

/// Resolve the change directory for a given change_id.
///
/// Checks two locations (in order):
/// 1. Worktree: `project_root/.aw/worktrees/{id}/.aw/changes/{id}/`
/// 2. Legacy (main): `project_root/.aw/changes/{id}/`
///
/// Returns the first path that exists, or the worktree path as default for new changes.
/// @spec projects/agentic-workflow/tech-design/core/tools/workflow_common/validation-and-paths.md#source
pub fn resolve_change_dir(project_root: &Path, change_id: &str) -> PathBuf {
    // Worktree path (preferred)
    let wt_path = project_root
        .join(".aw/worktrees")
        .join(change_id)
        .join(".aw/changes")
        .join(change_id);
    if wt_path.exists() {
        return wt_path;
    }

    // Legacy path (main branch)
    let legacy_path = project_root.join(".aw/changes").join(change_id);
    if legacy_path.exists() {
        return legacy_path;
    }

    // Default to legacy path (compatible with tests and non-worktree envs).
    // init_change explicitly uses worktree root when creating new changes.
    legacy_path
}

/// Convert StatePhase to string for JSON output
/// @spec projects/agentic-workflow/tech-design/core/tools/workflow_common/validation-and-paths.md#source
pub fn phase_to_string(phase: &StatePhase) -> &'static str {
    super::phase_transition::phase_to_string(phase)
}
// CODEGEN-END
// ─── Git status helpers (pre-flight gates) ─────────────────────────────────
// REQ: issue-centric-workflow#R7/R8/R11 — git cleanliness gates for init_change.

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/workflow_common/git-status.md#source
// CODEGEN-BEGIN
/// Probe for a git binary on PATH. `None` when git is unavailable.
fn find_git_bin() -> Option<PathBuf> {
    let output = std::process::Command::new("which")
        .arg("git")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        None
    } else {
        Some(PathBuf::from(path))
    }
}

/// Returns `true` when `project_root` is inside a git workspace.
/// Used to skip pre-flight gates gracefully in tempdir tests.
// REQ: issue-centric-workflow#R11
/// @spec projects/agentic-workflow/tech-design/core/tools/workflow_common/git-status.md#source
pub fn is_git_project(project_root: &Path) -> bool {
    let Some(git) = find_git_bin() else {
        return false;
    };
    std::process::Command::new(&git)
        .args(["rev-parse", "--git-dir"])
        .current_dir(project_root)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Returns `true` if `rel_path` (project-relative) is tracked in git's index.
/// Returns `false` if the file is untracked, ignored, or absent.
/// Returns `Ok(true)` with a warning-log skip when git isn't available — this
/// preserves test ergonomics in tempdirs.
// REQ: issue-centric-workflow#R7
/// @spec projects/agentic-workflow/tech-design/core/tools/workflow_common/git-status.md#source
pub fn is_git_tracked(project_root: &Path, rel_path: &str) -> Result<bool> {
    if !is_git_project(project_root) {
        return Ok(true); // Skip gate in non-git contexts.
    }
    let git = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let out = std::process::Command::new(&git)
        .args(["ls-files", "--error-unmatch", rel_path])
        .current_dir(project_root)
        .output()
        .map_err(|e| anyhow::anyhow!("git ls-files failed to run: {}", e))?;
    Ok(out.status.success())
}

/// Returns `true` when `rel_path` has staged or unstaged diffs against `HEAD`.
/// Returns `Ok(false)` in non-git contexts (gate skipped).
// REQ: issue-centric-workflow#R8
/// @spec projects/agentic-workflow/tech-design/core/tools/workflow_common/git-status.md#source
pub fn has_uncommitted_diff(project_root: &Path, rel_path: &str) -> Result<bool> {
    if !is_git_project(project_root) {
        return Ok(false);
    }
    let git = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let out = std::process::Command::new(&git)
        .args(["diff", "--quiet", "HEAD", "--", rel_path])
        .current_dir(project_root)
        .output()
        .map_err(|e| anyhow::anyhow!("git diff failed to run: {}", e))?;
    // `git diff --quiet` exits 0 when identical, 1 when different.
    Ok(!out.status.success())
}
// CODEGEN-END
/// Returns `true` when `project_root` is inside a git workspace.
/// Used to skip pre-flight gates gracefully in tempdir tests.
// REQ: issue-centric-workflow#R11
/// Returns `true` if `rel_path` (project-relative) is tracked in git's index.
/// Returns `false` if the file is untracked, ignored, or absent.
/// Returns `Ok(true)` with a warning-log skip when git isn't available — this
/// preserves test ergonomics in tempdirs.
// REQ: issue-centric-workflow#R7
/// Returns `true` when `rel_path` has staged or unstaged diffs against `HEAD`.
/// Returns `Ok(false)` in non-git contexts (gate skipped).
// REQ: issue-centric-workflow#R8
/// Update STATE.yaml phase after successful agent execution.
///
/// StateManager::save() now dual-writes all workflow fields (phase, branch,
/// iteration, task tracking) to issue frontmatter automatically.
// REQ: R1 — Issue frontmatter absorbs STATE.yaml
// REQ: R6 — StateManager reads/writes issue frontmatter

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/workflow_common/phase-and-executor.md#source
// CODEGEN-BEGIN
/// Update STATE.yaml phase after successful agent execution.
///
/// StateManager::save() now dual-writes all workflow fields (phase, branch,
/// iteration, task tracking) to issue frontmatter automatically.
// REQ: R1 — Issue frontmatter absorbs STATE.yaml
// REQ: R6 — StateManager reads/writes issue frontmatter
/// @spec projects/agentic-workflow/tech-design/core/tools/workflow_common/phase-and-executor.md#source
pub fn update_phase(change_dir: &Path, phase: StatePhase) -> Result<()> {
    let mut state_manager = StateManager::load(change_dir)?;
    state_manager.set_phase(phase)?;
    state_manager.save()?;
    Ok(())
}

/// Get the executor chain for a WorkflowArtifact.
///
/// Score uses a **fixed hybrid mapping**: heavy tasks (reference context, spec,
/// implementation, docs) run as Claude Code subagents — context-isolated,
/// parallelizable. Lightweight tasks (restructure, clarifications, merge) run
/// in the mainthread session because subagent startup overhead isn't worth it.
///
/// The executor string is `subagent:<agent_type>` — model is NOT specified here.
/// Model selection lives in the agent definition (`.claude/agents/<type>.md`
/// frontmatter `model:` field). This separation means model changes don't
/// require code changes.
///
/// This is **not configurable**. There is no `workflow.mode` knob, no alternate
/// preset. The host agent (Claude Code) is the only supported runner — score
/// does not spawn subprocesses (no `claude-agent:*`, `gemini:*`, `codex:*`).
/// @spec projects/agentic-workflow/tech-design/core/tools/workflow_common/phase-and-executor.md#source
pub fn get_executor_chain(_project_root: &Path, artifact: WorkflowArtifact) -> Vec<String> {
    let executor = match artifact {
        // Lightweight phases — stay in mainthread (subagent overhead > benefit)
        WorkflowArtifact::RestructureInput
        | WorkflowArtifact::CreatePreClarifications
        | WorkflowArtifact::CreatePostClarifications
        | WorkflowArtifact::ReviseReferenceContext
        | WorkflowArtifact::CreateChangeMerge => "mainthread",

        // Reference context building — Explore agent (purpose-built for code reading)
        WorkflowArtifact::CreateReferenceContext => "subagent:Explore",

        // Review phases — quality gate
        WorkflowArtifact::ReviewReferenceContext
        | WorkflowArtifact::ReviewChangeSpec
        | WorkflowArtifact::ReviewChangeImplementation
        | WorkflowArtifact::ReviewChangeDocs => "subagent:score-review",

        // Spec creation / revision
        WorkflowArtifact::CreateChangeSpec
        | WorkflowArtifact::ReviseChangeSpec
        | WorkflowArtifact::CreateChangeDocs
        | WorkflowArtifact::ReviseChangeDocs => "subagent:score-change-spec",

        // Implementation creation / revision
        WorkflowArtifact::CreateChangeImplementation
        | WorkflowArtifact::ReviseChangeImplementation => "subagent:score-change-implementation",
    };
    vec![executor.to_string()]
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/workflow_common/interface-and-groups.md#source
// CODEGEN-BEGIN
/// Load the SddInterface from config (defaults to Cli).
/// @spec projects/agentic-workflow/tech-design/core/tools/workflow_common/interface-and-groups.md#source
pub fn load_interface(project_root: &Path) -> SddInterface {
    SddConfig::load_validated(project_root)
        .map(|c| c.interface)
        .unwrap_or_default()
}

/// Build a `next_actions` entry with ONLY the relevant interface field.
///
/// Delegates to `crate::workflow::helpers::next_action`.
/// @spec projects/agentic-workflow/tech-design/core/tools/workflow_common/interface-and-groups.md#source
pub fn next_action(interface: SddInterface, tool: &str, args: Value) -> Value {
    crate::workflow::helpers::next_action(interface, tool, args)
}

/// List group IDs from groups/ subdirectories (sorted).
/// @spec projects/agentic-workflow/tech-design/core/tools/workflow_common/interface-and-groups.md#source
pub fn list_group_ids(groups_dir: &Path) -> Result<Vec<String>> {
    let mut ids = Vec::new();
    if !groups_dir.exists() {
        return Ok(ids);
    }
    for entry in std::fs::read_dir(groups_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                ids.push(name.to_string());
            }
        }
    }
    ids.sort();
    Ok(ids)
}

/// Determine the group_id for a change directory (simple heuristic).
///
/// If exactly one group exists, returns it. Otherwise returns `None`.
/// @spec projects/agentic-workflow/tech-design/core/tools/workflow_common/interface-and-groups.md#source
pub fn resolve_single_group_id(change_dir: &Path) -> Option<String> {
    let groups_dir = change_dir.join("groups");
    match list_group_ids(&groups_dir) {
        Ok(ids) if ids.len() == 1 => Some(ids.into_iter().next().unwrap()),
        _ => None,
    }
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/workflow_common/prompt-response.md#source
// CODEGEN-BEGIN
/// Write a prompt to a file in the change's `prompts/` directory.
///
/// When `group_id` is `Some`, writes to `groups/{gid}/prompts/` instead of
/// the change-level `prompts/` directory.
///
/// Returns the path to the written file.
/// @spec projects/agentic-workflow/tech-design/core/tools/workflow_common/prompt-response.md#source
pub fn write_prompt_file(
    change_dir: &Path,
    group_id: Option<&str>,
    action: &str,
    prompt: &str,
) -> Result<PathBuf> {
    let prompts_dir = match group_id {
        Some(gid) => change_dir.join("groups").join(gid).join("prompts"),
        None => change_dir.join("prompts"),
    };
    std::fs::create_dir_all(&prompts_dir)?;
    let path = prompts_dir.join(format!("{}.md", action));
    std::fs::write(&path, prompt)?;
    Ok(path)
}

/// Build a workflow response. Always writes prompt to file for clean tool responses.
///
/// Score is **not a runner** — it's a state machine that writes prompts and
/// returns metadata. The host agent (Claude Code mainthread or subagent)
/// handles actual LLM execution:
///
/// - `mainthread` executor → host agent reads the prompt file and executes inline
/// - `subagent:{type}` → host agent invokes Agent tool (model from agent definition)
///
/// No subprocess dispatch, no runtime agent selection. The executor was
/// decided by `get_executor_chain()` per-artifact (see that function for the
/// hardcoded hybrid mapping).
///
/// `extra_fields` are merged into the top-level response (e.g., `spec_id`, `group_id`).
/// @spec projects/agentic-workflow/tech-design/core/tools/workflow_common/prompt-response.md#source
pub async fn build_workflow_response(
    change_dir: &Path,
    change_id: &str,
    action: &str,
    prompt: String,
    executor: Vec<String>,
    extra_fields: Value,
    _interface: SddInterface,
    _project_root: &Path,
) -> Result<String> {
    // Extract group_id from extra_fields for group-scoped prompt path
    let group_id = extra_fields.get("group_id").and_then(|v| v.as_str());

    // Always write prompt to file
    write_prompt_file(change_dir, group_id, action, &prompt)?;
    let rel_path = match group_id {
        Some(gid) => format!(
            ".aw/changes/{}/groups/{}/prompts/{}.md",
            change_id, gid, action
        ),
        None => format!(".aw/changes/{}/prompts/{}.md", change_id, action),
    };

    // All executors (mainthread + subagent:*) return prompt_path + executor.
    // The host agent decides how to run it. This is the only dispatch path.
    let mut result = json!({
        "status": "ok",
        "prompt_path": rel_path,
        "executor": executor,
        "next_actions": []
    });

    // Merge extra fields into top-level
    if let Some(obj) = extra_fields.as_object() {
        for (k, v) in obj {
            result[k] = v.clone();
        }
    }

    Ok(serde_json::to_string_pretty(&result)?)
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/workflow_common/group-issues-hint.md#source
// CODEGEN-BEGIN
/// Build a hint string listing issues belonging to a group (from frontmatter).
/// @spec projects/agentic-workflow/tech-design/core/tools/workflow_common/group-issues-hint.md#source
pub fn build_group_issues_hint(change_dir: &Path, group_id: &str) -> String {
    let issues_dir = change_dir.join("issues");
    if !issues_dir.exists() {
        return String::new();
    }

    let mut hints = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&issues_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("issue_") && name.ends_with(".md") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if content.contains(&format!("group: \"{}\"", group_id)) {
                        if let Some(num_str) = name
                            .strip_prefix("issue_")
                            .and_then(|s| s.strip_suffix(".md"))
                        {
                            hints.push(format!("#{}", num_str));
                        }
                    }
                }
            }
        }
    }
    hints.join(", ")
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/workflow_common/tests.md#source
// CODEGEN-BEGIN
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    /// Create a minimal change directory structure for testing build_workflow_response.
    fn setup_change_dir(change_id: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes").join(change_id);
        std::fs::create_dir_all(change_dir.join("prompts")).unwrap();
        crate::test_util::write_minimal_issue(tmp.path(), change_id);
        tmp
    }

    // ======================================================================
    // Subagent executor routing tests (sdd-subagent-mode)
    // ======================================================================

    #[tokio::test]
    async fn test_build_workflow_response_subagent_returns_prompt_path_and_executor() {
        // subagent:* executors should be returned to caller (like mainthread),
        // NOT dispatched via run_agent()
        let tmp = setup_change_dir("test-sub");
        let change_dir = tmp.path().join(".aw/changes/test-sub");

        let result = build_workflow_response(
            &change_dir,
            "test-sub",
            "create_reference_context",
            "Test prompt".to_string(),
            vec!["subagent:Explore".to_string()],
            json!({}),
            SddInterface::default(),
            tmp.path(),
        )
        .await
        .unwrap();

        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert!(
            parsed["prompt_path"]
                .as_str()
                .unwrap()
                .contains("create_reference_context"),
            "prompt_path should contain action name"
        );
        assert_eq!(
            parsed["executor"][0], "subagent:Explore",
            "executor should be passed through as-is"
        );
        // next_actions should be empty (no run_agent dispatch)
        assert_eq!(
            parsed["next_actions"].as_array().unwrap().len(),
            0,
            "subagent executors must have empty next_actions"
        );
    }

    #[tokio::test]
    async fn test_build_workflow_response_subagent_writes_prompt_file() {
        // Prompt must be written to file even for subagent executors
        let tmp = setup_change_dir("test-sub-prompt");
        let change_dir = tmp.path().join(".aw/changes/test-sub-prompt");

        let prompt_content = "Explore specs for auth module";
        build_workflow_response(
            &change_dir,
            "test-sub-prompt",
            "create_reference_context",
            prompt_content.to_string(),
            vec!["subagent:Explore".to_string()],
            json!({}),
            SddInterface::default(),
            tmp.path(),
        )
        .await
        .unwrap();

        let prompt_file = change_dir.join("prompts/create_reference_context.md");
        assert!(prompt_file.exists(), "prompt file must be written");
        let written = std::fs::read_to_string(&prompt_file).unwrap();
        assert_eq!(written, prompt_content);
    }

    #[tokio::test]
    async fn test_build_workflow_response_subagent_with_group_id() {
        // When group_id is present, prompt goes to groups/{gid}/prompts/
        let tmp = setup_change_dir("test-sub-group");
        let change_dir = tmp.path().join(".aw/changes/test-sub-group");

        let result = build_workflow_response(
            &change_dir,
            "test-sub-group",
            "create_change_spec",
            "Write spec".to_string(),
            vec!["subagent:score-change-spec".to_string()],
            json!({"group_id": "auth-module"}),
            SddInterface::default(),
            tmp.path(),
        )
        .await
        .unwrap();

        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert!(
            parsed["prompt_path"]
                .as_str()
                .unwrap()
                .contains("groups/auth-module/prompts/"),
            "prompt_path should include group_id"
        );
        assert_eq!(parsed["executor"][0], "subagent:score-change-spec");

        // Verify group_id is merged into response
        assert_eq!(parsed["group_id"], "auth-module");

        // Verify prompt file written to group dir
        let prompt_file = change_dir.join("groups/auth-module/prompts/create_change_spec.md");
        assert!(prompt_file.exists(), "prompt file must be in group dir");
    }

    #[tokio::test]
    async fn test_build_workflow_response_mainthread_still_works() {
        // Mainthread executor should still work (regression test)
        let tmp = setup_change_dir("test-mt");
        let change_dir = tmp.path().join(".aw/changes/test-mt");

        let result = build_workflow_response(
            &change_dir,
            "test-mt",
            "restructure_input",
            "Restructure".to_string(),
            vec!["mainthread".to_string()],
            json!({}),
            SddInterface::default(),
            tmp.path(),
        )
        .await
        .unwrap();

        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["executor"][0], "mainthread");
        assert_eq!(
            parsed["next_actions"].as_array().unwrap().len(),
            0,
            "mainthread must have empty next_actions"
        );
    }

    #[tokio::test]
    async fn test_build_workflow_response_subagent_vs_mainthread_same_shape() {
        // Both subagent and mainthread produce the same response shape
        let tmp = setup_change_dir("test-shape");
        let change_dir = tmp.path().join(".aw/changes/test-shape");

        let subagent_result = build_workflow_response(
            &change_dir,
            "test-shape",
            "create_change_spec",
            "prompt1".to_string(),
            vec!["subagent:score-change-spec".to_string()],
            json!({}),
            SddInterface::default(),
            tmp.path(),
        )
        .await
        .unwrap();

        let mt_result = build_workflow_response(
            &change_dir,
            "test-shape",
            "restructure_input",
            "prompt2".to_string(),
            vec!["mainthread".to_string()],
            json!({}),
            SddInterface::default(),
            tmp.path(),
        )
        .await
        .unwrap();

        let sub_parsed: Value = serde_json::from_str(&subagent_result).unwrap();
        let mt_parsed: Value = serde_json::from_str(&mt_result).unwrap();

        // Both should have same top-level keys
        assert_eq!(sub_parsed["status"], mt_parsed["status"]);
        assert!(sub_parsed["prompt_path"].is_string());
        assert!(mt_parsed["prompt_path"].is_string());
        assert!(sub_parsed["executor"].is_array());
        assert!(mt_parsed["executor"].is_array());
        assert_eq!(
            sub_parsed["next_actions"].as_array().unwrap().len(),
            mt_parsed["next_actions"].as_array().unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_build_workflow_response_multiple_subagent_executors() {
        // Even with multiple subagent executors, should still return (not dispatch)
        let tmp = setup_change_dir("test-multi-sub");
        let change_dir = tmp.path().join(".aw/changes/test-multi-sub");

        let result = build_workflow_response(
            &change_dir,
            "test-multi-sub",
            "create_change_spec",
            "multi prompt".to_string(),
            vec![
                "subagent:score-change-spec".to_string(),
                "subagent:Explore".to_string(),
            ],
            json!({}),
            SddInterface::default(),
            tmp.path(),
        )
        .await
        .unwrap();

        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        // is_subagent is true because .any() matches first element
        assert_eq!(
            parsed["next_actions"].as_array().unwrap().len(),
            0,
            "multiple subagent executors should still return, not dispatch"
        );
    }

    // ======================================================================
    // get_executor_chain tests — hardcoded hybrid mapping (no config lookup)
    // ======================================================================

    #[test]
    fn test_get_executor_chain_is_pure_and_ignores_config() {
        // get_executor_chain is a pure function; it returns the same values
        // regardless of whether config.toml exists or what it contains.
        let tmp = TempDir::new().unwrap();
        let chain = get_executor_chain(tmp.path(), WorkflowArtifact::CreateReferenceContext);
        assert_eq!(chain, vec!["subagent:Explore".to_string()]);
    }

    #[test]
    fn test_get_executor_chain_hardcoded_mapping() {
        // Verify the hardcoded hybrid mapping: lightweight actions → mainthread,
        // heavy actions → subagent.
        let tmp = TempDir::new().unwrap();

        // Mainthread actions
        assert_eq!(
            get_executor_chain(tmp.path(), WorkflowArtifact::RestructureInput),
            vec!["mainthread".to_string()]
        );
        assert_eq!(
            get_executor_chain(tmp.path(), WorkflowArtifact::CreatePreClarifications),
            vec!["mainthread".to_string()]
        );
        assert_eq!(
            get_executor_chain(tmp.path(), WorkflowArtifact::ReviseReferenceContext),
            vec!["mainthread".to_string()]
        );
        assert_eq!(
            get_executor_chain(tmp.path(), WorkflowArtifact::CreateChangeMerge),
            vec!["mainthread".to_string()]
        );

        // Explore subagent (model defined in agent definition)
        assert_eq!(
            get_executor_chain(tmp.path(), WorkflowArtifact::CreateReferenceContext),
            vec!["subagent:Explore".to_string()]
        );

        // Review subagents — quality gate (model defined in agent definition)
        assert_eq!(
            get_executor_chain(tmp.path(), WorkflowArtifact::ReviewReferenceContext),
            vec!["subagent:score-review".to_string()]
        );
        assert_eq!(
            get_executor_chain(tmp.path(), WorkflowArtifact::ReviewChangeSpec),
            vec!["subagent:score-review".to_string()]
        );
        assert_eq!(
            get_executor_chain(tmp.path(), WorkflowArtifact::ReviewChangeImplementation),
            vec!["subagent:score-review".to_string()]
        );

        // Spec authoring (model defined in agent definition)
        assert_eq!(
            get_executor_chain(tmp.path(), WorkflowArtifact::CreateChangeSpec),
            vec!["subagent:score-change-spec".to_string()]
        );

        // Impl authoring (model defined in agent definition)
        assert_eq!(
            get_executor_chain(tmp.path(), WorkflowArtifact::CreateChangeImplementation),
            vec!["subagent:score-change-implementation".to_string()]
        );
    }

    // ======================================================================
    // is_subagent detection edge cases
    // ======================================================================

    #[test]
    fn test_subagent_prefix_must_be_exact() {
        // "subagent_foo" should NOT trigger subagent routing (missing colon)
        let executor = vec!["subagent_invalid".to_string()];
        let is_subagent = executor.iter().any(|e| e.starts_with("subagent:"));
        assert!(
            !is_subagent,
            "subagent_invalid should NOT be detected as subagent (no colon after 'subagent')"
        );

        // "sub" prefix should not match
        let executor2 = vec!["sub:agent:sonnet".to_string()];
        let is_subagent2 = executor2.iter().any(|e| e.starts_with("subagent:"));
        assert!(!is_subagent2, "sub:agent:sonnet must not match subagent:");
    }

    #[test]
    fn test_is_subagent_logic_matches_spec() {
        // Verify the detection logic from the spec's routing pseudocode
        let cases: Vec<(Vec<&str>, bool, bool)> = vec![
            // (executor, expected_is_mainthread_only, expected_is_subagent)
            (vec!["mainthread"], true, false),
            (vec!["subagent:Explore"], false, true),
            (vec!["subagent:score-change-spec"], false, true),
            (vec!["gemini:flash"], false, false),
            (vec!["claude-agent:change-spec"], false, false),
            (vec!["codex:balanced"], false, false),
        ];

        for (executor, exp_mt, exp_sub) in cases {
            let executor_strings: Vec<String> = executor.iter().map(|s| s.to_string()).collect();
            let is_mainthread_only =
                executor_strings.len() == 1 && executor_strings[0] == "mainthread";
            let is_subagent = executor_strings.iter().any(|e| e.starts_with("subagent:"));

            assert_eq!(
                is_mainthread_only, exp_mt,
                "executor {:?}: is_mainthread_only",
                executor
            );
            assert_eq!(is_subagent, exp_sub, "executor {:?}: is_subagent", executor);

            // When either is true, response should be returned (not dispatched)
            if exp_mt || exp_sub {
                assert!(
                    is_mainthread_only || is_subagent,
                    "executor {:?}: should be returned to caller",
                    executor
                );
            } else {
                assert!(
                    !is_mainthread_only && !is_subagent,
                    "executor {:?}: should be dispatched via run_agent()",
                    executor
                );
            }
        }
    }
}
// CODEGEN-END
