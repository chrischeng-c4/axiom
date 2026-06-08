// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/merge_git_ops/types-and-post-archive.md#source
// CODEGEN-BEGIN
//! Post-archive git operations for the change-merge workflow.
//!
//! After `create_change_merge` archives a change, these functions handle:
//! - Auto-commit of dirty `cclab/` paths (in the worktree branch)
//! - **Merge worktree branch back to main** (`git merge cclab/<slug> --no-ff`)
//! - **Cleanup** (`git worktree remove` + `git branch -d`)
//! - Auto-PR creation via `gh` CLI
//! - Git/gh binary detection
//! - Commit message and PR body construction
//!
//! The 5-step merge sequence (see `projects/agentic-workflow/tech-design/core/tools/merge_git_ops/types-and-post-archive.md`):
//! 1. SDD archive (handled upstream in `create_change_merge.rs`)
//! 2. Auto-commit in worktree branch — warning on failure
//! 5. `gh pr create` (when auto_pr=true and worktree exists) — **before** local merge so branch is still alive — warning on failure
//! 3. `git merge cclab/<slug>` into main — **hard error** on conflict; merge commit SHA captured after success
//! 4. `git worktree remove` + `git branch -d` — warning on failure

use crate::models::change::RepoPlatformConfig;
use crate::Result;
use serde_json::Value;
use std::path::{Path, PathBuf};

// ─── Git Operations Result ───────────────────────────────────────────────────

/// Result of post-archive git operations (auto-commit, auto-PR).
#[derive(Debug)]
pub(super) struct GitOpsResult {
    pub git_commit_sha: Option<String>,
    pub pr_url: Option<String>,
    pub git_warning: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/tools/merge_git_ops/types-and-post-archive.md#source
impl GitOpsResult {
    /// No-op result: no git operations performed.
    fn noop() -> Self {
        Self {
            git_commit_sha: None,
            pr_url: None,
            git_warning: None,
        }
    }

    /// Warning result: git operations skipped with a reason.
    fn warning(msg: impl Into<String>) -> Self {
        Self {
            git_commit_sha: None,
            pr_url: None,
            git_warning: Some(msg.into()),
        }
    }
}

// ─── Post-Archive Git Operations ─────────────────────────────────────────────

/// Execute the post-archive 5-step merge sequence.
///
/// Called after archive move (`fs::rename`). The change dir no longer exists at
/// `.aw/changes/{id}` — only the archive path is valid.
///
/// Steps (all relative to `project_root`):
/// 1. SDD archive — handled by the caller before invoking this.
/// 2. **Auto-commit** dirty `cclab/` AND `.aw/` paths in the worktree branch
///    (warning on failure). `.aw/` coverage is what picks up the archive
///    moves (`.aw/changes/<id>/` → `.aw/archive/<date>-<id>/`) and the
///    spec writes (`.aw/tech-design/...`) that the caller produced in step 1.
///    REQ: bug-create-change-merge-archive-moves-not-committed-sp
/// 3. **Merge** `cclab/<change_id>` into `config.default_branch`. Only runs
///    when a worktree exists at `.aw/worktrees/<change_id>`. Hard error on
///    conflict — merge is aborted and the user must resolve manually.
/// 4. **Cleanup** — `git worktree remove` + `git branch -d cclab/<change_id>`.
///    Warning on failure.
/// 5. **Auto-PR** via `gh pr create` (warning on failure).
///
/// # Errors
/// Returns `Err` only on step-3 merge conflicts. All other failures are
/// captured as warnings inside `GitOpsResult`.
// REQ: worktree-per-change — step 3/4 (merge + cleanup) is the new behavior
pub(super) fn post_archive_git_ops(
    project_root: &Path,
    change_id: &str,
    archive_path: &Path,
    repo_platform: Option<&RepoPlatformConfig>,
    merged_specs: &[Value],
) -> Result<GitOpsResult> {
    let config = match repo_platform {
        Some(c) => c,
        None => return Ok(GitOpsResult::noop()),
    };

    // Check auto_pr without auto_commit — warn but don't fail
    if config.auto_pr && !config.auto_commit {
        return Ok(GitOpsResult::warning(
            "auto_pr requires auto_commit — skipping PR creation",
        ));
    }

    if !config.auto_commit {
        return Ok(GitOpsResult::noop());
    }

    // Find git binary
    let git_bin = match find_git_binary() {
        Some(g) => g,
        None => {
            return Ok(GitOpsResult::warning(
                "git binary not found, skipping auto-commit",
            ))
        }
    };

    // ── Step 2: Auto-commit in the worktree branch ─────────────────────────
    //
    // If a worktree is active for this change, run the status/add/commit inside
    // the worktree dir so the commit lands on `cclab/<slug>`. If not, fall back
    // to project_root (legacy in-place flow).
    let worktree_dir = resolve_worktree_dir(project_root, change_id);
    let commit_cwd: &Path = worktree_dir.as_deref().unwrap_or(project_root);

    // REQ: bug-create-change-merge-archive-moves-not-committed-sp
    // Include `.aw/` so archive moves (changes → archive) and spec writes
    // (tech_design) produced by the caller in step 1 get staged too.
    // Previously only `cclab/` was queried, so the archive moves were left
    // dirty on disk and required manual `git add .aw/ && git commit`
    // cleanup after every merge.
    let status_output = match std::process::Command::new(&git_bin)
        .args(["status", "--porcelain", "--", "cclab/", ".aw/"])
        .current_dir(commit_cwd)
        .output()
    {
        Ok(o) => o,
        Err(e) => return Ok(GitOpsResult::warning(format!("git status failed: {}", e))),
    };

    let status_str = String::from_utf8_lossy(&status_output.stdout);
    let has_dirty = status_str.lines().any(|line| !line.is_empty());

    let mut commit_sha: Option<String> = None;

    if has_dirty {
        // Stage all changes under cclab/ and .aw/ atomically.
        // Use separate `git add` calls per pathspec so a missing directory
        // (e.g. no cclab/ in a spec-only change) doesn't block the other.
        // REQ: bug-create-change-merge-archive-moves-not-committed-sp (defect 1)
        for pathspec in &["cclab/", ".aw/"] {
            if !commit_cwd.join(pathspec).exists() {
                continue;
            }
            match std::process::Command::new(&git_bin)
                .args(["add", "--all", "--", pathspec])
                .current_dir(commit_cwd)
                .output()
            {
                Ok(output) => {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Ok(GitOpsResult::warning(format!(
                            "git add {} failed: {}",
                            pathspec,
                            stderr.trim()
                        )));
                    }
                }
                Err(e) => {
                    return Ok(GitOpsResult::warning(format!(
                        "git add {} failed: {}",
                        pathspec, e
                    )))
                }
            }
        }

        // Build commit message
        let summary = read_user_input_summary(archive_path);
        let commit_msg = build_commit_message(change_id, summary.as_deref());

        // Run git commit
        let commit_output = match std::process::Command::new(&git_bin)
            .args(["commit", "-m", &commit_msg])
            .current_dir(commit_cwd)
            .output()
        {
            Ok(o) => o,
            Err(e) => return Ok(GitOpsResult::warning(format!("git commit failed: {}", e))),
        };

        if !commit_output.status.success() {
            let stderr = String::from_utf8_lossy(&commit_output.stderr);
            return Ok(GitOpsResult::warning(format!(
                "git commit failed: {}",
                stderr.trim()
            )));
        }

        // Extract SHA from commit output
        let commit_stdout = String::from_utf8_lossy(&commit_output.stdout);
        commit_sha = extract_commit_sha(&commit_stdout, &git_bin, commit_cwd);
    }

