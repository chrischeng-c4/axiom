// SPEC-MANAGED: apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
// HANDWRITE-BEGIN conf-cli-surface
//! `aw conf` -- check and refresh `aw.toml` producer artifacts.

use crate::services::{project_discovery, project_registry};
use crate::Result;
use anyhow::{bail, Context};
use clap::{Args, Subcommand};
use std::path::{Path, PathBuf};

/// Manage Agentic Workflow configuration artifacts.
#[derive(Debug, Args, Clone)]
pub struct ConfArgs {
    #[command(subcommand)]
    pub command: ConfCommand,
}

#[derive(Debug, Subcommand, Clone)]
pub enum ConfCommand {
    /// Bootstrap a project-local `aw.toml` from a tracker project label.
    Init(ConfInitArgs),
    /// Check whether `aw.toml`'s project registry block is stale.
    Check,
    /// Auto-discover projects and refresh `aw.toml`'s registry block.
    Sync(ConfSyncArgs),
}

#[derive(Debug, Args, Clone)]
pub struct ConfInitArgs {
    /// Tracker identity label (`app:<name>`, `lib:<name>`, or `project:<name>`).
    #[arg(long)]
    pub project_label: String,
}

#[derive(Debug, Args, Clone)]
pub struct ConfSyncArgs {
    /// Print the registry diff without writing `aw.toml`.
    #[arg(long)]
    pub dry_run: bool,
}

pub fn run(args: ConfArgs) -> Result<()> {
    let root = crate::find_project_root()?;
    run_at_root(&root, args)
}

