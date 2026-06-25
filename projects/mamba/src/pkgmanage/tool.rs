// `mamba tool` — uv-style tool command family.

use anyhow::{Context, Result, bail};
use clap::ArgMatches;
use std::path::PathBuf;

use crate::pkgmanage::install::{install_tool, list_tools, resolve_tools_root, uninstall_tool};
use crate::pkgmanage::pkgmgr::shell::Shell;

const FROZEN_INDEX_ENV: &str = "MAMBA_FROZEN_INDEX";

pub fn cmd_tool(sub: &ArgMatches) -> Result<()> {
    match sub.subcommand() {
        Some(("install", cmd)) => cmd_install(cmd),
        Some(("upgrade", cmd)) => cmd_upgrade(cmd),
        Some(("list", _)) => list_tools(),
        Some(("uninstall", cmd)) => cmd_uninstall(cmd),
        Some(("dir", _)) => cmd_dir(),
        Some(("update-shell", cmd)) => cmd_update_shell(cmd),
        Some((other, _)) => bail!("unknown tool subcommand `{other}`"),
        None => bail!("`mamba tool` requires a subcommand"),
    }
}

fn cmd_install(sub: &ArgMatches) -> Result<()> {
    let name = required_name(sub)?;
    let explicit_version = sub.get_one::<String>("version").map(String::as_str);
    let index = resolve_index(sub)?;
    install_tool(name, explicit_version, &index)
}

fn cmd_upgrade(sub: &ArgMatches) -> Result<()> {
    let name = required_name(sub)?;
    let index = resolve_index(sub)?;
    install_tool(name, None, &index)
}

fn cmd_uninstall(sub: &ArgMatches) -> Result<()> {
    uninstall_tool(required_name(sub)?)
}

fn cmd_dir() -> Result<()> {
    println!("{}", resolve_tools_root()?.display());
    Ok(())
}

fn cmd_update_shell(sub: &ArgMatches) -> Result<()> {
    let shell = sub
        .get_one::<String>("shell")
        .map(|raw| {
            Shell::parse(raw).with_context(|| {
                format!(
                    "unsupported shell `{raw}`; expected bash|zsh|fish|powershell|cmd|nushell|elvish"
                )
            })
        })
        .transpose()?
        .unwrap_or(Shell::Bash);
    let bin_dir = sub
        .get_one::<String>("bin-dir")
        .map(PathBuf::from)
        .unwrap_or(resolve_tools_root()?.join("bin"));
    let snippet = shell.prepend_path_snippet(&bin_dir.display().to_string());
    print!("{}", shell.wrap_managed_block(&snippet));
    Ok(())
}

fn required_name(sub: &ArgMatches) -> Result<&str> {
    sub.get_one::<String>("name")
        .map(String::as_str)
        .context("missing required <name>")
}

fn resolve_index(sub: &ArgMatches) -> Result<PathBuf> {
    sub.get_one::<String>("index")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os(FROZEN_INDEX_ENV).map(PathBuf::from))
        .context("no frozen index configured (pass --index DIR or set $MAMBA_FROZEN_INDEX)")
}
