---
id: sdd-tools-merge-git-ops-worktree-merge-cleanup
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools merge git ops worktree merge cleanup

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
      - "resolve_worktree_dir"
      - "MergeOutcome"
      - "merge_worktree_branch"
      - "cleanup_merged_worktree"
    description: "Worktree detection, default-branch merge, and worktree/branch cleanup helpers."
```
