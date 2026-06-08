---
id: vat-source-projects-vat-src-paths-rs
summary: Source replay payload for projects/vat/src/paths.rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: copy-on-write-fork-and-snapshot-lifecycle
    claim: copy-on-write-fork-and-snapshot-lifecycle
    coverage: full
    rationale: "This source replay TD preserves vat's copy-on-write workspace, agent-legible state, resource isolation, and host GPU behavior."
---

# Source TD: projects/vat/src/paths.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/paths.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `file` | projects/vat/src/paths.rs | module | pub | 48 |  |
| `root` | projects/vat/src/paths.rs | function | pub | 26 | root() -> Result<PathBuf> |
| `vat_dir` | projects/vat/src/paths.rs | function | pub | 42 | vat_dir(id: &str) -> Result<PathBuf> |
| `vats_dir` | projects/vat/src/paths.rs | function | pub | 36 | vats_dir() -> Result<PathBuf> |
## Source
<!-- type: source lang: rust -->

`````rust
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
`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/vat/src/paths.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/vat/tech-design/semantic/vat-src.md#schema.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-vat-src-paths-rs-source-replay-superseded>"
```
