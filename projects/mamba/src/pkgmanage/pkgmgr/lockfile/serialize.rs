// HANDWRITE-BEGIN gap="missing-generator:hand-written:ac50240a" tracker="enhancement-sdd-codegen-rust-serde-toml" reason="Generator can emit serde derives but cannot yet emit deterministic-config wrappers around toml::to_string_pretty."

//! `ResolvedGraph` -> deterministic TOML bytes (R1, R5).
//!
//! See `.aw/tech-design/projects/mamba/pkgmgr/lockfile.md#logic` —
//! the `lockfile-write` flowchart drives the projection.

use crate::pkgmanage::pkgmgr::resolver::ResolvedGraph;

use super::{Lockfile, LockfileError, Package};

/// @spec .aw/tech-design/projects/mamba/pkgmgr/lockfile.md#logic (lockfile-write)
pub(super) fn to_toml(
    graph: &ResolvedGraph,
    input_hash: &str,
) -> Result<String, LockfileError> {
    let mut packages: Vec<Package> = graph
        .nodes
        .iter()
        .map(|node| {
            let sha256 = node
                .files
                .first()
                .map(|f| f.digest.clone())
                .unwrap_or_default();
            let mut deps: Vec<String> = node
                .requires
                .iter()
                .map(|r| r.name.clone())
                .collect();
            deps.sort();
            deps.dedup();
            Package {
                name: node.name.clone(),
                version: node.version.clone(),
                sha256,
                source: synthesise_source(&node.name, &node.version),
                dependencies: deps,
                markers: None,
                source_ref: None,
            }
        })
        .collect();
    packages.sort_by(|a, b| a.name.cmp(&b.name));

    let lockfile = Lockfile {
        format_version: 1,
        input_hash: input_hash.to_string(),
        packages,
    };

    toml::to_string_pretty(&lockfile)
        .map_err(|e| LockfileError::TomlEncode { detail: e.to_string() })
}

/// Synthesised provenance URL for Phase 1.4. The resolver does not yet
/// thread the index-client URL through `ResolvedNode`; until it does, the
/// lockfile records a stable `pypi://` placeholder so round-trip is
/// deterministic. Closed by a future resolver enhancement (carry url on
/// ResolvedNode); the format stays stable across that change.
fn synthesise_source(name: &str, version: &str) -> String {
    format!("pypi://{}/{}", name, version)
}

// HANDWRITE-END
