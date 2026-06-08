---
id: sdd-tools-workflow-common-git-status
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools workflow common git status

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/workflow_common.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `build_group_issues_hint` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 442 | build_group_issues_hint(change_dir: &Path, group_id: &str) -> String |
| `build_workflow_response` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 395 | build_workflow_response(     change_dir: &Path,     change_id: &str,     action: &str,     prompt: String,     executor: Vec<String>,     extra_fields: Value,     _interface: SddInterface,     _project_root: &Path, ) -> Result<String> |
| `get_executor_chain` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 273 | get_executor_chain(_project_root: &Path, artifact: WorkflowArtifact) -> Vec<String> |
| `has_uncommitted_diff` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 209 | has_uncommitted_diff(project_root: &Path, rel_path: &str) -> Result<bool> |
| `is_git_project` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 174 | is_git_project(project_root: &Path) -> bool |
| `is_git_tracked` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 192 | is_git_tracked(project_root: &Path, rel_path: &str) -> Result<bool> |
| `list_group_ids` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 325 | list_group_ids(groups_dir: &Path) -> Result<Vec<String>> |
| `load_interface` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 309 | load_interface(project_root: &Path) -> SddInterface |
| `next_action` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 319 | next_action(interface: SddInterface, tool: &str, args: Value) -> Value |
| `phase_to_string` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 144 | phase_to_string(phase: &StatePhase) -> &'static str |
| `resolve_active_change_id` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 76 | resolve_active_change_id(project_root: &Path) -> Result<String> |
| `resolve_change_dir` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 120 | resolve_change_dir(project_root: &Path, change_id: &str) -> PathBuf |
| `resolve_single_group_id` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 346 | resolve_single_group_id(change_dir: &Path) -> Option<String> |
| `update_phase` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 250 | update_phase(change_dir: &Path, phase: StatePhase) -> Result<()> |
| `validate_change_dir` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 39 | validate_change_dir(change_dir: &Path, project_root: &Path) -> Result<()> |
| `validate_change_id` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 18 | validate_change_id(change_id: &str) -> Result<()> |
| `write_prompt_file` | projects/agentic-workflow/src/tools/workflow_common.rs | function | pub | 364 | write_prompt_file(     change_dir: &Path,     group_id: Option<&str>,     action: &str,     prompt: &str, ) -> Result<PathBuf> |
## Source
<!-- type: source lang: rust -->

````rust
/// Probe for a git binary on PATH. `None` when git is unavailable.
fn find_git_bin() -> Option<PathBuf> {
    let output = std::process::Command::new("which")
        .arg("git")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        None
    } else {
        Some(PathBuf::from(path))
    }
}

/// Returns `true` when `project_root` is inside a git workspace.
/// Used to skip pre-flight gates gracefully in tempdir tests.
// REQ: issue-centric-workflow#R11
pub fn is_git_project(project_root: &Path) -> bool {
    let Some(git) = find_git_bin() else {
        return false;
    };
    std::process::Command::new(&git)
        .args(["rev-parse", "--git-dir"])
        .current_dir(project_root)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Returns `true` if `rel_path` (project-relative) is tracked in git's index.
/// Returns `false` if the file is untracked, ignored, or absent.
/// Returns `Ok(true)` with a warning-log skip when git isn't available — this
/// preserves test ergonomics in tempdirs.
// REQ: issue-centric-workflow#R7
pub fn is_git_tracked(project_root: &Path, rel_path: &str) -> Result<bool> {
    if !is_git_project(project_root) {
        return Ok(true); // Skip gate in non-git contexts.
    }
    let git = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let out = std::process::Command::new(&git)
        .args(["ls-files", "--error-unmatch", rel_path])
        .current_dir(project_root)
        .output()
        .map_err(|e| anyhow::anyhow!("git ls-files failed to run: {}", e))?;
    Ok(out.status.success())
}

/// Returns `true` when `rel_path` has staged or unstaged diffs against `HEAD`.
/// Returns `Ok(false)` in non-git contexts (gate skipped).
// REQ: issue-centric-workflow#R8
pub fn has_uncommitted_diff(project_root: &Path, rel_path: &str) -> Result<bool> {
    if !is_git_project(project_root) {
        return Ok(false);
    }
    let git = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let out = std::process::Command::new(&git)
        .args(["diff", "--quiet", "HEAD", "--", rel_path])
        .current_dir(project_root)
        .output()
        .map_err(|e| anyhow::anyhow!("git diff failed to run: {}", e))?;
    // `git diff --quiet` exits 0 when identical, 1 when different.
    Ok(!out.status.success())
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/workflow_common.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "find_git_bin"
      - "is_git_project"
      - "is_git_tracked"
      - "has_uncommitted_diff"
    description: "Git availability and cleanliness gate helpers."
```
