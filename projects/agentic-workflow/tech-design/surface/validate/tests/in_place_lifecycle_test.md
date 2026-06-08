---
id: projects-score-tests-in-place-lifecycle-test-rs
fill_sections: [overview, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Standardization TDs support brownfield takeover, semantic coverage, traceability, and production readiness gates."
---

# Standardized projects/agentic-workflow/tests/cli/tests/in_place_lifecycle_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/tests/cli/tests/in_place_lifecycle_test.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/tests/cli/tests/in_place_lifecycle_test.rs -->
```rust
//! Integration test for the post-Phase-C in-place workspace lifecycle.
//!
//! In-place lifecycle integration tests.
//!
//! Verifies the unconditional in-place behavior of
//! `crate::slug_workspace::enter_workspace_for_verb`:
//!
//!   - host repo HEAD switches to `<kind>-<slug>` for TD/CB from main
//!   - the `ActiveWorkspace.path` equals the project root
//!   - no `.aw/worktrees/` directory is created at any point
//!   - WI/Issue activation is rejected because WI state no longer uses
//!     `issue-*` branches.

use std::path::Path;
use std::process::Command;

use agentic_workflow::cli::slug_workspace::enter_workspace_for_verb;
use agentic_workflow::issues::slug::BranchKind;

fn skip_unless_git() -> Option<std::path::PathBuf> {
    agentic_workflow::git::find_git_bin()
}

fn bootstrap_repo(git: &Path, root: &Path) {
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["init", "-b", "main"])
        .status()
        .expect("git init");
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["config", "user.email", "t@t"])
        .status()
        .unwrap();
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["config", "user.name", "t"])
        .status()
        .unwrap();
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["config", "commit.gpgsign", "false"])
        .status()
        .unwrap();
    std::fs::write(root.join("README.md"), "seed\n").unwrap();
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "seed"])
        .status()
        .unwrap();
}

fn current_branch(git: &Path, root: &Path) -> String {
    let out = Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Single-namespace activation — no worktree dir, branch switched on host.
#[test]
fn in_place_activation_switches_branch_and_skips_worktrees_dir() {
    let Some(git) = skip_unless_git() else { return };
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    bootstrap_repo(&git, root);

    let aw = enter_workspace_for_verb(root, BranchKind::Td, "lifecycle-demo", true).unwrap();
    assert_eq!(
        aw.path, root,
        "ActiveWorkspace.path must equal project root post-Phase-C"
    );
    assert_eq!(aw.branch, "td-lifecycle-demo");
    assert_eq!(current_branch(&git, root), "td-lifecycle-demo");
    assert!(
        !root.join(".aw/worktrees").exists(),
        ".aw/worktrees/ must not be provisioned in-place"
    );
}

/// Routing split: WI/Issue activation is rejected, while TD/CB branch only
/// when starting from main. Off-main TD/CB calls stay on the current branch.
#[test]
fn in_place_cross_namespace_chain() {
    let Some(git) = skip_unless_git() else { return };
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    bootstrap_repo(&git, root);

    let slug = "chain-demo";

    let err = enter_workspace_for_verb(root, BranchKind::Issue, slug, true).unwrap_err();
    assert!(err.to_string().contains("WI workflow"), "{err}");
    assert_eq!(current_branch(&git, root), "main");

    let aw2 = enter_workspace_for_verb(root, BranchKind::Td, slug, true).unwrap();
    assert_eq!(aw2.branch, "td-chain-demo");
    assert_eq!(current_branch(&git, root), "td-chain-demo");

    let aw3 = enter_workspace_for_verb(root, BranchKind::Cb, slug, true).unwrap();
    assert_eq!(aw3.branch, "td-chain-demo");
    assert_eq!(current_branch(&git, root), "td-chain-demo");

    assert!(
        !root.join(".aw/worktrees").exists(),
        ".aw/worktrees/ must remain absent across the entire chain"
    );
}

/// Idempotent re-entry: switching back to an existing kind-slug branch
/// must succeed and not double-provision.
#[test]
fn in_place_reentry_is_idempotent() {
    let Some(git) = skip_unless_git() else { return };
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    bootstrap_repo(&git, root);

    let _ = enter_workspace_for_verb(root, BranchKind::Td, "idemp", true).unwrap();
    let aw = enter_workspace_for_verb(root, BranchKind::Td, "idemp", false).unwrap();
    assert_eq!(aw.branch, "td-idemp");
    assert_eq!(current_branch(&git, root), "td-idemp");
}

/// Refusal: dirty working tree blocks activation entirely.
#[test]
fn in_place_refuses_dirty_tree() {
    let Some(git) = skip_unless_git() else { return };
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    bootstrap_repo(&git, root);

    std::fs::write(root.join("dirty.txt"), "uncommitted\n").unwrap();
    let err = enter_workspace_for_verb(root, BranchKind::Td, "blocked", true).unwrap_err();
    assert!(
        err.to_string().contains("clean working tree"),
        "expected dirty-tree refusal, got: {err}",
    );
    assert_eq!(
        current_branch(&git, root),
        "main",
        "dirty refusal must leave HEAD on the original branch",
    );
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/cli/tests/in_place_lifecycle_test.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Existing source claimed by `aw standardize managed run`. The code is
      wrapped in a tracked HANDWRITE block until deterministic generator
      coverage can replace it with CODEGEN.
```
