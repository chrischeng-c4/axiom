---
id: vat-source-projects-vat-src-store-rs
summary: Source replay payload for projects/vat/src/store.rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: copy-on-write-fork-and-snapshot-lifecycle
    claim: copy-on-write-fork-and-snapshot-lifecycle
    coverage: full
    rationale: "This source replay TD preserves vat's copy-on-write workspace, agent-legible state, resource isolation, and host GPU behavior."
---

# Source TD: projects/vat/src/store.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/store.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Vat` | projects/vat/src/store.rs | struct | pub | 26 |  |
| `base_manifest` | projects/vat/src/store.rs | function | pub | 64 | base_manifest(&self) -> Result<Manifest> |
| `base_manifest_path` | projects/vat/src/store.rs | function | pub | 44 | base_manifest_path(&self) -> PathBuf |
| `changes` | projects/vat/src/store.rs | function | pub | 71 | changes(&self) -> Result<ChangeSet> |
| `create` | projects/vat/src/store.rs | function | pub | 115 | create(     id: &str,     name: Option<String>,     spec: EnvSpec,     rootfs_source: Option<&std::path::Path>,     lineage: Vec<String>, ) -> Result<Vat> |
| `events_path` | projects/vat/src/store.rs | function | pub | 41 | events_path(&self) -> PathBuf |
| `list` | projects/vat/src/store.rs | function | pub | 175 | list() -> Result<Vec<Vat>> |
| `load` | projects/vat/src/store.rs | function | pub | 161 | load(id: &str) -> Result<Vat> |
| `log` | projects/vat/src/store.rs | function | pub | 60 | log(&self, ev: Event) -> Result<()> |
| `meta_path` | projects/vat/src/store.rs | function | pub | 38 | meta_path(&self) -> PathBuf |
| `project` | projects/vat/src/store.rs | function | pub | 78 | project(&self) -> Result<VatState> |
| `remove` | projects/vat/src/store.rs | function | pub | 198 | remove(id: &str) -> Result<()> |
| `rootfs` | projects/vat/src/store.rs | function | pub | 35 | rootfs(&self) -> PathBuf |
| `save` | projects/vat/src/store.rs | function | pub | 51 | save(&mut self) -> Result<()> |
## Source
<!-- type: source lang: rust -->

