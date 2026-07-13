// SPEC-MANAGED: apps/lumen/tech-design/semantic/source/projects-lumen-src-segment_rdb-rs.md#rust-source-unit
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
//!
//! ## Crash-safe same-seq swap (#1444)
//!
//! A same-seq re-save (routine: reshard apply/evict mutate engine state
//! without advancing `applied_seq`, and the periodic snapshotter re-saves
//! every tick with no skip condition) must replace an ALREADY-COMMITTED
//! `gen-<seq>/` with the freshly-staged one. Deleting the old generation
//! before renaming staging in would leave a window where a crash destroys
//! the only durable copy of the previous good state while the new one is
//! still hidden under its dot-prefixed staging name. Instead the old
//! generation is moved ASIDE (`gen-<seq>.old/`) first, the staging dir is
//! renamed onto `gen-<seq>/` (the same atomic commit point as the
//! new-seq path), and only then is the aside copy removed:
//!   1. `gen-<seq>/` -> `gen-<seq>.old/` (rename, cheap, same filesystem),
//!   2. `.gen-<seq>.tmp/` -> `gen-<seq>/` (the commit point),
//!   3. remove `gen-<seq>.old/` (best-effort cleanup).
//! `.old`-suffixed names never parse as a `gen-<seq>` sequence (see
//! [`SegmentRdbStore::seq_of`]), so they are already invisible to
//! `generations()`/`load_latest`. [`SegmentRdbStore::reconcile_aside`]
//! additionally self-heals a crash between steps 1 and 2: if `gen-<seq>/`
//! is missing but `gen-<seq>.old/` is present, the aside copy is renamed
//! back — recovering exactly the last successful save's predecessor at
//! that seq. If both exist (crash after step 2, before step 3), the aside
//! is stale and is removed. `reconcile_aside` runs under `save_lock`
//! wherever a generation is committed or read back (`save`, `prune`,
//! `reopen_into`), so recovery is guaranteed by the time any caller reads
//! the newest generation, including at cold start.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};

use crate::storage::Engine;

/// Filesystem-backed segment-checkpoint store: `<root>/gen-<seq>/`. The newest
/// `gen-*` (by sequence) is the latest. Parallels [`crate::rdb::LocalFsRdbStore`]
/// but persists the columnar segment tree instead of a CBOR blob.
///
/// ## Concurrency (#1397)
///
/// The serving binary shares one store instance between the on-demand
/// checkpoint sink (`POST /admin/checkpoint`) and the periodic snapshotter —
/// both call [`Self::save`]/[`Self::prune`] on clones of the same
/// `Arc<SegmentRdbStore>`. Without serialization, one caller's
/// [`Self::sweep_staging`] can delete another caller's in-flight staging
/// dir, and two concurrent `save`s at the same `applied_seq` (routine during
/// a quiet cutover, since reshard apply/evict mutate state without
/// advancing `applied_seq`) can interleave their stage/flush/rename steps
/// into a torn `gen-<seq>` directory that `load_latest` would then pick with
/// no integrity check. `save_lock` fully serializes every mutating call on
/// this store, so a caller only ever observes a fully-old or fully-new
/// generation, never a partial one. It is wrapped in its own `Arc` (rather
/// than relying on the struct's derived `Clone`) so the lock is shared even
/// if a `SegmentRdbStore` value itself — not just an outer `Arc` around it —
/// is cloned.
#[derive(Debug, Clone)]
/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-segment_rdb-rs.md#source
pub struct SegmentRdbStore {
    root: PathBuf,
    save_lock: Arc<Mutex<()>>,
}

