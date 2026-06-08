// HANDWRITE-BEGIN gap="missing-generator:hand-written:baef157d" tracker="enhancement-sdd-codegen-rust-serde-toml" reason="Generator does not emit version-bound checks against const MAX_SUPPORTED_FORMAT_VERSION."

//! TOML bytes -> `ResolvedGraph` (R2, R3 enforcement).
//!
//! See `.aw/tech-design/projects/mamba/pkgmgr/lockfile.md#logic` —
//! the `lockfile-parse` flowchart drives the lift.

use std::collections::BTreeSet;

use crate::pkgmanage::pkgmgr::resolver::{ResolvedGraph, ResolvedNode, Requirement};
use crate::pkgmanage::pkgmgr::types::FileHash;

use super::{Lockfile, LockfileError};

/// @spec .aw/tech-design/projects/mamba/pkgmgr/lockfile.md#logic (lockfile-parse)
pub(super) fn from_toml(text: &str) -> Result<Lockfile, LockfileError> {
    toml::from_str::<Lockfile>(text)
        .map_err(|e| LockfileError::TomlDecode { detail: e.to_string() })
}

/// @spec .aw/tech-design/projects/mamba/pkgmgr/lockfile.md#logic (lockfile-parse)
///
/// Lift a parsed `Lockfile` to a `ResolvedGraph`. Version-range information
/// is intentionally lost: the lockfile pins exact versions, so the rebuilt
/// `Requirement` carries only the name with empty specifiers.
pub(super) fn lift_to_graph(lock: Lockfile) -> ResolvedGraph {
    // Roots = node names that appear in NO other node's `dependencies` list.
    let referenced: BTreeSet<&str> = lock
        .packages
        .iter()
        .flat_map(|p| p.dependencies.iter().map(|d| d.as_str()))
        .collect();
    let mut roots: Vec<String> = lock
        .packages
        .iter()
        .filter(|p| !referenced.contains(p.name.as_str()))
        .map(|p| p.name.clone())
        .collect();
    roots.sort();

    let nodes = lock
        .packages
        .into_iter()
        .map(|p| ResolvedNode {
            name: p.name,
            version: p.version,
            files: vec![FileHash {
                algorithm: "sha256".to_string(),
                digest: p.sha256,
            }],
            requires: p
                .dependencies
                .into_iter()
                .map(|name| Requirement {
                    name,
                    specifiers: Vec::new(),
                    extras: Vec::new(),
                    marker: None,
                })
                .collect(),
        })
        .collect();

    ResolvedGraph { nodes, roots }
}

// HANDWRITE-END
