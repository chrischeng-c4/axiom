//! Startup recovery — reconstruct engine state from snapshot + WAL.
//!
//! Recovery flow:
//!  1. Create empty engine.
//!  2. If a snapshot exists, restore entities/relations into the DashMaps and
//!     rebuild adjacency + type indexes.
//!  3. Resolve a [`WalReplayStart`] from the loaded snapshot (v2:
//!     `FromPoint { wal_file_timestamp, wal_position_in_file }`; v1: graceful
//!     fallback to `FullReplay`; no snapshot: `FullReplay`).
//!  4. Find all WAL files; for each one (in `find_wal_files` sort order):
//!     - If its parsed filename timestamp token is `<= wal_file_timestamp`, skip it
//!       entirely — those ops are already captured in the snapshot.
//!     - If its parsed filename timestamp token is `> wal_file_timestamp` and this
//!       is the FIRST such file encountered, apply a byte-offset skip at
//!       `wal_position_in_file` — this is the file that was active at
//!       snapshot-time (now possibly rotated-out).
//!     - If its parsed filename timestamp token is `> wal_file_timestamp` and a
//!       boundary file has already been handled, replay in full.
//!     - `wal-current.log` is treated as having timestamp `u64::MAX`.
//!  5. Skip corrupted entries (log warning); stop on first unrecoverable read
//!     error within a file.

use chrono::{DateTime, TimeZone, Utc};
use std::path::Path;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

use cclab_wal::{find_wal_files, WalReader};

use super::snapshot::{SnapshotLoader, WalReplayStart};
use super::wal_ops::GraphOp;
use crate::engine::CtxInfEngine;
use crate::error::{CtxInfError, Result};

/// `wal-current.log` has no embedded timestamp — treat it as strictly-newest
/// so it sorts after every rotated `wal-<timestamp-token>.log` during replay.
const WAL_CURRENT_SENTINEL_TS: u64 = u64::MAX;

/// Sentinel: WAL entries predating the bitemporal extension default this field to epoch.
/// On replay we treat epoch as "no history row to park" (backward-compatible).
fn is_epoch_sentinel(t: DateTime<Utc>) -> bool {
    t == Utc.timestamp_nanos(0)
}

/// Convert nanoseconds-since-UNIX_EPOCH to `DateTime<Utc>`. Matches the encoding used by
/// `SnapshotHeader::created_at`.
fn ns_to_datetime(ns: i64) -> Option<DateTime<Utc>> {
    let secs = ns.div_euclid(1_000_000_000);
    let sub_ns = ns.rem_euclid(1_000_000_000) as u32;
    DateTime::<Utc>::from_timestamp(secs, sub_ns)
}

/// Statistics from a recovery run.
#[derive(Debug, Clone)]
pub struct RecoveryStats {
    pub snapshot_loaded: bool,
    pub snapshot_entities: usize,
    pub snapshot_relations: usize,
    pub wal_entries_replayed: usize,
    pub corrupted_entries: usize,
    pub duration: Duration,
}

/// Recovery orchestrator.
pub struct RecoveryManager;

