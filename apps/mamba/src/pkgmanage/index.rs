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
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::pkgmanage::pkgmgr::name_normalize::pep503_normalize;
use crate::pkgmanage::pkgmgr::pip_install::is_extra_marker;
use crate::pkgmanage::pkgmgr::wheel_build::parse_core_metadata;
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
        copy_if_changed(&self.source, &dest)?;
        self.write_metadata(&version_dir, &dest)
    }

    /// Write `<version_dir>/metadata.toml` beside the staged wheel: the
    /// digest of the staged bytes and the dependency edges declared in the
    /// wheel's own `dist-info/METADATA`, so `mamba lock`/`mamba add` can
    /// resolve the transitive closure and verify the artifact without
    /// re-reading the wheel.
    fn write_metadata(&self, version_dir: &Path, wheel_path: &Path) -> Result<()> {
        let bytes = fs::read(wheel_path)
            .with_context(|| format!("read staged wheel {}", wheel_path.display()))?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let sha256 = format!("{:x}", hasher.finalize());
        // A staged wheel that is not a real zip (or has no dist-info/METADATA)
        // still gets indexed with an empty dependency set rather than
        // failing the whole `index build` — the sha256/path stay usable for
        // `mamba add`/`mamba lock` even when no `Requires-Dist:` can be read.
        let requires = read_requires_dist(wheel_path).unwrap_or_default();
        let meta_path = version_dir.join("metadata.toml");
        let body = render_metadata_toml(
            &self.normalized_name,
            &self.version,
            &self.filename,
            &sha256,
            &requires,
        );
        fs::write(&meta_path, body)
            .with_context(|| format!("write {}", meta_path.display()))?;
        Ok(())
    }
}

/// Read `Requires-Dist:` lines from a wheel's `dist-info/METADATA` through
/// `parse_core_metadata`, dropping an `extra ==` marker entry entirely and
/// stripping any other marker while keeping the requirement (frozen
/// decision: the dependency channel is the wheel's own METADATA, nothing
/// else).
fn read_requires_dist(wheel_path: &Path) -> Result<Vec<String>> {
    let file = fs::File::open(wheel_path)
        .with_context(|| format!("open wheel {}", wheel_path.display()))?;
    let mut zip = zip::ZipArchive::new(file)
        .with_context(|| format!("read wheel {}", wheel_path.display()))?;
    for i in 0..zip.len() {
        let mut entry = zip
            .by_index(i)
            .with_context(|| format!("read wheel entry {i} in {}", wheel_path.display()))?;
        let name = entry.name().to_string();
        if !name.ends_with("/METADATA") || !name.contains(".dist-info/") {
            continue;
        }
        let mut body = String::new();
        entry
            .read_to_string(&mut body)
            .with_context(|| format!("read METADATA in {}", wheel_path.display()))?;
        let meta = parse_core_metadata(&body)
            .map_err(|e| anyhow::anyhow!("parse METADATA in {}: {e}", wheel_path.display()))?;
        return Ok(clean_requires(&meta.requires_dist));
    }
    bail!("wheel {} has no dist-info/METADATA", wheel_path.display())
}

// -- #4206: `metadata.toml` rendering for `mamba index build` --
//
// These are the pure pieces of the frozen-index metadata channel: filtering
// a wheel's `Requires-Dist:` entries down to the ones `mamba add`/`mamba
// lock` should resolve against, and rendering the `metadata.toml` TOML body
// written beside each staged wheel. Colocated unit tests in
// `apps/mamba/src/pkgmanage/tests.rs` cover these directly (`pub(crate)` so
// that sibling-module test file can reach them); the black-box
// `apps/mamba/e2e/pkgmgr_lock_frozen_transitive.rs` case judges the
// externally observable `mamba.lock` shape these functions feed.

/// Drop `extra == "..."` marker requirements entirely (they gate optional
/// extras this index format does not model) and strip any other `;marker`
/// suffix from the rest while keeping the requirement itself.
pub(crate) fn clean_requires(raw: &[String]) -> Vec<String> {
    raw.iter()
        .filter(|r| !is_extra_marker(r))
        .map(|r| match r.split_once(';') {
            Some((head, _)) => head.trim().to_string(),
            None => r.trim().to_string(),
        })
        .collect()
}

/// Render the `metadata.toml` body written beside a staged wheel.
pub(crate) fn render_metadata_toml(
    name: &str,
    version: &str,
    filename: &str,
    sha256: &str,
    requires: &[String],
) -> String {
    let mut out = String::new();
    out.push_str(&format!("name = \"{}\"\n", escape_toml_string(name)));
    out.push_str(&format!("version = \"{}\"\n", escape_toml_string(version)));
    out.push_str(&format!(
        "filename = \"{}\"\n",
        escape_toml_string(filename)
    ));
    out.push_str(&format!("sha256 = \"{sha256}\"\n"));
    let items = requires
        .iter()
        .map(|r| format!("\"{}\"", escape_toml_string(r)))
        .collect::<Vec<_>>()
        .join(", ");
    out.push_str(&format!("requires = [{items}]\n"));
    out
}

/// Escape `\` and `"` for a TOML basic string body.
pub(crate) fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
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
