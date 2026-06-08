//! Create-change handler for sdd_write_artifact(artifact="change", action="create")
//! and standalone sdd_init_change MCP tool.
//!
//! Initializes a new change directory with STATE.yaml. Optionally fetches issues
//! and builds the DAG. This extracts the side-effectful initialization that was
//! previously embedded in `run_change::route()`.
//!
//! Two entry points:
//! - `execute()` — called by `sdd_write_artifact(artifact="change", action="create")`
//! - `execute_standalone()` — called by `sdd_init_change` (standalone tool with #632 format)

use super::ToolDefinition;
use crate::issues::{local_backend, IssueBackend};
use crate::services::issue_parser;
use crate::state::StateManager;
use crate::Result;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/init_change/definition.md#source
// CODEGEN-BEGIN
/// MCP tool definition for sdd_init_change
/// @spec projects/agentic-workflow/tech-design/core/logic/structured-issue.md#changes
/// @spec projects/agentic-workflow/tech-design/core/logic/state-machine.md#changes
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_init_change".to_string(),
        description: "Initialize a new change directory and sync workflow state to the issue frontmatter. Returns next_actions for workflow chaining.".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "description"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path"
                },
                "change_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Change identifier (lowercase alphanumeric + hyphens)"
                },
                "description": {
                    "type": "string",
                    "description": "User's description of the change"
                },
                "issues": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Issue references (e.g. [\"#188\", \"#189\"]). Triggers fetch_issues."
                },
                "git_workflow": {
                    "type": "string",
                    "enum": ["new_branch", "in_place"],
                    "description": "Git workflow chosen by user"
                },
                "branch": {
                    "type": "string",
                    "description": "Git branch name. Recorded in STATE.yaml."
                }
            }
        }),
    }
}
// CODEGEN-END
/// Standalone execute for sdd_init_change MCP tool.
///
/// Params are top-level (not nested under `payload`). Returns #632 format
/// with `status`, `artifacts_written`, and `next_actions`.

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/init_change/standalone-flow.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/logic/structured-issue.md#requirements
/// @spec projects/agentic-workflow/tech-design/core/logic/state-machine.md#requirements
pub fn execute_standalone(args: &Value, project_root: &Path) -> Result<String> {
    let raw_change_id = super::get_required_string(args, "change_id")?;
    let description = super::get_required_string(args, "description")?;

    // REQ: issue-centric-workflow#U1/U2 — expand UUID-prefix change_id to full slug
    // before any further processing. Ambiguous prefix errors here with candidate list.
    // Only expand if the raw value doesn't already match an on-disk slug (slug wins).
    let change_id = if issue_parser::looks_like_uuid_prefix(&raw_change_id)
        && issue_parser::load_issue_body(project_root, &raw_change_id).is_none()
    {
        match issue_parser::find_slug_by_uuid_prefix(project_root, &raw_change_id)? {
            Some(slug) => slug,
            None => raw_change_id.clone(),
        }
    } else {
        raw_change_id.clone()
    };

    // Validate change_id format
    super::workflow_common::validate_change_id(&change_id)?;

    let interface = super::workflow_common::load_interface(project_root);
    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);

    // Error if change directory already exists
    if change_dir.exists() {
        anyhow::bail!(
            "Change '{}' already exists. Use sdd_run_change to continue.",
            change_id
        );
    }

    // NOTE: `git_workflow` and `branch` are deprecated inputs. Worktree isolation
    // is now always on: each change is created on a dedicated branch `cclab/<slug>`
    // in a worktree at `.aw/worktrees/<slug>`. Log a warning if callers still
    // pass these fields, but do not act on them.
    // REQ: worktree-per-change — init_change always creates a worktree
    let deprecated_branch = super::get_optional_string(args, "branch");
    let deprecated_git_workflow = super::get_optional_string(args, "git_workflow");
    if deprecated_branch.is_some() {
        tracing::warn!(
            change_id = %change_id,
            branch = ?deprecated_branch,
            "init_change: `branch` parameter is deprecated — worktree branch is always cclab/<slug>"
        );
    }
    if deprecated_git_workflow.is_some() {
        tracing::warn!(
            change_id = %change_id,
            git_workflow = ?deprecated_git_workflow,
            "init_change: `git_workflow` parameter is deprecated — worktree is always created"
        );
    }

    // Worktree branch is derived deterministically from change_id (= issue slug).
    let worktree_branch = format!("cclab/{}", change_id);
    let worktree_relpath = format!(".aw/worktrees/{}", change_id);

    // Branch uniqueness check — skip if worktree is expected and this change owns it.
    // We still keep the check for legacy callers that pass an explicit branch.
    if let Some(ref branch_name) = deprecated_branch {
        check_branch_uniqueness(project_root, branch_name, &change_id)?;
    }

    let issue_refs: Option<Vec<String>> =
        args.get("issues").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });

    // ── Pre-flight validation (NO side effects before these pass) ──────────
    //
    // All three checks must pass before creating any files or worktrees.
    // This prevents orphaned change dirs / worktrees on validation failure.

    // REQ: issue-centric-workflow#R1 — resolve issue slug
    let resolved_slug =
        issue_parser::resolve_issue_slug(project_root, &description, issue_refs.as_deref());
    let issue_slug = match &resolved_slug {
        Some(s) => s.clone(),
        None => anyhow::bail!(
            "No issue found for this change. Provide --issue \"#<num>\" or \
             include issue:<slug> in the description. \
             See: issue-centric-workflow.md R1."
        ),
    };

    // @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R1
    // R1: init_change rejects requests where resolved issue_slug differs from change_id.
    // One issue = one change: the on-disk change_id MUST equal the issue slug.
    if change_id != issue_slug {
        anyhow::bail!(
            "change_id '{}' does not match resolved issue slug '{}'. \
             change_id must equal the issue slug (one issue = one change). \
             Re-run with change_id='{}' or fix the issue reference.",
            change_id,
            issue_slug,
            issue_slug
        );
    }

    // REQ: structured-issue#R2 — hard gate, must be structured
    let issue_body = issue_parser::load_issue_body(project_root, &issue_slug);
    match &issue_body {
        Some(body) if issue_parser::is_structured_issue(body) => { /* OK */ }
        Some(_) => anyhow::bail!(
            "Issue '{}' is not structured (missing ## Problem, ## Requirements, or ## Scope). \
             Run `/aw:issue update {}` to prepare the issue before starting SDD. \
             See: structured-issue.md R2, issue-centric-workflow.md R7.",
            issue_slug,
            issue_slug
        ),
        None => anyhow::bail!(
            "Issue '{}' found by slug but body could not be loaded.",
            issue_slug
        ),
    }

    // Issue working copies are temp-backed. There is no checkout-hosted issue
    // file to require in git before lifecycle side effects.

    // REQ: issue-centric-workflow#R9 — filesystem-only 1:1:1 enforcement.
    // If a worktree already exists for this slug, a change is already in
    // progress. This replaces the old async backend query (flaky in tests).
    let wt_path = project_root.join(".aw/worktrees").join(&issue_slug);
    if wt_path.exists() {
        anyhow::bail!(
            "error: change '{}' already in progress ({} exists). \
             Complete or abandon it before starting a new one.",
            issue_slug,
            wt_path.display()
        );
    }

    // state:open gate — promote required before SDD can start. We still use
    // the backend here because it's the only thing that resolves state from
    // frontmatter, and we need state/phase on a per-issue basis.
    {
        let backend = local_backend(project_root);
        let issue_opt = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| handle.block_on(backend.get(&issue_slug)))
                .ok()
                .flatten()
        } else if let Ok(rt) = tokio::runtime::Runtime::new() {
            rt.block_on(backend.get(&issue_slug)).ok().flatten()
        } else {
            None
        };
        if let Some(issue) = &issue_opt {
            // REQ: R5 — init_change requires state:open
            if issue.state == crate::issues::types::IssueState::Draft {
                anyhow::bail!(
                    "Issue '{}' is still in draft state. Run `aw wi validate {}` \
                     to review and promote to open before starting SDD.",
                    issue_slug,
                    issue_slug
                );
            }
        }
    }

    // ── Side effects start here (all validations passed) ─────────────────

    // 1. Create worktree FIRST — change artifacts will live inside it.
    let worktree_created_path: Option<PathBuf> = match create_worktree(project_root, &change_id) {
        Ok(p) => Some(p),
        Err(e) => {
            tracing::warn!(
                change_id = %change_id,
                error = %e,
                "init_change: failed to create git worktree — continuing without isolation"
            );
            None
        }
    };

    // 2. Change dir lives in the worktree (not on main).
    //    Falls back to project_root if worktree creation failed (tests, non-git).
    let change_root = worktree_created_path.as_deref().unwrap_or(project_root);
    let has_issues = issue_refs.as_ref().is_some_and(|r| !r.is_empty());

    create_change_internal(
        change_root,
        &change_id,
        &description,
        issue_refs.as_deref(),
        Some("worktree"),
        Some(&worktree_branch),
    )?;

    let change_dir = change_root.join(".aw/changes").join(&change_id);
    {
        let mut sm = StateManager::load(&change_dir)?;
        // Phase stays at ChangeInited (default) — route() handles it
        sm.state_mut().branch = Some(worktree_branch.clone());
        sm.state_mut().git_workflow = Some("worktree".to_string());
        sm.state_mut().change_id = change_id.clone();
        // @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R4
        // R4: save() bubbles sync_to_issue() errors up — no STATE.yaml fallback.
        sm.save()?; // Writes workflow fields to issue frontmatter + operational data to meta.yaml
    }

    let mut written = Vec::<String>::new();
    if has_issues {
        written.push("issues/".to_string());
    }

    let next_tool = "sdd_run_change";

    let artifacts_written = json!(written);

    let worktree_path_json = if worktree_created_path.is_some() {
        json!(worktree_relpath)
    } else {
        Value::Null
    };

    let result = json!({
        "status": "ok",
        "artifacts_written": artifacts_written,
        "worktree_path": worktree_path_json,
        "worktree_branch": worktree_branch,
        "structured_issue_detected": true,
        "next_actions": [
            super::workflow_common::next_action(interface, next_tool, json!({"change_id": change_id}))
        ]
    });

    Ok(serde_json::to_string_pretty(&result)?)
}
// CODEGEN-END
// ─── Worktree Helpers ────────────────────────────────────────────────────────
//
// Thin wrappers around `crate::worktree` that lock the stage prefix to
// `cclab` (SDD changes). Issue authoring calls `worktree::provision` directly
// with stage `issue`.

