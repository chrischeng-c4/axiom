---
id: sdd-tools-workflow-common-validation-and-paths
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools workflow common validation and paths

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
//! Shared workflow helpers
//!
//! Common functions used by workflow state machine modules
//! (decide_change, plan_change, impl_change, merge_change, run_change).

use crate::models::change::SddInterface;
use crate::models::state::StatePhase;
use crate::models::{SddConfig, WorkflowArtifact};
use crate::state::StateManager;
use crate::Result;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// Validate change_id format (security: prevent directory traversal)
pub fn validate_change_id(change_id: &str) -> Result<()> {
    if change_id.is_empty() {
        anyhow::bail!("Invalid change_id: cannot be empty");
    }
    if !change_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!("Invalid change_id: must be lowercase alphanumeric with hyphens only");
    }
    Ok(())
}

/// Validate change directory exists and is not a symlink escape.
///
/// Accepts both legacy and worktree-first layouts:
/// - Legacy:    `project_root/.aw/changes/<id>/`
/// - Worktree:  `project_root/.aw/worktrees/<slug>/.aw/changes/<id>/`
///
/// REQ: change-merge R9 — worktree-first path resolution.
pub fn validate_change_dir(change_dir: &Path, project_root: &Path) -> Result<()> {
    if !change_dir.exists() {
        anyhow::bail!("Change directory not found: {}", change_dir.display());
    }
    let canonical_change_dir = change_dir
        .canonicalize()
        .map_err(|e| anyhow::anyhow!("Failed to resolve change directory: {}", e))?;

    // Legacy parent: project_root/.aw/changes/
    let legacy_parent = project_root.join(".aw/changes");
    if let Ok(canonical) = legacy_parent.canonicalize() {
        if canonical_change_dir.starts_with(&canonical) {
            return Ok(());
        }
    }

    // Worktree parent: project_root/.aw/worktrees/ (any worktree's changes/
    // directory is nested two levels deeper and resolves to a real path under
    // the worktrees root, so `starts_with` on the worktrees root is sufficient
    // as a security containment check).
    let worktrees_parent = project_root.join(".aw/worktrees");
    if let Ok(canonical) = worktrees_parent.canonicalize() {
        if canonical_change_dir.starts_with(&canonical) {
            return Ok(());
        }
    }

    anyhow::bail!(
        "Security error: change directory escapes project boundary (possible symlink attack)"
    );
}

/// Resolve the active change_id on the current branch.
///
/// Scans `.aw/changes/*/STATE.yaml` for changes whose phase is non-terminal
/// (not `archived` or `rejected`). Returns the change_id if exactly one is found.
pub fn resolve_active_change_id(project_root: &Path) -> Result<String> {
    let changes_dir = project_root.join(".aw/changes");
    if !changes_dir.exists() {
        anyhow::bail!(
            "No .aw/changes/ directory found. Start a change with `score run-change`."
        );
    }

    let mut active = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&changes_dir) {
        for entry in entries.flatten() {
            let state_path = entry.path().join("STATE.yaml");
            if !state_path.exists() {
                continue;
            }
            if let Ok(sm) = StateManager::load(&entry.path()) {
                let phase = sm.phase();
                if !matches!(
                    phase,
                    StatePhase::ChangeArchived | StatePhase::ChangeRejected
                ) {
                    active.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
    }

    match active.len() {
        0 => anyhow::bail!("No active change on this branch. Start one with `score run-change`."),
        1 => Ok(active.into_iter().next().unwrap()),
        n => anyhow::bail!(
            "Multiple active changes found ({}): {}. Pass --change-id explicitly.",
            n,
            active.join(", ")
        ),
    }
}

/// Resolve the change directory for a given change_id.
///
/// Checks two locations (in order):
/// 1. Worktree: `project_root/.aw/worktrees/{id}/.aw/changes/{id}/`
/// 2. Legacy (main): `project_root/.aw/changes/{id}/`
///
/// Returns the first path that exists, or the worktree path as default for new changes.
pub fn resolve_change_dir(project_root: &Path, change_id: &str) -> PathBuf {
    // Worktree path (preferred)
    let wt_path = project_root
        .join(".aw/worktrees")
        .join(change_id)
        .join(".aw/changes")
        .join(change_id);
    if wt_path.exists() {
        return wt_path;
    }

    // Legacy path (main branch)
    let legacy_path = project_root.join(".aw/changes").join(change_id);
    if legacy_path.exists() {
        return legacy_path;
    }

    // Default to legacy path (compatible with tests and non-worktree envs).
    // init_change explicitly uses worktree root when creating new changes.
    legacy_path
}

/// Convert StatePhase to string for JSON output
pub fn phase_to_string(phase: &StatePhase) -> &'static str {
    super::phase_transition::phase_to_string(phase)
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
      - "<module-preamble>"
      - "validate_change_id"
      - "validate_change_dir"
      - "resolve_active_change_id"
      - "resolve_change_dir"
      - "phase_to_string"
    description: "Module preamble, change validation, active change lookup, and change directory resolution helpers."
```
