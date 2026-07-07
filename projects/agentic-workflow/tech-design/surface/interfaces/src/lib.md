---
id: projects-score-src-lib-rs
fill_sections: [overview, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Support CLI surfaces expose AW Core bootstrap, chat, hook, project, and workspace invariants."
---

# Standardized projects/agentic-workflow/src/cli/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/cli/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `capability` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 9 |  |
| `capability_type` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 10 |  |
| `cb` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 11 |  |
| `cb_fill` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 12 |  |
| `chain` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 13 |  |
| `chat` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 15 |  |
| `check_alignment` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 16 |  |
| `commands` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 17 |  |
| `doc_mirror` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 18 |  |
| `ec` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 19 |  |
| `fillback` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 20 |  |
| `find_project_root` | projects/agentic-workflow/src/cli/mod.rs | function | pub | 76 | find_project_root() -> anyhow::Result<std::path::PathBuf> |
| `generator` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 21 |  |
| `guard` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 22 |  |
| `hook` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 23 |  |
| `init` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 24 |  |
| `issues` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 25 |  |
| `llm` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 26 |  |
| `loop_state` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 27 |  |
| `merge_target` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 53 |  |
| `migrate` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 48 |  |
| `production` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 28 |  |
| `project` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 29 |  |
| `regenerability_policy` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 30 |  |
| `remote_push` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 31 |  |
| `run` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 32 |  |
| `shell_env` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 33 |  |
| `slug_workspace` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 34 |  |
| `standard_cli` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 35 |  |
| `standardize` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 36 |  |
| `sync` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 37 |  |
| `td` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 38 |  |
| `td_check_section_type` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 39 |  |
| `td_lock` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 40 |  |
| `td_migrate` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 41 |  |
| `update` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 42 |  |
| `validate_spec_structure` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 43 |  |
| `view` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 44 |  |
| `workflow_guard` | projects/agentic-workflow/src/cli/mod.rs | module | pub | 45 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/cli/mod.rs -->
```rust
// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/lib.md#source
// CODEGEN-BEGIN
//! Agentic Workflow — local spec-governed workflow orchestrator.
//!
//! Standalone binary + library. The binary entry point is `src/bin/aw.rs`;
//! this library exposes the `Commands` enum and `run_command` dispatch for
//! programmatic consumers and for the binary itself.

pub mod capability;
pub mod capability_type;
pub mod cb;
pub mod cb_fill;
pub(crate) mod chain;
#[path = "chat.rs"]
pub mod chat;
pub mod check_alignment;
pub mod commands;
pub mod conf;
pub mod doc_mirror;
pub mod ec;
pub mod fillback;
pub mod generator;
pub mod guard;
pub mod hook;
pub mod init;
pub mod issues;
pub mod llm;
pub mod loop_state;
pub mod production;
pub mod project;
pub mod regenerability_policy;
pub mod remote_push;
pub mod run;
pub(crate) mod shell_env;
pub mod slug_workspace;
pub mod standard_cli;
pub mod standardize;
pub mod td;
pub mod td_check_section_type;
pub mod td_lock;
pub mod td_migrate;
pub mod update;
pub mod validate_spec_structure;
pub mod view;
pub mod workflow_guard;

// Legacy modules kept for init.rs / update.rs dependencies
pub(crate) mod migrate;

// Legacy merge-target resolution logic retained for older merge-style recovery
// surfaces while those APIs age out.
// Public so integration tests in tests/ can call resolve_merge_target directly.
pub mod merge_target;

// Public library API — the binary (src/main.rs) and any programmatic consumer
// call into these.
pub use commands::{run_command, Commands};

const LEGACY_SCORE_WORKSPACE_DIR: &str = concat!(".", "score");

fn legacy_score_workspace_error(root: &std::path::Path) -> anyhow::Error {
    anyhow::anyhow!(
        "legacy Agentic Workflow state found at {}; active state now lives under .aw. Move or remove the old directory explicitly, then rerun this command.",
        root.join(LEGACY_SCORE_WORKSPACE_DIR).display()
    )
}

// Find the project root by walking up from CWD looking for `.aw/config.toml`.
// Falls back to CWD if no `.aw/` is found (e.g., during greenfield setup).
///
// This intentionally returns the repo root for the CLI process's current
// working tree. In a git linked-worktree checkout, do not use shared git
// metadata to redirect to another checkout: mutating Agentic Workflow commands must write
// to the `.aw/` tree visible from the user's current checkout.
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/lib.md#source
pub fn find_project_root() -> anyhow::Result<std::path::PathBuf> {
    let cwd = std::env::current_dir()?;
    let mut dir = cwd.as_path();
    let mut legacy_root = None;
    loop {
        if dir.join(".aw/config.toml").exists() {
            if dir.join(LEGACY_SCORE_WORKSPACE_DIR).exists() {
                return Err(legacy_score_workspace_error(dir));
            }
            return Ok(dir.to_path_buf());
        }
        if dir.join(LEGACY_SCORE_WORKSPACE_DIR).exists() && legacy_root.is_none() {
            legacy_root = Some(dir.to_path_buf());
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => {
                if let Some(root) = legacy_root {
                    return Err(legacy_score_workspace_error(&root));
                }
                // No .aw/ found — fall back to CWD for uninitialized repos.
                return Ok(cwd);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Mutex;

    // find_project_root() reads process-global CWD; serialize tests that
    // mutate it so parallel cargo test runs don't race.
    static CWD_LOCK: Mutex<()> = Mutex::new(());

    fn git_available() -> bool {
        std::process::Command::new("git")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn init_git_repo(path: &std::path::Path) {
        for args in [
            vec!["init", "-q", "-b", "main"],
            vec!["config", "user.email", "test@example.com"],
            vec!["config", "user.name", "Test"],
            vec!["commit", "--allow-empty", "-m", "init", "-q"],
        ] {
            let out = std::process::Command::new("git")
                .args(&args)
                .current_dir(path)
                .output()
                .expect("git command");
            assert!(
                out.status.success(),
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }

    // Canonicalize via `std::fs::canonicalize`. On macOS, tempdir paths go
    // through /private/var/… so comparisons need canonicalized values on both
    // sides.
    fn canon(p: &std::path::Path) -> PathBuf {
        std::fs::canonicalize(p).unwrap()
    }

    #[test]
    fn find_project_root_inside_worktree_returns_current_worktree() {
        if !git_available() {
            return;
        }
        let _guard = CWD_LOCK.lock().unwrap();
        let prev = std::env::current_dir().unwrap();

        let tmp = tempfile::TempDir::new().unwrap();
        let main_repo = tmp.path().join("main");
        std::fs::create_dir_all(&main_repo).unwrap();
        init_git_repo(&main_repo);
        std::fs::create_dir_all(main_repo.join(".aw")).unwrap();
        std::fs::write(main_repo.join(".aw/config.toml"), "").unwrap();

        let worktree_rel = "../linked/foo";
        let out = std::process::Command::new("git")
            .args(["worktree", "add", "-b", "cclab/foo", worktree_rel])
            .current_dir(&main_repo)
            .output()
            .expect("git worktree add");
        assert!(
            out.status.success(),
            "worktree add: {}",
            String::from_utf8_lossy(&out.stderr)
        );

        let worktree = main_repo.join(worktree_rel);
        std::fs::create_dir_all(worktree.join(".aw")).unwrap();
        std::fs::write(worktree.join(".aw/config.toml"), "").unwrap();

        std::env::set_current_dir(&worktree).unwrap();
        let resolved = find_project_root().unwrap();
        std::env::set_current_dir(&prev).unwrap();

        assert_eq!(
            canon(&resolved),
            canon(&worktree),
            "linked worktree CWD must resolve to the current worktree root"
        );
    }

    #[test]
    fn find_project_root_inside_main_repo_unchanged() {
        if !git_available() {
            return;
        }
        let _guard = CWD_LOCK.lock().unwrap();
        let prev = std::env::current_dir().unwrap();

        let tmp = tempfile::TempDir::new().unwrap();
        let repo = tmp.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        init_git_repo(&repo);
        std::fs::create_dir_all(repo.join(".aw")).unwrap();
        std::fs::write(repo.join(".aw/config.toml"), "").unwrap();
        let subdir = repo.join("crates/demo");
        std::fs::create_dir_all(&subdir).unwrap();

        std::env::set_current_dir(&repo).unwrap();
        let from_root = find_project_root().unwrap();
        std::env::set_current_dir(&subdir).unwrap();
        let from_subdir = find_project_root().unwrap();
        std::env::set_current_dir(&prev).unwrap();

        assert_eq!(canon(&from_root), canon(&repo));
        assert_eq!(canon(&from_subdir), canon(&repo));
    }

    #[test]
    fn find_project_root_non_git_tempdir_walks_up() {
        let _guard = CWD_LOCK.lock().unwrap();
        let prev = std::env::current_dir().unwrap();

        let tmp = tempfile::TempDir::new().unwrap();
        let proj = tmp.path().join("proj");
        std::fs::create_dir_all(proj.join(".aw")).unwrap();
        std::fs::write(proj.join(".aw/config.toml"), "").unwrap();

        std::env::set_current_dir(&proj).unwrap();
        let resolved = find_project_root().unwrap();
        std::env::set_current_dir(&prev).unwrap();

        assert_eq!(
            canon(&resolved),
            canon(&proj),
            "non-git context must fall back to walk-up"
        );
    }

    #[test]
    fn find_project_root_signature_unchanged_compile_check() {
        let _f: fn() -> anyhow::Result<std::path::PathBuf> = find_project_root;
    }
}

// CODEGEN-END

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/mod.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
  - path: projects/agentic-workflow/src/cli/mod.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Issue #848: whole-file resync to the current module list (adds
      capability_type, chain, ec, guard, llm, loop_state, shell_env,
      standard_cli, sync, td_lock, view; drops the removed cb_arbitrate,
      cb_review, cb_revise modules) and current find_project_root body.
  - path: projects/agentic-workflow/src/cli/mod.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Issue #984 (init-projector slice 1/3): registers the new `doc_mirror`
      module — the one shared whitelist definition consumed by both `aw
      init`'s AGENTS.md projection and `root_doc_mirror_test`.
```
