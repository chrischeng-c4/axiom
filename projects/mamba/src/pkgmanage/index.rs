// `mamba index build` — materialize a frozen local package index from wheels.
//
// The index layout is the one already consumed by `mamba add --index` and
// `mamba lock --index`:
//
//   <INDEX>/<pep503-normalized-name>/<version>/<filename>.whl
//
// The build path parses every selected wheel before creating output
// directories, so malformed wheel filenames fail without partial index writes.

use anyhow::{bail, Context, Result};
use clap::ArgMatches;
use std::fs;
use std::path::{Path, PathBuf};

use crate::pkgmanage::pkgmgr::name_normalize::pep503_normalize;
use crate::pkgmanage::pkgmgr::wheel_filename::parse_wheel_filename;

pub fn cmd_index(sub: &ArgMatches) -> Result<()> {
    match sub.subcommand() {
        Some(("build", build)) => cmd_build(build),
        _ => bail!("expected subcommand: mamba index build"),
    }
}

fn cmd_build(sub: &ArgMatches) -> Result<()> {
    let out_dir = PathBuf::from(
        sub.get_one::<String>("out")
            .context("missing required --out <DIR>")?,
    );
    let inputs = sub
        .get_many::<String>("paths")
        .context("missing required <wheel-or-dir> input")?;

    let mut wheels = Vec::new();
    for input in inputs {
        collect_wheels(Path::new(input), &mut wheels)?;
    }
    wheels.sort();
    wheels.dedup();
    if wheels.is_empty() {
        bail!("no .whl files found in input paths");
    }

    let entries = wheels
        .iter()
        .map(|path| IndexEntry::from_path(path))
        .collect::<Result<Vec<_>>>()?;

    for entry in &entries {
        entry.write_to(&out_dir)?;
    }

    eprintln!(
        "indexed {} wheel(s) into {}",
        entries.len(),
        out_dir.display()
    );
    Ok(())
}

fn collect_wheels(path: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !path.exists() {
        bail!("input path does not exist: {}", path.display());
    }
    if path.is_file() {
        if path.extension().and_then(|s| s.to_str()) != Some("whl") {
            bail!("input file is not a wheel: {}", path.display());
        }
        out.push(path.to_path_buf());
        return Ok(());
    }
    if !path.is_dir() {
        bail!("input path is not a file or directory: {}", path.display());
    }

    let mut children = fs::read_dir(path)
        .with_context(|| format!("read input directory {}", path.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()
        .with_context(|| format!("read input directory {}", path.display()))?;
    children.sort();
    for child in children {
        if child.is_dir() {
            collect_wheels(&child, out)?;
        } else if child.extension().and_then(|s| s.to_str()) == Some("whl") {
            out.push(child);
        }
    }
    Ok(())
}

#[derive(Debug)]
struct IndexEntry {
    source: PathBuf,
    normalized_name: String,
    version: String,
    filename: String,
}

impl IndexEntry {
    fn from_path(path: &Path) -> Result<Self> {
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .with_context(|| format!("wheel path is not valid UTF-8: {}", path.display()))?;
        let wheel = parse_wheel_filename(filename)
            .map_err(|e| anyhow::anyhow!("parse wheel filename `{filename}`: {e}"))?;
        let normalized_name = pep503_normalize(&wheel.distribution);
        if normalized_name.is_empty() {
            bail!("wheel filename `{filename}` has an empty distribution name");
        }
        Ok(IndexEntry {
            source: path.to_path_buf(),
            normalized_name,
            version: wheel.version,
            filename: filename.to_string(),
        })
    }

    fn write_to(&self, out_dir: &Path) -> Result<()> {
        let version_dir = out_dir.join(&self.normalized_name).join(&self.version);
        fs::create_dir_all(&version_dir)
            .with_context(|| format!("create index directory {}", version_dir.display()))?;
        let dest = version_dir.join(&self.filename);
        copy_if_changed(&self.source, &dest)
    }
}

fn copy_if_changed(src: &Path, dest: &Path) -> Result<()> {
    if dest.exists() && same_file_contents(src, dest)? {
        return Ok(());
    }

    let tmp_name = format!(
        ".{}.tmp.{}",
        dest.file_name().and_then(|s| s.to_str()).unwrap_or("wheel"),
        std::process::id()
    );
    let tmp = dest.with_file_name(tmp_name);
    if tmp.exists() {
        fs::remove_file(&tmp).with_context(|| format!("remove stale temp {}", tmp.display()))?;
    }
    fs::copy(src, &tmp).with_context(|| format!("copy {} to {}", src.display(), tmp.display()))?;
    if dest.exists() {
        fs::remove_file(dest)
            .with_context(|| format!("replace indexed wheel {}", dest.display()))?;
    }
    fs::rename(&tmp, dest).with_context(|| {
        format!(
            "move indexed wheel from {} to {}",
            tmp.display(),
            dest.display()
        )
    })?;
    Ok(())
}

fn same_file_contents(a: &Path, b: &Path) -> Result<bool> {
    let a_meta = fs::metadata(a).with_context(|| format!("stat {}", a.display()))?;
    let b_meta = fs::metadata(b).with_context(|| format!("stat {}", b.display()))?;
    if a_meta.len() != b_meta.len() {
        return Ok(false);
    }
    let a_bytes = fs::read(a).with_context(|| format!("read {}", a.display()))?;
    let b_bytes = fs::read(b).with_context(|| format!("read {}", b.display()))?;
    Ok(a_bytes == b_bytes)
}