impl RecoveryManager {
    /// Recover an engine from `data_dir`.
    /// Returns the rebuilt engine and stats. The returned engine has no
    /// persistence handle attached — `CtxInfEngine::open` wires that up.
    ///
    /// # Failure modes
    ///
    /// Recovery is designed to yield a prefix-consistent engine state
    /// whenever possible. Specific conditions:
    ///
    /// - **Missing snapshot.** No `snapshot-*.snap` file is present.
    ///   Recovery starts from an empty engine and replays all WAL files
    ///   from position 0. Reported via
    ///   [`RecoveryStats::snapshot_loaded`]` = false` and
    ///   [`RecoveryStats::snapshot_entities`]`/`[`RecoveryStats::snapshot_relations`]` = 0`
    ///   (see the "no snapshot" branch at `recovery.rs` ~L91–93).
    ///
    /// - **Corrupt snapshot checksum.** The snapshot payload's SHA-256 does
    ///   not match the header's recorded checksum. `SnapshotLoader::load_latest`
    ///   returns `Err(CtxInfError::Snapshot("checksum mismatch: snapshot
    ///   data corrupted"))` (see `snapshot.rs` ~L252–261) and `recover`
    ///   propagates that error to the caller — **no partial recovery is
    ///   attempted** from a corrupted snapshot. The caller is responsible
    ///   for quarantining or deleting the bad `.snap` file before retry.
    ///
    /// - **Corrupt WAL tail.** A WAL entry near end-of-file fails CRC or
    ///   returns a read error. `replay_wal` logs a `warn!`, increments
    ///   [`RecoveryStats::corrupted_entries`], and **stops replay of that
    ///   WAL file** at the corrupt position (see `replay_wal` ~L173–181).
    ///   Earlier entries in the same file are kept; subsequent WAL files in
    ///   the directory are still attempted. Recovery itself returns `Ok`;
    ///   inspect `corrupted_entries` to detect the truncation.
    ///
    /// - **Mid-entry truncation.** Treated identically to a corrupt tail —
    ///   `WalReader::read_entry` surfaces an `Err` which is counted in
    ///   [`RecoveryStats::corrupted_entries`] and causes replay of that
    ///   file to stop (`replay_wal` ~L173–181). The clean prefix up to the
    ///   truncation is preserved in the recovered engine.
    ///
    /// - **WAL enumeration error.** `cclab_wal::find_wal_files` returning
    ///   an error (e.g. permission denied on the data directory) is
    ///   propagated as `CtxInfError::Wal` and aborts recovery
    ///   (`recovery.rs` ~L96–97). Per-file read errors during replay are
    ///   logged with `warn!` but do not abort — subsequent files are
    ///   still attempted (~L111–114).
    ///
    /// The full set of observable outcomes for a caller is captured in
    /// [`RecoveryStats`]: `snapshot_loaded`, `snapshot_entities`,
    /// `snapshot_relations`, `wal_entries_replayed`, `corrupted_entries`,
    /// and `duration`.
    pub fn recover(data_dir: impl AsRef<Path>) -> Result<(CtxInfEngine, RecoveryStats)> {
        let data_dir = data_dir.as_ref();
        let start = Instant::now();

        info!("Starting recovery from {}", data_dir.display());

        let engine = CtxInfEngine::new();

        let mut snapshot_entities = 0;
        let mut snapshot_relations = 0;
        let mut wal_entries_replayed = 0;
        let mut corrupted_entries = 0;
        let mut replay_start = WalReplayStart::FullReplay;
        let mut snapshot_loaded = false;

        // Step 1: snapshot.
        if let Some((data, start_from, created_at_ns)) = SnapshotLoader::load_latest_full(data_dir)?
        {
            snapshot_loaded = true;
            info!(
                "Found snapshot: {} entities, {} relations (replay_start={:?})",
                data.entities.len(),
                data.relations.len(),
                start_from
            );

            // Bitemporal stamping (D1 / R6): pre-bitemporal snapshots don't carry
            // `tx_from` / `tx_to`, so the serde defaults kick in and give us `Utc::now()`
            // for `tx_from` — nondeterministic and in the future relative to the snapshot.
            // When we detect that (tx_from > snapshot.created_at), rewrite the row to
            // `tx_from = snapshot.created_at, tx_to = None`. Post-bitemporal snapshots
            // carry honest tx_from values ≤ created_at and are left alone.
            let snapshot_tx_from = ns_to_datetime(created_at_ns).unwrap_or_else(Utc::now);

            // Restore entities — direct DashMap inserts, then rebuild adjacency + type index.
            for mut entity in data.entities {
                if entity.tx_from > snapshot_tx_from {
                    entity.tx_from = snapshot_tx_from;
                    entity.tx_to = None;
                }
                let id = entity.id;
                let entity_type = entity.entity_type.clone();
                engine.entities.insert(id, entity);
                engine.adj_out.entry(id).or_default();
                engine.adj_in.entry(id).or_default();
                engine.type_index.entry(entity_type).or_default().push(id);
                snapshot_entities += 1;
            }

            // Restore relations — rebuild adjacency.
            for mut relation in data.relations {
                if relation.tx_from > snapshot_tx_from {
                    relation.tx_from = snapshot_tx_from;
                    relation.tx_to = None;
                }
                let rid = relation.id;
                let source = relation.source;
                let target = relation.target;
                engine
                    .adj_out
                    .entry(source)
                    .or_default()
                    .push((rid, target));
                engine.adj_in.entry(target).or_default().push((rid, source));
                engine.relations.insert(rid, relation);
                snapshot_relations += 1;
            }

            replay_start = start_from;
            info!(
                "Restored {} entities, {} relations from snapshot",
                snapshot_entities, snapshot_relations
            );
        } else {
            info!("No snapshot found, starting from empty state");
        }

        // Step 2: WAL replay.
        let wal_files = find_wal_files(data_dir)
            .map_err(|e| CtxInfError::Wal(format!("find WAL files: {}", e)))?;

        if wal_files.is_empty() {
            info!("No WAL files found");
        } else {
            info!(
                "Found {} WAL file(s) to replay (start={:?})",
                wal_files.len(),
                replay_start
            );

            // Track whether we've already applied the byte-offset skip to the
            // boundary file (first file with ts > wal_file_timestamp). After
            // that, subsequent files are fully post-snapshot and replayed whole.
            let mut boundary_skip_used = false;

            for wal_path in wal_files {
                let file_ts = parse_wal_file_timestamp(&wal_path);
                let per_file_skip = match replay_start {
                    WalReplayStart::FullReplay => Some(0u64),
                    WalReplayStart::FromPoint {
                        wal_file_timestamp,
                        wal_position_in_file,
                    } => {
                        if file_ts <= wal_file_timestamp {
                            // Pre-snapshot file — already captured; skip entirely.
                            debug!(
                                "Skipping pre-snapshot WAL {} (ts={} <= {})",
                                wal_path.display(),
                                file_ts,
                                wal_file_timestamp
                            );
                            None
                        } else if !boundary_skip_used {
                            // Boundary file: byte-offset skip at the stored offset.
                            boundary_skip_used = true;
                            Some(wal_position_in_file)
                        } else {
                            // Post-boundary file: fully post-snapshot.
                            Some(0u64)
                        }
                    }
                };

                if let Some(skip) = per_file_skip {
                    debug!(
                        "Replaying WAL: {} (skip_before={})",
                        wal_path.display(),
                        skip
                    );
                    match Self::replay_wal(&engine, &wal_path, skip) {
                        Ok((replayed, corrupted)) => {
                            wal_entries_replayed += replayed;
                            corrupted_entries += corrupted;
                        }
                        Err(e) => {
                            warn!("Failed to replay WAL {}: {}", wal_path.display(), e);
                        }
                    }
                }
            }

            info!(
                "Replayed {} WAL entries ({} corrupted/skipped)",
                wal_entries_replayed, corrupted_entries
            );
        }

        let duration = start.elapsed();
        let stats = RecoveryStats {
            snapshot_loaded,
            snapshot_entities,
            snapshot_relations,
            wal_entries_replayed,
            corrupted_entries,
            duration,
        };

        info!(
            "Recovery complete in {:?}: {} entities + {} relations from snapshot, {} from WAL ({} corrupted)",
            duration,
            snapshot_entities,
            snapshot_relations,
            wal_entries_replayed,
            corrupted_entries
        );

        Ok((engine, stats))
    }

