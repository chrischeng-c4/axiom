// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-src.md#schema
// CODEGEN-BEGIN
//! Segment-checkpoint persistence store (Stage 2 Phase 2f-2) — the disk engine
//! wired in as the running binary's "RDB".
//!
//! A segment checkpoint SUPERSEDES the CBOR RDB ([`crate::rdb`]): instead of
//! serializing the whole materialized index into one `rdb-<seq>.lrb` blob, the
//! engine seals every collection into columnar mmap segments
//! (`<collection>/<field>.lseg` + EID column + a `_schema.json` sidecar) under a
//! generation directory `gen-<seq>/`. Cold start reopens those segments WITHOUT a
//! whole-collection load (the forward payload stays demand-paged on the mmaps),
//! then tails the WAL from `<seq> + 1`.
//!
//! ## Atomicity
//!
//! A checkpoint is written by:
//!   1. staging the whole generation under a temp dir `.gen-<seq>.tmp/` (removed
//!      first if a prior torn attempt left one),
//!   2. having [`Engine::flush_to_segments`] seal every collection into it,
//!   3. atomically `rename`-ing the temp dir to `gen-<seq>/`.
//!
//! The rename is the commit point: a `gen-<seq>/` directory exists IFF the whole
//! checkpoint was staged successfully, so a torn checkpoint (a crash mid-stage)
//! leaves only a `.gen-*.tmp` dir, which is ignored by load and swept on the next
//! write. This is exactly [`crate::rdb::LocalFsRdbStore`]'s temp-file+rename model
//! lifted to a directory. The sequence in the generation name is the total order
//! — no separate pointer file is needed; the highest `gen-*` is the latest.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};

use crate::storage::Engine;

/// Filesystem-backed segment-checkpoint store: `<root>/gen-<seq>/`. The newest
/// `gen-*` (by sequence) is the latest. Parallels [`crate::rdb::LocalFsRdbStore`]
/// but persists the columnar segment tree instead of a CBOR blob.
/// @spec projects/lumen/tech-design/semantic/lumen-src.md#schema
#[derive(Debug, Clone)]
pub struct SegmentRdbStore {
    root: PathBuf,
}