#[allow(dead_code)] // retained for in-module tests that exercise cclab-stage provisioning
fn find_git_bin() -> Option<PathBuf> {
    crate::git::find_git_bin()
}

/// Create a `cclab/<slug>` worktree for an SDD change.
///
/// Returns the absolute worktree path on success. Uses legacy naming
/// (`.aw/worktrees/<slug>` + branch `cclab/<slug>`) pending the X3
/// migration to `change-<slug>` tracked as a separate epic.
///
/// Phase C note: the inline `git worktree add` call below is the last
/// remaining tenant of the legacy worktree-creation path. The `score`
/// CLI's CRRR verbs no longer call this; only the deprecated `tools`
/// MCP layer does (slated for deletion per the legacy-cleanup backlog).
// REQ: worktree-per-change
pub(super) fn create_worktree(project_root: &Path, slug: &str) -> Result<PathBuf> {
    let worktree_rel = format!(".aw/worktrees/{}", slug);
    let branch = format!("cclab/{}", slug);

    let git_bin = crate::git::find_git_bin()
        .ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;

    cleanup_stale_worktree(project_root, slug)?;

    let worktree_abs = project_root.join(&worktree_rel);
    if let Some(parent) = worktree_abs.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("failed to create worktree parent dir: {}", e))?;
    }

    let output = std::process::Command::new(&git_bin)
        .args(["worktree", "add", "-b", &branch, &worktree_rel])
        .current_dir(project_root)
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run git worktree add: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git worktree add failed: {}", stderr.trim());
    }

    Ok(worktree_abs)
}

