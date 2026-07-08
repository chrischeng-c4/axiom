---
id: projects-sdd-src-shared-workspace-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Shared workflow utilities are part of the AW Core protocol support surface used across clients and lifecycle phases."
---

# Standardized apps/agentic-workflow/src/shared/workspace.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/shared/workspace.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ARCHIVE_DIR` | apps/agentic-workflow/src/shared/workspace.rs | constant | pub | 49 |  |
| `AW_TMP_ROOT` | apps/agentic-workflow/src/shared/workspace.rs | constant | pub | 43 |  |
| `CHANGES_DIR` | apps/agentic-workflow/src/shared/workspace.rs | constant | pub | 46 |  |
| `CONFIG_FILE` | apps/agentic-workflow/src/shared/workspace.rs | constant | pub | 20 |  |
| `ISSUES_DIR` | apps/agentic-workflow/src/shared/workspace.rs | constant | pub | 34 |  |
| `PAYLOADS_DIR` | apps/agentic-workflow/src/shared/workspace.rs | constant | pub | 40 |  |
| `SYNC_BEGIN_MARKER` | apps/agentic-workflow/src/shared/workspace.rs | constant | pub | 24 |  |
| `SYNC_END_MARKER` | apps/agentic-workflow/src/shared/workspace.rs | constant | pub | 28 |  |
| `TECH_DESIGN_DIR` | apps/agentic-workflow/src/shared/workspace.rs | constant | pub | 31 |  |
| `WORKITEMS_DIR` | apps/agentic-workflow/src/shared/workspace.rs | constant | pub | 37 |  |
| `WORKSPACE_DIR` | apps/agentic-workflow/src/shared/workspace.rs | constant | pub | 17 |  |
| `archive_path` | apps/agentic-workflow/src/shared/workspace.rs | function | pub | 236 | archive_path(project_root: &Path) -> PathBuf |
| `aw_tmp_path` | apps/agentic-workflow/src/shared/workspace.rs | function | pub | 59 | aw_tmp_path() -> PathBuf |
| `change_path` | apps/agentic-workflow/src/shared/workspace.rs | function | pub | 230 | change_path(project_root: &Path, change_id: &str) -> PathBuf |
| `changes_path` | apps/agentic-workflow/src/shared/workspace.rs | function | pub | 224 | changes_path(project_root: &Path) -> PathBuf |
| `config_path` | apps/agentic-workflow/src/shared/workspace.rs | function | pub | 74 | config_path(project_root: &Path) -> PathBuf |
| `issues_path` | apps/agentic-workflow/src/shared/workspace.rs | function | pub | 157 | issues_path(project_root: &Path) -> PathBuf |
| `payloads_path` | apps/agentic-workflow/src/shared/workspace.rs | function | pub | 176 | payloads_path(project_root: &Path) -> PathBuf |
| `project_tech_design_paths` | apps/agentic-workflow/src/shared/workspace.rs | function | pub | 95 | project_tech_design_paths(project_root: &Path) -> Vec<(String, PathBuf)> |
| `tech_design_path` | apps/agentic-workflow/src/shared/workspace.rs | function | pub | 84 | tech_design_path(project_root: &Path) -> PathBuf |
| `workitems_path` | apps/agentic-workflow/src/shared/workspace.rs | function | pub | 164 | workitems_path(project_root: &Path) -> PathBuf |
| `workspace_path` | apps/agentic-workflow/src/shared/workspace.rs | function | pub | 53 | workspace_path(project_root: &Path) -> PathBuf |
| `workspace_runtime_path` | apps/agentic-workflow/src/shared/workspace.rs | function | pub | 66 | workspace_runtime_path(project_root: &Path) -> PathBuf |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=apps/agentic-workflow/src/shared/workspace.rs -->
```rust
// SPEC-MANAGED: apps/agentic-workflow/tech-design/core/interfaces/shared/workspace.md#source
// CODEGEN-BEGIN
//! Agentic Workflow workspace directory constants and path helpers.
//!
//! Versioned Agentic Workflow state lives under a single top-level `.aw/`
//! directory in the consumer's project root, following the dev-tool convention
//! of hidden dot-prefix state dirs (`.git/`, `.cargo/`, `.claude/`).
//! Ephemeral runtime/cache state lives under `/tmp/aw`.
//!
//! Centralizing these constants here makes future renames a one-line change.

use std::path::{Path, PathBuf};

use crate::services::project_registry::resolve_td_root_from_config;

/// Top-level workspace directory name.
pub const WORKSPACE_DIR: &str = ".aw";

/// Config file name (inside workspace dir).
pub const CONFIG_FILE: &str = "config.toml";

/// Begin marker for the auto-generated [[projects]] block in config.toml.
// @spec apps/agentic-workflow/tech-design/surface/specs/sync-command.md#R2
pub const SYNC_BEGIN_MARKER: &str = "# BEGIN AW SYNC \u{2014} auto-generated, do not edit by hand";

/// End marker for the auto-generated [[projects]] block in config.toml.
// @spec apps/agentic-workflow/tech-design/surface/specs/sync-command.md#R2
pub const SYNC_END_MARKER: &str = "# END AW SYNC";

/// Tech design artifact directory (previously "specs").
pub const TECH_DESIGN_DIR: &str = "tech-design";

/// Local issue artifact directory (pre-tracker).
pub const ISSUES_DIR: &str = "issues";

/// Work-item draft and planning artifact directory.
pub const WORKITEMS_DIR: &str = "workitems";

/// Ephemeral payload round-trip artifact directory.
pub const PAYLOADS_DIR: &str = "payloads";

/// Agentic Workflow runtime/cache root.
pub const AW_TMP_ROOT: &str = "/tmp/aw";

/// In-flight change directory.
pub const CHANGES_DIR: &str = "changes";

/// Completed change archive directory.
pub const ARCHIVE_DIR: &str = "archive";

/// Path to the workspace root: `{project_root}/.aw`
/// @spec apps/agentic-workflow/tech-design/core/interfaces/shared/workspace.md#source
pub fn workspace_path(project_root: &Path) -> PathBuf {
    project_root.join(WORKSPACE_DIR)
}

/// Path to the runtime/cache root: `/tmp/aw`.
/// @spec apps/agentic-workflow/tech-design/core/interfaces/shared/workspace.md#source
pub fn aw_tmp_path() -> PathBuf {
    PathBuf::from(AW_TMP_ROOT)
}

/// Path to the workspace-scoped runtime/cache root:
/// `/tmp/aw/workspaces/<workspace>`.
/// @spec apps/agentic-workflow/tech-design/core/interfaces/shared/workspace.md#source
pub fn workspace_runtime_path(project_root: &Path) -> PathBuf {
    aw_tmp_path()
        .join("workspaces")
        .join(workspace_cache_slug(project_root))
}

/// Path to the config file: `{project_root}/.aw/config.toml`
/// @spec apps/agentic-workflow/tech-design/core/interfaces/shared/workspace.md#source
pub fn config_path(project_root: &Path) -> PathBuf {
    workspace_path(project_root).join(CONFIG_FILE)
}

/// Path to the default tech design directory.
///
/// Reads `[agentic_workflow.tech_design_platform].path` from `.aw/config.toml` when it
/// is present, and falls back to `{project_root}/.aw/tech-design` for
/// workspaces.
/// @spec apps/agentic-workflow/tech-design/core/interfaces/shared/workspace.md#source
pub fn tech_design_path(project_root: &Path) -> PathBuf {
    configured_tech_design_base(project_root)
        .unwrap_or_else(|| workspace_path(project_root).join(TECH_DESIGN_DIR))
}

/// Return all registered project TD roots resolved from `.aw/config.toml`.
///
/// Per-project `td_path` values are resolved by the same primitive used by TD
/// creation. Invalid rows are skipped here so callers can still render the
/// resolvable subset of a partially edited config.
/// @spec apps/agentic-workflow/tech-design/core/interfaces/shared/workspace.md#source
pub fn project_tech_design_paths(project_root: &Path) -> Vec<(String, PathBuf)> {
    #[derive(serde::Deserialize, Default)]
    struct Config {
        #[serde(default)]
        projects: Vec<ProjectRow>,
    }

    #[derive(serde::Deserialize)]
    struct ProjectRow {
        name: String,
    }

    let config_file = config_path(project_root);
    let Ok(content) = std::fs::read_to_string(config_file) else {
        return vec![];
    };
    let Ok(parsed) = toml::from_str::<Config>(&content) else {
        return vec![];
    };

    parsed
        .projects
        .into_iter()
        .filter_map(|project| {
            let resolved = resolve_td_root_from_config(project_root, &project.name).ok()?;
            Some((project.name, PathBuf::from(resolved.root)))
        })
        .collect()
}

fn configured_tech_design_base(project_root: &Path) -> Option<PathBuf> {
    #[derive(serde::Deserialize, Default)]
    struct Config {
        #[serde(default)]
        agentic_workflow: SddSection,
    }

    #[derive(serde::Deserialize, Default)]
    struct SddSection {
        #[serde(default)]
        tech_design_platform: Option<TdPlatform>,
    }

    #[derive(serde::Deserialize, Default)]
    struct TdPlatform {
        #[serde(default)]
        path: Option<String>,
    }

    let content = std::fs::read_to_string(config_path(project_root)).ok()?;
    let parsed = toml::from_str::<Config>(&content).ok()?;
    let path = parsed.agentic_workflow.tech_design_platform?.path?;
    if path.is_empty() {
        None
    } else {
        Some(project_root.join(path))
    }
}

/// Path to the ephemeral issue working-copy directory:
/// `/tmp/aw/workspaces/<workspace>/issues`.
/// @spec apps/agentic-workflow/tech-design/core/interfaces/shared/workspace.md#source
pub fn issues_path(project_root: &Path) -> PathBuf {
    workspace_runtime_path(project_root).join(ISSUES_DIR)
}

/// Path to the ephemeral work-item draft/planning directory:
/// `/tmp/aw/workspaces/<workspace>/workitems`.
/// @spec apps/agentic-workflow/tech-design/core/interfaces/shared/workspace.md#source
pub fn workitems_path(project_root: &Path) -> PathBuf {
    workspace_runtime_path(project_root).join(WORKITEMS_DIR)
}

/// Path to the ephemeral payload round-trip directory:
/// `/tmp/aw/workspaces/<workspace>/payloads`.
///
/// CRRR round-trip fragments (TD section drafts, HANDWRITE marker fills, EC
/// draft sections, work-item fill-section/review bodies) are consumed
/// within a single agent turn and are never git-tracked, so they live here
/// alongside `issues_path` rather than under the project's `.aw/` tree.
/// @spec apps/agentic-workflow/tech-design/core/interfaces/shared/workspace.md#source
pub fn payloads_path(project_root: &Path) -> PathBuf {
    workspace_runtime_path(project_root).join(PAYLOADS_DIR)
}

fn workspace_cache_slug(project_root: &Path) -> String {
    let identity_root = issue_workspace_identity_root(project_root);
    let resolved = identity_root.canonicalize().unwrap_or(identity_root);
    let raw = resolved.to_string_lossy();
    let mut out = String::with_capacity(raw.len());
    let mut last_dash = true;
    for c in raw.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "workspace".to_string()
    } else {
        trimmed.to_string()
    }
}

fn issue_workspace_identity_root(project_root: &Path) -> PathBuf {
    let Some(worktrees_dir) = project_root.parent() else {
        return project_root.to_path_buf();
    };
    if worktrees_dir.file_name().and_then(|name| name.to_str()) != Some("worktrees") {
        return project_root.to_path_buf();
    }
    let Some(aw_dir) = worktrees_dir.parent() else {
        return project_root.to_path_buf();
    };
    if aw_dir.file_name().and_then(|name| name.to_str()) != Some(WORKSPACE_DIR) {
        return project_root.to_path_buf();
    }
    aw_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| project_root.to_path_buf())
}

/// Path to the changes directory: `{project_root}/.aw/changes`
/// @spec apps/agentic-workflow/tech-design/core/interfaces/shared/workspace.md#source
pub fn changes_path(project_root: &Path) -> PathBuf {
    workspace_path(project_root).join(CHANGES_DIR)
}

/// Path to a specific change directory: `{project_root}/.aw/changes/{change_id}`
/// @spec apps/agentic-workflow/tech-design/core/interfaces/shared/workspace.md#source
pub fn change_path(project_root: &Path, change_id: &str) -> PathBuf {
    changes_path(project_root).join(change_id)
}

/// Path to the archive directory: `{project_root}/.aw/archive`
/// @spec apps/agentic-workflow/tech-design/core/interfaces/shared/workspace.md#source
pub fn archive_path(project_root: &Path) -> PathBuf {
    workspace_path(project_root).join(ARCHIVE_DIR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issues_path_uses_main_checkout_identity_for_aw_worktrees() {
        let tmp = tempfile::TempDir::new().unwrap();
        let main_root = tmp.path();
        let worktree_root = main_root.join(".aw/worktrees/change-slug");

        assert_eq!(issues_path(&worktree_root), issues_path(main_root));
    }

    #[test]
    fn payloads_path_uses_main_checkout_identity_for_aw_worktrees() {
        let tmp = tempfile::TempDir::new().unwrap();
        let main_root = tmp.path();
        let worktree_root = main_root.join(".aw/worktrees/change-slug");

        assert_eq!(payloads_path(&worktree_root), payloads_path(main_root));
    }

    #[test]
    fn payloads_path_lives_under_aw_tmp_root_alongside_issues() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();

        let payloads = payloads_path(root);
        assert!(payloads.starts_with(aw_tmp_path().join("workspaces")));
        assert_eq!(payloads.file_name().unwrap(), PAYLOADS_DIR);
        // Same workspace slug as issues_path — siblings under one workspace dir.
        assert_eq!(payloads.parent(), issues_path(root).parent());
    }

    #[test]
    fn workitems_path_lives_under_workspace_runtime_root() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();

        let workitems = workitems_path(root);
        assert!(workitems.starts_with(aw_tmp_path().join("workspaces")));
        assert_eq!(workitems.file_name().unwrap(), WORKITEMS_DIR);
        assert_eq!(workitems.parent(), payloads_path(root).parent());
        assert_eq!(workitems.parent(), issues_path(root).parent());
    }
}

// CODEGEN-END
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/shared/workspace.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete shared workspace constants and helpers.
```