fn run_at_root(root: &Path, args: ConfArgs) -> Result<()> {
    match args.command {
        ConfCommand::Init(args) => run_project_init(root, args),
        ConfCommand::Check => run_drift_check(root, true),
        ConfCommand::Sync(args) => run_registry_sync(root, args),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BootstrapProjectIdentity {
    name: String,
    label: String,
    relative_path: PathBuf,
}

// @spec apps/agentic-workflow/tech-design/semantic/aw-epic-project-label-dispatch.md#R5
fn bootstrap_project_identity(label: &str) -> Result<BootstrapProjectIdentity> {
    let (prefix, name) = label.split_once(':').ok_or_else(|| {
        anyhow::anyhow!("--project-label must be app:<name>, lib:<name>, or project:<name>")
    })?;
    let parent = match prefix {
        "app" => "apps",
        "lib" => "libs",
        "project" => "projects",
        _ => bail!("unsupported --project-label prefix `{prefix}`; expected app, lib, or project"),
    };
    if name.is_empty()
        || name == "."
        || name == ".."
        || !name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        bail!(
            "invalid project name `{name}` in --project-label; use letters, digits, dot, dash, or underscore"
        );
    }
    Ok(BootstrapProjectIdentity {
        name: name.to_string(),
        label: label.to_string(),
        relative_path: PathBuf::from(parent).join(name),
    })
}

// @spec apps/agentic-workflow/tech-design/semantic/aw-epic-project-label-dispatch.md#R5
fn run_project_init(root: &Path, args: ConfInitArgs) -> Result<()> {
    let identity = bootstrap_project_identity(&args.project_label)?;
    let target = root.join(&identity.relative_path);
    let config_path = target.join(project_registry::PROJECT_AW_CONFIG_FILE);

    if config_path.exists() {
        let row = project_registry::resolve_project_config_row(root, &identity.name).with_context(
            || {
                format!(
                    "{} exists but is not discoverable from root aw.toml",
                    config_path.display()
                )
            },
        )?;
        if row.path != identity.relative_path.to_string_lossy()
            || row.label_or_default() != identity.label
        {
            bail!(
                "project `{}` is already configured as path `{}` with label `{}`; refusing to overwrite {}",
                identity.name,
                row.path,
                row.label_or_default(),
                config_path.display()
            );
        }
        println!(
            "aw conf init: project `{}` is already registered.",
            identity.name
        );
        println!("next: aw meta init --project {}", identity.name);
        return Ok(());
    }

    std::fs::create_dir_all(&target)
        .with_context(|| format!("creating project directory {}", target.display()))?;
    let body = format!(
        "[project]\nname = {:?}\ncap_path = \"CAPABILITIES.md\"\nlabel = {:?}\n\n[[workspaces]]\nname = {:?}\npaths = [\"**\"]\ntarget = \"schemas\"\ntest_cmd = \"true\"\n",
        identity.name, identity.label, identity.name
    );
    std::fs::write(&config_path, body)
        .with_context(|| format!("writing {}", config_path.display()))?;

    let row = project_registry::resolve_project_config_row(root, &identity.name).with_context(|| {
        format!(
            "created {}, but root aw.toml does not discover it; add `{}` to [agentic_workflow.projects].discover",
            config_path.display(),
            format!("{}/*/aw.toml", identity.relative_path.parent().unwrap().display())
        )
    })?;
    if row.path != identity.relative_path.to_string_lossy()
        || row.label_or_default() != identity.label
    {
        bail!(
            "created project config resolved to path `{}` and label `{}`, expected `{}` and `{}`",
            row.path,
            row.label_or_default(),
            identity.relative_path.display(),
            identity.label
        );
    }

    println!(
        "aw conf init: registered `{}` at {}.",
        identity.name,
        identity.relative_path.display()
    );
    println!("next: aw meta init --project {}", identity.name);
    Ok(())
}

fn run_registry_sync(root: &Path, args: ConfSyncArgs) -> Result<()> {
    if args.dry_run {
        return run_drift_check(root, false);
    }

    let projects = project_discovery::discover_projects(root)?;
    let count = projects.len();
    project_registry::write_projects_config(root, &projects)?;

    println!("aw conf sync: wrote aw.toml with {count} discovered project(s).");
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
                bail!("drift detected: aw.toml project registry is out of date");
            }
        }
        None => println!("aw.toml project registry is up to date."),
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
        fs::write(root.join("aw.toml"), content).unwrap();
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
        let content = fs::read_to_string(tmp.path().join("aw.toml")).unwrap();
        assert!(content.contains("ghost"));
    }

    #[test]
    fn conf_init_bootstraps_discoverable_project_local_config() {
        let tmp = make_root();
        write_config(
            tmp.path(),
            "[agentic_workflow.projects]\ndiscover = [\"apps/*/aw.toml\", \"libs/*/aw.toml\", \"projects/*/aw.toml\"]\n",
        );

        run_at_root(
            tmp.path(),
            ConfArgs {
                command: ConfCommand::Init(ConfInitArgs {
                    project_label: "app:workbench".to_string(),
                }),
            },
        )
        .unwrap();

        let config = fs::read_to_string(tmp.path().join("apps/workbench/aw.toml")).unwrap();
        assert!(config.contains("name = \"workbench\""));
        assert!(config.contains("label = \"app:workbench\""));
        assert!(config.contains("target = \"schemas\""));
        let row = project_registry::resolve_project_config_row(tmp.path(), "workbench").unwrap();
        assert_eq!(row.path, "apps/workbench");
        assert_eq!(row.label_or_default(), "app:workbench");

        // Re-running the producer is an idempotent no-op, never an overwrite.
        run_at_root(
            tmp.path(),
            ConfArgs {
                command: ConfCommand::Init(ConfInitArgs {
                    project_label: "app:workbench".to_string(),
                }),
            },
        )
        .unwrap();
    }

    #[test]
    fn conf_init_rejects_unsafe_or_unsupported_project_labels() {
        for label in ["workbench", "service:workbench", "app:../workbench", "app:"] {
            let err = bootstrap_project_identity(label).unwrap_err().to_string();
            assert!(!err.is_empty(), "{label} must be rejected");
        }
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

        let content = fs::read_to_string(tmp.path().join("aw.toml")).unwrap();
        assert!(content.contains("alpha"));
        assert!(!content.contains("ghost"));
        assert!(project_registry::check_drift(tmp.path()).unwrap().is_none());
    }
}
// HANDWRITE-END conf-cli-surface