    /// Replay a single WAL file, skipping entries whose pre-read position is
    /// below `skip_before_position`. Returns `(entries_replayed, corrupted_entries)`.
    fn replay_wal(
        engine: &CtxInfEngine,
        wal_path: impl AsRef<Path>,
        skip_before_position: u64,
    ) -> Result<(usize, usize)> {
        let mut reader = WalReader::<GraphOp>::new(wal_path)
            .map_err(|e| CtxInfError::Wal(format!("open WAL: {}", e)))?;

        let mut replayed = 0;
        let mut corrupted = 0;

        loop {
            // Track position before reading the entry. WalReader::position() returns
            // the position *after* the next read, so we capture before-state here.
            let pos_before = reader.position();

            match reader.read_entry() {
                Ok(Some(entry)) => {
                    if pos_before < skip_before_position {
                        // Already in snapshot — skip.
                        continue;
                    }
                    Self::apply_op(engine, entry.op);
                    replayed += 1;
                }
                Ok(None) => break, // EOF
                Err(e) => {
                    warn!(
                        "Corrupted WAL entry at position {}: {} — stopping replay of this file",
                        pos_before, e
                    );
                    corrupted += 1;
                    break;
                }
            }
        }

        Ok((replayed, corrupted))
    }

    /// Apply a single WAL operation to the engine.
    ///
    /// Bypasses the public CRUD methods to avoid (a) re-logging to WAL and
    /// (b) double-pushing into `type_index` / adjacency lists when an op is
    /// already captured by the snapshot. Idempotent for create/update.
    /// `delete_entity` / `delete_relation` go through the engine method (which
    /// no-ops `log_op` since persistence is `None` during recovery).
    fn apply_op(engine: &CtxInfEngine, op: GraphOp) {
        match op {
            GraphOp::CreateEntity { entity } => {
                let id = entity.id;
                let entity_type = entity.entity_type.clone();
                let already_exists = engine.entities.contains_key(&id);
                engine.entities.insert(id, entity);
                engine.adj_out.entry(id).or_default();
                engine.adj_in.entry(id).or_default();
                if !already_exists {
                    engine.type_index.entry(entity_type).or_default().push(id);
                }
            }
            GraphOp::UpdateEntity {
                id,
                entity,
                frozen_tx_to,
            } => {
                // Park the previous current-state row into history with the recorded freeze
                // stamp, then swap in the new current row. Bitemporal-aware replay (D1 / R6):
                // this reconstructs exactly what the live engine produced.
                if !is_epoch_sentinel(frozen_tx_to) {
                    if let Some((_, mut prev)) = engine.entities.remove(&id) {
                        let prev_version = prev.version;
                        prev.tx_to = Some(frozen_tx_to);
                        engine.entities_history.insert((id, prev_version), prev);
                    }
                }
                engine.entities.insert(id, entity);
            }
            GraphOp::DeleteEntity { id, cascade, tx_to } => {
                if is_epoch_sentinel(tx_to) {
                    // Backward-compatible path for pre-bitemporal WAL entries.
                    let _ = engine.delete_entity(id, cascade);
                    return;
                }
                // Bitemporal freeze-on-delete: remove the entity from current-state, park
                // it in history with `tx_to = tx_to`. Cascade freezes adjacent relations.
                if cascade {
                    let mut to_freeze = Vec::new();
                    if let Some(adj) = engine.adj_out.get(&id) {
                        to_freeze.extend(adj.iter().map(|(rid, _)| *rid));
                    }
                    if let Some(adj) = engine.adj_in.get(&id) {
                        to_freeze.extend(adj.iter().map(|(rid, _)| *rid));
                    }
                    for rid in &to_freeze {
                        if let Some((_, mut rel)) = engine.relations.remove(rid) {
                            if rel.source == id {
                                if let Some(mut adj) = engine.adj_in.get_mut(&rel.target) {
                                    adj.retain(|(r, _)| r != rid);
                                }
                            }
                            if rel.target == id {
                                if let Some(mut adj) = engine.adj_out.get_mut(&rel.source) {
                                    adj.retain(|(r, _)| r != rid);
                                }
                            }
                            rel.tx_to = Some(tx_to);
                            let rel_version = rel.version;
                            engine.relations_history.insert((*rid, rel_version), rel);
                        }
                    }
                }
                engine.adj_out.remove(&id);
                engine.adj_in.remove(&id);
                if let Some((_, mut entity)) = engine.entities.remove(&id) {
                    if let Some(mut ids) = engine.type_index.get_mut(&entity.entity_type) {
                        ids.retain(|eid| *eid != id);
                    }
                    entity.tx_to = Some(tx_to);
                    let prev_version = entity.version;
                    engine.entities_history.insert((id, prev_version), entity);
                }
            }
            GraphOp::CreateRelation { relation } => {
                let rid = relation.id;
                let source = relation.source;
                let target = relation.target;
                let already_exists = engine.relations.contains_key(&rid);
                engine.relations.insert(rid, relation);
                if !already_exists {
                    engine
                        .adj_out
                        .entry(source)
                        .or_default()
                        .push((rid, target));
                    engine.adj_in.entry(target).or_default().push((rid, source));
                }
            }
            GraphOp::UpdateRelation {
                id,
                relation,
                frozen_tx_to,
            } => {
                if !is_epoch_sentinel(frozen_tx_to) {
                    if let Some((_, mut prev)) = engine.relations.remove(&id) {
                        let prev_version = prev.version;
                        prev.tx_to = Some(frozen_tx_to);
                        engine.relations_history.insert((id, prev_version), prev);
                    }
                }
                engine.relations.insert(id, relation);
            }
            GraphOp::DeleteRelation { id, tx_to } => {
                if is_epoch_sentinel(tx_to) {
                    let _ = engine.delete_relation(id);
                    return;
                }
                if let Some((_, mut rel)) = engine.relations.remove(&id) {
                    if let Some(mut adj) = engine.adj_out.get_mut(&rel.source) {
                        adj.retain(|(r, _)| *r != id);
                    }
                    if let Some(mut adj) = engine.adj_in.get_mut(&rel.target) {
                        adj.retain(|(r, _)| *r != id);
                    }
                    rel.tx_to = Some(tx_to);
                    let rel_version = rel.version;
                    engine.relations_history.insert((id, rel_version), rel);
                }
            }
        }
    }
}

