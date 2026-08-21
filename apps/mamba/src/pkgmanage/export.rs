// `mamba export` — uv-compatible lockfile export entrypoint.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::ArgMatches;

use crate::pkgmanage::lockfile;
use crate::pkgmanage::pkgmgr::pylock_export::{render_pylock_toml, PylockOptions};
use crate::pkgmanage::pkgmgr::requirements_export::{export_requirements_txt, ExportOptions};

const LOCKFILE_FILE: &str = "mamba.lock";

pub fn cmd_export(sub: &ArgMatches) -> Result<()> {
    let project_dir = std::env::current_dir().context("read current directory")?;
    let lock_path = project_dir.join(LOCKFILE_FILE);
    if !lock_path.exists() {
        bail!(
            "no {LOCKFILE_FILE} in {} — run `mamba lock` or `mamba add <dep>` first",
            project_dir.display()
        );
    }
    let lock = lockfile::read_user_lockfile(&lock_path)?;
    let format = sub
        .get_one::<String>("format")
        .map(String::as_str)
        .unwrap_or("requirements-txt");
    let body = match format {
        "requirements-txt" | "requirements.txt" | "requirements" => {
            export_requirements_txt(&lock, &requirements_options(sub))
        }
        "pylock.toml" | "pylock" => render_pylock_toml(&lock, &pylock_options(sub)),
        other => bail!("unsupported export format `{other}`"),
    };
    write_export(sub.get_one::<String>("output-file"), &body)?;
    Ok(())
}

fn requirements_options(sub: &ArgMatches) -> ExportOptions {
    ExportOptions {
        include_hashes: !sub.get_flag("no-hashes"),
        include_header: !sub.get_flag("no-header"),
        exclude_packages: sub
            .get_many::<String>("no-emit-package")
            .map(|vals| vals.cloned().collect())
            .unwrap_or_default(),
        global_marker: sub.get_one::<String>("marker").cloned(),
        annotate: sub.get_flag("annotate"),
    }
}

fn pylock_options(sub: &ArgMatches) -> PylockOptions {
    PylockOptions {
        requires_python: sub.get_one::<String>("requires-python").cloned(),
        environments: sub
            .get_many::<String>("environment")
            .map(|vals| vals.cloned().collect())
            .unwrap_or_default(),
        ..Default::default()
    }
}

fn write_export(output: Option<&String>, body: &str) -> Result<()> {
    match output.map(String::as_str) {
        Some("-") | None => {
            print!("{body}");
            Ok(())
        }
        Some(path) => {
            let dest = PathBuf::from(path);
            if let Some(parent) = dest.parent().filter(|p| !p.as_os_str().is_empty()) {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            atomic_write(&dest, body.as_bytes())
        }
    }
}

fn atomic_write(dest: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = dest.with_extension("tmp");
    std::fs::write(&tmp, bytes).with_context(|| format!("write {}", tmp.display()))?;
    std::fs::rename(&tmp, dest).with_context(|| format!("rename {}", dest.display()))?;
    Ok(())
}
