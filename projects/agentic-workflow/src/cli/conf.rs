// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
// HANDWRITE-BEGIN conf-cli-surface
//! `aw conf` -- check and refresh `.aw/config.toml` producer artifacts.

use crate::services::{project_discovery, project_registry};
use crate::Result;
use anyhow::bail;
use clap::{Args, Subcommand};
use std::path::Path;

/// Manage Agentic Workflow configuration artifacts.
#[derive(Debug, Args, Clone)]
pub struct ConfArgs {
    #[command(subcommand)]
    pub command: ConfCommand,
}

#[derive(Debug, Subcommand, Clone)]
pub enum ConfCommand {
    /// Check whether `.aw/config.toml`'s project registry block is stale.
    Check,
    /// Auto-discover projects and refresh `.aw/config.toml`'s registry block.
    Sync(ConfSyncArgs),
}

#[derive(Debug, Args, Clone)]
pub struct ConfSyncArgs {
    /// Print the registry diff without writing `.aw/config.toml`.
    #[arg(long)]
    pub dry_run: bool,
}

pub fn run(args: ConfArgs) -> Result<()> {
    let root = crate::find_project_root()?;
    run_at_root(&root, args)
}

fn run_at_root(root: &Path, args: ConfArgs) -> Result<()> {
    match args.command {
        ConfCommand::Check => run_drift_check(root, true),
        ConfCommand::Sync(args) => run_registry_sync(root, args),
    }
}

fn run_registry_sync(root: &Path, args: ConfSyncArgs) -> Result<()> {
    if args.dry_run {
        return run_drift_check(root, false);
    }

    let projects = project_discovery::discover_projects(root)?;
    let count = projects.len();
    project_registry::write_projects_config(root, &projects)?;

    println!("aw conf sync: wrote .aw/config.toml with {count} discovered project(s).");
    for project in &projects {
        println!(
            "  {} ({} workspace(s))",
            project.name,
            project.workspaces.len()
        );
    }
    Ok(())
}

fn run_drift_check(root: &Path, fail_on_drift: bool) -> Result<()> {
    match project_registry::check_drift(root)? {
        Some(diff) => {
            println!("{diff}");
            if fail_on_drift {
                bail!("drift detected: .aw/config.toml project registry is out of date");
            }
        }
        None => println!(".aw/config.toml project registry is up to date."),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::workspace::{SYNC_BEGIN_MARKER, SYNC_END_MARKER};
    use std::fs;
    use tempfile::TempDir;

    fn make_root() -> TempDir {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".aw")).unwrap();
        tmp
    }

    fn write_config(root: &Path, content: &str) {
        fs::write(root.join(".aw").join("config.toml"), content).unwrap();
    }

    fn stale_config() -> String {
        format!(
            "{SYNC_BEGIN_MARKER}\n\n[[projects]]\nname = \"ghost\"\npath = \"crates/ghost\"\n\n[[projects.workspaces]]\nname = \"ghost\"\npaths = [\"crates/ghost/**\"]\ntarget = \"rust\"\n\n{SYNC_END_MARKER}\n"
        )
    }

    #[test]
    fn conf_check_reports_drift_without_writing_config() {
        let tmp = make_root();
        write_config(tmp.path(), &stale_config());

        let result = run_at_root(
            tmp.path(),
            ConfArgs {
                command: ConfCommand::Check,
            },
        );

        assert!(result.is_err());
        let content = fs::read_to_string(tmp.path().join(".aw").join("config.toml")).unwrap();
        assert!(content.contains("ghost"));
    }

    #[test]
    fn conf_sync_updates_registry_and_clears_drift() {
        let tmp = make_root();
        write_config(tmp.path(), &stale_config());
        let project_dir = tmp.path().join("crates").join("alpha");
        fs::create_dir_all(&project_dir).unwrap();
        fs::write(
            project_dir.join("Cargo.toml"),
            "[package]\nname = \"alpha\"\n",
        )
        .unwrap();

        run_at_root(
            tmp.path(),
            ConfArgs {
                command: ConfCommand::Sync(ConfSyncArgs { dry_run: false }),
            },
        )
        .unwrap();

        let content = fs::read_to_string(tmp.path().join(".aw").join("config.toml")).unwrap();
        assert!(content.contains("alpha"));
        assert!(!content.contains("ghost"));
        assert!(project_registry::check_drift(tmp.path()).unwrap().is_none());
    }
}
// HANDWRITE-END conf-cli-surface