/// Parse the numeric timestamp token from a WAL filename.
///
/// - `wal-current.log` → [`WAL_CURRENT_SENTINEL_TS`] (= `u64::MAX`).
/// - `wal-<timestamp-token>.log` → the parsed number.
/// - Anything else (shouldn't happen — `find_wal_files` already filtered) →
///   `0`, treating the file as pre-snapshot to be safe.
fn parse_wal_file_timestamp(path: &Path) -> u64 {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return 0;
    };
    if name == "wal-current.log" {
        return WAL_CURRENT_SENTINEL_TS;
    }
    name.strip_prefix("wal-")
        .and_then(|s| s.strip_suffix(".log"))
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_wal_file_timestamp_current() {
        let p = PathBuf::from("/data/wal-current.log");
        assert_eq!(parse_wal_file_timestamp(&p), u64::MAX);
    }

    #[test]
    fn test_parse_wal_file_timestamp_rotated() {
        let p = PathBuf::from("/data/wal-1700000000.log");
        assert_eq!(parse_wal_file_timestamp(&p), 1_700_000_000);
    }

    #[test]
    fn test_parse_wal_file_timestamp_rotated_nanosecond_token() {
        let p = PathBuf::from("/data/wal-1781800000123456789.log");
        assert_eq!(parse_wal_file_timestamp(&p), 1_781_800_000_123_456_789);
    }

    #[test]
    fn test_parse_wal_file_timestamp_malformed() {
        let p = PathBuf::from("/data/wal-garbage.log");
        assert_eq!(parse_wal_file_timestamp(&p), 0);
    }
}
