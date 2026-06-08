---
id: projects-score-src-merge-target-rs
fill_sections: [overview, changes]
capability_refs:
  - id: work-item-planning
    role: primary
    gap: capability-to-epic-planning
    claim: capability-to-epic-planning
    coverage: full
    rationale: "Issue/update CLI surfaces support work-item planning, projection, and platform synchronization."
---

# Standardized projects/agentic-workflow/src/cli/merge_target.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/cli/merge_target.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `resolve_merge_target` | projects/agentic-workflow/src/cli/merge_target.rs | function | pub | 30 | resolve_merge_target(     override_branch: Option<String>,     frontmatter_branch: Option<String>,     project_root: &Path, ) -> anyhow::Result<String> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/cli/merge_target.rs -->
```rust
//! Resolve the target branch for `aw td merge` and `aw wi merge`.
//!
//! Resolution order (per the Logic flowchart in the TD spec
//! `projects/agentic-workflow/tech-design/core/issues/issue-merge-target.md`):
//!   1. `override_branch` (CLI `--target-branch`) if `Some` → return verbatim
//!      (no branch-exists check; user is explicit).
//!   2. `frontmatter_branch` (`issue.target_branch`) if `Some` → return it.
//!   3. `git -C <project_root> rev-parse --abbrev-ref HEAD` → if output ≠ "HEAD"
//!      return the detected branch name.
//!   4. Detached HEAD: read `.aw/config.toml`, extract
//!      `[agentic_workflow.repo_platform].default_branch`. If present + non-empty → return it.
//!   5. Err: "cannot determine merge target: detached HEAD and no
//!      default_branch in .aw/config.toml"
//!
//! Never silently falls back to "main".
//!
//! The R5 branch-exists validation step from the TD spec is deferred to a
//! follow-up issue — it is a behavior change for steps 2–4 that needs its
//! own coverage matrix.

use std::path::Path;

// Resolve the effective merge-target branch.
///
// See module-level docs for the 5-step resolution order.
///
// @spec projects/agentic-workflow/tech-design/core/issues/issue-merge-target.md#logic
pub fn resolve_merge_target(
    override_branch: Option<String>,
    frontmatter_branch: Option<String>,
    project_root: &Path,
) -> anyhow::Result<String> {
    // Step 1: explicit CLI override wins (verbatim, no validation).
    if let Some(branch) = override_branch {
        return Ok(branch);
    }

    // Step 2: issue frontmatter `target_branch` is the next source of truth.
    if let Some(branch) = frontmatter_branch {
        return Ok(branch);
    }

    // Step 3: detect current branch via git.
    let git_bin =
        agentic_workflow::git::find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;

    let output = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(project_root)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run git rev-parse: {}", e))?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !branch.is_empty() && branch != "HEAD" {
            return Ok(branch);
        }
    }

    // Step 4: detached HEAD — fall back to .aw/config.toml default_branch.
    let config_path = project_root.join(".aw/config.toml");
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| anyhow::anyhow!("failed to read .aw/config.toml: {}", e))?;
        let parsed: toml::Value = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("failed to parse .aw/config.toml: {}", e))?;
        let default_branch = parsed
            .get("sdd")
            .and_then(|s| s.get("repo_platform"))
            .and_then(|rp| rp.get("default_branch"))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        if let Some(branch) = default_branch {
            return Ok(branch);
        }
    }

    // Step 5: nothing worked.
    Err(anyhow::anyhow!(
        "cannot determine merge target: detached HEAD and no default_branch in .aw/config.toml"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    /// Helper: initialise a bare-minimum git repo in a temp dir and return the path.
    fn init_repo(dir: &std::path::Path) -> std::path::PathBuf {
        Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["init", "--initial-branch=main"])
            .output()
            .unwrap();
        // git init always needs at least one commit to create a real branch pointer
        Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["config", "user.email", "test@example.com"])
            .output()
            .unwrap();
        Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["config", "user.name", "Test"])
            .output()
            .unwrap();
        // Create initial commit so HEAD points to a real branch
        Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["commit", "--allow-empty", "-m", "init"])
            .output()
            .unwrap();
        dir.to_path_buf()
    }

    /// Write a minimal .aw/config.toml with an optional default_branch.
    fn write_config(dir: &std::path::Path, default_branch: Option<&str>) {
        let score_dir = dir.join(".aw");
        std::fs::create_dir_all(&score_dir).unwrap();
        let content = match default_branch {
            Some(b) => format!(
                "[agentic_workflow.repo_platform]\ntype = \"github\"\nrepo = \"test/repo\"\ndefault_branch = \"{}\"\n",
                b
            ),
            None => "[agentic_workflow.repo_platform]\ntype = \"github\"\nrepo = \"test/repo\"\n".to_string(),
        };
        std::fs::write(score_dir.join("config.toml"), content).unwrap();
    }

    // REQ: TP-C
    #[test]
    fn test_override_branch_wins() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        write_config(tmp.path(), Some("main"));
        let result =
            resolve_merge_target(Some("release-1.0".to_string()), None, tmp.path()).unwrap();
        assert_eq!(result, "release-1.0");
    }

    // REQ: TP-A, TP-B
    #[test]
    fn test_detects_current_branch() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        // Create and checkout a feature branch
        Command::new("git")
            .arg("-C")
            .arg(tmp.path())
            .args(["checkout", "-b", "feature-xyz"])
            .output()
            .unwrap();
        let result = resolve_merge_target(None, None, tmp.path()).unwrap();
        assert_eq!(result, "feature-xyz");
    }

    // REQ: TP-B
    #[test]
    fn test_detects_main_branch() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        let result = resolve_merge_target(None, None, tmp.path()).unwrap();
        assert_eq!(result, "main");
    }

    // REQ: TP-D
    #[test]
    fn test_detached_head_uses_config() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        write_config(tmp.path(), Some("develop"));
        // Detach HEAD by checking out the commit hash
        let rev_out = Command::new("git")
            .arg("-C")
            .arg(tmp.path())
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap();
        let sha = String::from_utf8_lossy(&rev_out.stdout).trim().to_string();
        Command::new("git")
            .arg("-C")
            .arg(tmp.path())
            .args(["checkout", &sha])
            .output()
            .unwrap();
        let result = resolve_merge_target(None, None, tmp.path()).unwrap();
        assert_eq!(result, "develop");
    }

    // REQ: TP-D (error path)
    #[test]
    fn test_detached_head_no_config_returns_error() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        // No .aw/config.toml
        let rev_out = Command::new("git")
            .arg("-C")
            .arg(tmp.path())
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap();
        let sha = String::from_utf8_lossy(&rev_out.stdout).trim().to_string();
        Command::new("git")
            .arg("-C")
            .arg(tmp.path())
            .args(["checkout", &sha])
            .output()
            .unwrap();
        let result = resolve_merge_target(None, None, tmp.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot determine merge target"));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/merge_target.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
```
