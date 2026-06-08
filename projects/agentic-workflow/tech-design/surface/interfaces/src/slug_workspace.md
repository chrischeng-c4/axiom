---
id: projects-score-src-slug-workspace-rs
fill_sections: [overview, changes]
capability_refs:
  - id: work-item-planning
    role: primary
    gap: capability-to-epic-planning
    claim: capability-to-epic-planning
    coverage: full
    rationale: "Issue/update CLI surfaces support work-item planning, projection, and platform synchronization."
---

# Standardized projects/agentic-workflow/src/cli/slug_workspace.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/cli/slug_workspace.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ActiveWorkspace` | projects/agentic-workflow/src/cli/slug_workspace.rs | struct | pub | 25 |  |
| `enter_workspace_for_verb` | projects/agentic-workflow/src/cli/slug_workspace.rs | function | pub | 40 | enter_workspace_for_verb(     project_root: &Path,     kind: BranchKind,     slug: &str,     provision_if_missing: bool, ) -> Result<ActiveWorkspace> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/cli/slug_workspace.rs -->
```rust
//! Phase C: in-place workspace activation for TD/CB artifacts.
//!
//! WI is the workflow root but does not materialize `issue-*` git branches.
//! TD/CB commands run in-place on the current checkout. They create/switch to
//! `<kind>-<slug>` only when invoked from `main`; project branches stay on the
//! current branch.
//!
//! Path resolution moved to `repo_root` directly — callers no longer
//! call `slug_workspace_path*`. Branch activation lives in
//! `enter_workspace_for_verb`, which is now in-place-only and is the
//! only public entry point in this module.
//!
use anyhow::Result;
use agentic_workflow::issues::slug::BranchKind;
use std::path::{Path, PathBuf};

// Resolved active workspace for a `<kind>-<slug>` verb call.
///
// Post-Phase-C `path` is always `repo_root`; the host repo is on
// branch `branch` after `enter_workspace_for_verb` returns.
#[derive(Debug, Clone)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/slug_workspace.md#source
pub struct ActiveWorkspace {
    /// Filesystem root for all writes — equals `repo_root`.
    pub path: PathBuf,
    /// Branch name the workspace is bound to after activation.
    pub branch: String,
}

// Activate the host repo for a TD/CB lifecycle verb.
///
// WI commands must not call this helper. For TD/CB, project branches stay on
// the current branch. From `main`, this creates or switches to
// `<kind>-<slug>` when `provision_if_missing` allows it.
///
// Returns an `ActiveWorkspace { path: repo_root, branch }`.
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/slug_workspace.md#source
pub fn enter_workspace_for_verb(
    project_root: &Path,
    kind: BranchKind,
    slug: &str,
    provision_if_missing: bool,
) -> Result<ActiveWorkspace> {
    if matches!(kind, BranchKind::Issue) {
        anyhow::bail!("WI workflow does not use issue-* git branches");
    }
    let current = agentic_workflow::branch_switch::current_branch(project_root)?;
    if current != "main" {
        return Ok(ActiveWorkspace {
            path: project_root.to_path_buf(),
            branch: current,
        });
    }

    let branch = format!("{}-{}", kind.as_prefix(), slug);
    let branch_exists =
        agentic_workflow::branch_switch::branch_exists_local(project_root, &branch).unwrap_or(false);
    if !branch_exists && !provision_if_missing {
        anyhow::bail!("workspace not found: branch '{}' does not exist", branch);
    }
    agentic_workflow::branch_switch::ensure_branch_clean(project_root)
        .map_err(|e| anyhow::anyhow!("in-place verb requires clean working tree: {}", e))?;
    agentic_workflow::branch_switch::switch_or_create_branch(project_root, &branch, &current)?;
    Ok(ActiveWorkspace {
        path: project_root.to_path_buf(),
        branch,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    fn init_git_repo(dir: &Path) -> bool {
        let Some(git) = agentic_workflow::git::find_git_bin() else {
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
        std::fs::write(dir.join("README.md"), "# test\n").unwrap();
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
    fn enter_creates_td_branch_from_main_when_provisioning() {
        let tmp = TempDir::new().unwrap();
        if !init_git_repo(tmp.path()) {
            return;
        }
        let aw = enter_workspace_for_verb(tmp.path(), BranchKind::Td, "demo", true).unwrap();
        assert_eq!(aw.path, tmp.path());
        assert_eq!(aw.branch, "td-demo");
        assert!(
            agentic_workflow::branch_switch::branch_exists_local(tmp.path(), "td-demo").unwrap(),
            "branch should be provisioned"
        );
    }

    #[test]
    fn enter_refuses_missing_branch_without_provision() {
        let tmp = TempDir::new().unwrap();
        if !init_git_repo(tmp.path()) {
            return;
        }
        let err =
            enter_workspace_for_verb(tmp.path(), BranchKind::Td, "missing", false).unwrap_err();
        assert!(err.to_string().contains("workspace not found"), "{err}");
    }

    #[test]
    fn enter_path_is_project_root() {
        let tmp = TempDir::new().unwrap();
        if !init_git_repo(tmp.path()) {
            return;
        }
        let aw = enter_workspace_for_verb(tmp.path(), BranchKind::Cb, "demo", true).unwrap();
        assert_eq!(aw.path, tmp.path());
    }

    #[test]
    fn enter_rejects_issue_kind() {
        let tmp = TempDir::new().unwrap();
        if !init_git_repo(tmp.path()) {
            return;
        }
        let err =
            enter_workspace_for_verb(tmp.path(), BranchKind::Issue, "demo", true).unwrap_err();
        assert!(err.to_string().contains("WI workflow"), "{err}");
    }

    #[test]
    fn enter_off_main_stays_on_current_branch() {
        let tmp = TempDir::new().unwrap();
        if !init_git_repo(tmp.path()) {
            return;
        }
        let git = agentic_workflow::git::find_git_bin().unwrap();
        let out = Command::new(&git)
            .args(["switch", "-c", "project-score"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        assert!(out.status.success());
        let aw = enter_workspace_for_verb(tmp.path(), BranchKind::Td, "demo", true).unwrap();
        assert_eq!(aw.branch, "project-score");
        assert!(
            !agentic_workflow::branch_switch::branch_exists_local(tmp.path(), "td-demo").unwrap(),
            "off-main activation should not create td branch"
        );
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/slug_workspace.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
```