/// Remove a stale `cclab/<slug>` worktree. Idempotent.
// REQ: worktree-per-change
#[allow(dead_code)] // retained for in-module tests that exercise cclab-stage provisioning
pub(super) fn cleanup_stale_worktree(project_root: &Path, slug: &str) -> Result<()> {
    let worktree_rel = format!(".aw/worktrees/{}", slug);
    let branch = format!("cclab/{}", slug);

    let git_bin = match crate::git::find_git_bin() {
        Some(g) => g,
        None => return Ok(()),
    };

    let worktree_abs = project_root.join(&worktree_rel);

    if worktree_abs.exists() {
        let _ = std::process::Command::new(&git_bin)
            .args(["worktree", "remove", "--force", &worktree_rel])
            .current_dir(project_root)
            .output();

        if worktree_abs.exists() {
            let _ = std::fs::remove_dir_all(&worktree_abs);
        }
    }

    let _ = std::process::Command::new(&git_bin)
        .args(["worktree", "prune"])
        .current_dir(project_root)
        .output();

    let branch_check = std::process::Command::new(&git_bin)
        .args([
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{}", branch),
        ])
        .current_dir(project_root)
        .status();
    if let Ok(status) = branch_check {
        if status.success() {
            let _ = std::process::Command::new(&git_bin)
                .args(["branch", "-D", &branch])
                .current_dir(project_root)
                .output();
        }
    }

    Ok(())
}

// NOTE: try_structured_issue_skip and write_phase_to_issue removed per R5/R12.
// - Intermediate artifacts (requirements.md, pre_clarifications.md, etc.) no longer generated.
// - StateManager::save() dual-writes all workflow fields to issue frontmatter.
// - Spec/impl phases read the issue file directly as context.

/// Check that no other active (non-terminal) change uses the same branch.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R9
/// R9: Scans `.aw/worktrees/` plus issue frontmatter (not STATE.yaml).
///
/// Algorithm:
/// 1. Iterate `.aw/worktrees/{slug}/` directories — each represents an active change.
/// 2. For each, derive the expected branch `cclab/{slug}`.
/// 3. Cross-reference the temp-backed issue working copy: if its
///    `branch` frontmatter matches `branch_name` and its phase is not terminal,
///    we have a conflict.
/// 4. Fall back to directory-name heuristic if issue is unavailable (non-terminal assumed).
fn check_branch_uniqueness(
    project_root: &Path,
    branch_name: &str,
    new_change_id: &str,
) -> Result<()> {
    let worktrees_dir = project_root.join(".aw/worktrees");
    if !worktrees_dir.exists() {
        return Ok(());
    }

    let entries = std::fs::read_dir(&worktrees_dir)?;
    for entry in entries.flatten() {
        let existing_id = entry.file_name().to_str().unwrap_or_default().to_string();
        if existing_id.is_empty() || existing_id == new_change_id {
            continue;
        }
        // Only directories represent live worktrees
        if !entry.path().is_dir() {
            continue;
        }

        // Read issue frontmatter to learn branch + phase.
        // Issue slug = worktree dir name (1:1:1 mapping per R1).
        let issue_info = load_issue_branch_and_phase(project_root, &existing_id);

        let (existing_branch, phase_is_terminal) = match issue_info {
            Some((b, t)) => (b, t),
            None => {
                // No issue found — fall back to deterministic branch mapping.
                // If a worktree exists with the target branch name under
                // `cclab/{slug}`, treat as non-terminal conflict.
                (Some(format!("cclab/{}", existing_id)), false)
            }
        };

        if existing_branch.as_deref() == Some(branch_name) && !phase_is_terminal {
            let phase_label = if phase_is_terminal {
                "terminal"
            } else {
                "active"
            };
            anyhow::bail!(
                "Branch '{}' already has an active change '{}' (phase: {}). \
                 Complete or archive it first.",
                branch_name,
                existing_id,
                phase_label,
            );
        }
    }
    Ok(())
}

/// Load `(branch, phase_is_terminal)` from an issue's frontmatter.
///
/// Returns `None` if the issue file cannot be located or parsed.
/// Looks in both open and closed temp-backed issue working-copy states.
fn load_issue_branch_and_phase(project_root: &Path, slug: &str) -> Option<(Option<String>, bool)> {
    use crate::models::state::StatePhase;

    let backend = local_backend(project_root);
    let issue_opt = if let Ok(handle) = tokio::runtime::Handle::try_current() {
        tokio::task::block_in_place(|| handle.block_on(backend.get(slug)))
            .ok()
            .flatten()
    } else if let Ok(rt) = tokio::runtime::Runtime::new() {
        rt.block_on(backend.get(slug)).ok().flatten()
    } else {
        None
    }?;

    let phase_terminal = issue_opt
        .phase
        .as_deref()
        .and_then(|p| super::phase_transition::parse_phase(p).ok())
        .map(|p: StatePhase| p.is_terminal())
        .unwrap_or(false);

    Some((issue_opt.branch, phase_terminal))
}

/// Shared creation logic used by both `execute()` and `execute_standalone()`.
fn create_change_internal(
    project_root: &Path,
    change_id: &str,
    _description: &str,
    issue_refs: Option<&[String]>,
    git_workflow: Option<&str>,
    branch: Option<&str>,
) -> Result<()> {
    let change_dir = project_root.join(".aw/changes").join(change_id);
    std::fs::create_dir_all(&change_dir)?;

    // Fetch issues from remote (GitHub/GitLab) if provided as #<num> refs.
    // Local slug refs (non-numeric) are already on disk and don't need fetching.
    if let Some(refs) = issue_refs {
        let remote_refs: Vec<&str> = refs
            .iter()
            .filter(|r| r.starts_with('#') || r.parse::<u64>().is_ok())
            .map(|s| s.as_str())
            .collect();
        if !remote_refs.is_empty() {
            let fetch_args = json!({
                "change_id": change_id,
                "issue_refs": remote_refs,
            });
            super::fetch_issues::execute(&fetch_args, project_root)?;
        }
    }

    // user_input.md no longer written — issue body provides all context.
    // change_id = issue slug, so StateManager resolves the issue directly.

    let mut sm = StateManager::load(&change_dir)?;
    if let Some(wf) = git_workflow {
        sm.state_mut().git_workflow = Some(wf.to_string());
    }
    if let Some(br) = branch {
        sm.state_mut().branch = Some(br.to_string());
    }

    sm.save()?;
    Ok(())
}