    // ── Step 5 (early): Auto-PR before local merge ─────────────────────────
    //
    // When auto_pr=true and a worktree exists, create the PR BEFORE local
    // merge steps 3+4. The PR targets the worktree branch which is still
    // alive at this point. In legacy in-place flow (no worktree), auto-PR
    // is skipped since there is no separate branch to PR from.
    // @spec projects/agentic-workflow/tech-design/core/logic/merge-gaps-fix.md#R2
    let mut step_warnings: Vec<String> = Vec::new();
    let (pr_url, pr_warning) = if config.auto_pr && worktree_dir.is_some() {
        match create_pr(project_root, change_id, archive_path, merged_specs, config) {
            Ok(url) => (Some(url), None),
            Err(warning) => (None, Some(warning)),
        }
    } else {
        (None, None)
    };
    if let Some(pw) = pr_warning {
        step_warnings.push(pw);
    }

    // ── Step 3: Merge worktree branch into default branch ──────────────────
    //
    // Only runs when a worktree exists. Fail-fast on merge conflict.
    if worktree_dir.is_some() {
        match merge_worktree_branch(&git_bin, project_root, change_id, &config.default_branch) {
            MergeOutcome::Ok => {
                // @spec projects/agentic-workflow/tech-design/core/logic/merge-gaps-fix.md#R1
                // After a successful merge, capture the merge commit SHA from the
                // default branch HEAD (project_root). This is the canonical SHA —
                // the commit that actually landed on the default branch.
                if let Some(sha) = capture_head_sha(&git_bin, project_root) {
                    commit_sha = Some(sha);
                }
            }
            MergeOutcome::Warning(w) => step_warnings.push(w),
            MergeOutcome::Conflict(msg) => {
                anyhow::bail!(
                    "git merge conflict on cclab/{slug}: {msg}. \
                     Resolve manually: cd {root} && git merge cclab/{slug} --no-ff",
                    slug = change_id,
                    msg = msg,
                    root = project_root.display()
                );
            }
        }

        // ── Step 4: Cleanup worktree + delete branch ───────────────────────
        if let Err(w) = cleanup_merged_worktree(&git_bin, project_root, change_id) {
            step_warnings.push(w);
        }
    }

    // Combine all warnings.
    let combined_warning = if step_warnings.is_empty() {
        None
    } else {
        Some(step_warnings.join(" | "))
    };

    Ok(GitOpsResult {
        git_commit_sha: commit_sha,
        pr_url,
        git_warning: combined_warning,
    })
}
// CODEGEN-END

// ─── Worktree Merge & Cleanup (Steps 3 & 4) ──────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/merge_git_ops/worktree-merge-cleanup.md#source
// CODEGEN-BEGIN
// ─── Worktree Merge & Cleanup (Steps 3 & 4) ──────────────────────────────────

/// Returns the absolute path to the worktree directory for a change, if it
/// exists on disk. Used to detect whether we're in worktree mode or legacy
/// in-place mode.
pub(super) fn resolve_worktree_dir(project_root: &Path, change_id: &str) -> Option<PathBuf> {
    let wt = project_root.join(".aw/worktrees").join(change_id);
    if wt.is_dir() {
        Some(wt)
    } else {
        None
    }
}

/// Outcome of `git merge cclab/<slug>` in step 3 of the merge sequence.
#[derive(Debug)]
enum MergeOutcome {
    /// Merge landed cleanly on the default branch.
    Ok,
    /// Non-fatal failure — the merge could not proceed but the change can still
    /// archive (e.g. git checkout main failed, detached HEAD, nothing to merge).
    Warning(String),
    /// **Hard failure** — merge conflicts. Caller must surface this to the user.
    Conflict(String),
}

