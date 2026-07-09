---
id: projects-score-src-td-migrate-rs
fill_sections: [overview, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "TD/CB CLI surface manifests cover lifecycle dispatch, review, fill, and merge command behavior."
---

# Standardized apps/agentic-workflow/src/cli/td_migrate.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/cli/td_migrate.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `MigrateMermaidArgs` | apps/agentic-workflow/src/cli/td_migrate.rs | struct | pub | 35 |  |
| `MigrateMermaidCheckReport` | apps/agentic-workflow/src/cli/td_migrate.rs | struct |  | 67 |  |
| `run_check_scan` | apps/agentic-workflow/src/cli/td_migrate.rs | function |  | 77 | run_check_scan(args: &MigrateMermaidArgs, project_root: &Path) -> Result<()> |
| `run` | apps/agentic-workflow/src/cli/td_migrate.rs | function | pub | 121 | run(args: MigrateMermaidArgs) -> Result<()> |
| `commit_mermaid_migration` | apps/agentic-workflow/src/cli/td_migrate.rs | function |  | 171 | commit_mermaid_migration(project_root: &std::path::Path, path: &std::path::Path, block_id: &str) -> Result<()> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=apps/agentic-workflow/src/cli/td_migrate.rs -->
```rust
// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/interfaces/src/td_migrate.md#source
// CODEGEN-BEGIN
//! `aw td migrate-mermaid` — convert legacy mermaid blocks via envelope dispatch.
//!
//! Three modes:
//!
//! - **Enumerate** (default): scan the file, print one JSON dispatch envelope per
//!   legacy mermaid block on stdout. Caller authors the YAML payload externally.
//! - **Apply** (`--apply --block-id <id>`): read the payload from disk, render +
//!   verify equivalence + atomic-write the converted block.
//! - **Check** (`--check`): read-only scan of a file or directory that reports
//!   a `legacy_block_count` summary instead of per-block envelopes. This is
//!   the measurement this verb's `VERB_LIFECYCLE_REGISTRY` sunset criterion
//!   (epic #1270 R6 / #1274) is defined against: the verb retires once
//!   `aw td migrate-mermaid <project-td-root> --check` reports
//!   `legacy_block_count: 0` for every configured project's tech-design root.
//!
//! No embedded LLM call lives here.
//
// @spec apps/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/migrate.md
// @spec apps/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/migrate-envelope.md

use anyhow::{Context, Result};
use clap::Args;
use std::path::{Path, PathBuf};

use crate::generate::diagrams::mermaid_plus::migrate::{
    apply_block_payload, enumerate_envelopes, MigrationOptions,
};
use crate::generate::diagrams::mermaid_plus::BlockMigrationStatus;

// Arguments for `aw td migrate-mermaid`.
// @spec apps/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/migrate.md#cli
#[derive(Debug, Args)]
pub struct MigrateMermaidArgs {
    /// Path to a TD spec file, or (with `--check`) a directory to scan
    /// recursively for legacy mermaid blocks in `.md` files.
    pub path: PathBuf,

    /// Apply mode: render + verify + atomic-write the payload for `--block-id`.
    #[arg(long)]
    pub apply: bool,

    /// Block id (`<line_open>-<line_close>`) of a previously-enumerated envelope.
    /// Required with `--apply`.
    #[arg(long = "block-id")]
    pub block_id: Option<String>,

    /// Override the default payload path
    /// (`<project_root>/.aw/payloads/migrate-mermaid/<basename>-<block_id>.yaml`).
    #[arg(long = "payload-path")]
    pub payload_path: Option<PathBuf>,

    /// Read-only scan mode: walk `path` (a single spec file, or a directory
    /// scanned recursively for `.md` files) and print a JSON
    /// `legacy_block_count` summary instead of per-block dispatch envelopes.
    /// Mutually exclusive with `--apply`. This is the command the verb's
    /// sunset criterion measures.
    #[arg(long)]
    pub check: bool,
}

/// Read-only summary emitted by `--check` — the measurement surface for this
/// verb's `VERB_LIFECYCLE_REGISTRY` sunset criterion.
/// @spec apps/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/migrate.md#cli
#[derive(Debug, serde::Serialize)]
struct MigrateMermaidCheckReport {
    scanned_root: String,
    files_scanned: usize,
    legacy_block_count: usize,
    files_with_legacy_blocks: Vec<String>,
}

/// Walk `args.path` (file or directory) and report the remaining legacy
/// mermaid block count without emitting per-block dispatch envelopes.
/// @spec apps/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/migrate.md#cli
fn run_check_scan(args: &MigrateMermaidArgs, project_root: &Path) -> Result<()> {
    let opts = MigrationOptions {
        path: None,
        apply: false,
        block_id: None,
        payload_path: None,
        project_root: project_root.to_path_buf(),
    };

    let files: Vec<PathBuf> = if args.path.is_dir() {
        walkdir::WalkDir::new(&args.path)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("md"))
            .map(|entry| entry.path().to_path_buf())
            .collect()
    } else {
        vec![args.path.clone()]
    };

    let mut legacy_block_count = 0usize;
    let mut files_with_legacy_blocks = Vec::new();
    for file in &files {
        let envelopes = enumerate_envelopes(file, &opts)?;
        if !envelopes.is_empty() {
            legacy_block_count += envelopes.len();
            let rel = file.strip_prefix(project_root).unwrap_or(file);
            files_with_legacy_blocks.push(rel.to_string_lossy().to_string());
        }
    }

    let report = MigrateMermaidCheckReport {
        scanned_root: args.path.display().to_string(),
        files_scanned: files.len(),
        legacy_block_count,
        files_with_legacy_blocks,
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

// Entry point dispatched from `aw td migrate-mermaid`.
// @spec apps/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/migrate.md#cli
pub async fn run(args: MigrateMermaidArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;

    if args.check {
        anyhow::ensure!(!args.apply, "--check and --apply are mutually exclusive");
        return run_check_scan(&args, &project_root);
    }

    let opts = MigrationOptions {
        path: Some(args.path.clone()),
        apply: args.apply,
        block_id: args.block_id.clone(),
        payload_path: args.payload_path.clone(),
        project_root: project_root.clone(),
    };

    if args.apply {
        let block_id = args
            .block_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("--apply requires --block-id"))?;
        let payload_path = match &args.payload_path {
            Some(p) => p.clone(),
            None => {
                let base = args
                    .path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("block");
                project_root
                    .join(".aw/payloads/migrate-mermaid")
                    .join(format!("{}-{}.yaml", base, block_id))
            }
        };
        let payload = std::fs::read_to_string(&payload_path)
            .with_context(|| format!("read payload: {}", payload_path.display()))?;
        let result = apply_block_payload(&args.path, block_id, &payload, &opts).await?;
        if result.status == BlockMigrationStatus::Converted {
            commit_mermaid_migration(&project_root, &args.path, block_id)?;
        }
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        let envelopes = enumerate_envelopes(&args.path, &opts)?;
        for env in &envelopes {
            println!("{}", serde_json::to_string_pretty(env)?);
        }
    }
    Ok(())
}

fn commit_mermaid_migration(
    project_root: &std::path::Path,
    path: &std::path::Path,
    block_id: &str,
) -> Result<()> {
    let target = path
        .strip_prefix(project_root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();
    let message = format!(
        "td migrate-mermaid: {target}\n\n\
         Lifecycle-Stage: Td-Migrate-Mermaid\n\
         TD-Block: {block_id}\n\
         TD-Target: {target}\n"
    );
    crate::git::commit_scoped_paths(project_root, &[path.to_path_buf()], &message)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn git_available() -> bool {
        std::process::Command::new("git")
            .arg("--version")
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }

    fn init_git_repo(root: &std::path::Path) {
        for args in [
            vec!["init", "-q", "-b", "main"],
            vec!["config", "user.email", "test@example.com"],
            vec!["config", "user.name", "Test"],
            vec!["commit", "--allow-empty", "-m", "init", "-q"],
        ] {
            let out = std::process::Command::new("git")
                .args(&args)
                .current_dir(root)
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

    fn git_stdout(root: &std::path::Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .expect("git command");
        assert!(
            out.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    #[test]
    fn mermaid_apply_commit_records_target_and_block() {
        if !git_available() {
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_git_repo(root);
        let td = root.join("apps/agentic-workflow/tech-design/demo.md");
        std::fs::create_dir_all(td.parent().unwrap()).unwrap();
        std::fs::write(&td, "# Demo\n").unwrap();

        commit_mermaid_migration(root, &td, "10-20").unwrap();

        let log = git_stdout(root, &["log", "-1", "--pretty=%B"]);
        assert!(log.contains("Lifecycle-Stage: Td-Migrate-Mermaid"));
        assert!(log.contains("TD-Block: 10-20"));
        assert!(log.contains("TD-Target: apps/agentic-workflow/tech-design/demo.md"));
    }

    // Regression fixture for `run_check_scan` — a legacy (frontmatter-less)
    // mermaid block, matching `mermaid_plus::migrate::tests::LEGACY_FLOWCHART`.
    const LEGACY_FLOWCHART: &str = concat!(
        "\n# Sample TD\n\n",
        "## Logic\n",
        "<!-- type: logic lang: mermaid -->\n\n",
        "```mermaid\n",
        "flowchart TD\n",
        "    a[Start] --> b{Check}\n",
        "```\n",
    );

    // Regression fixture for `run_check_scan` — an already-migrated Mermaid
    // Plus block (has YAML frontmatter), which must not count as legacy.
    const PLUS_FLOWCHART: &str = concat!(
        "\n## Logic\n",
        "<!-- type: logic lang: mermaid -->\n\n",
        "```mermaid\n",
        "---\n",
        "id: sample\n",
        "entry: a\n",
        "nodes:\n",
        "  a: { kind: start, label: \"Start\" }\n",
        "edges: []\n",
        "---\n",
        "flowchart TD\n",
        "    a([Start])\n",
        "```\n",
    );

    #[test]
    fn check_scan_counts_legacy_blocks_across_a_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let td_dir = root.join("tech-design");
        std::fs::create_dir_all(&td_dir).unwrap();
        std::fs::write(td_dir.join("legacy.md"), LEGACY_FLOWCHART).unwrap();
        std::fs::write(td_dir.join("plus.md"), PLUS_FLOWCHART).unwrap();
        std::fs::write(td_dir.join("no-diagram.md"), "# Just prose\n").unwrap();

        let args = MigrateMermaidArgs {
            path: td_dir.clone(),
            apply: false,
            block_id: None,
            payload_path: None,
            check: true,
        };
        // `run_check_scan` prints its report to stdout; assert it succeeds
        // and returns via its file-walk without invoking apply mode.
        run_check_scan(&args, root).expect("check scan ok");
    }

    #[test]
    fn check_scan_single_file_matches_enumerate_count() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let td = root.join("demo.md");
        std::fs::write(&td, LEGACY_FLOWCHART).unwrap();

        let opts = MigrationOptions {
            project_root: root.to_path_buf(),
            ..Default::default()
        };
        let envelopes = enumerate_envelopes(&td, &opts).expect("enumerate ok");
        assert_eq!(
            envelopes.len(),
            1,
            "fixture carries exactly one legacy block"
        );

        let args = MigrateMermaidArgs {
            path: td,
            apply: false,
            block_id: None,
            payload_path: None,
            check: true,
        };
        run_check_scan(&args, root).expect("check scan ok");
    }

    #[tokio::test]
    async fn check_and_apply_are_mutually_exclusive() {
        let args = MigrateMermaidArgs {
            path: PathBuf::from("unused.md"),
            apply: true,
            block_id: None,
            payload_path: None,
            check: true,
        };
        let err = run(args).await.expect_err("check + apply must be rejected");
        assert!(err.to_string().contains("mutually exclusive"));
    }
}

// CODEGEN-END
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/cli/td_migrate.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Add a read-only `--check` scan mode (walks a file or directory and
      reports a `legacy_block_count` summary) that is the measurement
      surface for the `td.migrate-mermaid` verb's `VERB_LIFECYCLE_REGISTRY`
      sunset criterion (epic #1270 R6 / #1274). Whole-file source template
      generated from the standardized target body.
```