/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-segment_rdb-rs.md#source
impl SegmentRdbStore {
    /// Open (creating) the checkpoint root directory.
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        std::fs::create_dir_all(&root)
            .with_context(|| format!("create segment-checkpoint dir {}", root.display()))?;
        Ok(Self {
            root,
            save_lock: Arc::new(Mutex::new(())),
        })
    }

    /// The committed generation path for `seq`.
    fn gen_path(&self, seq: u64) -> PathBuf {
        self.root.join(format!("gen-{seq}"))
    }

    /// The staging path for `seq` (renamed to `gen_path` on commit).
    fn staging_path(&self, seq: u64) -> PathBuf {
        self.root.join(format!(".gen-{seq}.tmp"))
    }

    /// The aside path a same-seq re-save moves the OLD committed generation
    /// to before renaming staging onto `gen_path` (#1444). Never parses as a
    /// sequence via [`Self::seq_of`], so it is invisible to `generations()`
    /// until [`Self::reconcile_aside`] resolves it.
    fn aside_path(&self, seq: u64) -> PathBuf {
        self.root.join(format!("gen-{seq}.old"))
    }

    /// Parse the sequence out of a committed `gen-<seq>` directory name.
    fn seq_of(path: &Path) -> Option<u64> {
        path.file_name()?
            .to_str()?
            .strip_prefix("gen-")?
            .parse()
            .ok()
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

    /// Resolve any leftover `gen-<seq>.old` aside directories left by a crash
    /// during a same-seq re-save's swap (#1444, see the module doc's
    /// "Crash-safe same-seq swap" section). For each `gen-<seq>.old` found:
    /// if `gen-<seq>` is missing, the commit rename never landed — restore
    /// the aside by renaming it back, recovering the last successful save's
    /// predecessor. If `gen-<seq>` exists, the commit already landed and the
    /// aside is stale cleanup leftover — remove it. Must run under
    /// `save_lock` (called from `save`/`prune`/`reopen_into`, all of which
    /// hold it) so it never races a concurrent swap.
    fn reconcile_aside(&self) {
        let Ok(rd) = std::fs::read_dir(&self.root) else {
            return;
        };
        for entry in rd.flatten() {
            let p = entry.path();
            if !p.is_dir() {
                continue;
            }
            let Some(seq_str) = p
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| n.strip_prefix("gen-"))
                .and_then(|n| n.strip_suffix(".old"))
            else {
                continue;
            };
            if seq_str.parse::<u64>().is_err() {
                continue;
            }
            let committed = self.root.join(format!("gen-{seq_str}"));
            if committed.exists() {
                let _ = std::fs::remove_dir_all(&p);
            } else {
                let _ = std::fs::rename(&p, &committed);
            }
        }
    }

    /// Checkpoint `engine` at `up_to_seq`: stage a full generation under a temp
    /// dir, seal every collection into it via [`Engine::flush_to_segments`], then
    /// atomically rename it to `gen-<up_to_seq>/`. The rename is the commit point
    /// — a crash before it leaves only the temp dir (swept on the next call), so a
    /// torn checkpoint never replaces a good one.
    ///
    /// Serialized via `save_lock` (#1397): the checkpoint sink and the
    /// periodic snapshotter can both call this on the same store instance,
    /// including at an unchanged `up_to_seq` (reshard apply/evict mutate
    /// engine state without advancing `applied_seq`, so a same-seq save is
    /// NOT a no-op and must still run — it is only made safe, not skipped).
    /// Holding the lock across sweep+stage+flush+rename means a concurrent
    /// caller's `sweep_staging` can never delete this call's in-flight
    /// staging dir, and two saves at the same seq simply run one after the
    /// other instead of interleaving into a torn generation.
    ///
    /// The commit itself is crash-safe even when replacing an
    /// already-committed generation at the same seq (#1444): the old
    /// generation is moved aside — never deleted — before the new one is
    /// renamed in, so `load_latest` always finds a complete generation at
    /// least as new as this save's predecessor at every crash point. See
    /// the module doc's "Crash-safe same-seq swap" section.
    pub fn save(&self, engine: &Arc<Engine>, up_to_seq: u64) -> Result<()> {
        let _guard = self
            .save_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.sweep_staging();
        self.reconcile_aside();
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
        if committed.exists() {
            // A previously-committed generation at the same seq: move it
            // aside instead of deleting it, so a crash before the commit
            // rename below still leaves a complete generation recoverable
            // by `reconcile_aside` — never a window with no good copy.
            let aside = self.aside_path(up_to_seq);
            let _ = std::fs::remove_dir_all(&aside);
            std::fs::rename(&committed, &aside).with_context(|| {
                format!(
                    "move superseded checkpoint {} aside to {}",
                    committed.display(),
                    aside.display()
                )
            })?;
            std::fs::rename(&staging, &committed).with_context(|| {
                format!(
                    "commit checkpoint {} -> {}",
                    staging.display(),
                    committed.display()
                )
            })?;
            let _ = std::fs::remove_dir_all(&aside);
        } else {
            // No existing generation at this seq: the plain rename is
            // already the atomic commit point, exactly as for a new seq.
            std::fs::rename(&staging, &committed).with_context(|| {
                format!(
                    "commit checkpoint {} -> {}",
                    staging.display(),
                    committed.display()
                )
            })?;
        }
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
    ///
    /// Runs [`Self::reconcile_aside`] under `save_lock` first (#1444): this is
    /// the real cold-start entry point, so any `gen-<seq>.old` left by a
    /// crash mid same-seq-swap is resolved before the newest generation is
    /// picked, guaranteeing `load_latest`/`reopen_into` never miss a
    /// generation that a completed save actually committed.
    pub fn reopen_into(&self, engine: &Arc<Engine>) -> Result<Option<u64>> {
        let _guard = self
            .save_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.reconcile_aside();
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
    ///
    /// Shares `save_lock` with [`Self::save`] (#1397): `sweep_staging` here
    /// could otherwise delete a concurrent `save`'s in-flight staging dir.
    pub fn prune(&self, keep: usize) -> Result<usize> {
        let _guard = self
            .save_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.sweep_staging();
        self.reconcile_aside();
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
        Ok(self
            .generations()?
            .into_iter()
            .map(|(seq, _)| seq)
            .collect())
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
                    version: None,
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

    /// #1444 AC1: a same-seq re-save's swap has two rename steps (old
    /// generation moved aside, then staging renamed onto the committed
    /// path). This structurally — not probabilistically — proves the crash
    /// window between those two steps is closed: it performs exactly the
    /// first rename (`gen-<seq>` -> `gen-<seq>.old`) and then a cold start
    /// (`load_latest`, a fresh store handle, no state carried over) must
    /// still find a complete generation at `seq` containing the PRE-crash
    /// (predecessor) committed data — not the never-committed replacement
    /// staged alongside it, and not nothing.
    #[test]
    fn same_seq_resave_crash_between_aside_and_commit_recovers_predecessor() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();

        // The predecessor: a first successful save at seq 7.
        let engine_a = Arc::new(Engine::new());
        engine_a.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine_a, "a1", "a1@x.com");
        store.save(&engine_a, 7).unwrap();
        assert_eq!(store.load_latest().unwrap().unwrap().1, 7);

        // A same-seq re-save's replacement generation, staged the way `save`
        // stages it (flush completes) but never committed.
        let engine_b = Arc::new(Engine::new());
        engine_b.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine_b, "b1", "b1@x.com");
        index_kw(&engine_b, "b2", "b2@x.com");
        let staging = store.staging_path(7);
        std::fs::create_dir_all(&staging).unwrap();
        engine_b.flush_to_segments(&staging, 7).unwrap();

        // Perform ONLY the swap's first rename — the exact crash point named
        // by the issue: "between the old-generation removal point and the
        // commit point". `committed` (gen-7) no longer exists; the aside
        // (gen-7.old) holds the predecessor; the never-committed staging
        // dir (.gen-7.tmp) still sits alongside both.
        let committed = store.gen_path(7);
        let aside = store.aside_path(7);
        std::fs::rename(&committed, &aside).unwrap();
        assert!(!committed.exists(), "commit rename never ran (the crash)");
        assert!(staging.exists(), "staged replacement is untouched");
        assert!(aside.exists(), "predecessor survives, just moved aside");

        // Cold start from scratch: a brand-new store handle over the same
        // root, exactly what a restarted pod does.
        let cold_store = SegmentRdbStore::new(dir.path()).unwrap();
        let (reloaded, seq) = cold_store
            .load_latest()
            .unwrap()
            .expect("a complete generation survives the crash window");
        assert_eq!(seq, 7);
        assert_eq!(
            reloaded.stats("u").unwrap().documents_indexed,
            1,
            "recovered generation must be the predecessor (1 doc), not the \
             never-committed replacement (2 docs) or nothing"
        );

        // Reconciliation also cleaned up: gen-7 is restored, gen-7.old is
        // gone, and a normal save at 7 still works afterward (the swap
        // machinery is not left in a wedged state by the recovery).
        assert!(committed.is_dir());
        assert!(!aside.exists());
        cold_store.save(&engine_b, 7).unwrap();
        assert_eq!(
            cold_store
                .load_latest()
                .unwrap()
                .unwrap()
                .0
                .stats("u")
                .unwrap()
                .documents_indexed,
            2
        );
    }

    /// #1389 AC1: a `reshard:apply` batch applied to a target shard, and a
    /// `reshard:evict` on a source shard, both survive a cold start from a
    /// checkpoint written after those mutations — independent of any
    /// periodic-snapshot cadence, closing the restart gap `#1387`'s embedded
    /// persistence left open for reshard's direct-state-mutation admin verbs
    /// (`Engine::apply_reshard_batch` / `Engine::evict_not_owned`, added by
    /// `#1380`). This is the engine-level half of `#1389`'s proof; the
    /// driver-level half (cutover cannot fire before every touched shard's
    /// checkpoint completes) lives in `tests/reshard_driver_e2e.rs`.
    #[test]
    fn reshard_apply_and_evict_survive_checkpoint_and_cold_start() {
        use crate::routing::VirtualBucketShardMap;

        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();

        // Target shard: receives a reshard:apply batch on top of its own
        // pre-existing data — mirrors what a shard actually looks like
        // mid-migration.
        let target = Arc::new(Engine::new());
        target.create_collection("u", kw_schema()).unwrap();
        index_kw(&target, "t-existing", "existing@x.com");

        let source = Arc::new(Engine::new());
        source.create_collection("u", kw_schema()).unwrap();
        index_kw(&source, "migrated-1", "migrated1@x.com");
        let batch = source.snapshot().unwrap();
        let apply_outcome = target.apply_reshard_batch(batch, None).unwrap();
        assert_eq!(apply_outcome.documents_upserted, 1);
        assert_eq!(target.stats("u").unwrap().documents_indexed, 2);

        // Source shard: post-cutover eviction of the bucket that just moved
        // off of it, under a 2-shard map where bucket 0 now belongs to shard
        // 1 (mirrors `reshard_evict_removes_only_moved_bucket_docs`).
        let source_after_cutover = Arc::new(Engine::new());
        source_after_cutover
            .create_collection("u", kw_schema())
            .unwrap();
        let ids: Vec<String> = (0..8).map(|i| format!("s-{i:02}")).collect();
        for id in &ids {
            index_kw(&source_after_cutover, id, &format!("{id}@x.com"));
        }
        let mut assignments = vec![0u32; 4];
        assignments[0] = 1;
        let new_map = VirtualBucketShardMap::new(1, assignments, 2).unwrap();
        let evict_outcome = source_after_cutover.evict_not_owned(&new_map, 0).unwrap();
        assert!(evict_outcome.documents_evicted > 0);
        let remaining_before_checkpoint =
            source_after_cutover.stats("u").unwrap().documents_indexed;
        assert!(remaining_before_checkpoint < ids.len() as u64);

        // Checkpoint both post-mutation states, exactly like
        // `checkpoint_touched_shards` (#1389) drives per shard before
        // cutover — this is the synchronous, awaited durability step, not a
        // background snapshot the driver has no visibility into.
        store.save(&target, 100).unwrap();
        let target_docs_before_drop = target.stats("u").unwrap().documents_indexed;
        drop(target);

        let store2 = SegmentRdbStore::new(dir.path().join("source")).unwrap();
        store2.save(&source_after_cutover, 100).unwrap();
        drop(source_after_cutover);

        // Cold start: reload from the checkpoint alone, as a restarted pod
        // would (WAL replay from `seq + 1` is orthogonal to this proof —
        // there are no un-checkpointed writes here).
        let (reloaded_target, seq) = store.load_latest().unwrap().expect("target checkpoint");
        assert_eq!(seq, 100);
        assert_eq!(
            reloaded_target.stats("u").unwrap().documents_indexed,
            target_docs_before_drop
        );

        let (reloaded_source, seq2) = store2.load_latest().unwrap().expect("source checkpoint");
        assert_eq!(seq2, 100);
        assert_eq!(
            reloaded_source.stats("u").unwrap().documents_indexed,
            remaining_before_checkpoint
        );
    }

    /// #1397 AC1: `POST /admin/checkpoint` (the checkpoint sink) and the
    /// periodic snapshotter share one `SegmentRdbStore` and can both fire at
    /// an unchanged `applied_seq` (reshard apply/evict mutate engine state
    /// without advancing `applied_seq`, so this is a routine, not a rare,
    /// interleaving). Loop the interleaving many rounds with several
    /// concurrent `save` callers per round: every round must cold-start to a
    /// complete engine, never a torn one — proving `save_lock` actually
    /// prevents `sweep_staging`/`rename` races rather than merely narrowing
    /// them.
    #[test]
    fn concurrent_saves_at_same_seq_never_produce_torn_checkpoint() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SegmentRdbStore::new(dir.path()).unwrap());
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        for i in 0..20 {
            index_kw(&engine, &format!("u{i:02}"), &format!("u{i:02}@x.com"));
        }
        let expected_docs = engine.stats("u").unwrap().documents_indexed;

        for round in 0..50u64 {
            // Same `up_to_seq` across every concurrent caller this round,
            // mirroring a quiet cutover where `applied_seq` hasn't moved.
            let seq = round;
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let store = store.clone();
                    let engine = engine.clone();
                    std::thread::spawn(move || store.save(&engine, seq))
                })
                .collect();
            for h in handles {
                h.join().unwrap().unwrap();
            }

            // Cold-start from scratch after the interleaving: the committed
            // generation must always be complete and loadable, never torn.
            let (reloaded, loaded_seq) = store.load_latest().unwrap().expect("a checkpoint");
            assert_eq!(loaded_seq, seq);
            assert_eq!(
                reloaded.stats("u").unwrap().documents_indexed,
                expected_docs,
                "round {round}: cold start after concurrent saves must be complete"
            );
        }
    }
}
// CODEGEN-END
