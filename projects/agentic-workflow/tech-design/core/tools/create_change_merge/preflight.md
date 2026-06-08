---
id: sdd-tools-create-change-merge-preflight
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create change merge preflight

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_change_merge.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `execute_workflow` | projects/agentic-workflow/src/tools/create_change_merge.rs | function | pub | 69 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_change_merge.rs | function | pub | 29 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
// ─── Pre-flight Gates (R6, R7, R8) ───────────────────────────────────────────

/// Run the two pre-flight gates before any side effect.
///
/// - G1 (dirty worktree): `git -C <work_root> status --porcelain` must be empty.
/// - G2 (branch exists): `refs/heads/cclab/<slug>` must exist (worktree mode only).
///
/// Empty `change_dir/specs/` is intentionally allowed — callers downstream
/// handle the "no-specs archive" case.
///
/// Skipped entirely (with warning log) when no git binary is available — this
/// keeps tempdir/unit tests working without a git repo.
///
/// REQ: change-merge R6, R7.
fn pre_flight_validate(
    work_root: &Path,
    project_root: &Path,
    change_id: &str,
    _change_dir: &Path,
) -> Result<()> {
    // G1 + G2 require git. Skip gracefully in non-git contexts.
    let git_bin = match find_git_binary() {
        Some(g) => g,
        None => {
            tracing::warn!(
                change_id = %change_id,
                "pre-flight G1/G2 skipped: git binary not found"
            );
            return Ok(());
        }
    };

    // G1: dirty worktree
    let status = std::process::Command::new(&git_bin)
        .args(["status", "--porcelain"])
        .current_dir(work_root)
        .output();
    match status {
        Ok(out) if out.status.success() => {
            if !out.stdout.is_empty() {
                anyhow::bail!(
                    "uncommitted work in {} — commit or stash before merging.\n  Run: cd {} && git status",
                    work_root.display(),
                    work_root.display()
                );
            }
        }
        Ok(out) => {
            // Non-zero git status (e.g. not a git repo) — skip with warning
            tracing::warn!(
                stderr = %String::from_utf8_lossy(&out.stderr),
                "pre-flight G1 skipped: git status failed"
            );
            return Ok(());
        }
        Err(e) => {
            tracing::warn!(error = %e, "pre-flight G1 skipped: git status errored");
            return Ok(());
        }
    }

    // G2: branch exists (only when worktree mode). In legacy mode
    // (work_root == project_root), there is no dedicated cclab/<slug> branch
    // to verify.
    if work_root != project_root {
        let branch = format!("cclab/{}", change_id);
        let ref_name = format!("refs/heads/{}", branch);
        let exists = std::process::Command::new(&git_bin)
            .args(["show-ref", "--verify", "--quiet", &ref_name])
            .current_dir(project_root)
            .status();
        match exists {
            Ok(s) if !s.success() => {
                anyhow::bail!(
                    "branch {} not found — re-run init_change or use legacy flow",
                    branch
                );
            }
            Ok(_) => {}
            Err(e) => {
                tracing::warn!(error = %e, "pre-flight G2 skipped: show-ref errored");
            }
        }
    }

    Ok(())
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/create_change_merge.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "pre_flight_validate"
    description: "Pre-flight gates for dirty worktree and worktree branch existence."
```
