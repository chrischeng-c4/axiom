// `mamba workspace` — inspect uv-compatible workspace membership.

use anyhow::{Result, bail};
use clap::ArgMatches;
use std::path::PathBuf;

use crate::pkgmanage::pkgmgr::workspace::{
    discover_workspace_members, normalize_workspace_package_name, read_workspace_config,
};

pub fn cmd_workspace(sub: &ArgMatches) -> Result<()> {
    match sub.subcommand() {
        Some(("metadata", cmd)) => cmd_metadata(cmd),
        Some(("dir", cmd)) => cmd_dir(cmd),
        Some(("list", cmd)) => cmd_list(cmd),
        Some((other, _)) => bail!("unknown workspace subcommand `{other}`"),
        None => bail!("`mamba workspace` requires a subcommand: metadata | dir | list"),
    }
}

fn workspace_root(sub: &ArgMatches) -> Result<PathBuf> {
    Ok(sub
        .get_one::<String>("root")
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir()?))
}

fn cmd_metadata(sub: &ArgMatches) -> Result<()> {
    let root = workspace_root(sub)?;
    let config = read_workspace_config(&root)?;
    let members = discover_workspace_members(&root)?;
    let value = serde_json::json!({
        "root": root,
        "pyproject": root.join("pyproject.toml"),
        "workspace": config.map(|cfg| serde_json::json!({
            "members": cfg.members,
            "exclude": cfg.exclude,
        })),
        "members": members.iter().map(|m| serde_json::json!({
            "name": m.name,
            "version": m.version,
            "root": m.root,
            "pyproject": m.pyproject,
        })).collect::<Vec<_>>(),
    });
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

fn cmd_dir(sub: &ArgMatches) -> Result<()> {
    let root = workspace_root(sub)?;
    if let Some(package) = sub.get_one::<String>("package") {
        let package = normalize_workspace_package_name(package);
        let members = discover_workspace_members(&root)?;
        let member = members
            .iter()
            .find(|m| m.name == package)
            .ok_or_else(|| anyhow::anyhow!("workspace package not found: {package}"))?;
        println!("{}", member.root.display());
        return Ok(());
    }

    let pyproject = root.join("pyproject.toml");
    if !pyproject.exists() {
        bail!(
            "workspace pyproject.toml not found: {}",
            pyproject.display()
        );
    }
    println!("{}", root.display());
    Ok(())
}

fn cmd_list(sub: &ArgMatches) -> Result<()> {
    let root = sub
        .get_one::<String>("root")
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir()?);
    let members = discover_workspace_members(&root)?;
    if sub.get_flag("json") {
        let rows: Vec<_> = members
            .iter()
            .map(|m| {
                serde_json::json!({
                    "name": m.name,
                    "version": m.version,
                    "root": m.root,
                    "pyproject": m.pyproject,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }
    if sub.get_flag("paths") {
        for member in members {
            println!("{}", member.root.display());
        }
        return Ok(());
    }
    for member in members {
        println!("{}", member.name);
    }
    Ok(())
}