/// Execute step 3: `git checkout <default_branch>` then
/// `git merge --no-ff cclab/<slug>` from `project_root`.
///
/// Uses `git -C <project_root>` so the checkout/merge happen on the main
/// workspace (not the worktree).
// REQ: worktree-per-change — step 3 merge
fn merge_worktree_branch(
    git_bin: &Path,
    project_root: &Path,
    change_id: &str,
    default_branch: &str,
) -> MergeOutcome {
    // Checkout the default branch first.
    let checkout = match std::process::Command::new(git_bin)
        .args(["checkout", default_branch])
        .current_dir(project_root)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return MergeOutcome::Warning(format!("git checkout {} failed: {}", default_branch, e))
        }
    };
    if !checkout.status.success() {
        let stderr = String::from_utf8_lossy(&checkout.stderr);
        return MergeOutcome::Warning(format!(
            "git checkout {} failed: {}",
            default_branch,
            stderr.trim()
        ));
    }

    // Run `git merge --no-ff cclab/<slug>`.
    let branch = format!("cclab/{}", change_id);
    let merge = match std::process::Command::new(git_bin)
        .args(["merge", "--no-ff", &branch])
        .current_dir(project_root)
        .output()
    {
        Ok(o) => o,
        Err(e) => return MergeOutcome::Warning(format!("git merge {} failed: {}", branch, e)),
    };

    if merge.status.success() {
        return MergeOutcome::Ok;
    }

    // Distinguish conflict vs other failure via stderr/stdout content.
    let stdout = String::from_utf8_lossy(&merge.stdout);
    let stderr = String::from_utf8_lossy(&merge.stderr);
    let combined = format!("{}{}", stdout, stderr);
    let is_conflict = combined.to_lowercase().contains("conflict")
        || combined.contains("Automatic merge failed")
        || combined.contains("CONFLICT (");

    if is_conflict {
        // Best-effort abort so the working tree is clean again.
        let _ = std::process::Command::new(git_bin)
            .args(["merge", "--abort"])
            .current_dir(project_root)
            .output();
        MergeOutcome::Conflict(combined.trim().to_string())
    } else {
        MergeOutcome::Warning(format!("git merge {} failed: {}", branch, combined.trim()))
    }
}

/// Execute step 4: remove the worktree directory and delete the branch.
///
/// Returns `Err(warning_message)` for cleanup failures; caller adds them to the
/// overall `git_warning` field (non-blocking).
// REQ: worktree-per-change — step 4 cleanup
fn cleanup_merged_worktree(
    git_bin: &Path,
    project_root: &Path,
    change_id: &str,
) -> std::result::Result<(), String> {
    let worktree_rel = format!(".aw/worktrees/{}", change_id);
    let branch = format!("cclab/{}", change_id);

    let mut warnings: Vec<String> = Vec::new();

    // Remove worktree (use --force to cover stray untracked files left behind).
    let remove = std::process::Command::new(git_bin)
        .args(["worktree", "remove", "--force", &worktree_rel])
        .current_dir(project_root)
        .output();
    match remove {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
            warnings.push(format!("git worktree remove failed: {}", err));
        }
        Err(e) => warnings.push(format!("git worktree remove failed: {}", e)),
    }

    // Prune so the admin metadata stays tidy regardless of the remove result.
    let _ = std::process::Command::new(git_bin)
        .args(["worktree", "prune"])
        .current_dir(project_root)
        .output();

    // Delete the branch (use -d since the branch has been merged).
    let del = std::process::Command::new(git_bin)
        .args(["branch", "-d", &branch])
        .current_dir(project_root)
        .output();
    match del {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
            warnings.push(format!("git branch -d {} failed: {}", branch, err));
        }
        Err(e) => warnings.push(format!("git branch -d {} failed: {}", branch, e)),
    }

    if warnings.is_empty() {
        Ok(())
    } else {
        Err(warnings.join(" | "))
    }
}
// CODEGEN-END
/// Execute step 3: `git checkout <default_branch>` then
/// `git merge --no-ff cclab/<slug>` from `project_root`.
///
/// Uses `git -C <project_root>` so the checkout/merge happen on the main
/// workspace (not the worktree).
// REQ: worktree-per-change — step 3 merge
/// Execute step 4: remove the worktree directory and delete the branch.
///
/// Returns `Err(warning_message)` for cleanup failures; caller adds them to the
/// overall `git_warning` field (non-blocking).
// REQ: worktree-per-change — step 4 cleanup
// ─── Git Binary Detection ───────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/merge_git_ops/binary-and-pr.md#source
// CODEGEN-BEGIN
// ─── Git Binary Detection ───────────────────────────────────────────────────

/// Locate the `git` binary on PATH.
///
/// Returns `Some(path)` if found, `None` otherwise.
pub(super) fn find_git_binary() -> Option<PathBuf> {
    // Try `which git` on Unix-like systems
    let output = std::process::Command::new("which")
        .arg("git")
        .output()
        .ok()?;
    if output.status.success() {
        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path_str.is_empty() {
            return Some(PathBuf::from(path_str));
        }
    }
    None
}

/// Locate the `gh` CLI binary on PATH.
fn find_gh_binary() -> Option<PathBuf> {
    let output = std::process::Command::new("which")
        .arg("gh")
        .output()
        .ok()?;
    if output.status.success() {
        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path_str.is_empty() {
            return Some(PathBuf::from(path_str));
        }
    }
    None
}

// ─── Auto-PR via gh CLI ─────────────────────────────────────────────────────

