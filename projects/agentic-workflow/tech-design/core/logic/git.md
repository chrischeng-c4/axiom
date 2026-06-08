---
id: projects-sdd-src-git-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Change/context/git/spec-store logic supports TD/CB artifact lifecycle dispatch and review state."
---

# Standardized projects/agentic-workflow/src/git.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/git.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `commit_scoped_paths` | projects/agentic-workflow/src/git.rs | function | pub | 66 | commit_scoped_paths(project_root: &Path, paths: &[PathBuf], message: &str) -> Result<bool> |
| `dirty_paths` | projects/agentic-workflow/src/git.rs | function | pub | 124 | dirty_paths(     project_root: &Path,     scopes: &[PathBuf],     include_untracked: bool, ) -> Result<Vec<String>> |
| `ensure_no_staged_changes` | projects/agentic-workflow/src/git.rs | function | pub | 48 | ensure_no_staged_changes(project_root: &Path) -> Result<()> |
| `find_git_bin` | projects/agentic-workflow/src/git.rs | function | pub | 17 | find_git_bin() -> Option<PathBuf> |
| `is_git_repo` | projects/agentic-workflow/src/git.rs | function | pub | 33 | is_git_repo(project_root: &Path) -> bool |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/git.rs -->
````rust
//! Neutral home for `find_git_bin` — formerly under `crate::worktree`.
//!
//! @spec projects/agentic-workflow/tech-design/core/worktree-retirement.md#schema (R5)
//!
//! `worktree.rs` was gutted in Phase C. Callers that still need to
//! locate the `git` binary (in-place CRRR helpers, the one-shot
//! `score migrate-worktrees` verb) import this module instead.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Locate the `git` binary on `PATH`. Returns `None` if `which git`
/// fails or returns an empty string.
/// @spec projects/agentic-workflow/tech-design/core/logic/git.md#source
pub fn find_git_bin() -> Option<PathBuf> {
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

/// Return true when `project_root` is inside a git worktree.
/// @spec projects/agentic-workflow/tech-design/core/logic/git.md#source
pub fn is_git_repo(project_root: &Path) -> bool {
    let Some(git_bin) = find_git_bin() else {
        return false;
    };
    std::process::Command::new(git_bin)
        .arg("-C")
        .arg(project_root)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// Refuse to mix lifecycle commits with already-staged user changes.
/// @spec projects/agentic-workflow/tech-design/core/logic/git.md#source
pub fn ensure_no_staged_changes(project_root: &Path) -> Result<()> {
    let git_bin = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let status = std::process::Command::new(git_bin)
        .arg("-C")
        .arg(project_root)
        .args(["diff", "--cached", "--quiet"])
        .status()
        .context("git diff --cached failed")?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("refusing to commit lifecycle changes with pre-existing staged changes")
    }
}

/// Stage exactly `paths`, create `message` as a lifecycle commit, and no-op
/// when those paths have no staged diff.
/// @spec projects/agentic-workflow/tech-design/core/logic/git.md#source
pub fn commit_scoped_paths(project_root: &Path, paths: &[PathBuf], message: &str) -> Result<bool> {
    if paths.is_empty() || !is_git_repo(project_root) {
        return Ok(false);
    }
    ensure_no_staged_changes(project_root)?;
    let rel_paths = repo_relative_paths(project_root, paths)?;
    if rel_paths.is_empty() {
        return Ok(false);
    }

    let git_bin = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let mut add = std::process::Command::new(&git_bin);
    add.arg("-C").arg(project_root).arg("add").arg("--");
    for path in &rel_paths {
        add.arg(path);
    }
    let add_out = add.output().context("git add failed")?;
    if !add_out.status.success() {
        anyhow::bail!(
            "git add failed: {}",
            String::from_utf8_lossy(&add_out.stderr).trim()
        );
    }

    let mut diff = std::process::Command::new(&git_bin);
    diff.arg("-C")
        .arg(project_root)
        .args(["diff", "--cached", "--quiet", "--"]);
    for path in &rel_paths {
        diff.arg(path);
    }
    let staged = diff.status().context("git diff --cached failed")?;
    if staged.success() {
        return Ok(false);
    }

    let mut commit = std::process::Command::new(&git_bin);
    commit
        .arg("-C")
        .arg(project_root)
        .args(["commit", "-m"])
        .arg(message)
        .arg("--");
    for path in &rel_paths {
        commit.arg(path);
    }
    let commit_out = commit.output().context("git commit failed")?;
    if !commit_out.status.success() {
        anyhow::bail!(
            "git commit failed: {}",
            String::from_utf8_lossy(&commit_out.stderr).trim()
        );
    }
    Ok(true)
}

/// Return dirty paths under the supplied repo-relative or absolute scopes.
/// @spec projects/agentic-workflow/tech-design/core/logic/git.md#source
pub fn dirty_paths(
    project_root: &Path,
    scopes: &[PathBuf],
    include_untracked: bool,
) -> Result<Vec<String>> {
    if scopes.is_empty() || !is_git_repo(project_root) {
        return Ok(Vec::new());
    }
    let rel_scopes = repo_relative_paths(project_root, scopes)?;
    if rel_scopes.is_empty() {
        return Ok(Vec::new());
    }
    let git_bin = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let mut status = std::process::Command::new(git_bin);
    status
        .arg("-C")
        .arg(project_root)
        .args(["status", "--porcelain=v1"]);
    if include_untracked {
        status.arg("--untracked-files=all");
    } else {
        status.arg("--untracked-files=no");
    }
    status.arg("--");
    for scope in &rel_scopes {
        status.arg(scope);
    }
    let out = status.output().context("git status failed")?;
    if !out.status.success() {
        anyhow::bail!(
            "git status failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    let mut paths = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line| line.get(3..))
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn repo_relative_paths(project_root: &Path, paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut rel_paths = Vec::new();
    for path in paths {
        let rel = if path.is_absolute() {
            path.strip_prefix(project_root).with_context(|| {
                format!(
                    "{} is outside git worktree {}",
                    path.display(),
                    project_root.display()
                )
            })?
        } else {
            path.as_path()
        };
        if rel.as_os_str().is_empty() {
            continue;
        }
        rel_paths.push(rel.to_path_buf());
    }
    rel_paths.sort();
    rel_paths.dedup();
    Ok(rel_paths)
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/git.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete git utility module.
```
