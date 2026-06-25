// `mamba pip` — pip-compatible installed-environment inspection commands.

use anyhow::{Context, Result, bail};
use clap::ArgMatches;
use std::path::PathBuf;

use crate::pkgmanage::pkgmgr::pip_check::check_consistency;
use crate::pkgmanage::pkgmgr::pip_inventory::{
    ListOptions, enumerate_installed, find_by_name, render_freeze, render_list, render_show,
};
use crate::pkgmanage::pkgmgr::pip_tree::render_installed_tree;
use crate::pkgmanage::pkgmgr::tree::TreeOptions;

pub fn cmd_pip(sub: &ArgMatches) -> Result<()> {
    match sub.subcommand() {
        Some(("list", cmd)) => cmd_pip_list(cmd),
        Some(("freeze", cmd)) => cmd_pip_freeze(cmd),
        Some(("show", cmd)) => cmd_pip_show(cmd),
        Some(("tree", cmd)) => cmd_pip_tree(cmd),
        Some(("check", cmd)) => cmd_pip_check(cmd),
        _ => Ok(()),
    }
}

fn cmd_pip_list(sub: &ArgMatches) -> Result<()> {
    let dists = enumerate_installed(&site_packages_path(sub)?);
    let format = sub
        .get_one::<String>("format")
        .map(String::as_str)
        .unwrap_or("columns");
    let body = match format {
        "columns" => render_list(
            &dists,
            &ListOptions {
                include_header: !sub.get_flag("no-header"),
                sort_by_version: sub.get_flag("sort-by-version"),
            },
        ),
        "freeze" => render_freeze(&dists),
        other => bail!("unsupported pip list format `{other}`"),
    };
    print!("{body}");
    Ok(())
}

fn cmd_pip_freeze(sub: &ArgMatches) -> Result<()> {
    let dists = enumerate_installed(&site_packages_path(sub)?);
    print!("{}", render_freeze(&dists));
    Ok(())
}

fn cmd_pip_show(sub: &ArgMatches) -> Result<()> {
    let dists = enumerate_installed(&site_packages_path(sub)?);
    let name = sub
        .get_one::<String>("name")
        .context("pip show requires a package name")?;
    let Some(dist) = find_by_name(&dists, name) else {
        bail!("package `{name}` is not installed");
    };
    print!("{}", render_show(dist));
    Ok(())
}

fn cmd_pip_check(sub: &ArgMatches) -> Result<()> {
    let dists = enumerate_installed(&site_packages_path(sub)?);
    let issues = check_consistency(&dists);
    if issues.is_empty() {
        println!("No broken requirements found.");
        return Ok(());
    }
    for issue in &issues {
        println!("{}", issue.detail);
    }
    std::process::exit(1);
}

fn cmd_pip_tree(sub: &ArgMatches) -> Result<()> {
    let dists = enumerate_installed(&site_packages_path(sub)?);
    let opts = TreeOptions {
        max_depth: sub
            .get_one::<String>("depth")
            .map(|s| s.parse::<usize>())
            .transpose()
            .context("parse --depth")?,
        focus: sub.get_one::<String>("package").cloned(),
        invert: sub.get_flag("invert"),
        prune: sub
            .get_many::<String>("prune")
            .map(|vals| vals.cloned().collect())
            .unwrap_or_default(),
        no_dedupe: sub.get_flag("no-dedupe"),
    };
    let rendered = render_installed_tree(&dists, &opts);
    if opts.focus.is_some() && rendered.trim().is_empty() {
        bail!("package not found in site-packages inventory");
    }
    print!("{rendered}");
    Ok(())
}

fn site_packages_path(sub: &ArgMatches) -> Result<PathBuf> {
    if let Some(path) = sub.get_one::<String>("site-packages") {
        return Ok(PathBuf::from(path));
    }
    let cwd = std::env::current_dir().context("read current directory")?;
    Ok(cwd.join(".venv").join("site-packages"))
}