`````rust
//! The vat store: create, load, list, and remove vats on disk, and project a
//! [`VatState`] from persisted [`VatMeta`] plus live computation.

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use chrono::Utc;

use crate::event::{self, Event, EventKind};
use crate::gpu;
use crate::overlay::{self, Manifest};
use crate::paths::{self, file};
use crate::spec::EnvSpec;
use crate::state::{ChangeSet, Status, VatMeta, VatState, WorkspaceInfo};

/// Bounded sample size per change category in projected state.
const CHANGE_SAMPLE: usize = 20;

/// Number of trailing events surfaced in projected state.
const EVENTS_TAIL: usize = 12;

/// A handle to one vat directory plus its loaded metadata.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-store-rs.md#source
pub struct Vat {
    pub dir: PathBuf,
    pub meta: VatMeta,
}

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-store-rs.md#source
impl Vat {
    // --- paths -----------------------------------------------------------

    pub fn rootfs(&self) -> PathBuf {
        self.dir.join(file::ROOTFS)
    }
    pub fn meta_path(&self) -> PathBuf {
        self.dir.join(file::META)
    }
    pub fn events_path(&self) -> PathBuf {
        self.dir.join(file::EVENTS)
    }
    pub fn base_manifest_path(&self) -> PathBuf {
        self.dir.join(file::BASE_MANIFEST)
    }

    // --- persistence -----------------------------------------------------

    /// Write `meta.json` (touches `updated_at`).
    pub fn save(&mut self) -> Result<()> {
        self.meta.updated_at = Utc::now();
        let json = serde_json::to_vec_pretty(&self.meta).context("serialize meta")?;
        std::fs::write(self.meta_path(), json)
            .with_context(|| format!("write {}", self.meta_path().display()))?;
        Ok(())
    }

    /// Append an event to this vat's log.
    pub fn log(&self, ev: Event) -> Result<()> {
        event::append(&self.events_path(), &ev)
    }

    pub fn base_manifest(&self) -> Result<Manifest> {
        overlay::load_manifest(&self.base_manifest_path())
    }

    // --- projection ------------------------------------------------------

    /// Live filesystem changes vs. the captured baseline.
    pub fn changes(&self) -> Result<ChangeSet> {
        let base = self.base_manifest()?;
        let now = overlay::manifest_of(&self.rootfs())?;
        Ok(overlay::diff(&base, &now))
    }

    /// Build the full agent-legible [`VatState`].
    pub fn project(&self) -> Result<VatState> {
        let changes = self.changes().unwrap_or_default();
        let now = overlay::manifest_of(&self.rootfs()).unwrap_or_default();
        let size_bytes = now.values().map(|s| s.size).sum();
        let events_tail = event::tail(&self.events_path(), EVENTS_TAIL)?;

        Ok(VatState {
            id: self.meta.id.clone(),
            name: self.meta.name.clone(),
            status: self.meta.status.clone(),
            created_at: self.meta.created_at,
            updated_at: self.meta.updated_at,
            spec: self.meta.spec.clone(),
            lineage: self.meta.lineage.clone(),
            last_run: self.meta.last_run.clone(),
            test_run: self.meta.test_run.clone(),
            workspace: WorkspaceInfo {
                rootfs: self.rootfs().to_string_lossy().into_owned(),
                file_count: now.len(),
                size_bytes,
            },
            changes: changes.summary(CHANGE_SAMPLE),
            gpu: gpu::detect(),
            events_tail,
        })
    }
}

// --- store-level operations ----------------------------------------------

/// Create a new vat directory with the given spec and a fresh rootfs.
///
/// `rootfs_source` is the directory to copy-on-write clone into the vat's
/// rootfs; `None` creates an empty rootfs. `lineage` carries ancestor ids when
/// forking. The base manifest is captured immediately so later diffs are
/// relative to creation time.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-store-rs.md#source
pub fn create(
    id: &str,
    name: Option<String>,
    spec: EnvSpec,
    rootfs_source: Option<&std::path::Path>,
    lineage: Vec<String>,
) -> Result<Vat> {
    let dir = paths::vat_dir(id)?;
    if dir.exists() {
        bail!("vat already exists: {id}");
    }
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;

    let rootfs = dir.join(file::ROOTFS);
    match rootfs_source {
        Some(src) => overlay::clone_tree(src, &rootfs)
            .with_context(|| format!("clone {} into rootfs", src.display()))?,
        None => std::fs::create_dir_all(&rootfs).context("create empty rootfs")?,
    }

    // Capture the diff baseline up front.
    let manifest = overlay::manifest_of(&rootfs)?;
    overlay::save_manifest(&dir.join(file::BASE_MANIFEST), &manifest)?;

    let now = Utc::now();
    let mut vat = Vat {
        dir,
        meta: VatMeta {
            id: id.to_string(),
            name,
            status: Status::Created,
            created_at: now,
            updated_at: now,
            spec,
            lineage,
            last_run: None,
            test_run: None,
        },
    };
    vat.save()?;
    vat.log(Event::new(EventKind::Created, format!("created vat {id}")))?;
    Ok(vat)
}

/// Load a vat by id.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-store-rs.md#source
pub fn load(id: &str) -> Result<Vat> {
    let dir = paths::vat_dir(id)?;
    let meta_path = dir.join(file::META);
    if !meta_path.exists() {
        bail!("no such vat: {id}");
    }
    let bytes =
        std::fs::read(&meta_path).with_context(|| format!("read {}", meta_path.display()))?;
    let meta: VatMeta = serde_json::from_slice(&bytes).context("parse meta.json")?;
    Ok(Vat { dir, meta })
}

/// List all vats (unsorted directory order; callers sort as needed).
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-store-rs.md#source
pub fn list() -> Result<Vec<Vat>> {
    let dir = paths::vats_dir()?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().into_owned();
        match load(&id) {
            Ok(v) => out.push(v),
            // A half-written vat dir shouldn't break `vat ls`.
            Err(_) => continue,
        }
    }
    Ok(out)
}

/// Remove a vat directory entirely.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-store-rs.md#source
pub fn remove(id: &str) -> Result<()> {
    let dir = paths::vat_dir(id)?;
    if !dir.exists() {
        bail!("no such vat: {id}");
    }
    std::fs::remove_dir_all(&dir).with_context(|| format!("remove {}", dir.display()))?;
    Ok(())
}

`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/vat/src/store.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/vat/tech-design/semantic/vat-src.md#schema.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-vat-src-store-rs-source-replay-superseded>"
```
