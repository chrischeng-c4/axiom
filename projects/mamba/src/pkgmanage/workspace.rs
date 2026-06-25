// `mamba workspace` — inspect uv-compatible workspace membership.

use anyhow::{Result, bail};
use clap::ArgMatches;
use std::path::PathBuf;

use crate::pkgmanage::pkgmgr::workspace::discover_workspace_members;

pub fn cmd_workspace(sub: &ArgMatches) -> Result<()> {
    match sub.subcommand() {
        Some(("list", cmd)) => cmd_list(cmd),
        Some((other, _)) => bail!("unknown workspace subcommand `{other}`"),
        None => bail!("`mamba workspace` requires a subcommand: list"),
    }
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
    for member in members {
        println!(
            "{}=={}\t{}",
            member.name,
            member.version,
            member.root.display()
        );
    }
    Ok(())
}
