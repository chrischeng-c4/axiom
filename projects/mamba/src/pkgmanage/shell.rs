// `mamba shell` — print shell integration snippets.

use anyhow::{Context, Result, bail};
use clap::ArgMatches;
use std::path::PathBuf;

use crate::pkgmanage::pkgmgr::shell::{Shell, detect_from_shell_env};
use crate::pkgmanage::pkgmgr::tools::default_bin_root;

pub fn cmd_shell(sub: &ArgMatches) -> Result<()> {
    match sub.subcommand() {
        Some(("path", cmd)) => cmd_path(cmd),
        Some(("init", cmd)) => cmd_init(cmd),
        Some((other, _)) => bail!("unknown shell subcommand `{other}`"),
        None => bail!("`mamba shell` requires a subcommand: path | init"),
    }
}

fn cmd_path(sub: &ArgMatches) -> Result<()> {
    let shell = resolve_shell(sub)?;
    let bin_dir = resolve_bin_dir(sub);
    println!(
        "{}",
        shell.prepend_path_snippet(&bin_dir.display().to_string())
    );
    Ok(())
}

fn cmd_init(sub: &ArgMatches) -> Result<()> {
    let shell = resolve_shell(sub)?;
    let bin_dir = resolve_bin_dir(sub);
    let snippet = shell.prepend_path_snippet(&bin_dir.display().to_string());
    print!("{}", shell.wrap_managed_block(&snippet));
    Ok(())
}

fn resolve_bin_dir(sub: &ArgMatches) -> PathBuf {
    sub.get_one::<String>("bin-dir")
        .map(PathBuf::from)
        .unwrap_or_else(default_bin_root)
}

fn resolve_shell(sub: &ArgMatches) -> Result<Shell> {
    if let Some(raw) = sub.get_one::<String>("shell") {
        return Shell::parse(raw).with_context(|| {
            format!(
                "unsupported shell `{raw}`; expected bash|zsh|fish|powershell|cmd|nushell|elvish"
            )
        });
    }

    let shell_env = std::env::var("SHELL")
        .or_else(|_| std::env::var("ComSpec"))
        .context("no --shell provided and neither $SHELL nor $ComSpec is set")?;
    detect_from_shell_env(&shell_env)
        .with_context(|| format!("could not detect supported shell from `{shell_env}`"))
}
