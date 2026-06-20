// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/specs/sync-command.md#R11
// HANDWRITE-BEGIN sync-cli-surface
//! `aw sync` -- refresh the marker-delimited project registry block.

use crate::services::{project_discovery, project_registry};
use crate::Result;
use anyhow::bail;
use clap::Args;
use std::path::Path;

/// Auto-discover project/workspace hierarchy and update `.aw/config.toml`.
#[derive(Debug, Args, Clone)]
pub struct SyncArgs {
    /// Print the registry diff without writing `.aw/config.toml`.
    #[arg(long)]
    pub dry_run: bool,
    /// Print the registry diff and fail when `.aw/config.toml` is stale.
    #[arg(long)]
    pub check: bool,
}

pub fn run(args: SyncArgs) -> Result<()> {
    let root = crate::find_project_root()?;
    run_at_root(&root, args)
}

fn run_at_root(root: &Path, args: SyncArgs) -> Result<()> {
    if args.dry_run || args.check {
        return run_drift_check(root, args.check);
    }

    let projects = project_discovery::discover_projects(root)?;
    let count = projects.len();
    project_registry::write_projects_config(root, &projects)?;

    println!("aw sync: wrote .aw/config.toml with {count} discovered project(s).");
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
    fn sync_check_reports_drift_without_writing_config() {
        let tmp = make_root();
        write_config(tmp.path(), &stale_config());

        let result = run_at_root(
            tmp.path(),
            SyncArgs {
                dry_run: false,
                check: true,
            },
        );

        assert!(result.is_err());
        let content = fs::read_to_string(tmp.path().join(".aw").join("config.toml")).unwrap();
        assert!(content.contains("ghost"));
    }

    #[test]
    fn sync_write_updates_registry_and_clears_drift() {
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
            SyncArgs {
                dry_run: false,
                check: false,
            },
        )
        .unwrap();

        let content = fs::read_to_string(tmp.path().join(".aw").join("config.toml")).unwrap();
        assert!(content.contains("alpha"));
        assert!(!content.contains("ghost"));
        assert!(project_registry::check_drift(tmp.path()).unwrap().is_none());
    }
}
// HANDWRITE-END sync-cli-surface
