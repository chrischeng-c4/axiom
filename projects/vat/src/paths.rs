// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-src.md#schema
// CODEGEN-BEGIN
//! On-disk layout for vat state.
//!
//! Everything lives under a single root (default `~/.vat`, override with
//! `$VAT_HOME`). One directory per vat keeps the store trivially inspectable
//! by a human *or* an agent with nothing but `ls`:
//!
//! ```text
//! ~/.vat/
//!   vats/
//!     vat-7f3k1q9/
//!       meta.json          persisted VatMeta (id, status, spec, lineage, last_run)
//!       events.jsonl       append-only structured event log
//!       base_manifest.json file stats captured at clone time (diff baseline)
//!       rootfs/            the copy-on-write workspace the command runs in
//!       logs/              per-run stdout/stderr (future)
//! ```

use std::path::PathBuf;

use anyhow::{Context, Result};

/// Root of all vat state. Honors `$VAT_HOME`, else `~/.vat`.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-paths-rs.md#source
pub fn root() -> Result<PathBuf> {
    if let Some(custom) = std::env::var_os("VAT_HOME") {
        return Ok(PathBuf::from(custom));
    }
    let home = dirs::home_dir().context("could not determine home directory (set $VAT_HOME)")?;
    Ok(home.join(".vat"))
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