/// Execute the create-change handler (legacy entry point for sdd_write_artifact).
///
/// Payload fields:
/// - `description` (required): User's description of the change
/// - `issue_refs` (optional): Array of issue references (e.g. `["#188", "#189"]`)
/// - `git_workflow` (optional): `"new_branch"` | `"in_place"`

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/init_change/legacy-artifact-flow.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/logic/structured-issue.md#changes
/// @spec projects/agentic-workflow/tech-design/core/logic/state-machine.md#requirements
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = super::get_required_string(args, "change_id")?;
    let payload = args.get("payload").cloned().unwrap_or_else(|| json!({}));

    let description = payload
        .get("description")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("payload.description is required for change/create"))?
        .to_string();

    let issue_refs: Option<Vec<String>> =
        payload
            .get("issue_refs")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            });

    let git_workflow = payload
        .get("git_workflow")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    create_change_internal(
        project_root,
        &change_id,
        &description,
        issue_refs.as_deref(),
        git_workflow.as_deref(),
        None,
    )?;

    // @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R7
    // R7: user_input.md is no longer generated. STATE.yaml is no longer the
    // primary store — workflow fields live in the issue frontmatter.
    let mut result = json!({
        "status": "ok",
        "change_id": change_id,
        "artifact": "change",
        "action": "create",
        "artifacts_written": [],
        "phase": "clarified",
    });

    if issue_refs.as_ref().is_some_and(|r| !r.is_empty()) {
        result["has_issues"] = json!(true);
    }

    Ok(serde_json::to_string_pretty(&result)?)
}
// CODEGEN-END
#[cfg(test)]
mod tests {
    use crate::models::StatePhase;

    use super::*;
    use tempfile::TempDir;

    fn setup_project() -> TempDir {
        // R4: tests that call save() need an issue file backing `test-change`.
        crate::test_util::setup_project_with_issue("test-change")
    }

    fn issue_store_dir(project_root: &std::path::Path, state: &str) -> std::path::PathBuf {
        crate::shared::workspace::issues_path(project_root).join(state)
    }

    fn issue_store_path(
        project_root: &std::path::Path,
        state: &str,
        slug: &str,
    ) -> std::path::PathBuf {
        issue_store_dir(project_root, state).join(format!("{slug}.md"))
    }

    #[test]
    fn test_create_change_basic() {
        let tmp = setup_project();
        let args = json!({
            "change_id": "test-change",
            "payload": {
                "description": "Add new feature",
                "git_workflow": "new_branch"
            }
        });
        let result = execute(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["artifact"], "change");
        assert_eq!(parsed["action"], "create");

        // Verify artifacts were created. R11: meta.yaml only written when
        // operational data non-empty. Change dir itself must exist.
        let change_dir = tmp.path().join(".aw/changes/test-change");
        assert!(change_dir.exists(), "change dir must be created");
        // user_input.md no longer written — issue body is the source of context

        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(sm.state().git_workflow.as_deref(), Some("new_branch"));
    }

    #[test]
    fn test_create_change_requires_description() {
        let tmp = setup_project();
        let args = json!({
            "change_id": "test-change",
            "payload": {}
        });
        let result = execute(&args, tmp.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("description is required"));
    }

    #[test]
    fn test_create_change_without_git_workflow() {
        let tmp = setup_project();
        let args = json!({
            "change_id": "test-change",
            "payload": {
                "description": "Simple change"
            }
        });
        let result = execute(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        let change_dir = tmp.path().join(".aw/changes/test-change");
        // user_input.md no longer written

        let sm = StateManager::load(&change_dir).unwrap();
        assert!(sm.state().git_workflow.is_none());
    }

    // --- Standalone (sdd_init_change) tests ---

    #[test]
    fn test_standalone_basic() {
        let tmp = setup_project();
        let slug = "enhancement-test-basic";
        create_structured_issue(tmp.path(), slug);

        // R1: change_id must equal issue_slug.
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": slug,
            "description": format!("Add authentication issue:{}", slug),
        });
        let result = execute_standalone(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["status"], "ok");
        assert!(parsed["next_actions"].is_array());
        // Structured issue detected — phase skips to post_clarifications_created
        assert_eq!(parsed["structured_issue_detected"], true);

        // Verify files
        let change_dir = tmp.path().join(".aw/changes").join(slug);
        // R5: STATE.yaml is deprecated — meta.yaml is the operational store.
        assert!(change_dir.join("meta.yaml").exists() || !change_dir.join("STATE.yaml").exists());

