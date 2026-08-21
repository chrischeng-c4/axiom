// `mamba tree` — uv-compatible dependency tree entrypoint.

use anyhow::{bail, Context, Result};
use clap::ArgMatches;

use crate::pkgmanage::lockfile;
use crate::pkgmanage::pkgmgr::tree::{render_lockfile_tree, TreeOptions};

const LOCKFILE_FILE: &str = "mamba.lock";

pub fn cmd_tree(sub: &ArgMatches) -> Result<()> {
    let project_dir = std::env::current_dir().context("read current directory")?;
    let lock_path = project_dir.join(LOCKFILE_FILE);
    if !lock_path.exists() {
        bail!(
            "no {LOCKFILE_FILE} in {} — run `mamba lock` or `mamba add <dep>` first",
            project_dir.display()
        );
    }
    let lock = lockfile::read_user_lockfile(&lock_path)?;
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
    let rendered = render_lockfile_tree(&lock, &opts);
    if opts.focus.is_some() && rendered.trim().is_empty() {
        bail!("package not found in mamba.lock");
    }
    print!("{rendered}");
    Ok(())
}