/// @spec projects/lumen/tech-design/semantic/lumen-src.md#schema
impl SegmentRdbStore {
    /// Open (creating) the checkpoint root directory.
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        std::fs::create_dir_all(&root)
            .with_context(|| format!("create segment-checkpoint dir {}", root.display()))?;
        Ok(Self { root })
    }

    /// The committed generation path for `seq`.
    fn gen_path(&self, seq: u64) -> PathBuf {
        self.root.join(format!("gen-{seq}"))
    }

    /// The staging path for `seq` (renamed to `gen_path` on commit).
    fn staging_path(&self, seq: u64) -> PathBuf {
        self.root.join(format!(".gen-{seq}.tmp"))
    }

    /// Parse the sequence out of a committed `gen-<seq>` directory name.
    fn seq_of(path: &Path) -> Option<u64> {
        path.file_name()?.to_str()?.strip_prefix("gen-")?.parse().ok()
    }

    /// Every committed generation, ascending by sequence. Staging dirs
    /// (`.gen-*.tmp`) and stray entries are ignored.
    fn generations(&self) -> Result<Vec<(u64, PathBuf)>> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(&self.root)
            .with_context(|| format!("read checkpoint root {}", self.root.display()))?
        {
            let path = entry?.path();
            if !path.is_dir() {
                continue;
            }
            if let Some(seq) = Self::seq_of(&path) {
                out.push((seq, path));
            }
        }
        out.sort_by_key(|(seq, _)| *seq);
        Ok(out)
    }

    /// Remove any leftover staging directories from torn prior attempts.
    fn sweep_staging(&self) {
        if let Ok(rd) = std::fs::read_dir(&self.root) {
            for entry in rd.flatten() {
                let p = entry.path();
                if p.is_dir()
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.starts_with(".gen-") && n.ends_with(".tmp"))
                {
                    let _ = std::fs::remove_dir_all(&p);
                }
            }
        }
    }

    /// Checkpoint `engine` at `up_to_seq`: stage a full generation under a temp
    /// dir, seal every collection into it via [`Engine::flush_to_segments`], then
    /// atomically rename it to `gen-<up_to_seq>/`. The rename is the commit point
    /// — a crash before it leaves only the temp dir (swept on the next call), so a
    /// torn checkpoint never replaces a good one.
    pub fn save(&self, engine: &Arc<Engine>, up_to_seq: u64) -> Result<()> {
        self.sweep_staging();
        let staging = self.staging_path(up_to_seq);
        // A re-run at the same seq would collide; start from a clean staging dir.
        let _ = std::fs::remove_dir_all(&staging);
        std::fs::create_dir_all(&staging)
            .with_context(|| format!("create staging {}", staging.display()))?;

        // Seal every collection into the staging dir.
        if let Err(e) = engine.flush_to_segments(&staging, up_to_seq) {
            let _ = std::fs::remove_dir_all(&staging);
            return Err(e).context("flush collections to segment checkpoint");
        }

        let committed = self.gen_path(up_to_seq);
        // A previously-committed generation at the same seq is replaced wholesale;
        // remove it first so the rename of a directory onto a non-empty directory
        // does not fail on platforms that reject it.
        let _ = std::fs::remove_dir_all(&committed);
        std::fs::rename(&staging, &committed).with_context(|| {
            format!("commit checkpoint {} -> {}", staging.display(), committed.display())
        })?;
        Ok(())
    }

    /// Reopen the newest committed checkpoint into a FRESH engine, returning
    /// `(engine, up_to_seq)`, or `None` when the store has no committed
    /// generation. The WAL is tailed from `up_to_seq + 1`. Skips (and the caller
    /// may prune) any torn generation — `generations()` only lists committed dirs,
    /// and a generation with no readable collections returns seq 0.
    pub fn load_latest(&self) -> Result<Option<(Arc<Engine>, u64)>> {
        let engine = Arc::new(Engine::new());
        match self.reopen_into(&engine)? {
            Some(seq) => Ok(Some((engine, seq))),
            None => Ok(None),
        }
    }

    /// Reopen the newest committed checkpoint INTO an existing `engine`, returning
    /// `Some(up_to_seq)` or `None` when the store has no committed generation. Used
    /// by the serving binary's cold start so the checkpoint lands in the same
    /// engine the rest of the node (drain hooks, API state) already wraps.
    pub fn reopen_into(&self, engine: &Arc<Engine>) -> Result<Option<u64>> {
        let Some((_, path)) = self.generations()?.into_iter().next_back() else {
            return Ok(None);
        };
        let seq = engine
            .reopen_from_segment_dir(&path)
            .with_context(|| format!("reopen checkpoint {}", path.display()))?;
        Ok(Some(seq))
    }

    /// Drop committed generations older than the newest `keep` (retention). The
    /// newest `keep` survive; returns how many were removed. Also sweeps any
    /// torn staging dirs.
    pub fn prune(&self, keep: usize) -> Result<usize> {
        self.sweep_staging();
        let all = self.generations()?;
        if all.len() <= keep {
            return Ok(0);
        }
        let to_drop = all.len() - keep;
        let mut removed = 0;
        for (_, path) in all.into_iter().take(to_drop) {
            if std::fs::remove_dir_all(&path).is_ok() {
                removed += 1;
            }
        }
        Ok(removed)
    }

    /// The committed generation sequences, ascending (observability / tests).
    pub fn generation_seqs(&self) -> Result<Vec<u64>> {
        Ok(self.generations()?.into_iter().map(|(seq, _)| seq).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    };
    use std::collections::BTreeMap;

    fn kw_schema() -> CreateCollectionRequest {
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
        CreateCollectionRequest { fields }
    }

    fn index_kw(e: &Engine, eid: &str, v: &str) {
        e.index(
            "u",
            IndexRequest {
                items: vec![IndexItem {
                    external_id: eid.into(),
                    field: "email".into(),
                    value: FieldValue::String(v.into()),
                }],
                request_id: None,
            },
        )
        .unwrap();
    }

    #[test]
    fn save_then_load_round_trips_at_seq() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();

        let src = Arc::new(Engine::new());
        src.create_collection("u", kw_schema()).unwrap();
        index_kw(&src, "u1", "a@x.com");
        store.save(&src, 42).unwrap();

        let (eng, seq) = store.load_latest().unwrap().expect("a checkpoint");
        assert_eq!(seq, 42);
        assert_eq!(eng.stats("u").unwrap().documents_indexed, 1);
    }

    #[test]
    fn load_latest_picks_highest_seq() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let e = Arc::new(Engine::new());
        e.create_collection("u", kw_schema()).unwrap();
        index_kw(&e, "u1", "a@x.com");
        for seq in [10u64, 5, 99, 50] {
            store.save(&e, seq).unwrap();
        }
        assert_eq!(store.load_latest().unwrap().unwrap().1, 99);
    }

    #[test]
    fn prune_keeps_newest() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let e = Arc::new(Engine::new());
        e.create_collection("u", kw_schema()).unwrap();
        index_kw(&e, "u1", "a@x.com");
        for seq in 1..=5u64 {
            store.save(&e, seq).unwrap();
        }
        let removed = store.prune(2).unwrap();
        assert_eq!(removed, 3);
        assert_eq!(store.generation_seqs().unwrap(), vec![4, 5]);
        assert_eq!(store.load_latest().unwrap().unwrap().1, 5);
    }

    #[test]
    fn torn_staging_dir_is_ignored_and_swept() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let e = Arc::new(Engine::new());
        e.create_collection("u", kw_schema()).unwrap();
        index_kw(&e, "u1", "a@x.com");
        store.save(&e, 7).unwrap();

        // Simulate a crash mid-stage: a leftover `.gen-<seq>.tmp` dir.
        std::fs::create_dir_all(dir.path().join(".gen-9.tmp")).unwrap();
        // load_latest still returns the good committed generation, not the torn one.
        assert_eq!(store.load_latest().unwrap().unwrap().1, 7);
        // A subsequent save sweeps the torn staging dir.
        store.save(&e, 8).unwrap();
        assert!(!dir.path().join(".gen-9.tmp").exists());
        assert_eq!(store.load_latest().unwrap().unwrap().1, 8);
    }
}
// CODEGEN-END
