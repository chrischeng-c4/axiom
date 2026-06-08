// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/td_migrate.md#source
// CODEGEN-BEGIN
//! `aw td migrate-mermaid` — convert legacy mermaid blocks via envelope dispatch.
//!
//! Two modes:
//!
//! - **Enumerate** (default): scan the file, print one JSON dispatch envelope per
//!   legacy mermaid block on stdout. Caller authors the YAML payload externally.
//! - **Apply** (`--apply --block-id <id>`): read the payload from disk, render +
//!   verify equivalence + atomic-write the converted block.
//!
//! No embedded LLM call lives here.
//
// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/migrate.md
// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/migrate-envelope.md

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;

use crate::generate::diagrams::mermaid_plus::migrate::{
    apply_block_payload, enumerate_envelopes, MigrationOptions,
};
use crate::generate::diagrams::mermaid_plus::BlockMigrationStatus;

// Arguments for `aw td migrate-mermaid`.
// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/migrate.md#cli
#[derive(Debug, Args)]
pub struct MigrateMermaidArgs {
    /// Path to a TD spec file.
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
}

// Entry point dispatched from `aw td migrate-mermaid`.
// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/migrate.md#cli
pub async fn run(args: MigrateMermaidArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
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
        let td = root.join("projects/agentic-workflow/tech-design/demo.md");
        std::fs::create_dir_all(td.parent().unwrap()).unwrap();
        std::fs::write(&td, "# Demo\n").unwrap();

        commit_mermaid_migration(root, &td, "10-20").unwrap();

        let log = git_stdout(root, &["log", "-1", "--pretty=%B"]);
        assert!(log.contains("Lifecycle-Stage: Td-Migrate-Mermaid"));
        assert!(log.contains("TD-Block: 10-20"));
        assert!(log.contains("TD-Target: projects/agentic-workflow/tech-design/demo.md"));
    }
}

// CODEGEN-END
