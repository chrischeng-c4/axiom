// `mamba venv` — uv-style virtual environment create/remove CLI driver.

use anyhow::{Result, bail};
use clap::ArgMatches;
use std::path::PathBuf;

use crate::pkgmanage::pkgmgr::venv::{
    VenvCreationOutcome, VenvOptions, create_venv, first_python_on_path, remove_venv,
};

pub fn cmd_venv(sub: &ArgMatches) -> Result<()> {
    match sub.subcommand() {
        Some(("create", cmd)) => cmd_create(cmd),
        Some(("remove", cmd)) => cmd_remove(cmd),
        Some((other, _)) => bail!("unknown venv subcommand `{other}`"),
        None => cmd_create(sub),
    }
}

fn cmd_create(sub: &ArgMatches) -> Result<()> {
    let root = venv_root(sub);
    let mut opts = VenvOptions::new(resolve_python(sub), &root);
    opts.system_site_packages = sub.get_flag("system-site-packages");
    opts.copies = sub.get_flag("copies");
    opts.without_pip = !sub.get_flag("seed");
    opts.prompt = sub.get_one::<String>("prompt").cloned();
    opts.clobber = sub.get_flag("clear");

    match create_venv(&opts)? {
        VenvCreationOutcome::Created { version, .. } => {
            println!(
                "Created virtual environment at {} using Python {version}",
                root.display()
            );
            Ok(())
        }
        VenvCreationOutcome::Refused { reason } => bail!(reason),
    }
}

fn cmd_remove(sub: &ArgMatches) -> Result<()> {
    let root = venv_root(sub);
    match remove_venv(&root)? {
        outcome if outcome == "removed" => {
            println!("Removed virtual environment at {}", root.display());
            Ok(())
        }
        outcome => bail!("{outcome}: {}", root.display()),
    }
}

fn venv_root(sub: &ArgMatches) -> PathBuf {
    sub.get_one::<String>("path")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".venv"))
}

fn resolve_python(sub: &ArgMatches) -> PathBuf {
    sub.get_one::<String>("python")
        .map(PathBuf::from)
        .or_else(first_python_on_path)
        .unwrap_or_else(|| PathBuf::from("python3"))
}
