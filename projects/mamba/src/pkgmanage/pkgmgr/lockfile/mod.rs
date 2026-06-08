// HANDWRITE-BEGIN gap="missing-generator:hand-written:9c172004" tracker="enhancement-mamba-codegen-rust-public-api" reason="No generator emits a multi-fn public API + a thiserror enum from a yaml schema. Closes when sdd-codegen ships rust-api-from-schema."

//! Lockfile public API.
//!
//! See `.aw/tech-design/projects/mamba/pkgmgr/lockfile.md` for the full
//! design — schema definitions, write/parse flowcharts, R1-R5 contracts.

use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::pkgmanage::pkgmgr::resolver::ResolvedGraph;

mod invalidate;
mod parse;
mod serialize;

/// Highest `format_version` integer this build understands.
/// Bump on every breaking change to the on-disk shape.
pub const MAX_SUPPORTED_FORMAT_VERSION: u32 = 1;

/// @spec .aw/tech-design/projects/mamba/pkgmgr/lockfile.md#schema (Lockfile)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lockfile {
    pub format_version: u32,
    pub input_hash: String,
    #[serde(default, rename = "package")]
    pub packages: Vec<Package>,
}

/// @spec .aw/tech-design/projects/mamba/pkgmgr/lockfile.md#schema (Package)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub sha256: String,
    pub source: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub markers: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<SourceRef>,
}

/// @spec .aw/tech-design/projects/mamba/pkgmgr/lockfile.md#schema (SourceRef)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRef {
    pub kind: SourceRefKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceRefKind {
    Registry,
    Path,
    Git,
}

/// @spec .aw/tech-design/projects/mamba/pkgmgr/lockfile.md#schema (LockfileDiff)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockfileDiff {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<PackageChange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageChange {
    pub name: String,
    pub old_version: String,
    pub new_version: String,
}

/// @spec .aw/tech-design/projects/mamba/pkgmgr/lockfile.md#schema (LockfileError)
#[derive(Debug, Error)]
pub enum LockfileError {
    #[error("io: {detail}")]
    Io { detail: String },
    #[error("toml decode: {detail}")]
    TomlDecode { detail: String },
    #[error("toml encode: {detail}")]
    TomlEncode { detail: String },
    #[error("unsupported format_version {found} (max supported {max_supported})")]
    UnsupportedFormatVersion { found: u32, max_supported: u32 },
    #[error("lockfile stale: stored input_hash {stored} != current {current}")]
    Stale { stored: String, current: String },
    #[error("missing field: {detail}")]
    MissingField { detail: String },
    #[error("unknown source kind: {detail}")]
    UnknownSourceKind { detail: String },
}

impl Lockfile {
    /// R1, R5 — serialize a resolved graph to a deterministic TOML lockfile.
    pub fn write(
        graph: &ResolvedGraph,
        dest: &Path,
        input_hash: &str,
    ) -> Result<(), LockfileError> {
        let toml_text = serialize::to_toml(graph, input_hash)?;
        // Atomic write: tmp + rename.
        let tmp = dest.with_extension("tmp");
        std::fs::write(&tmp, toml_text.as_bytes())
            .map_err(|e| LockfileError::Io { detail: e.to_string() })?;
        std::fs::rename(&tmp, dest)
            .map_err(|e| LockfileError::Io { detail: e.to_string() })?;
        Ok(())
    }

    /// R2, R3, R4 — parse a TOML lockfile back into a `ResolvedGraph`. When
    /// `current_pyproject` is `Some`, recompute its `input_hash` and compare
    /// against the stored value; mismatch returns `LockfileError::Stale`.
    pub fn parse(
        src: &Path,
        current_pyproject: Option<&Path>,
    ) -> Result<ResolvedGraph, LockfileError> {
        let text = std::fs::read_to_string(src)
            .map_err(|e| LockfileError::Io { detail: e.to_string() })?;
        let lockfile = parse::from_toml(&text)?;
        if lockfile.format_version > MAX_SUPPORTED_FORMAT_VERSION {
            return Err(LockfileError::UnsupportedFormatVersion {
                found: lockfile.format_version,
                max_supported: MAX_SUPPORTED_FORMAT_VERSION,
            });
        }
        if let Some(pyproject) = current_pyproject {
            let current = invalidate::compute(pyproject)?;
            if current != lockfile.input_hash {
                return Err(LockfileError::Stale {
                    stored: lockfile.input_hash,
                    current,
                });
            }
        }
        Ok(parse::lift_to_graph(lockfile))
    }

    /// R4 — compute the canonical sha256-hex fingerprint of the source
    /// `pyproject.toml`'s dependency tables.
    pub fn compute_input_hash(pyproject_path: &Path) -> Result<String, LockfileError> {
        invalidate::compute(pyproject_path)
    }

    /// R8 — pure-data diff of two lockfiles.
    pub fn diff(old: &Lockfile, new: &Lockfile) -> LockfileDiff {
        use std::collections::BTreeMap;
        let old_map: BTreeMap<&str, &str> = old
            .packages
            .iter()
            .map(|p| (p.name.as_str(), p.version.as_str()))
            .collect();
        let new_map: BTreeMap<&str, &str> = new
            .packages
            .iter()
            .map(|p| (p.name.as_str(), p.version.as_str()))
            .collect();
        let mut added: Vec<String> = new_map
            .keys()
            .filter(|k| !old_map.contains_key(*k))
            .map(|k| k.to_string())
            .collect();
        let mut removed: Vec<String> = old_map
            .keys()
            .filter(|k| !new_map.contains_key(*k))
            .map(|k| k.to_string())
            .collect();
        let mut changed: Vec<PackageChange> = old_map
            .iter()
            .filter_map(|(name, old_ver)| match new_map.get(name) {
                Some(new_ver) if new_ver != old_ver => Some(PackageChange {
                    name: name.to_string(),
                    old_version: old_ver.to_string(),
                    new_version: new_ver.to_string(),
                }),
                _ => None,
            })
            .collect();
        added.sort();
        removed.sort();
        changed.sort_by(|a, b| a.name.cmp(&b.name));
        LockfileDiff { added, removed, changed }
    }

    /// Convenience: read a lockfile without converting to ResolvedGraph
    /// (used by `diff` callers and tests for AC4 round-trip checks).
    pub fn read_raw(src: &Path) -> Result<Lockfile, LockfileError> {
        let text = std::fs::read_to_string(src)
            .map_err(|e| LockfileError::Io { detail: e.to_string() })?;
        parse::from_toml(&text)
    }
}

// HANDWRITE-END
