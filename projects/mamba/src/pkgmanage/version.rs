// `mamba version` — uv-compatible PEP 621 version read/set/bump entrypoint.

use anyhow::{Context, Result, bail};
use clap::ArgMatches;
use std::path::PathBuf;

use crate::pkgmanage::pkgmgr::version_bump::{
    BumpKind, bump, parse_version, read_pyproject_version, write_pyproject_version,
};

const PYPROJECT_FILE: &str = "pyproject.toml";

pub fn cmd_version(sub: &ArgMatches) -> Result<()> {
    let project_dir = std::env::current_dir().context("read current directory")?;
    let pyproject = project_dir.join(PYPROJECT_FILE);
    if !pyproject.exists() {
        bail!(
            "no {PYPROJECT_FILE} in {} — `mamba version` operates on PEP 621 [project].version",
            project_dir.display()
        );
    }
    let src = std::fs::read_to_string(&pyproject)
        .with_context(|| format!("read {}", pyproject.display()))?;
    let current = read_pyproject_version(&src).context("[project].version missing")?;
    let explicit = sub.get_one::<String>("version").cloned();
    let bump_kind = sub
        .get_one::<String>("bump")
        .map(|s| parse_bump_kind(s))
        .transpose()?;
    if explicit.is_some() && bump_kind.is_some() {
        bail!("pass either an explicit version or --bump, not both");
    }

    let next = match (explicit, bump_kind) {
        (Some(v), None) => {
            if parse_version(&v).is_none() {
                bail!("invalid PEP 440 version `{v}`");
            }
            v
        }
        (None, Some(kind)) => {
            let parsed = parse_version(&current)
                .with_context(|| format!("invalid current PEP 440 version `{current}`"))?;
            bump(&parsed, kind).render()
        }
        (None, None) => {
            println!("{current}");
            return Ok(());
        }
        (Some(_), Some(_)) => unreachable!(),
    };

    if !sub.get_flag("dry-run") {
        let rewritten = write_pyproject_version(&src, &next).map_err(anyhow::Error::msg)?;
        atomic_write(&pyproject, rewritten.as_bytes())?;
    }
    println!("{next}");
    Ok(())
}

fn parse_bump_kind(raw: &str) -> Result<BumpKind> {
    match raw {
        "major" => Ok(BumpKind::Major),
        "minor" => Ok(BumpKind::Minor),
        "patch" => Ok(BumpKind::Patch),
        "alpha" => Ok(BumpKind::Alpha),
        "beta" => Ok(BumpKind::Beta),
        "rc" => Ok(BumpKind::Rc),
        "post" => Ok(BumpKind::Post),
        "dev" => Ok(BumpKind::Dev),
        "release" => Ok(BumpKind::Release),
        _ => bail!("unknown bump kind `{raw}`"),
    }
}

fn atomic_write(dest: &PathBuf, bytes: &[u8]) -> Result<()> {
    let tmp = dest.with_extension("tmp");
    std::fs::write(&tmp, bytes).with_context(|| format!("write {}", tmp.display()))?;
    std::fs::rename(&tmp, dest).with_context(|| format!("rename {}", dest.display()))?;
    Ok(())
}
