---
id: sdd-tools-merge-git-ops-commit-and-pr-body
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools merge git ops commit and pr body

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
      - "read_user_input_summary"
      - "build_commit_message"
      - "capture_head_sha"
      - "extract_commit_sha"
      - "build_pr_body"
    description: "Archive summary reading, commit-message formatting, SHA capture, and generated PR body helpers."
```