        // REQ: worktree-per-change — issue frontmatter records the worktree branch cclab/<slug>.
        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(sm.state().git_workflow.as_deref(), Some("worktree"));
        assert_eq!(
            sm.state().branch.as_deref(),
            Some(format!("cclab/{}", slug).as_str())
        );
    }

    #[test]
    fn test_standalone_duplicate_change_dir() {
        let tmp = setup_project();
        // Pre-create the change directory
        std::fs::create_dir_all(tmp.path().join(".aw/changes/existing")).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "existing",
            "description": "Should fail"
        });
        let result = execute_standalone(&args, tmp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    // @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R9
    // R9: check_branch_uniqueness now scans `.aw/worktrees/` + issue
    // frontmatter. Fixture mirrors the new storage layout.
    #[test]
    fn test_standalone_branch_conflict() {
        let tmp = setup_project();
        // Simulate an existing active change as a worktree directory whose
        // issue frontmatter carries the shared branch.
        let existing_slug = "old-change";
        std::fs::create_dir_all(tmp.path().join(".aw/worktrees").join(existing_slug)).unwrap();
        let issues_dir = issue_store_dir(tmp.path(), "open");
        std::fs::create_dir_all(&issues_dir).unwrap();
        std::fs::write(
            issues_dir.join(format!("{}.md", existing_slug)),
            "---\n\
             type: refactor\n\
             title: 'existing'\n\
             state: open\n\
             phase: change_spec_created\n\
             branch: feat/shared-branch\n\
             ---\n\n## Problem\n\nExisting.\n",
        )
        .unwrap();

        // New change tries to reuse the same branch
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "new-change",
            "description": "Should fail due to branch conflict",
            "branch": "feat/shared-branch"
        });
        let result = execute_standalone(&args, tmp.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("feat/shared-branch"));
        assert!(err.contains("old-change"));
    }

    // @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R9
    #[test]
    fn test_standalone_branch_ok_when_existing_is_terminal() {
        let tmp = setup_project();
        let slug = "enhancement-test-branch-ok";
        create_structured_issue(tmp.path(), slug);

        // Simulate an existing terminal change — worktree dir + issue with
        // `phase: change_archived` (terminal).
        let existing_slug = "archived-change";
        std::fs::create_dir_all(tmp.path().join(".aw/worktrees").join(existing_slug)).unwrap();
        let issues_dir = issue_store_dir(tmp.path(), "closed");
        std::fs::create_dir_all(&issues_dir).unwrap();
        std::fs::write(
            issues_dir.join(format!("{}.md", existing_slug)),
            "---\n\
             type: refactor\n\
             title: 'archived'\n\
             state: closed\n\
             phase: change_archived\n\
             branch: feat/reusable-branch\n\
             ---\n\n## Problem\n\nArchived.\n",
        )
        .unwrap();

        // New change on the same branch should succeed (existing is terminal)
        // R1: change_id must equal issue_slug.
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": slug,
            "description": format!("Should succeed issue:{}", slug),
            "branch": "feat/reusable-branch"
        });
        let result = execute_standalone(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
    }

    // --- Structured issue skip tests ---

    /// Helper: create a structured issue working copy in the temp open store.
    fn create_structured_issue(project_root: &std::path::Path, slug: &str) {
        let issues_dir = issue_store_dir(project_root, "open");
        std::fs::create_dir_all(&issues_dir).unwrap();
        let content = format!(
            r#"---
type: enhancement
title: "Test structured issue"
state: open
---

## Problem

SDD phases re-derive information that a well-structured issue already contains.

## Requirements

- **R1**: Parse markdown body by splitting on headers
- **R2** (high): Extract ID patterns from list items
- **R3**: Backward compat with unstructured issues

## Acceptance Criteria

- **AC1**: Structured issues skip to PostClarificationsCreated
- **AC2**: Unstructured issues proceed through normal flow

## Scope

### In Scope
- Issue section parser
- init_change update for phase skip

### Out of Scope
- CLI enrich command

## Key Decisions

- **D1**: No new state machine states
- **D2**: Reference Context is agent-filled

## Reference Context

See tech_design/sdd/ for state machine specs.
"#
        );
        std::fs::write(issues_dir.join(format!("{}.md", slug)), content).unwrap();
    }

    // REQ: REQ-012 - Structured issue detection + phase skip
    #[test]
    fn test_standalone_structured_issue_skip() {
        let tmp = setup_project();
        let slug = "enhancement-test-structured";
        create_structured_issue(tmp.path(), slug);

        // R1: change_id must equal issue_slug.
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": slug,
            "description": format!("Implement issue:{}", slug),
        });
        let result = execute_standalone(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["structured_issue_detected"], true);

        // Verify next_actions routes to sdd_run_change (not restructure_input)
        let next = &parsed["next_actions"][0];
        assert_eq!(next["args"]["change_id"], slug);

        // REQ: R12 — No intermediate artifacts generated (requirements.md, etc.)
        // Spec/impl phases read the issue file directly as context.
        let change_dir = tmp.path().join(".aw/changes").join(slug);
        let group_dir = change_dir.join("groups/default");
        assert!(!group_dir.exists(), "No group dir should be created (R12)");

        // Verify phase via StateManager (reads issue frontmatter per R5).
        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(*sm.phase(), StatePhase::ChangeInited);
    }

    // REQ: REQ-013 - Unstructured issue falls through to normal flow
    #[test]
    // REQ: structured-issue#R2 — hard gate rejects unstructured issues
    fn test_standalone_unstructured_issue_no_skip() {
        let tmp = setup_project();

        // Create an unstructured issue (missing ## Scope)
        let issues_dir = issue_store_dir(tmp.path(), "open");
        std::fs::create_dir_all(&issues_dir).unwrap();
        std::fs::write(
            issues_dir.join("enhancement-plain.md"),
            "---\ntype: enhancement\ntitle: Plain issue\nstate: open\n---\n\n## Problem\n\nSome problem.\n\n## Requirements\n\n- R1: do thing\n",
        )
        .unwrap();

        // R1: change_id must equal issue_slug.
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "enhancement-plain",
            "description": "Implement issue:enhancement-plain",
        });
        // Hard gate: unstructured → error (no side effects)
        let result = execute_standalone(&args, tmp.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("not structured"),
            "Expected structured-gate error, got: {}",
            err
        );

        // No side effects: change dir should NOT exist
        assert!(!tmp.path().join(".aw/changes/enhancement-plain").exists());
    }

    // REQ: issue-centric-workflow#R1 — no issue slug → error
    #[test]
    fn test_standalone_no_issue_slug_no_skip() {
        let tmp = setup_project();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "no-slug-test",
            "description": "Add a new feature without referencing any issue",
        });
        // Hard gate: no issue found → error (no side effects)
        let result = execute_standalone(&args, tmp.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("No issue found"),
            "Expected no-issue error, got: {}",
            err
        );

        // No side effects
        assert!(!tmp.path().join(".aw/changes/no-slug-test").exists());
    }

    // REQ: structured-issue#R2 — nonexistent issue slug → error
    #[test]
    fn test_standalone_nonexistent_issue_slug_no_skip() {
        let tmp = setup_project();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "ghost-test",
            "description": "Implement issue:does-not-exist",
        });
        // Slug resolves but file doesn't exist → body load fails → error
        let result = execute_standalone(&args, tmp.path());
        assert!(result.is_err());
        // Could be "No issue found" (slug doesn't resolve) or "body could not be loaded"
        // depending on whether the slug matched a file
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("No issue found")
                || err.contains("not structured")
                || err.contains("body could not be loaded"),
            "Expected issue resolution error, got: {}",
            err
        );

        // No side effects
        assert!(!tmp.path().join(".aw/changes/ghost-test").exists());
    }

    // ─── Worktree tests ──────────────────────────────────────────────────

    /// Initialize a bare-minimum git repo in the given dir so worktree
    /// commands can run. Returns true if git is available.
    fn init_git_repo(dir: &std::path::Path) -> bool {
        let Some(git) = find_git_bin() else {
            return false;
        };
        let ok = std::process::Command::new(&git)
            .args(["init", "-b", "main"])
            .current_dir(dir)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !ok {
            return false;
        }
        // Configure identity so commits can be made
        let _ = std::process::Command::new(&git)
            .args(["config", "user.email", "test@example.com"])
            .current_dir(dir)
            .output();
        let _ = std::process::Command::new(&git)
            .args(["config", "user.name", "Test"])
            .current_dir(dir)
            .output();
        // Need an initial commit so `worktree add -b` has a base
        std::fs::write(dir.join("README.md"), "init\n").unwrap();
        let _ = std::process::Command::new(&git)
            .args(["add", "README.md"])
            .current_dir(dir)
            .output();
        let _ = std::process::Command::new(&git)
            .args(["commit", "-m", "init"])
            .current_dir(dir)
            .output();
        true
    }

    // REQ: worktree-per-change — create_worktree makes a branch and directory
    #[test]
    fn test_create_worktree_happy_path() {
        let tmp = TempDir::new().unwrap();
        if !init_git_repo(tmp.path()) {
            return; // skip if git unavailable
        }
        let slug = "enhancement-worktree-test";
        let result = create_worktree(tmp.path(), slug);
        assert!(
            result.is_ok(),
            "create_worktree should succeed: {:?}",
            result.err()
        );
        let worktree_path = result.unwrap();
        assert!(worktree_path.exists(), "worktree directory should exist");
        assert_eq!(
            worktree_path,
            tmp.path().join(format!(".aw/worktrees/{}", slug))
        );

        // Verify branch was created
        let git = find_git_bin().unwrap();
        let status = std::process::Command::new(&git)
            .args([
                "show-ref",
                "--verify",
                &format!("refs/heads/cclab/{}", slug),
            ])
            .current_dir(tmp.path())
            .status()
            .unwrap();
        assert!(status.success(), "branch cclab/{} should exist", slug);
    }

    // REQ: worktree-per-change — create_worktree cleans up stale state
    #[test]
    fn test_create_worktree_cleans_stale() {
        let tmp = TempDir::new().unwrap();
        if !init_git_repo(tmp.path()) {
            return;
        }
        let slug = "enhancement-stale-test";

        // First creation
        create_worktree(tmp.path(), slug).unwrap();
        assert!(tmp.path().join(format!(".aw/worktrees/{}", slug)).exists());

        // Second call on the same slug should auto-cleanup and recreate
        let result = create_worktree(tmp.path(), slug);
        assert!(
            result.is_ok(),
            "create_worktree should succeed on stale state: {:?}",
            result.err()
        );
        assert!(tmp.path().join(format!(".aw/worktrees/{}", slug)).exists());
    }

    // REQ: worktree-per-change — cleanup_stale_worktree is idempotent
    #[test]
    fn test_cleanup_stale_worktree_idempotent() {
        let tmp = TempDir::new().unwrap();
        if !init_git_repo(tmp.path()) {
            return;
        }
        let slug = "enhancement-idempotent";
        // Cleanup with nothing present — must not error
        cleanup_stale_worktree(tmp.path(), slug).unwrap();

        // Create then cleanup twice — must not error
        create_worktree(tmp.path(), slug).unwrap();
        cleanup_stale_worktree(tmp.path(), slug).unwrap();
        cleanup_stale_worktree(tmp.path(), slug).unwrap();
    }

    // REQ: worktree-per-change — execute_standalone creates worktree when git is available
    #[test]
    fn test_standalone_creates_worktree() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".aw/changes")).unwrap();
        let slug = "enhancement-wt-standalone";
        // Issue working copy is temp-backed; git only owns the lifecycle worktree.
        if !init_git_repo_with_temp_issue(tmp.path(), slug) {
            return;
        }

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": slug,
            "description": format!("Add worktree isolation issue:{}", slug),
        });
        let result = execute_standalone(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["worktree_branch"], format!("cclab/{}", slug));
        assert_eq!(parsed["worktree_path"], format!(".aw/worktrees/{}", slug));
        assert!(tmp.path().join(format!(".aw/worktrees/{}", slug)).exists());
    }

    // REQ: REQ-016 - Structured issue without optional sections
    #[test]
    fn test_standalone_minimal_structured_issue() {
        let tmp = setup_project();

        // Create a minimal structured issue (only required sections)
        let issues_dir = issue_store_dir(tmp.path(), "open");
        std::fs::create_dir_all(&issues_dir).unwrap();
        std::fs::write(
            issues_dir.join("enhancement-minimal.md"),
            "---\ntype: enhancement\ntitle: Minimal\nstate: open\n---\n\n## Problem\n\nMinimal problem.\n\n## Requirements\n\n- **R1**: Single requirement\n\n## Scope\n\nJust this one thing.\n",
        )
        .unwrap();

        // R1: change_id must equal issue_slug.
        let slug = "enhancement-minimal";
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": slug,
            "description": format!("Implement issue:{}", slug),
        });
        let result = execute_standalone(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["structured_issue_detected"], true);

        // REQ: R12 — No intermediate artifacts generated
        let change_dir = tmp.path().join(".aw/changes").join(slug);
        let group_dir = change_dir.join("groups/default");
        assert!(!group_dir.exists(), "No group dir should be created (R12)");

        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(*sm.phase(), StatePhase::ChangeInited);
    }

    // ─── Git-backed lifecycle with temp issue working copies ─────────────
    // These tests exercise real git mode while issue working copies stay
    // outside the checkout under `/tmp/aw/workspaces/.../issues`.

    /// Initialize a git repo and create a structured temp issue working copy.
    fn init_git_repo_with_temp_issue(dir: &std::path::Path, slug: &str) -> bool {
        if !init_git_repo(dir) {
            return false;
        }
        create_structured_issue(dir, slug);
        true
    }

    // REQ: issue-centric-workflow#R7 — temp issue working copies are not git-tracked.
    #[test]
    fn test_temp_backed_issue_does_not_require_git_tracking() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".aw/changes")).unwrap();
        if !init_git_repo(tmp.path()) {
            return;
        }

        let slug = "enhancement-untracked";
        create_structured_issue(tmp.path(), slug);

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": slug,
            "description": format!("Implement issue:{}", slug),
        });
        let result = execute_standalone(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert!(!tmp.path().join(".aw").join("issues").exists());
    }

    // REQ: issue-centric-workflow#R8 — temp issue edits do not dirty the repo.
    #[test]
    fn test_temp_backed_issue_edits_do_not_require_clean_git_status() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".aw/changes")).unwrap();
        let slug = "enhancement-dirty";
        if !init_git_repo_with_temp_issue(tmp.path(), slug) {
            return;
        }

        let abs = issue_store_path(tmp.path(), "open", slug);
        let current = std::fs::read_to_string(&abs).unwrap();
        std::fs::write(&abs, format!("{}\nextra line\n", current)).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": slug,
            "description": format!("Implement issue:{}", slug),
        });
        let result = execute_standalone(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert!(!tmp.path().join(".aw").join("issues").exists());
    }

    // REQ: issue-centric-workflow#R9 — existing worktree must block re-init.
    #[test]
    fn test_preflight_r9_existing_worktree_aborts() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".aw/changes")).unwrap();
        let slug = "enhancement-existing-wt";
        if !init_git_repo_with_temp_issue(tmp.path(), slug) {
            return;
        }

        // Pre-create a worktree directory (as if a previous run owned it).
        std::fs::create_dir_all(tmp.path().join(format!(".aw/worktrees/{}", slug))).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": slug,
            "description": format!("Implement issue:{}", slug),
        });
        let result = execute_standalone(&args, tmp.path());
        assert!(result.is_err(), "existing worktree must abort");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("already in progress"),
            "expected 1:1:1 error, got: {}",
            err
        );
    }

    // REQ: issue-centric-workflow#R7/R8 — temp-backed issue allows init.
    #[test]
    fn test_preflight_happy_path_temp_issue_store() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".aw/changes")).unwrap();
        let slug = "enhancement-clean";
        if !init_git_repo_with_temp_issue(tmp.path(), slug) {
            return;
        }

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": slug,
            "description": format!("Implement issue:{}", slug),
        });
        let result = execute_standalone(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert!(tmp.path().join(format!(".aw/worktrees/{}", slug)).exists());
    }

    // ─── Refactor tests (T1, T2, T5, T6) ──────────────────────────────────
    // @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md

    /// Helper: write a minimal structured issue file for R1/R2 tests. Distinct
    /// from `create_structured_issue` so the slug-mismatch test can inspect a
    /// known-good payload without ambiguity.
    fn write_minimal_open_issue(project_root: &std::path::Path, slug: &str) {
        let issues_dir = issue_store_dir(project_root, "open");
        std::fs::create_dir_all(&issues_dir).unwrap();
        let body = "---\n\
type: enhancement\n\
title: \"Minimal issue for slug invariant\"\n\
state: open\n\
---\n\n\
## Problem\n\nMismatched change_id must be rejected.\n\n\
## Requirements\n\n- **R1**: Enforce change_id == issue_slug.\n\n\
## Scope\n\nJust the invariant.\n";
        std::fs::write(issues_dir.join(format!("{}.md", slug)), body).unwrap();
    }

    // @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R1
    // T1: init_change rejects when change_id != resolved issue_slug.
    // Verifies the structured error message, and that no worktree / change
    // dir / issue mutation side-effects occurred.
    #[test]
    fn test_r1_init_change_rejects_mismatched_change_id() {
        let tmp = setup_project();
        let real_slug = "enhancement-real-module-import-system";
        write_minimal_open_issue(tmp.path(), real_slug);

        // change_id (feat-mamba-import-system) != resolved slug (enhancement-real-module-import-system)
        let bad_change_id = "feat-mamba-import-system";
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": bad_change_id,
            "description": format!("issue:{}", real_slug),
        });

        let result = execute_standalone(&args, tmp.path());
        assert!(result.is_err(), "mismatched change_id must be rejected");
        let err = result.unwrap_err().to_string();

        // Error message must contain the invariant-enforcing phrase and both slugs.
        assert!(
            err.contains(&format!(
                "change_id '{}' does not match resolved issue slug '{}'",
                bad_change_id, real_slug
            )),
            "error must quote both slugs, got: {}",
            err
        );
        assert!(
            err.contains("change_id must equal the issue slug"),
            "error must spell out the invariant, got: {}",
            err
        );

        // No worktree created for either identifier.
        assert!(
            !tmp.path()
                .join(format!(".aw/worktrees/{}", bad_change_id))
                .exists(),
            "no worktree for the bad change_id"
        );
        assert!(
            !tmp.path()
                .join(format!(".aw/worktrees/{}", real_slug))
                .exists(),
            "no worktree for the real slug"
        );

        // No change directory created for either identifier.
        assert!(
            !tmp.path()
                .join(format!(".aw/changes/{}", bad_change_id))
                .exists(),
            "no change dir for the bad change_id"
        );
        assert!(
            !tmp.path()
                .join(format!(".aw/changes/{}", real_slug))
                .exists(),
            "no change dir for the real slug"
        );

        // Issue file must remain untouched (no phase/change_id/branch injected).
        let issue_path = issue_store_path(tmp.path(), "open", real_slug);
        let issue_body = std::fs::read_to_string(&issue_path).unwrap();
        assert!(
            !issue_body.contains("phase:"),
            "issue frontmatter must not gain phase"
        );
        assert!(
            !issue_body.contains("change_id:"),
            "issue frontmatter must not gain change_id"
        );
        assert!(
            !issue_body.contains("branch:"),
            "issue frontmatter must not gain branch"
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R2
    // T2: init_change refuses to operate when no issue file exists for the
    // given change_id. This covers the `score run-change <change_id>` entry
    // point, which routes to init_change when the change dir does not yet
    // exist. Since no issue file resolves the slug, issue_parser returns
    // None and init_change bails with the "No issue found" gate — the same
    // invariant surface as R1 (one issue = one change).
    #[test]
    fn test_r2_run_change_refuses_when_issue_file_missing() {
        let tmp = setup_project();
        let change_id = "feat-without-backing-issue";

        // Sanity: no issue file exists anywhere.
        assert!(!issue_store_path(tmp.path(), "open", change_id).exists());
        assert!(!issue_store_path(tmp.path(), "closed", change_id).exists());

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": change_id,
            "description": format!("issue:{}", change_id),
        });

        let result = execute_standalone(&args, tmp.path());
        assert!(result.is_err(), "missing issue file must abort init_change");
        let err = result.unwrap_err().to_string();
        // Either the slug resolves to nothing ("No issue found") or body load
        // fails — both are acceptable surfaces enforcing the invariant.
        assert!(
            err.contains("No issue found") || err.contains("body could not be loaded"),
            "expected missing-issue error, got: {}",
            err
        );

        // No side-effects: no change dir, no worktree.
        assert!(
            !tmp.path()
                .join(format!(".aw/changes/{}", change_id))
                .exists(),
            "no change dir on rejected init"
        );
        assert!(
            !tmp.path()
                .join(format!(".aw/worktrees/{}", change_id))
                .exists(),
            "no worktree on rejected init"
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R8
    // T5: create_change_internal no longer creates groups/{gid}/ subtrees.
    // Specs, prompts, payloads live flat at .aw/changes/{id}/{...}/.
    #[test]
    fn test_r8_no_groups_subtree_after_init() {
        let tmp = setup_project();
        let slug = "enhancement-r8-flat-layout";
        create_structured_issue(tmp.path(), slug);

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": slug,
            "description": format!("Implement issue:{}", slug),
        });
        let result = execute_standalone(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        // Whether or not a worktree was created (depends on git availability),
        // the change dir must exist somewhere and must NOT have groups/.
        let wt_change_dir = tmp
            .path()
            .join(format!(".aw/worktrees/{}/.aw/changes/{}", slug, slug));
        let legacy_change_dir = tmp.path().join(format!(".aw/changes/{}", slug));
        let change_dir = if wt_change_dir.exists() {
            wt_change_dir
        } else {
            legacy_change_dir
        };
        assert!(change_dir.exists(), "change dir should exist");

        // R8 invariant: no groups/ nesting. Check both common ids.
        assert!(
            !change_dir.join("groups").exists(),
            "groups/ subtree must not exist (R8)"
        );
        assert!(
            !change_dir.join("groups/default").exists(),
            "groups/default/ must not exist (R8)"
        );
        assert!(
            !change_dir.join("groups").join(slug).exists(),
            "groups/{{slug}}/ must not exist (R8)"
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R9
    // T6: check_branch_uniqueness detects collisions via .aw/worktrees/
    // scan + issue frontmatter (not STATE.yaml). Mock two worktree dirs
    // with the same target branch and assert the conflict is reported.
    #[test]
    fn test_r9_check_branch_uniqueness_scans_worktrees_not_state_yaml() {
        let tmp = setup_project();

        // Mock an "active" change: worktree dir exists + issue frontmatter
        // records a matching branch and a non-terminal phase.
        let existing_slug = "enhancement-r9-existing";
        std::fs::create_dir_all(tmp.path().join(format!(".aw/worktrees/{}", existing_slug)))
            .unwrap();
        let issues_dir = issue_store_dir(tmp.path(), "open");
        std::fs::create_dir_all(&issues_dir).unwrap();
        let existing_issue = format!(
            "---\n\
type: enhancement\n\
title: \"R9 existing\"\n\
state: open\n\
branch: cclab/{slug}\n\
phase: change_spec_created\n\
change_id: {slug}\n\
---\n\n\
## Problem\n\nExisting change holding the branch.\n\n\
## Requirements\n\n- R1: reserve the branch.\n\n\
## Scope\n\nHolding.\n",
            slug = existing_slug
        );
        std::fs::write(
            issues_dir.join(format!("{}.md", existing_slug)),
            existing_issue,
        )
        .unwrap();

        // New change attempting the same branch name must fail.
        let target_branch = format!("cclab/{}", existing_slug);
        let new_change_id = "enhancement-r9-new";
        let result = check_branch_uniqueness(tmp.path(), &target_branch, new_change_id);
        assert!(
            result.is_err(),
            "branch collision must be detected from worktree scan + issue frontmatter"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains(&target_branch),
            "error must quote the colliding branch, got: {}",
            err
        );
        assert!(
            err.contains(existing_slug),
            "error must quote the existing change id, got: {}",
            err
        );

        // R9 invariant: decision is NOT driven by STATE.yaml. Remove
        // STATE.yaml (or prove it was never required) and repeat the check
        // to confirm the algorithm relies solely on worktrees/ + frontmatter.
        let stray_state = tmp
            .path()
            .join(format!(".aw/worktrees/{}/STATE.yaml", existing_slug));
        assert!(
            !stray_state.exists(),
            "no STATE.yaml fixture was created — algorithm must not need one"
        );
        let result_again = check_branch_uniqueness(tmp.path(), &target_branch, new_change_id);
        assert!(
            result_again.is_err(),
            "collision still detected without STATE.yaml"
        );

        // And different branch name on same worktree set is fine.
        let unrelated = "cclab/unrelated-branch";
        let ok = check_branch_uniqueness(tmp.path(), unrelated, new_change_id);
        assert!(ok.is_ok(), "non-colliding branch must pass: {:?}", ok.err());
    }

    // @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R9
    // T6b: check_branch_uniqueness is a no-op when .aw/worktrees/ does
    // not exist — guards against regressions in the empty-state fast path.
    #[test]
    fn test_r9_check_branch_uniqueness_no_worktrees_dir_is_ok() {
        let tmp = setup_project();
        // setup_project creates .aw/changes but NOT .aw/worktrees.
        assert!(!tmp.path().join(".aw/worktrees").exists());
        let ok = check_branch_uniqueness(tmp.path(), "cclab/anything", "new-change");
        assert!(
            ok.is_ok(),
            "empty worktree tree must not error: {:?}",
            ok.err()
        );
    }
}
