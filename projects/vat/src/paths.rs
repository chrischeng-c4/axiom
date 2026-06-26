// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-paths-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! On-disk layout for vat state.
//!
//! Everything lives under a single root (default `<repo>/.vat`, override with
//! `$VAT_HOME`). One directory per vat keeps the store trivially inspectable
//! by a human *or* an agent with nothing but `ls`:
//!
//! ```text
//! .vat/
//!   vats/
//!     vat-7f3k1q9/
//!       meta.json          persisted VatMeta (id, status, spec, lineage, last_run)
//!       events.jsonl       append-only structured event log
//!       base_manifest.json file stats captured at clone time (diff baseline)
//!       rootfs/            the copy-on-write workspace the command runs in
//!       logs/              per-run stdout/stderr (future)
//! ```

use std::path::PathBuf;

use anyhow::Result;

/// Root of all vat state. Honors `$VAT_HOME`, else `<repo>/.vat`, else `./.vat`.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-paths-rs.md#source
pub fn root() -> Result<PathBuf> {
    if let Some(custom) = std::env::var_os("VAT_HOME") {
        return Ok(PathBuf::from(custom));
    }
    let cwd = std::env::current_dir()?;
    Ok(repo_root_from(&cwd).unwrap_or(cwd).join(".vat"))
}

fn repo_root_from(start: &std::path::Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join(".git").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Directory holding every vat (`<root>/vats`).
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-paths-rs.md#source
pub fn vats_dir() -> Result<PathBuf> {
    Ok(root()?.join("vats"))
}

/// Directory for a single vat (`<root>/vats/<id>`).
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-paths-rs.md#source
pub fn vat_dir(id: &str) -> Result<PathBuf> {
    Ok(vats_dir()?.join(id))
}

/// Directory holding standalone `vat cluster` registry entries
/// (`<root>/clusters`). Standalone clusters are not vats, so they live in a
/// sibling tree, one directory per cluster.
/// @spec projects/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#cli
pub fn clusters_dir() -> Result<PathBuf> {
    Ok(root()?.join("clusters"))
}

/// Directory for a single standalone cluster (`<root>/clusters/<name>`).
/// @spec projects/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#cli
pub fn cluster_dir(name: &str) -> Result<PathBuf> {
    Ok(clusters_dir()?.join(name))
}

/// Filenames within a vat directory. Centralized so the layout has one source
/// of truth.
pub mod file {
    pub const META: &str = "meta.json";
    pub const EVENTS: &str = "events.jsonl";
    pub const BASE_MANIFEST: &str = "base_manifest.json";
    pub const ROOTFS: &str = "rootfs";
    pub const LOGS: &str = "logs";
}
// CODEGEN-END
