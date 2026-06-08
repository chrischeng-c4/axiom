---
id: projects-score-src-hook-rs
fill_sections: [overview, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Support CLI surfaces expose AW Core bootstrap, chat, hook, project, and workspace invariants."
---

# Standardized projects/agentic-workflow/src/cli/hook.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/cli/hook.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `HookArgs` | projects/agentic-workflow/src/cli/hook.rs | struct | pub | 26 |  |
| `HookEvent` | projects/agentic-workflow/src/cli/hook.rs | enum | pub | 33 |  |
| `PosttooluseKind` | projects/agentic-workflow/src/cli/hook.rs | enum | pub | 61 |  |
| `PretooluseKind` | projects/agentic-workflow/src/cli/hook.rs | enum | pub | 48 |  |
| `run` | projects/agentic-workflow/src/cli/hook.rs | function | pub | 81 | run(args: HookArgs) -> Result<()> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/cli/hook.rs -->
```rust
//! Internal Claude Code hook handlers.
//!
//! Reads a hook payload JSON from stdin, applies an event-/kind-specific
//! decision, and emits the Claude Code hook contract on stdout/stderr
//! with exit code 0 (allow) or 2 (block). Fail-open on every error
//! path: any unexpected condition produces exit 0 with a single-line
//! `aw-hook: <reason>` stderr warning. Hook failures must never
//! silently block edits.
//!
//! @spec projects/agentic-workflow/tech-design/core/specs/score-hook-pretooluse-write-scope.md

use std::io::Read;
use std::panic;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::services::path_scope::{self, AllowedScope};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use serde_json::Value;

#[derive(Debug, Args)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/hook.md#source
pub struct HookArgs {
    #[command(subcommand)]
    pub event: HookEvent,
}

#[derive(Debug, Subcommand)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/hook.md#source
pub enum HookEvent {
    /// PreToolUse hook handlers
    Pretooluse {
        #[command(subcommand)]
        kind: PretooluseKind,
    },
    /// PostToolUse hook handlers
    Posttooluse {
        #[command(subcommand)]
        kind: PosttooluseKind,
    },
}

#[derive(Debug, Subcommand)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/hook.md#source
pub enum PretooluseKind {
    /// Enforce project-<name> branch ↔ edit scope per .aw/config.toml
    /// [[projects]] entry. Reads PreToolUse JSON on stdin; emits
    /// `{"decision":"block","reason":"..."}` + exit 2 to block, or
    /// empty stdout + exit 0 to allow. Fail-open on every error.
    WriteScope,
    /// Enforce WI workflow projection locks for TD/CB payload writes and
    /// mutating Agentic Workflow commands.
    WorkflowGuard,
}

#[derive(Debug, Subcommand)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/hook.md#source
pub enum PosttooluseKind {
    /// When the expected TD/CB payload was written, run the fixed command
    /// recorded in the WI workflow projection.
    WorkflowApply,
}

// Outcome of a hook handler — encodes the Claude Code hook contract.
enum Decision {
    /// Allow the tool call. stdout empty, exit 0.
    Allow,
    /// Allow the tool call and surface hook-produced output.
    AllowWithOutput(String),
    /// Block the tool call. stdout = `{"decision":"block","reason":...}`,
    /// exit 2.
    Block(String),
    /// Fail-open. stdout empty, stderr `aw-hook: <reason>`, exit 0.
    FailOpen(String),
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/hook.md#source
pub async fn run(args: HookArgs) -> Result<()> {
    let decision = match args.event {
        HookEvent::Pretooluse { kind } => match kind {
            PretooluseKind::WriteScope => run_write_scope_guarded(),
            PretooluseKind::WorkflowGuard => run_workflow_guard().await,
        },
        HookEvent::Posttooluse { kind } => match kind {
            PosttooluseKind::WorkflowApply => run_workflow_apply().await,
        },
    };
    emit_and_exit(decision);
}

async fn run_workflow_guard() -> Decision {
    match read_json_payload() {
        Ok(payload) => match crate::workflow_guard::hook_pretooluse_workflow_guard(
            &std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            &payload,
        )
        .await
        {
            Ok(decision) => workflow_hook_decision(decision),
            Err(e) => Decision::FailOpen(format!("workflow guard failed: {e}")),
        },
        Err(_) => Decision::Allow,
    }
}

async fn run_workflow_apply() -> Decision {
    match read_json_payload() {
        Ok(payload) => match crate::workflow_guard::hook_posttooluse_workflow_apply(
            &std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            &payload,
        )
        .await
        {
            Ok(decision) => workflow_hook_decision(decision),
            Err(e) => Decision::FailOpen(format!("workflow apply failed: {e}")),
        },
        Err(_) => Decision::Allow,
    }
}

fn read_json_payload() -> Result<Value> {
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .context("failed to read hook stdin")?;
    serde_json::from_str(&buf).context("failed to parse hook JSON")
}

fn workflow_hook_decision(decision: crate::workflow_guard::HookDecision) -> Decision {
    match decision {
        crate::workflow_guard::HookDecision::Allow => Decision::Allow,
        crate::workflow_guard::HookDecision::AllowWithOutput(out) => Decision::AllowWithOutput(out),
        crate::workflow_guard::HookDecision::Block(reason) => Decision::Block(reason),
    }
}

// Wrap the handler in `catch_unwind` so any panic inside our logic
// produces exit 0 + a fail-open stderr warning instead of a process
// abort. Per R4 and TP-11, the hook must NEVER silently block edits
// because the hook itself is broken.
fn run_write_scope_guarded() -> Decision {
    match panic::catch_unwind(panic::AssertUnwindSafe(write_scope::decide)) {
        Ok(d) => d,
        Err(payload) => {
            let msg = panic_message(&payload);
            Decision::FailOpen(format!("panic: {msg}"))
        }
    }
}

fn panic_message(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&'static str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic payload".to_string()
    }
}

fn emit_and_exit(decision: Decision) -> ! {
    match decision {
        Decision::Allow => std::process::exit(0),
        Decision::AllowWithOutput(output) => {
            if !output.trim().is_empty() {
                println!("{}", output.trim_end());
            }
            std::process::exit(0);
        }
        Decision::Block(reason) => {
            let body = serde_json::json!({ "decision": "block", "reason": reason });
            println!("{}", body);
            std::process::exit(2);
        }
        Decision::FailOpen(reason) => {
            eprintln!("aw-hook: {reason}");
            std::process::exit(0);
        }
    }
}

mod write_scope {
    use super::*;

    /// Pure decision function for the internal pretooluse write-scope hook.
    /// All I/O (stdin read, git invocations, TOML load) happens inside;
    /// the only side effect is reading stdin once. Returns the
    /// Decision; the caller emits + exits.
    pub fn decide() -> Decision {
        // Step 1: read stdin. JSON parse error → allow (matches the
        // .py stopgap; this is "we have nothing to scope, so don't
        // block"). This is fail-open for the absent-payload case but
        // not flagged as a fail-open warning because invalid stdin is
        // expected for non-Edit invocations.
        let mut buf = String::new();
        if std::io::stdin().read_to_string(&mut buf).is_err() {
            return Decision::Allow;
        }
        let payload: Value = match serde_json::from_str(&buf) {
            Ok(v) => v,
            Err(_) => return Decision::Allow,
        };

        // Step 2: extract tool_input.file_path. Missing/empty → allow.
        let file_path = match payload
            .get("tool_input")
            .and_then(|t| t.get("file_path"))
            .and_then(|v| v.as_str())
        {
            Some(s) if !s.is_empty() => PathBuf::from(s),
            _ => return Decision::Allow,
        };
        let abs_target = if file_path.is_absolute() {
            file_path.clone()
        } else {
            std::env::current_dir().unwrap_or_default().join(&file_path)
        };

        // Step 3: locate the git repo via the closest existing
        // ancestor of the target (Write may target a not-yet-existing
        // file).
        let probe = closest_existing_ancestor(&abs_target);
        let repo_root = match git_show_toplevel(&probe) {
            Some(p) => p,
            None => return Decision::Allow, // not in a git repo
        };

        // Step 4: current branch. Empty / detached HEAD → allow.
        let branch = match git_current_branch(&repo_root) {
            Some(b) if !b.is_empty() => b,
            _ => return Decision::Allow,
        };

        // Step 5: only project-<name> branches are scoped. main,
        // issue-*, td-*, cb-*, etc. all unscoped (R6).
        let project_name = match branch.strip_prefix("project-") {
            Some(rest) if !rest.is_empty() => rest.to_string(),
            _ => return Decision::Allow,
        };

        // Step 6: compute target path relative to repo root. Outside
        // the repo → not our concern, allow.
        let resolved = resolve_existing(&abs_target);
        let canon_root = repo_root.canonicalize().unwrap_or(repo_root.clone());
        let rel = match resolved.strip_prefix(&canon_root) {
            Ok(p) => p.to_string_lossy().replace('\\', "/"),
            Err(_) => return Decision::Allow,
        };

        // Step 7: load .aw/config.toml. Missing → allow (a repo
        // without a config has no scopes to enforce). Parse error →
        // fail-open with stderr (R4 + TP-8 — diverges from .py which
        // blocked on parse error).
        let cfg = match path_scope::load_scope(&canon_root) {
            Ok(Some(c)) => c,
            Ok(None) => return Decision::Allow,
            Err(e) => {
                return Decision::FailOpen(format!("failed to load .aw/config.toml: {e}"));
            }
        };

        // Step 8: locate matching [[projects]] entry. Missing → BLOCK
        // (the branch claims to scope to a project that does not exist
        // — most likely a typo or stale branch).
        let project = match path_scope::project_by_name(&cfg, &project_name) {
            Some(p) => p,
            None => {
                return Decision::Block(format!(
                    "branch '{branch}' has no matching [[projects]] entry \
                     (name='{project_name}') in .aw/config.toml — refusing edit of {rel}"
                ));
            }
        };

        // Step 9: build the scope and check the target.
        let scope = match AllowedScope::for_project(project) {
            Ok(s) => s,
            Err(e) => {
                return Decision::FailOpen(format!(
                    "failed to build allowed-scope matcher for project '{project_name}': {e}"
                ));
            }
        };
        if scope.contains(&rel) {
            Decision::Allow
        } else {
            Decision::Block(format!(
                "branch '{branch}' restricts edits to {} (per .aw/config.toml \
                 [[projects]] name='{project_name}'); got: {rel}",
                scope.describe()
            ))
        }
    }

    /// Walk up from `p` to the closest existing directory ancestor.
    /// Used to locate the git repo when the target file does not yet
    /// exist (Write of a new file).
    fn closest_existing_ancestor(p: &Path) -> PathBuf {
        let mut probe = p.to_path_buf();
        while !probe.is_dir() {
            match probe.parent() {
                Some(parent) if parent != probe => probe = parent.to_path_buf(),
                _ => return std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            }
        }
        probe
    }

    /// Resolve symlinks via the closest existing ancestor — handles
    /// macOS /tmp → /private/tmp for not-yet-existing files.
    fn resolve_existing(p: &Path) -> PathBuf {
        let mut parent = match p.parent() {
            Some(par) => par.to_path_buf(),
            None => return p.to_path_buf(),
        };
        while !parent.exists() {
            match parent.parent() {
                Some(par) if par != parent => parent = par.to_path_buf(),
                _ => return p.to_path_buf(),
            }
        }
        let canon = parent.canonicalize().unwrap_or(parent.clone());
        match p.strip_prefix(&parent) {
            Ok(rest) => canon.join(rest),
            Err(_) => p.to_path_buf(),
        }
    }

    fn git_show_toplevel(cwd: &Path) -> Option<PathBuf> {
        let out = Command::new("git")
            .args(["-C", cwd.to_str()?, "rev-parse", "--show-toplevel"])
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
        if s.is_empty() {
            None
        } else {
            Some(PathBuf::from(s))
        }
    }

    fn git_current_branch(cwd: &Path) -> Option<String> {
        let out = Command::new("git")
            .args(["-C", cwd.to_str()?, "branch", "--show-current"])
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        Some(String::from_utf8(out.stdout).ok()?.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the dispatcher shell.

    use super::*;

    #[test]
    fn panic_message_extracts_str() {
        let payload: Box<dyn std::any::Any + Send> = Box::new("boom");
        assert_eq!(panic_message(&payload), "boom");
    }

    #[test]
    fn panic_message_extracts_string() {
        let payload: Box<dyn std::any::Any + Send> = Box::new(String::from("kaboom"));
        assert_eq!(panic_message(&payload), "kaboom");
    }

    #[test]
    fn panic_message_unknown_payload() {
        let payload: Box<dyn std::any::Any + Send> = Box::new(42_i32);
        assert_eq!(panic_message(&payload), "unknown panic payload");
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/hook.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Add workflow guard/apply hook subcommands and delegate lifecycle decisions
      to workflow_guard.
```
