---
id: sdd-tools-merge-git-ops-types-and-post-archive
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools merge git ops types and post archive

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/merge_git_ops.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `GitOpsResult` | projects/agentic-workflow/src/tools/merge_git_ops.rs | struct | pub | 29 |  |
| `find_git_binary` | projects/agentic-workflow/src/tools/merge_git_ops.rs | function | pub | 442 | find_git_binary() -> Option<PathBuf> |
| `post_archive_git_ops` | projects/agentic-workflow/src/tools/merge_git_ops.rs | function | pub | 81 | post_archive_git_ops(     project_root: &Path,     change_id: &str,     archive_path: &Path,     repo_platform: Option<&RepoPlatformConfig>,     merged_specs: &[Value], ) -> Result<GitOpsResult> |
| `resolve_worktree_dir` | projects/agentic-workflow/src/tools/merge_git_ops.rs | function | pub | 278 | resolve_worktree_dir(project_root: &Path, change_id: &str) -> Option<PathBuf> |
## Source
<!-- type: source lang: rust -->

````rust
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/merge_git_ops.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
      - "GitOpsResult"
      - "impl GitOpsResult"
      - "post_archive_git_ops"
    description: "Module preamble, git operation result type, and post-archive merge orchestration."
```
