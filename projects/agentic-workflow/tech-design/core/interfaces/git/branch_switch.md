---
id: projects-sdd-src-branch-switch-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Branch switching is part of TD/CB lifecycle automation because lifecycle commands create, reuse, and merge short-lived implementation branches."
---

# Standardized projects/agentic-workflow/src/branch_switch.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/branch_switch.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DirtyTreeError` | projects/agentic-workflow/src/branch_switch.rs | struct | pub | 38 |  |
| `MergeOutcome` | projects/agentic-workflow/src/branch_switch.rs | enum | pub | 25 |  |
| `branch_exists_local` | projects/agentic-workflow/src/branch_switch.rs | function | pub | 88 | branch_exists_local(repo_root: &Path, branch: &str) -> Result<bool> |
| `current_branch` | projects/agentic-workflow/src/branch_switch.rs | function | pub | 104 | current_branch(repo_root: &Path) -> Result<String> |
| `delete_local_branch` | projects/agentic-workflow/src/branch_switch.rs | function | pub | 199 | delete_local_branch(repo_root: &Path, branch: &str) -> Result<()> |
| `ensure_branch_clean` | projects/agentic-workflow/src/branch_switch.rs | function | pub | 61 | ensure_branch_clean(repo_root: &Path) -> Result<()> |
| `merge_branch_into_default` | projects/agentic-workflow/src/branch_switch.rs | function | pub | 152 | merge_branch_into_default(     repo_root: &Path,     source: &str,     default: &str, ) -> Result<MergeOutcome> |
| `switch_or_create_branch` | projects/agentic-workflow/src/branch_switch.rs | function | pub | 122 | switch_or_create_branch(repo_root: &Path, target: &str, fork_point: &str) -> Result<()> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/branch_switch.rs -->
```rust
//! Phase C: in-place branch-switch helpers.
//!
//! @spec projects/agentic-workflow/tech-design/core/worktree-retirement.md#schema
//!
//! Replaces the worktree-creation contract from `worktree.rs`. Phase C
//! lifecycle work (`aw wi`, `aw td`, `aw cb`) runs on the
//! host repo's working tree via `git switch -c <branch>`, NOT inside
//! `.aw/worktrees/<kind>-<slug>/`.
//!
//! The legacy `worktree::provision` API is preserved for one release so
//! `score migrate-worktrees` can still walk existing worktree dirs and
//! lift their commits onto host-repo branches.

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::git::find_git_bin;

/// Outcome of `merge_branch_into_default`.
#[derive(Debug, Clone, PartialEq, Eq)]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/git/branch_switch.md#source
pub enum MergeOutcome {
    /// Merge succeeded with a new merge commit.
    Merged,
    /// Merge succeeded as a fast-forward (no merge commit needed).
    FastForward,
    /// Merge produced conflicts; operator must resolve and commit, then
    /// re-run the verb's `--continue` form.
    Conflict { stderr: String },
}

/// Returned by `ensure_branch_clean` when the working tree has uncommitted changes.
#[derive(Debug, Clone)]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/git/branch_switch.md#source
pub struct DirtyTreeError {
    pub repo_root: String,
    pub porcelain: String,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/git/branch_switch.md#source
impl std::fmt::Display for DirtyTreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "working tree at {} is dirty (uncommitted changes); commit or stash before continuing.\n\
             Run `git status` to inspect; `git stash` to set aside; then re-run the verb.\n\
             porcelain output:\n{}",
            self.repo_root, self.porcelain
        )
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/git/branch_switch.md#source
impl std::error::Error for DirtyTreeError {}

/// Run `git status --porcelain`. Empty output => Ok; non-empty => `DirtyTreeError`.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/git/branch_switch.md#source
pub fn ensure_branch_clean(repo_root: &Path) -> Result<()> {
    let git = find_git_bin().context("git binary not found on PATH")?;
    let output = Command::new(&git)
        .args(["status", "--porcelain"])
        .current_dir(repo_root)
        .output()
        .with_context(|| format!("running git status in {}", repo_root.display()))?;
    if !output.status.success() {
        anyhow::bail!(
            "git status failed in {}: {}",
            repo_root.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let porcelain = String::from_utf8_lossy(&output.stdout).into_owned();
    if porcelain.trim().is_empty() {
        return Ok(());
    }
    Err(DirtyTreeError {
        repo_root: repo_root.display().to_string(),
        porcelain,
    }
    .into())
}

/// Branch existence check: returns true iff `refs/heads/<branch>` resolves.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/git/branch_switch.md#source
pub fn branch_exists_local(repo_root: &Path, branch: &str) -> Result<bool> {
    let git = find_git_bin().context("git binary not found on PATH")?;
    let output = Command::new(&git)
        .args([
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{branch}"),
        ])
        .current_dir(repo_root)
        .output()?;
    Ok(output.status.success())
}

/// Current branch name (HEAD's short symbolic ref). Returns "HEAD" if detached.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/git/branch_switch.md#source
pub fn current_branch(repo_root: &Path) -> Result<String> {
    let git = find_git_bin().context("git binary not found on PATH")?;
    let output = Command::new(&git)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo_root)
        .output()?;
    if !output.status.success() {
        anyhow::bail!(
            "git rev-parse failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Switch to `target` if it exists locally; otherwise create it from `fork_point`.
/// Idempotent — re-run on already-on-target is a no-op.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/git/branch_switch.md#source
pub fn switch_or_create_branch(repo_root: &Path, target: &str, fork_point: &str) -> Result<()> {
    if current_branch(repo_root)? == target {
        return Ok(());
    }
    let git = find_git_bin().context("git binary not found on PATH")?;
    let args: Vec<String> = if branch_exists_local(repo_root, target)? {
        vec!["switch".into(), target.into()]
    } else {
        vec![
            "switch".into(),
            "-c".into(),
            target.into(),
            fork_point.into(),
        ]
    };
    let output = Command::new(&git)
        .args(&args)
        .current_dir(repo_root)
        .output()?;
    if !output.status.success() {
        anyhow::bail!(
            "git switch failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

/// Switch to `default`, then `git merge --no-ff <source>`. Returns the merge outcome.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/git/branch_switch.md#source
pub fn merge_branch_into_default(
    repo_root: &Path,
    source: &str,
    default: &str,
) -> Result<MergeOutcome> {
    switch_or_create_branch(repo_root, default, default)?;
    let git = find_git_bin().context("git binary not found on PATH")?;

    // Check fast-forward eligibility first so the caller can omit the merge commit.
    let mb = Command::new(&git)
        .args(["merge-base", "--is-ancestor", default, source])
        .current_dir(repo_root)
        .output()?;
    let is_ff = mb.status.success();

    let merge_args: Vec<&str> = if is_ff {
        vec!["merge", "--ff-only", source]
    } else {
        vec!["merge", "--no-ff", "-m", "&", source]
    };
    // Replace the placeholder "&" with a real message.
    let mut args_owned: Vec<String> = merge_args.iter().map(|s| s.to_string()).collect();
    if !is_ff {
        if let Some(pos) = args_owned.iter().position(|s| s == "&") {
            args_owned[pos] = format!("Merge {source}");
        }
    }

    let output = Command::new(&git)
        .args(&args_owned)
        .current_dir(repo_root)
        .output()?;

    if output.status.success() {
        return Ok(if is_ff {
            MergeOutcome::FastForward
        } else {
            MergeOutcome::Merged
        });
    }

    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    Ok(MergeOutcome::Conflict { stderr })
}

/// `git branch -d <branch>`. Idempotent — already-absent or HEAD-points-to-it are Ok-with-warning.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/git/branch_switch.md#source
pub fn delete_local_branch(repo_root: &Path, branch: &str) -> Result<()> {
    if !branch_exists_local(repo_root, branch)? {
        return Ok(());
    }
    if current_branch(repo_root)? == branch {
        // Refuse to delete the currently-checked-out branch; caller must switch first.
        anyhow::bail!(
            "refusing to delete currently-checked-out branch '{branch}' \
             — switch to default first"
        );
    }
    let git = find_git_bin().context("git binary not found on PATH")?;
    let output = Command::new(&git)
        .args(["branch", "-d", branch])
        .current_dir(repo_root)
        .output()?;
    if !output.status.success() {
        anyhow::bail!(
            "git branch -d {} failed: {}",
            branch,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn init_git_repo(dir: &Path) -> bool {
        let Some(git) = find_git_bin() else {
            return false;
        };
        let ok = Command::new(&git)
            .args(["init", "--initial-branch=main"])
            .current_dir(dir)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !ok {
            return false;
        }
        let _ = Command::new(&git)
            .args(["config", "user.email", "test@example.com"])
            .current_dir(dir)
            .output();
        let _ = Command::new(&git)
            .args(["config", "user.name", "Test"])
            .current_dir(dir)
            .output();
        fs::write(dir.join("README.md"), "# test\n").unwrap();
        let _ = Command::new(&git)
            .args(["add", "."])
            .current_dir(dir)
            .output();
        let _ = Command::new(&git)
            .args(["commit", "-m", "init"])
            .current_dir(dir)
            .output();
        true
    }

    #[test]
    fn ensure_clean_passes_on_clean_tree() {
        let tmp = TempDir::new().unwrap();
        if !init_git_repo(tmp.path()) {
            return;
        }
        ensure_branch_clean(tmp.path()).unwrap();
    }

    #[test]
    fn ensure_clean_returns_dirty_tree_error() {
        let tmp = TempDir::new().unwrap();
        if !init_git_repo(tmp.path()) {
            return;
        }
        fs::write(tmp.path().join("scratch.txt"), "uncommitted\n").unwrap();
        let err = ensure_branch_clean(tmp.path()).unwrap_err();
        let s = err.to_string();
        assert!(s.contains("dirty"), "{s}");
        assert!(s.contains("scratch.txt"), "{s}");
    }

    #[test]
    fn switch_or_create_idempotent_on_current_branch() {
        let tmp = TempDir::new().unwrap();
        if !init_git_repo(tmp.path()) {
            return;
        }
        // Already on main; no-op.
        switch_or_create_branch(tmp.path(), "main", "main").unwrap();
        assert_eq!(current_branch(tmp.path()).unwrap(), "main");
    }

    #[test]
    fn switch_or_create_creates_then_switches_back() {
        let tmp = TempDir::new().unwrap();
        if !init_git_repo(tmp.path()) {
            return;
        }
        switch_or_create_branch(tmp.path(), "issue-42-foo", "main").unwrap();
        assert_eq!(current_branch(tmp.path()).unwrap(), "issue-42-foo");
        // Re-run on existing branch from main: should switch to it (already on it, no-op).
        switch_or_create_branch(tmp.path(), "issue-42-foo", "main").unwrap();
        assert_eq!(current_branch(tmp.path()).unwrap(), "issue-42-foo");
        // Switch back to main.
        switch_or_create_branch(tmp.path(), "main", "main").unwrap();
        assert_eq!(current_branch(tmp.path()).unwrap(), "main");
        // Switch to existing issue branch (no -c needed).
        switch_or_create_branch(tmp.path(), "issue-42-foo", "main").unwrap();
        assert_eq!(current_branch(tmp.path()).unwrap(), "issue-42-foo");
    }

    #[test]
    fn merge_fast_forward_path() {
        let tmp = TempDir::new().unwrap();
        if !init_git_repo(tmp.path()) {
            return;
        }
        switch_or_create_branch(tmp.path(), "feat-x", "main").unwrap();
        fs::write(tmp.path().join("a.txt"), "a\n").unwrap();
        let git = find_git_bin().unwrap();
        Command::new(&git)
            .args(["add", "."])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        Command::new(&git)
            .args(["commit", "-m", "x"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        let outcome = merge_branch_into_default(tmp.path(), "feat-x", "main").unwrap();
        assert_eq!(outcome, MergeOutcome::FastForward);
        assert_eq!(current_branch(tmp.path()).unwrap(), "main");
    }

    #[test]
    fn delete_branch_idempotent_when_absent() {
        let tmp = TempDir::new().unwrap();
        if !init_git_repo(tmp.path()) {
            return;
        }
        delete_local_branch(tmp.path(), "never-existed").unwrap();
    }

    #[test]
    fn delete_refuses_currently_checked_out_branch() {
        let tmp = TempDir::new().unwrap();
        if !init_git_repo(tmp.path()) {
            return;
        }
        switch_or_create_branch(tmp.path(), "issue-1-foo", "main").unwrap();
        let err = delete_local_branch(tmp.path(), "issue-1-foo").unwrap_err();
        assert!(err.to_string().contains("refusing"), "{err}");
    }

    #[test]
    fn delete_succeeds_after_switch() {
        let tmp = TempDir::new().unwrap();
        if !init_git_repo(tmp.path()) {
            return;
        }
        switch_or_create_branch(tmp.path(), "issue-1-foo", "main").unwrap();
        switch_or_create_branch(tmp.path(), "main", "main").unwrap();
        delete_local_branch(tmp.path(), "issue-1-foo").unwrap();
        assert!(!branch_exists_local(tmp.path(), "issue-1-foo").unwrap());
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/branch_switch.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete in-place branch switch helper module.
```