/// Create a PR via `gh pr create` targeting the configured default branch.
///
/// Builds the PR body from change context: user_input.md summary and list of
/// merged spec targets. Returns the PR URL on success, or an error string
/// describing why PR creation failed (non-fatal to the merge).
fn create_pr(
    project_root: &Path,
    change_id: &str,
    archive_path: &Path,
    merged_specs: &[Value],
    config: &RepoPlatformConfig,
) -> std::result::Result<String, String> {
    let gh_bin = find_gh_binary()
        .ok_or_else(|| "gh CLI not found, skipping auto-PR creation".to_string())?;

    let title = format!("chore(sdd): merge {}", change_id);
    let body = build_pr_body(change_id, archive_path, merged_specs);

    let output = std::process::Command::new(&gh_bin)
        .args([
            "pr",
            "create",
            "--title",
            &title,
            "--body",
            &body,
            "--base",
            &config.default_branch,
            "--repo",
            &config.repo,
        ])
        .current_dir(project_root)
        .output()
        .map_err(|e| format!("gh pr create failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gh pr create failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let url = stdout.trim().to_string();
    if url.is_empty() {
        Err("gh pr create returned empty output".to_string())
    } else {
        Ok(url)
    }
}
// CODEGEN-END
// ─── Auto-PR via gh CLI ─────────────────────────────────────────────────────

// ─── Commit Message & PR Body ───────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/merge_git_ops/commit-and-pr-body.md#source
// CODEGEN-BEGIN
// ─── Commit Message & PR Body ───────────────────────────────────────────────

/// Read the first line of `user_input.md` from the archive directory.
/// Returns `None` if file doesn't exist or is empty.
fn read_user_input_summary(archive_path: &Path) -> Option<String> {
    let user_input_path = archive_path.join("user_input.md");
    let content = std::fs::read_to_string(&user_input_path).ok()?;
    let first_line = content.lines().find(|l| !l.trim().is_empty())?;
    let trimmed = first_line.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Build conventional commit message for SDD merge.
///
/// Format: `chore(sdd): merge {change_id} — {summary}`
/// Summary is truncated to 72 chars. If missing, just `chore(sdd): merge {change_id}`.
fn build_commit_message(change_id: &str, summary: Option<&str>) -> String {
    match summary {
        Some(desc) => {
            let truncated: String = desc.chars().take(72).collect();
            format!("chore(sdd): merge {} — {}", change_id, truncated)
        }
        None => format!("chore(sdd): merge {}", change_id),
    }
}

/// Capture the current HEAD SHA by running `git rev-parse HEAD` in `dir`.
///
/// Used after step 3 (worktree merge into default branch) to record the merge
/// commit SHA on the default branch.
// @spec projects/agentic-workflow/tech-design/core/logic/merge-gaps-fix.md#R1
fn capture_head_sha(git_bin: &Path, dir: &Path) -> Option<String> {
    let output = std::process::Command::new(git_bin)
        .args(["rev-parse", "HEAD"])
        .current_dir(dir)
        .output()
        .ok()?;
    if output.status.success() {
        let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !sha.is_empty() {
            return Some(sha);
        }
    }
    None
}

/// Extract the commit SHA from git output or by running `git rev-parse HEAD`.
fn extract_commit_sha(commit_stdout: &str, git_bin: &Path, project_root: &Path) -> Option<String> {
    // Try to parse SHA from commit output (format varies by git version)
    // Common: "[branch abc1234] commit message"
    for word in commit_stdout.split_whitespace() {
        let cleaned = word.trim_matches(|c: char| !c.is_ascii_hexdigit());
        if cleaned.len() >= 7 && cleaned.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(cleaned.to_string());
        }
    }

    // Fallback: git rev-parse HEAD
    let output = std::process::Command::new(git_bin)
        .args(["rev-parse", "HEAD"])
        .current_dir(project_root)
        .output()
        .ok()?;

    if output.status.success() {
        let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !sha.is_empty() {
            return Some(sha);
        }
    }

    None
}

/// Build a PR body from change context.
///
/// Includes the change description from `user_input.md` and a list of merged
/// spec targets. Used by `create_pr()` for the `--body` argument.
fn build_pr_body(change_id: &str, archive_path: &Path, merged_specs: &[Value]) -> String {
    let mut body = format!("## SDD Merge: {}\n\n", change_id);

    // Add user input summary
    if let Some(summary) = read_user_input_summary(archive_path) {
        body.push_str(&format!("**Description:** {}\n\n", summary));
    }

    // Add merged specs list
    if !merged_specs.is_empty() {
        body.push_str("### Merged Specs\n\n");
        for spec in merged_specs {
            if let Some(target) = spec.get("target").and_then(|v| v.as_str()) {
                body.push_str(&format!("- `{}`\n", target));
            }
        }
        body.push('\n');
    }

    body.push_str("---\n*Auto-generated by SDD merge workflow*\n");
    body
}
// CODEGEN-END
/// Capture the current HEAD SHA by running `git rev-parse HEAD` in `dir`.
///
/// Used after step 3 (worktree merge into default branch) to record the merge
/// commit SHA on the default branch.
// @spec projects/agentic-workflow/tech-design/core/logic/merge-gaps-fix.md#R1
// ─── Tests ──────────────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/merge_git_ops/tests.md#source
// CODEGEN-BEGIN
// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn test_build_commit_message_with_summary() {
        let msg = build_commit_message("1136", Some("feat(sdd): platform config restructure"));
        assert_eq!(
            msg,
            "chore(sdd): merge 1136 — feat(sdd): platform config restructure"
        );
    }

    #[test]
    fn test_build_commit_message_without_summary() {
        let msg = build_commit_message("42", None);
        assert_eq!(msg, "chore(sdd): merge 42");
    }

    #[test]
    fn test_build_commit_message_truncates_at_72_chars() {
        let long_summary = "a".repeat(80);
        let msg = build_commit_message("100", Some(&long_summary));
        let expected = format!("chore(sdd): merge 100 — {}", "a".repeat(72));
        assert_eq!(msg, expected);
    }

    #[test]
    fn test_read_user_input_summary_with_content() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("user_input.md"),
            "My change description\nMore detail",
        )
        .unwrap();
        let result = read_user_input_summary(tmp.path());
        assert_eq!(result, Some("My change description".to_string()));
    }

    #[test]
    fn test_read_user_input_summary_missing_file() {
        let tmp = TempDir::new().unwrap();
        let result = read_user_input_summary(tmp.path());
        assert_eq!(result, None);
    }

    #[test]
    fn test_read_user_input_summary_empty_file() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("user_input.md"), "").unwrap();
        let result = read_user_input_summary(tmp.path());
        assert_eq!(result, None);
    }

    #[test]
    fn test_read_user_input_summary_blank_lines_then_content() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("user_input.md"), "\n\n  \nActual content\n").unwrap();
        let result = read_user_input_summary(tmp.path());
        assert_eq!(result, Some("Actual content".to_string()));
    }

    #[test]
    fn test_extract_commit_sha_from_typical_output() {
        let output = "[main abc1234f] chore(sdd): merge 42";
        let sha = extract_commit_sha(output, Path::new("/nonexistent"), Path::new("/nonexistent"));
        assert!(sha.is_some());
        let sha_val = sha.unwrap();
        assert!(sha_val.len() >= 7);
        assert!(sha_val.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_extract_commit_sha_from_detached_head() {
        let output = "[detached HEAD a1b2c3d] chore(sdd): merge 99";
        let sha = extract_commit_sha(output, Path::new("/nonexistent"), Path::new("/nonexistent"));
        assert!(sha.is_some());
        assert_eq!(sha.unwrap(), "a1b2c3d");
    }

    #[test]
    fn test_build_pr_body_with_specs() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("user_input.md"), "Add new feature").unwrap();

        let specs = vec![
            json!({"spec_id": "auth-flow", "target": ".aw/tech-design/sdd/logic/auth-flow.md"}),
            json!({"spec_id": "config", "target": ".aw/tech-design/sdd/config/config.md"}),
        ];
        let body = build_pr_body("1136", tmp.path(), &specs);
        assert!(body.contains("SDD Merge: 1136"));
        assert!(body.contains("Add new feature"));
        assert!(body.contains(".aw/tech-design/sdd/logic/auth-flow.md"));
        assert!(body.contains(".aw/tech-design/sdd/config/config.md"));
        assert!(body.contains("Merged Specs"));
    }

    #[test]
    fn test_build_pr_body_no_specs_no_input() {
        let tmp = TempDir::new().unwrap();
        let body = build_pr_body("42", tmp.path(), &[]);
        assert!(body.contains("SDD Merge: 42"));
        assert!(!body.contains("Merged Specs"));
        assert!(!body.contains("Description"));
    }

    #[test]
    fn test_repo_platform_config_deserialization() {
        use crate::models::change::RepoPlatformConfig;

        let toml_content = r#"
type = "github"
repo = "owner/repo"
default_branch = "main"
auto_commit = true
auto_pr = false
"#;
        let rp: RepoPlatformConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(rp.type_, "github");
        assert_eq!(rp.repo, "owner/repo");
        assert_eq!(rp.default_branch, "main");
        assert!(rp.auto_commit);
        assert!(!rp.auto_pr);
    }

    #[test]
    fn test_repo_platform_config_absent_is_none() {
        use crate::models::SddConfig;

        let toml_content = "";
        let config: SddConfig = toml::from_str(toml_content).unwrap();
        assert!(
            config.repo_platform.is_none(),
            "absent repo_platform must be None"
        );
    }

    #[test]
    fn test_repo_platform_defaults() {
        use crate::models::change::RepoPlatformConfig;

        let toml_content = r#"
type = "github"
repo = "owner/repo"
"#;
        let rp: RepoPlatformConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(
            rp.default_branch, "main",
            "default_branch must default to 'main'"
        );
        assert!(!rp.auto_commit, "auto_commit must default to false");
        assert!(!rp.auto_pr, "auto_pr must default to false");
    }

    #[test]
    fn test_tech_design_platform_config_deserialization() {
        use crate::models::change::TechDesignPlatformConfig;

        let toml_content = r#"
type = "local"
path = ".aw/tech-design"
"#;
        let sp: TechDesignPlatformConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(sp.type_, "local");
        assert_eq!(sp.path, ".aw/tech-design");
    }

    #[test]
    fn test_sdd_config_load_with_repo_platform() {
        use crate::models::SddConfig;

        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join(".aw");
        std::fs::create_dir_all(&config_dir).unwrap();
        let toml_content = r#"
[agentic_workflow.repo_platform]
type = "github"
repo = "owner/repo"
auto_commit = true

[agentic_workflow.tech_design_platform]
type = "local"
path = ".aw/tech-design"
"#;
        std::fs::write(config_dir.join("config.toml"), toml_content).unwrap();

        let config = SddConfig::load(tmp.path()).unwrap();
        let rp = config
            .repo_platform
            .expect("repo_platform must be loaded from [agentic_workflow.repo_platform]");
        assert_eq!(rp.type_, "github");
        assert_eq!(rp.repo, "owner/repo");
        assert!(rp.auto_commit);

        let sp = config.tech_design_platform.expect(
            "tech_design_platform must be loaded from [agentic_workflow.tech_design_platform]",
        );
        assert_eq!(sp.type_, "local");
        assert_eq!(sp.path, ".aw/tech-design");
    }

    #[test]
    fn test_sdd_config_load_without_repo_platform() {
        use crate::models::SddConfig;

        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join(".aw");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(config_dir.join("config.toml"), "").unwrap();

        let config = SddConfig::load(tmp.path()).unwrap();
        assert!(
            config.repo_platform.is_none(),
            "absent [agentic_workflow.repo_platform] must leave None"
        );
        assert!(
            config.tech_design_platform.is_none(),
            "absent [agentic_workflow.tech_design_platform] must leave None"
        );
    }

    // ─── Worktree Merge Sequence Tests ───────────────────────────────────

    /// Initialize a bare git repo with an initial commit. Returns true if git
    /// is available and the repo was initialized.
    fn init_repo(dir: &Path) -> bool {
        let Some(git) = find_git_binary() else {
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
        for (k, v) in [("user.email", "test@example.com"), ("user.name", "Test")] {
            let _ = std::process::Command::new(&git)
                .args(["config", k, v])
                .current_dir(dir)
                .output();
        }
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

    // REQ: worktree-per-change — resolve_worktree_dir detects worktree presence
    #[test]
    fn test_resolve_worktree_dir_none_when_missing() {
        let tmp = TempDir::new().unwrap();
        assert!(resolve_worktree_dir(tmp.path(), "nothing-here").is_none());
    }

    // REQ: worktree-per-change — resolve_worktree_dir returns Some when dir exists
    #[test]
    fn test_resolve_worktree_dir_some_when_present() {
        let tmp = TempDir::new().unwrap();
        let wt = tmp.path().join(".aw/worktrees/my-change");
        std::fs::create_dir_all(&wt).unwrap();
        assert_eq!(resolve_worktree_dir(tmp.path(), "my-change"), Some(wt));
    }

    /// End-to-end test of the 5-step merge sequence: create worktree,
    /// make a commit on the branch, invoke post_archive_git_ops, verify
    /// the branch is merged into main and the worktree is cleaned up.
    // REQ: worktree-per-change — full 5-step sequence happy path
    #[test]
    fn test_post_archive_merges_worktree_and_cleans_up() {
        let tmp = TempDir::new().unwrap();
        if !init_repo(tmp.path()) {
            return;
        }

        let slug = "enhancement-full-merge";
        let Some(git) = find_git_binary() else { return };

        // Create the worktree and a commit on the new branch so there's
        // something for step 3 to merge.
        let wt_rel = format!(".aw/worktrees/{}", slug);
        std::fs::create_dir_all(tmp.path().join(".aw/worktrees")).unwrap();
        let add_out = std::process::Command::new(&git)
            .args(["worktree", "add", "-b", &format!("cclab/{}", slug), &wt_rel])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        assert!(
            add_out.status.success(),
            "git worktree add failed: {}",
            String::from_utf8_lossy(&add_out.stderr)
        );

        // Write a file under cclab/ in the worktree so the status check picks it up.
        let wt_abs = tmp.path().join(&wt_rel);
        std::fs::create_dir_all(wt_abs.join("cclab")).unwrap();
        std::fs::write(wt_abs.join("cclab/change-file.txt"), "worktree content\n").unwrap();

        // Prepare a stub archive path with user_input.md so commit msg builds.
        let archive_abs = tmp
            .path()
            .join(".aw/archive/20260101-enhancement-full-merge");
        std::fs::create_dir_all(&archive_abs).unwrap();
        std::fs::write(
            archive_abs.join("user_input.md"),
            "Add worktree isolation\n",
        )
        .unwrap();

        let config = RepoPlatformConfig {
            type_: "github".to_string(),
            repo: "test/repo".to_string(),
            host: None,
            default_branch: "main".to_string(),
            auto_commit: true,
            auto_pr: false,
        };

        let result = post_archive_git_ops(tmp.path(), slug, &archive_abs, Some(&config), &[]);
        assert!(
            result.is_ok(),
            "post_archive_git_ops failed: {:?}",
            result.err()
        );
        let ops = result.unwrap();
        assert!(ops.git_commit_sha.is_some(), "expected a commit SHA");

        // Verify the worktree directory is gone
        assert!(
            !wt_abs.exists(),
            "worktree directory should have been removed"
        );

        // Verify the branch was deleted
        let branch_check = std::process::Command::new(&git)
            .args([
                "show-ref",
                "--verify",
                "--quiet",
                &format!("refs/heads/cclab/{}", slug),
            ])
            .current_dir(tmp.path())
            .status()
            .unwrap();
        assert!(
            !branch_check.success(),
            "branch cclab/{} should have been deleted",
            slug
        );

        // Verify we're on main and the change file landed here
        let branch = std::process::Command::new(&git)
            .args(["branch", "--show-current"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&branch.stdout).trim(),
            "main",
            "should be on main branch after merge"
        );
        assert!(
            tmp.path().join("cclab/change-file.txt").exists(),
            "change file should be present on main after merge"
        );
    }

    /// Simulate a merge conflict: create a worktree, commit file X with content
    /// A on the branch, and commit the same file with content B on main.
    /// `post_archive_git_ops` must return an Err with a message mentioning
    /// conflict and manual resolution.
    // REQ: worktree-per-change — step 3 fails fast on conflict
    #[test]
    fn test_post_archive_fails_fast_on_merge_conflict() {
        let tmp = TempDir::new().unwrap();
        if !init_repo(tmp.path()) {
            return;
        }

        let slug = "enhancement-conflict";
        let Some(git) = find_git_binary() else { return };

        // Create a file on main first so both branches diverge from a common base
        std::fs::create_dir_all(tmp.path().join("cclab")).unwrap();
        std::fs::write(tmp.path().join("cclab/contested.txt"), "base\n").unwrap();
        std::process::Command::new(&git)
            .args(["add", "cclab/contested.txt"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        std::process::Command::new(&git)
            .args(["commit", "-m", "base"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        // Now create the worktree branch from current main
        let wt_rel = format!(".aw/worktrees/{}", slug);
        std::fs::create_dir_all(tmp.path().join(".aw/worktrees")).unwrap();
        std::process::Command::new(&git)
            .args(["worktree", "add", "-b", &format!("cclab/{}", slug), &wt_rel])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        // Modify contested.txt on main (uncommitted won't diverge — need a commit)
        std::fs::write(tmp.path().join("cclab/contested.txt"), "main version\n").unwrap();
        std::process::Command::new(&git)
            .args(["add", "cclab/contested.txt"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        std::process::Command::new(&git)
            .args(["commit", "-m", "main change"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        // Modify contested.txt inside the worktree (different content)
        let wt_abs = tmp.path().join(&wt_rel);
        std::fs::write(wt_abs.join("cclab/contested.txt"), "branch version\n").unwrap();

        // Archive stub
        let archive_abs = tmp.path().join(".aw/archive/20260101-enhancement-conflict");
        std::fs::create_dir_all(&archive_abs).unwrap();
        std::fs::write(archive_abs.join("user_input.md"), "conflict test\n").unwrap();

        let config = RepoPlatformConfig {
            type_: "github".to_string(),
            repo: "test/repo".to_string(),
            host: None,
            default_branch: "main".to_string(),
            auto_commit: true,
            auto_pr: false,
        };

        let result = post_archive_git_ops(tmp.path(), slug, &archive_abs, Some(&config), &[]);
        assert!(
            result.is_err(),
            "expected hard error on merge conflict, got {:?}",
            result.ok().map(|r| r.git_warning)
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.to_lowercase().contains("conflict"),
            "error must mention conflict: {}",
            err_msg
        );
        assert!(
            err_msg.contains("Resolve manually") || err_msg.contains("resolve"),
            "error must suggest manual resolution: {}",
            err_msg
        );
    }

    /// When no worktree exists, post_archive_git_ops falls back to the legacy
    /// flow (steps 2 + 5 only) and does not error. The archive file under
    /// `.aw/archive/` is treated as a dirty path and committed.
    ///
    /// REQ: bug-create-change-merge-archive-moves-not-committed-sp
    /// Before the fix, the status pathspec was `-- cclab/` only, so the
    /// archive file was NOT seen as dirty and no commit was made
    /// (git_commit_sha was None). After the fix, the pathspec is
    /// `-- cclab/ .aw/`, so the archive file IS staged and committed.
    // REQ: worktree-per-change — backward compat legacy in-place changes
    #[test]
    fn test_post_archive_legacy_no_worktree() {
        let tmp = TempDir::new().unwrap();
        if !init_repo(tmp.path()) {
            return;
        }

        // Legacy flow: no worktree, one archive file under .aw/archive/.
        let archive_abs = tmp.path().join(".aw/archive/20260101-legacy");
        std::fs::create_dir_all(&archive_abs).unwrap();
        std::fs::write(archive_abs.join("user_input.md"), "legacy change\n").unwrap();

        let config = RepoPlatformConfig {
            type_: "github".to_string(),
            repo: "test/repo".to_string(),
            host: None,
            default_branch: "main".to_string(),
            auto_commit: true,
            auto_pr: false,
        };

        let result = post_archive_git_ops(
            tmp.path(),
            "legacy-change",
            &archive_abs,
            Some(&config),
            &[],
        );
        assert!(
            result.is_ok(),
            "legacy path should not error: {:?}",
            result.err()
        );
        let ops = result.unwrap();
        // The dirty .aw/ path is committed — SHA should be populated.
        assert!(
            ops.git_commit_sha.is_some(),
            "expected auto-commit of .aw/ archive path; got None"
        );
        assert!(
            ops.git_warning.is_none()
                || !ops
                    .git_warning
                    .as_deref()
                    .unwrap_or("")
                    .contains("conflict")
        );
    }

    /// Fix 1: After step 3 merge, git_commit_sha must be the merge commit SHA
    /// on project_root (not the worktree branch commit), and must not be null.
    // @spec projects/agentic-workflow/tech-design/core/logic/merge-gaps-fix.md#R1
    #[test]
    fn test_commit_sha_is_merge_commit_on_default_branch() {
        let tmp = TempDir::new().unwrap();
        if !init_repo(tmp.path()) {
            return;
        }

        let slug = "fix1-sha-capture";
        let Some(git) = find_git_binary() else { return };

        // Create a worktree branch and add a file to it.
        let wt_rel = format!(".aw/worktrees/{}", slug);
        std::fs::create_dir_all(tmp.path().join(".aw/worktrees")).unwrap();
        let add_wt = std::process::Command::new(&git)
            .args(["worktree", "add", "-b", &format!("cclab/{}", slug), &wt_rel])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        assert!(
            add_wt.status.success(),
            "git worktree add failed: {}",
            String::from_utf8_lossy(&add_wt.stderr)
        );

        let wt_abs = tmp.path().join(&wt_rel);
        std::fs::create_dir_all(wt_abs.join("cclab")).unwrap();
        std::fs::write(wt_abs.join("cclab/test-file.txt"), "sha test\n").unwrap();

        let archive_abs = tmp.path().join(".aw/archive/20260101-fix1-sha");
        std::fs::create_dir_all(&archive_abs).unwrap();
        std::fs::write(archive_abs.join("user_input.md"), "SHA capture fix\n").unwrap();

        let config = RepoPlatformConfig {
            type_: "github".to_string(),
            repo: "test/repo".to_string(),
            host: None,
            default_branch: "main".to_string(),
            auto_commit: true,
            auto_pr: false,
        };

        let result = post_archive_git_ops(tmp.path(), slug, &archive_abs, Some(&config), &[]);
        assert!(
            result.is_ok(),
            "post_archive_git_ops failed: {:?}",
            result.err()
        );
        let ops = result.unwrap();

        // Must have a SHA — the merge commit on main.
        assert!(
            ops.git_commit_sha.is_some(),
            "git_commit_sha must not be null after merge"
        );
        let sha = ops.git_commit_sha.unwrap();
        assert!(sha.len() >= 7, "SHA must be at least 7 chars, got: {}", sha);
        assert!(
            sha.chars().all(|c| c.is_ascii_hexdigit()),
            "SHA must be hex, got: {}",
            sha
        );

        // Verify the SHA matches HEAD on project_root (main branch after merge).
        let head_output = std::process::Command::new(&git)
            .args(["rev-parse", "HEAD"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let head_sha = String::from_utf8_lossy(&head_output.stdout)
            .trim()
            .to_string();
        assert!(
            head_sha.starts_with(&sha)
                || sha.starts_with(&head_sha[..sha.len().min(head_sha.len())]),
            "reported SHA {} must match or be prefix of HEAD SHA {}",
            sha,
            head_sha
        );
    }

    /// Fix 2: auto_pr=true in legacy in-place flow (no worktree) must NOT attempt PR creation.
    // @spec projects/agentic-workflow/tech-design/core/logic/merge-gaps-fix.md#R2
    #[test]
    fn test_auto_pr_skipped_in_legacy_flow() {
        let tmp = TempDir::new().unwrap();
        if !init_repo(tmp.path()) {
            return;
        }

        // No worktree created — legacy in-place flow.
        let archive_abs = tmp.path().join(".aw/archive/20260101-legacy-pr");
        std::fs::create_dir_all(&archive_abs).unwrap();
        std::fs::write(archive_abs.join("user_input.md"), "legacy auto-pr test\n").unwrap();

        let config = RepoPlatformConfig {
            type_: "github".to_string(),
            repo: "test/repo".to_string(),
            host: None,
            default_branch: "main".to_string(),
            auto_commit: true,
            auto_pr: true, // enabled, but no worktree → must be skipped
        };

        let result = post_archive_git_ops(
            tmp.path(),
            "legacy-pr-change",
            &archive_abs,
            Some(&config),
            &[],
        );
        assert!(
            result.is_ok(),
            "should not error in legacy flow: {:?}",
            result.err()
        );
        let ops = result.unwrap();
        // PR URL must be None (no worktree → PR skipped).
        assert!(
            ops.pr_url.is_none(),
            "pr_url must be None in legacy flow when no worktree exists; got: {:?}",
            ops.pr_url
        );
    }

    /// Fix 2: auto_pr=false with a worktree must produce no PR URL and no PR warning.
    // @spec projects/agentic-workflow/tech-design/core/logic/merge-gaps-fix.md#R2
    #[test]
    fn test_auto_pr_disabled_produces_no_pr_url() {
        let tmp = TempDir::new().unwrap();
        if !init_repo(tmp.path()) {
            return;
        }

        let slug = "fix2-no-pr";
        let Some(git) = find_git_binary() else { return };

        let wt_rel = format!(".aw/worktrees/{}", slug);
        std::fs::create_dir_all(tmp.path().join(".aw/worktrees")).unwrap();
        std::process::Command::new(&git)
            .args(["worktree", "add", "-b", &format!("cclab/{}", slug), &wt_rel])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        let wt_abs = tmp.path().join(&wt_rel);
        std::fs::create_dir_all(wt_abs.join("cclab")).unwrap();
        std::fs::write(wt_abs.join("cclab/test.txt"), "content\n").unwrap();

        let archive_abs = tmp.path().join(".aw/archive/20260101-fix2-no-pr");
        std::fs::create_dir_all(&archive_abs).unwrap();
        std::fs::write(archive_abs.join("user_input.md"), "auto-pr disabled test\n").unwrap();

        let config = RepoPlatformConfig {
            type_: "github".to_string(),
            repo: "test/repo".to_string(),
            host: None,
            default_branch: "main".to_string(),
            auto_commit: true,
            auto_pr: false,
        };

        let result = post_archive_git_ops(tmp.path(), slug, &archive_abs, Some(&config), &[]);
        assert!(result.is_ok(), "should not error: {:?}", result.err());
        let ops = result.unwrap();
        assert!(
            ops.pr_url.is_none(),
            "pr_url must be None when auto_pr=false"
        );
    }

    /// Fix 3: capture_head_sha returns the current HEAD SHA for a given dir.
    // @spec projects/agentic-workflow/tech-design/core/logic/merge-gaps-fix.md#R1
    #[test]
    fn test_capture_head_sha_returns_valid_sha() {
        let tmp = TempDir::new().unwrap();
        if !init_repo(tmp.path()) {
            return;
        }
        let Some(git) = find_git_binary() else { return };
        let sha = capture_head_sha(&git, tmp.path());
        assert!(
            sha.is_some(),
            "capture_head_sha must return Some for a valid repo"
        );
        let sha_str = sha.unwrap();
        assert_eq!(
            sha_str.len(),
            40,
            "SHA must be 40 hex chars, got len={}",
            sha_str.len()
        );
        assert!(
            sha_str.chars().all(|c| c.is_ascii_hexdigit()),
            "SHA must be hex, got: {}",
            sha_str
        );
    }
}
// CODEGEN-END
