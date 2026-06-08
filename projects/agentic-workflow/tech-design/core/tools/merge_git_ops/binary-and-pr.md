---
id: sdd-tools-merge-git-ops-binary-and-pr
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools merge git ops binary and pr

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
      - "find_git_binary"
      - "find_gh_binary"
      - "create_pr"
    description: "Git/gh binary lookup and GitHub PR creation helper."
```
