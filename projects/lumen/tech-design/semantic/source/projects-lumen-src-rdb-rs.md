---
id: projects-lumen-src-rdb-rs
capability_refs:
  - id: "search"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/rdb.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/rdb.rs` captured as a per-file rust-source-unit (td_ast) during lumen standardization onto the per-file codegen ladder.

### Symbols

| Name | Target | Kind | Visibility |
|------|--------|------|------------|
| `RdbSnapshot` | projects/lumen/src/rdb.rs | struct | pub |
| `capture` | projects/lumen/src/rdb.rs | function | pub |
| `restore_into` | projects/lumen/src/rdb.rs | function | pub |
| `encode` | projects/lumen/src/rdb.rs | function | pub |
| `decode` | projects/lumen/src/rdb.rs | function | pub |
| `RdbStore` | projects/lumen/src/rdb.rs | trait | pub |
| `LocalFsRdbStore` | projects/lumen/src/rdb.rs | struct | pub |
| `new` | projects/lumen/src/rdb.rs | function | pub |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! RDB — point-in-time snapshots of the materialized index, tagged with
//! the WAL sequence they correspond to.
//!
//! This is the Redis-RDB half of the data plane (the WAL is the AOF
//! half). Its job is to **bound cold-start and broker retention**: a
//! fresh serving node loads the latest RDB to get a baseline at
//! `up_to_seq`, then tails the log from `up_to_seq + 1` instead of
//! replaying the whole stream. The broker (NATS) then only needs to
//! retain history back to the oldest live RDB.
//!
//! An [`RdbStore`] is where snapshots live. v1 ships [`LocalFsRdbStore`]
//! (a directory of `rdb-<seq>.lrb` — bincode + lz4); S3/GCS adapters slot in
//! behind the same trait (the bytes and layout are identical, only the
//! put/get changes).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::storage::{Engine, SnapshotV1};

/// A snapshot plus the log sequence it is current as of.
#[derive(Debug, Serialize, Deserialize)]
pub struct RdbSnapshot {
    /// The WAL sequence this snapshot incorporates. A node that loads
    /// this baseline tails the log from `up_to_seq + 1`.
    pub up_to_seq: u64,
    pub snapshot: SnapshotV1,
}

impl RdbSnapshot {
    /// Capture the engine's current state as a snapshot tagged with
    /// `up_to_seq` (the caller passes the coordinator's applied
    /// sequence so the tag matches exactly what the snapshot contains).
    pub fn capture(engine: &Engine, up_to_seq: u64) -> Result<Self> {
        Ok(Self {
            up_to_seq,
            snapshot: engine.snapshot()?,
        })
    }

    /// Restore this snapshot into a (fresh) engine.
    pub fn restore_into(self, engine: &Engine) -> Result<()> {
        engine.restore(self.snapshot)
    }

    /// Encode as CBOR + lz4 (compact binary). Vectors are raw IEEE-754, not
    /// text float arrays — far smaller and faster to parse than JSON.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut raw = Vec::new();
        ciborium::into_writer(self, &mut raw).map_err(|e| anyhow::anyhow!("cbor encode RDB: {e}"))?;
        Ok(lz4_flex::compress_prepend_size(&raw))
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let raw = lz4_flex::decompress_size_prepended(bytes).context("lz4 decompress RDB")?;
        ciborium::from_reader(&raw[..]).map_err(|e| anyhow::anyhow!("cbor decode RDB: {e}"))
    }
}

/// Where RDB snapshots are persisted. Object-store adapters (S3/GCS)
/// implement this with the same byte layout as [`LocalFsRdbStore`].
#[async_trait]
pub trait RdbStore: Send + Sync {
    /// Persist `rdb` and make it the new latest.
    async fn save(&self, rdb: &RdbSnapshot) -> Result<()>;

    /// Load the most recent snapshot, or `None` if the store is empty.
    async fn load_latest(&self) -> Result<Option<RdbSnapshot>>;

    /// Drop snapshots older than the newest `keep` (retention).
    async fn prune(&self, keep: usize) -> Result<usize>;
}

/// Filesystem-backed RDB store: `<root>/rdb-<seq>.lrb`. The newest
/// `rdb-*.lrb` (by sequence) is the latest — no separate pointer file
/// needed, the sequence in the name is the total order.
#[derive(Debug, Clone)]
pub struct LocalFsRdbStore {
    root: PathBuf,
}

impl LocalFsRdbStore {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        std::fs::create_dir_all(&root)
            .with_context(|| format!("create RDB dir {}", root.display()))?;
        Ok(Self { root })
    }

    fn seq_of(path: &Path) -> Option<u64> {
        path.file_stem()?
            .to_str()?
            .strip_prefix("rdb-")?
            .parse()
            .ok()
    }

    fn snapshots(&self) -> Result<Vec<(u64, PathBuf)>> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(&self.root)? {
            let path = entry?.path();
            if path.extension().and_then(|e| e.to_str()) == Some("lrb") {
                if let Some(seq) = Self::seq_of(&path) {
                    out.push((seq, path));
                }
            }
        }
        out.sort_by_key(|(seq, _)| *seq);
        Ok(out)
    }
}

#[async_trait]
impl RdbStore for LocalFsRdbStore {
    async fn save(&self, rdb: &RdbSnapshot) -> Result<()> {
        let bytes = rdb.encode()?;
        let path = self.root.join(format!("rdb-{}.lrb", rdb.up_to_seq));
        // Write to a temp file then rename for atomic visibility.
        let tmp = self.root.join(format!(".rdb-{}.lrb.tmp", rdb.up_to_seq));
        let (tmp2, path2) = (tmp.clone(), path.clone());
        tokio::task::spawn_blocking(move || -> Result<()> {
            std::fs::write(&tmp2, &bytes).with_context(|| format!("write {}", tmp2.display()))?;
            std::fs::rename(&tmp2, &path2)
                .with_context(|| format!("rename to {}", path2.display()))?;
            Ok(())
        })
        .await
        .context("spawn_blocking join")??;
        Ok(())
    }

    async fn load_latest(&self) -> Result<Option<RdbSnapshot>> {
        let Some((_, path)) = self.snapshots()?.into_iter().last() else {
            return Ok(None);
        };
        let bytes = tokio::task::spawn_blocking(move || std::fs::read(&path))
            .await
            .context("spawn_blocking join")??;
        Ok(Some(RdbSnapshot::decode(&bytes)?))
    }

    async fn prune(&self, keep: usize) -> Result<usize> {
        let all = self.snapshots()?;
        if all.len() <= keep {
            return Ok(0);
        }
        let to_drop = all.len() - keep;
        let mut removed = 0;
        for (_, path) in all.into_iter().take(to_drop) {
            if std::fs::remove_file(&path).is_ok() {
                removed += 1;
            }
        }
        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    };
    use std::collections::BTreeMap;

    fn seeded_engine() -> Engine {
        let e = Engine::new();
        let mut fields = BTreeMap::new();
        fields.insert(
            "email".to_string(),
            FieldSpec {
                field_type: FieldType::Keyword,
                analyzer: None,
                multi: None,
                dim: None,
                metric: None,
                backend: None,
                quantize: None,
            },
        );
        e.create_collection("u", CreateCollectionRequest { fields })
            .unwrap();
        e.index(
            "u",
            IndexRequest {
                items: vec![IndexItem {
                    external_id: "u1".into(),
                    field: "email".into(),
                    value: FieldValue::String("a@x.com".into()),
                }],
                request_id: None,
            },
        )
        .unwrap();
        e
    }

    #[tokio::test]
    async fn save_then_load_latest_round_trips_at_seq() {
        let dir = std::env::temp_dir().join(format!("lumen-rdb-{}", std::process::id()));
        let store = LocalFsRdbStore::new(&dir).unwrap();

        let src = seeded_engine();
        let rdb = RdbSnapshot::capture(&src, 42).unwrap();
        store.save(&rdb).await.unwrap();

        let loaded = store.load_latest().await.unwrap().expect("a snapshot");
        assert_eq!(loaded.up_to_seq, 42);

        // Restore into a fresh engine and confirm the data is there.
        let dst = Engine::new();
        loaded.restore_into(&dst).unwrap();
        assert_eq!(dst.stats("u").unwrap().documents_indexed, 1);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn load_latest_picks_highest_seq() {
        let dir = std::env::temp_dir().join(format!("lumen-rdb-hi-{}", std::process::id()));
        let store = LocalFsRdbStore::new(&dir).unwrap();
        let e = seeded_engine();
        for seq in [10u64, 5, 99, 50] {
            store
                .save(&RdbSnapshot::capture(&e, seq).unwrap())
                .await
                .unwrap();
        }
        assert_eq!(store.load_latest().await.unwrap().unwrap().up_to_seq, 99);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn prune_keeps_newest() {
        let dir = std::env::temp_dir().join(format!("lumen-rdb-prune-{}", std::process::id()));
        let store = LocalFsRdbStore::new(&dir).unwrap();
        let e = seeded_engine();
        for seq in 1..=5u64 {
            store
                .save(&RdbSnapshot::capture(&e, seq).unwrap())
                .await
                .unwrap();
        }
        let removed = store.prune(2).await.unwrap();
        assert_eq!(removed, 3);
        // Newest (seq 5) survives.
        assert_eq!(store.load_latest().await.unwrap().unwrap().up_to_seq, 5);
        std::fs::remove_dir_all(&dir).ok();
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/rdb.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/rdb.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```
