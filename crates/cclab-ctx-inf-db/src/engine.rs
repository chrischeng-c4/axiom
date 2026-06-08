use chrono::{DateTime, Utc};
use dashmap::DashMap;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tracing::{debug, warn};

use crate::error::{CtxInfError, Result};
use crate::storage::snapshot::{cleanup_old_snapshots, SnapshotWriter};
use crate::storage::{
    GraphOp, PersistenceConfig, PersistenceHandle, RecoveryManager, RecoveryStats,
};
use crate::types::*;

/// Core engine — in-memory temporal knowledge graph with CRUD + adjacency index.
///
/// Phase 1: RAM-only via DashMaps.
/// Phase 2: optional write-behind persistence (`persistence` field).
///   - `new()` is in-memory only (no WAL, no snapshots) — preserves all Phase 1 tests.
///   - `open(config)` recovers from disk + starts WAL background thread.
///   - `with_persistence(config)` starts persistence on an empty engine (no recovery).
pub struct CtxInfEngine {
    /// Current rows only — holds entities whose `tx_to = None`. Updates and deletes
    /// remove the previous version from here and push it into `entities_history`.
    pub(crate) entities: DashMap<EntityId, Entity>,
    /// Current rows only — holds relations whose `tx_to = None`. See `entities`.
    pub(crate) relations: DashMap<RelationId, Relation>,
    /// Outgoing adjacency: source → [(relation_id, target)]. Reflects only current rows.
    pub(crate) adj_out: DashMap<EntityId, Vec<(RelationId, EntityId)>>,
    /// Incoming adjacency: target → [(relation_id, source)]. Reflects only current rows.
    pub(crate) adj_in: DashMap<EntityId, Vec<(RelationId, EntityId)>>,
    /// Type index: entity_type → [entity_id]. Reflects only current rows.
    pub(crate) type_index: DashMap<EntityType, Vec<EntityId>>,
    /// Frozen (historical) entity rows — keyed by (id, version) of the frozen row.
    /// Populated on update (old row frozen with `tx_to = now`, new row replaces current)
    /// and on delete (current row moved here with `tx_to = now`).
    /// Used by `get_entity_as_of_tx` to reconstruct the view at a past transaction time.
    pub(crate) entities_history: DashMap<(EntityId, u64), Entity>,
    /// Frozen (historical) relation rows. See `entities_history`.
    pub(crate) relations_history: DashMap<(RelationId, u64), Relation>,
    /// Optional persistence handle — `None` for pure in-memory mode.
    pub(crate) persistence: Option<PersistenceHandle>,
}

impl CtxInfEngine {
    /// Create a fresh in-memory engine (no persistence).
    pub fn new() -> Self {
        Self {
            entities: DashMap::new(),
            relations: DashMap::new(),
            adj_out: DashMap::new(),
            adj_in: DashMap::new(),
            type_index: DashMap::new(),
            entities_history: DashMap::new(),
            relations_history: DashMap::new(),
            persistence: None,
        }
    }

    /// Open a persistent engine: recover state from `data_dir`, then start the
    /// WAL background thread so future mutations are durable.
    pub fn open(config: PersistenceConfig) -> Result<Self> {
        let (mut engine, _stats) = RecoveryManager::recover(&config.data_dir)?;
        let handle = PersistenceHandle::new(config)?;
        engine.persistence = Some(handle);
        Ok(engine)
    }

    /// Open a persistent engine and return recovery stats alongside.
    pub fn open_with_stats(config: PersistenceConfig) -> Result<(Self, RecoveryStats)> {
        let (mut engine, stats) = RecoveryManager::recover(&config.data_dir)?;
        let handle = PersistenceHandle::new(config)?;
        engine.persistence = Some(handle);
        Ok((engine, stats))
    }

    /// Start a fresh persistent engine — no recovery, just attach the WAL.
    /// Useful when you know `data_dir` is empty.
    pub fn with_persistence(config: PersistenceConfig) -> Result<Self> {
        let mut engine = Self::new();
        let handle = PersistenceHandle::new(config)?;
        engine.persistence = Some(handle);
        Ok(engine)
    }

    /// Internal: log a mutation to the WAL (no-op if persistence disabled).
    fn log_op(&self, op: GraphOp) {
        if let Some(ref p) = self.persistence {
            p.log_operation(op);
        }
    }

    /// Force a WAL fsync. No-op if persistence is disabled.
    ///
    /// # Durability
    ///
    /// This call enqueues a `Flush` command onto the persistence channel and
    /// returns immediately — it is **not synchronous**. The background thread
    /// will pick up the command and call `WalWriter::flush` (which performs an
    /// `fsync`) as soon as it drains the channel. If the channel is full the
    /// flush request is silently dropped (`try_send`).
    ///
    /// For a hard durability barrier, use [`Self::shutdown`], which joins the
    /// background thread after its final `fsync`. See `handle.rs::flush`
    /// (~L122–125) and the flush branch of `persistence_thread` (~L188–195).
    pub fn flush(&self) {
        if let Some(ref p) = self.persistence {
            p.flush();
        }
    }

    /// Take a synchronous snapshot of current engine state.
    /// Requires persistence to be enabled.
    ///
    /// # Durability
    ///
    /// The snapshot is written atomically: the payload (72-byte header plus
    /// JSON-encoded entities/relations) is written to a `snapshot-<ts>.tmp`
    /// file, `fsync`ed (`sync_all`), and then renamed to
    /// `snapshot-<ts>.snap`. Readers (including [`Self::open`]) only observe
    /// the final `.snap` file, so a crash mid-write cannot yield a
    /// partially-readable snapshot.
    ///
    /// The snapshot captures a **rotation-safe** WAL replay starting point:
    /// a pair `(wal_file_timestamp, wal_position_in_file)` where
    /// `wal_file_timestamp` is the maximum unix-second timestamp among the
    /// WAL files that have already been rotated-out (`0` if none) and
    /// `wal_position_in_file` is the byte offset within the currently-active
    /// `wal-current.log`.
    ///
    /// On recovery, replay skips every WAL file with
    /// `parsed_ts <= wal_file_timestamp` entirely, applies `wal_position_in_file`
    /// as a byte-offset skip only to the first file with
    /// `parsed_ts > wal_file_timestamp` (the file that was active at
    /// snapshot-time, possibly since rotated-out), and replays later files in
    /// full. This is the v2 snapshot shape; v1 snapshots (legacy per-file byte
    /// offset) are loaded with a graceful fallback to full WAL replay.
    ///
    /// Prior to capturing position, the WAL is synchronously flushed via
    /// [`PersistenceHandle::snapshot_barrier`] so the captured offset refers
    /// to bytes already fsynced to disk.
    ///
    /// After the snapshot is durable on disk, old snapshots beyond
    /// `snapshot_keep_count` are garbage-collected. GC failures are logged
    /// but never abort the snapshot path.
    pub fn create_snapshot(&self) -> Result<PathBuf> {
        let p = self
            .persistence
            .as_ref()
            .ok_or_else(|| CtxInfError::Storage("create_snapshot requires persistence".into()))?;
        // Synchronous barrier: drain the command channel, fsync the WAL, and
        // return the durable byte offset. Unlike fire-and-forget `flush()`,
        // this blocks until the background thread has processed every
        // enqueued command and called sync_data.
        let wal_position_in_file = p.snapshot_barrier()?;
        let wal_file_timestamp = p.wal_file_timestamp();
        let path =
            SnapshotWriter::create(self, p.data_dir(), wal_file_timestamp, wal_position_in_file)?;

        // Post-snapshot GC: prune old `.snap` files beyond `snapshot_keep_count`.
        // Failure here must never abort the snapshot path — log and continue.
        let keep = p.snapshot_keep_count();
        match cleanup_old_snapshots(p.data_dir(), keep) {
            Ok(deleted) => {
                debug!(
                    "snapshot GC: deleted {} old snapshots, kept {}",
                    deleted, keep
                );
            }
            Err(e) => {
                warn!("snapshot GC failed (continuing): {}", e);
            }
        }

        Ok(path)
    }

    /// Graceful shutdown of the persistence background thread (consumes self).
    /// Idempotent if persistence is disabled.
    ///
    /// # Durability
    ///
    /// On return, all operations previously queued for persistence that
    /// were accepted by the channel are **drained and fsynced to disk**, and
    /// the background thread has been joined. The `Shutdown` command is
    /// sent via a blocking `send` on the persistence channel
    /// (`handle.rs::shutdown_internal` ~L229–240); the thread then drains
    /// every remaining command via the `recv_timeout` loop and exits via the
    /// `Shutdown` branch, which falls through to a final `wal_writer.flush()`
    /// (`handle.rs::persistence_thread` ~L197–225). Only operations that
    /// were **dropped on a full channel** before this call are not covered
    /// (see `PersistenceHandle::log_operation` in `storage::handle`).
    pub fn shutdown(mut self) -> Result<()> {
        if let Some(handle) = self.persistence.take() {
            handle.shutdown()?;
        }
        Ok(())
    }

    // ── Entity CRUD ──────────────────────────────────────────────────

    /// Insert a new entity into the in-memory graph.
    ///
    /// # Durability
    ///
    /// This call returns **before** the operation is on stable storage.
    /// After applying the insert to the in-memory DashMaps, a
    /// `GraphOp::CreateEntity` is enqueued on the persistence channel via
    /// `log_op` (`engine.rs` ~L70–75 → `handle.rs::log_operation` ~L114–120).
    /// The background thread fsyncs the WAL at the interval configured by
    /// `PersistenceConfig::wal_flush_interval_ms` — see the crate-level
    /// durability notes on [`crate::storage`]. If the channel
    /// is full the op is dropped with a warn log; if the process crashes
    /// before the next fsync the op may be lost. Use [`Self::flush`] (plus a
    /// subsequent [`Self::shutdown`], or rely on shutdown alone) to force a
    /// durability barrier.
    pub fn create_entity(&self, entity: Entity) -> Result<Entity> {
        let id = entity.id;
        let entity_type = entity.entity_type.clone();

        self.entities.insert(id, entity.clone());
        self.adj_out.entry(id).or_default();
        self.adj_in.entry(id).or_default();
        self.type_index.entry(entity_type).or_default().push(id);

        self.log_op(GraphOp::CreateEntity {
            entity: entity.clone(),
        });

        Ok(entity)
    }

    pub fn get_entity(&self, id: EntityId) -> Result<Entity> {
        self.entities
            .get(&id)
            .map(|e| e.clone())
            .ok_or(CtxInfError::EntityNotFound(id))
    }

    /// Compare-and-swap update of an entity (bumps `version`, updates
    /// `updated_at`).
    ///
    /// # Durability
    ///
    /// Write-behind, identical to [`Self::create_entity`]: the mutation is
    /// applied in memory (`engine.rs` ~L145–152) and then a
    /// `GraphOp::UpdateEntity` is enqueued via `log_op` (~L156–159). Return
    /// does **not** guarantee the update is on disk. See the crate-level
    /// durability notes on [`crate::storage`].
    pub fn update_entity(
        &self,
        id: EntityId,
        expected_version: u64,
        mut updater: impl FnMut(&mut Entity),
    ) -> Result<Entity> {
        // Freeze-on-mutate (D1 bitemporal):
        //   1. Check version under the DashMap lock.
        //   2. Snapshot the pre-change row; stamp its `tx_to = now` and park it in history.
        //   3. Apply the caller's mutation, bump version, set `tx_from = now, tx_to = None`.
        let mut entry = self
            .entities
            .get_mut(&id)
            .ok_or(CtxInfError::EntityNotFound(id))?;

        if entry.version != expected_version {
            return Err(CtxInfError::VersionConflict {
                expected: expected_version,
                actual: entry.version,
            });
        }

        let now = Utc::now();

        // Freeze the previous row.
        let mut frozen = entry.clone();
        frozen.tx_to = Some(now);
        let frozen_version = frozen.version;

        // Apply caller mutation to the live row.
        updater(&mut entry);
        entry.version += 1;
        entry.updated_at = now;
        entry.tx_from = now;
        entry.tx_to = None;

        let updated = entry.clone();
        drop(entry); // release DashMap lock before further work

        // Park the frozen row.
        self.entities_history.insert((id, frozen_version), frozen);

        self.log_op(GraphOp::UpdateEntity {
            id,
            entity: updated.clone(),
            frozen_tx_to: now,
        });

        Ok(updated)
    }

    /// Delete an entity; optionally cascade-delete its incident relations.
    ///
    /// # Durability
    ///
    /// Write-behind. After the in-memory cascade (adjacency cleanup + entry
    /// removal, `engine.rs` ~L181–220), a single `GraphOp::DeleteEntity`
    /// is enqueued via `log_op` (~L224). Cascaded relation deletes are
    /// **not** individually logged — recovery re-derives them by replaying
    /// the `DeleteEntity { cascade: true }` op (see `recovery.rs::apply_op`
    /// ~L211–213). Return does **not** guarantee the delete is on disk.
    /// See the crate-level durability notes on [`crate::storage`].
    pub fn delete_entity(&self, id: EntityId, cascade: bool) -> Result<DeleteResult> {
        if !self.entities.contains_key(&id) {
            return Err(CtxInfError::EntityNotFound(id));
        }

        let now = Utc::now();
        let mut relations_deleted = 0;

        if cascade {
            // Collect relation IDs to freeze (outgoing + incoming).
            let mut to_freeze = Vec::new();
            if let Some(out) = self.adj_out.get(&id) {
                to_freeze.extend(out.iter().map(|(rid, _)| *rid));
            }
            if let Some(inc) = self.adj_in.get(&id) {
                to_freeze.extend(inc.iter().map(|(rid, _)| *rid));
            }

            for rid in &to_freeze {
                if let Some((_, mut rel)) = self.relations.remove(rid) {
                    // Clean up counterpart adjacency entries (current-state).
                    if rel.source == id {
                        if let Some(mut adj) = self.adj_in.get_mut(&rel.target) {
                            adj.retain(|(r, _)| r != rid);
                        }
                    }
                    if rel.target == id {
                        if let Some(mut adj) = self.adj_out.get_mut(&rel.source) {
                            adj.retain(|(r, _)| r != rid);
                        }
                    }

                    // Freeze the relation row into history (D1 bitemporal: do NOT physically lose).
                    rel.tx_to = Some(now);
                    let frozen_version = rel.version;
                    self.relations_history.insert((*rid, frozen_version), rel);
                    relations_deleted += 1;
                }
            }
        }

        // Remove adjacency entries from current-state indexes.
        self.adj_out.remove(&id);
        self.adj_in.remove(&id);

        // Move the entity's current row into history rather than losing it.
        if let Some((_, mut entity)) = self.entities.remove(&id) {
            if let Some(mut ids) = self.type_index.get_mut(&entity.entity_type) {
                ids.retain(|eid| *eid != id);
            }
            entity.tx_to = Some(now);
            let frozen_version = entity.version;
            self.entities_history.insert((id, frozen_version), entity);
        }

        self.log_op(GraphOp::DeleteEntity {
            id,
            cascade,
            tx_to: now,
        });

        Ok(DeleteResult {
            entity_deleted: true,
            relations_deleted,
        })
    }

    // ── Relation CRUD ────────────────────────────────────────────────

    /// Insert a new relation; errors if source or target entity does not
    /// exist in the current in-memory view.
    ///
    /// # Durability
    ///
    /// Write-behind, identical to [`Self::create_entity`]: the relation is
    /// inserted and adjacency lists are updated synchronously, then a
    /// `GraphOp::CreateRelation` is enqueued via `log_op`. Return does
    /// **not** guarantee the operation is on disk. See the crate-level
    /// durability notes on [`crate::storage`].
    pub fn create_relation(&self, relation: Relation) -> Result<Relation> {
        // Validate source and target exist.
        if !self.entities.contains_key(&relation.source) {
            return Err(CtxInfError::DanglingReference(relation.source));
        }
        if !self.entities.contains_key(&relation.target) {
            return Err(CtxInfError::DanglingReference(relation.target));
        }

        let rid = relation.id;
        let source = relation.source;
        let target = relation.target;

        self.relations.insert(rid, relation.clone());
        self.adj_out.entry(source).or_default().push((rid, target));
        self.adj_in.entry(target).or_default().push((rid, source));

        self.log_op(GraphOp::CreateRelation {
            relation: relation.clone(),
        });

        Ok(relation)
    }

    pub fn get_relation(&self, id: RelationId) -> Result<Relation> {
        self.relations
            .get(&id)
            .map(|r| r.clone())
            .ok_or(CtxInfError::RelationNotFound(id))
    }

    /// Compare-and-swap update of a relation (bumps `version`).
    ///
    /// # Durability
    ///
    /// Write-behind, identical to [`Self::create_entity`]. The mutation is
    /// applied in memory then a `GraphOp::UpdateRelation` is enqueued via
    /// `log_op`. Return does **not** guarantee durability. See the
    /// crate-level durability notes on [`crate::storage`].
    pub fn update_relation(
        &self,
        id: RelationId,
        expected_version: u64,
        mut updater: impl FnMut(&mut Relation),
    ) -> Result<Relation> {
        // Freeze-on-mutate (D1 bitemporal) — see `update_entity`.
        let mut entry = self
            .relations
            .get_mut(&id)
            .ok_or(CtxInfError::RelationNotFound(id))?;

        if entry.version != expected_version {
            return Err(CtxInfError::VersionConflict {
                expected: expected_version,
                actual: entry.version,
            });
        }

        let now = Utc::now();

        let mut frozen = entry.clone();
        frozen.tx_to = Some(now);
        let frozen_version = frozen.version;

        updater(&mut entry);
        entry.version += 1;
        entry.tx_from = now;
        entry.tx_to = None;

        let updated = entry.clone();
        drop(entry);

        self.relations_history.insert((id, frozen_version), frozen);

        self.log_op(GraphOp::UpdateRelation {
            id,
            relation: updated.clone(),
            frozen_tx_to: now,
        });

        Ok(updated)
    }

    /// Remove a relation and its adjacency entries.
    ///
    /// # Durability
    ///
    /// Write-behind. The adjacency cleanup happens in memory, then a
    /// `GraphOp::DeleteRelation` is enqueued via `log_op`. Return does
    /// **not** guarantee durability. See the crate-level durability notes
    /// on [`crate::storage`].
    pub fn delete_relation(&self, id: RelationId) -> Result<bool> {
        let (_, mut rel) = self
            .relations
            .remove(&id)
            .ok_or(CtxInfError::RelationNotFound(id))?;

        if let Some(mut adj) = self.adj_out.get_mut(&rel.source) {
            adj.retain(|(r, _)| *r != id);
        }
        if let Some(mut adj) = self.adj_in.get_mut(&rel.target) {
            adj.retain(|(r, _)| *r != id);
        }

        let now = Utc::now();
        rel.tx_to = Some(now);
        let frozen_version = rel.version;
        self.relations_history.insert((id, frozen_version), rel);

        self.log_op(GraphOp::DeleteRelation { id, tx_to: now });

        Ok(true)
    }

    // ── Bitemporal "as-of" queries (D1) ──────────────────────────────

    /// Return the row visible for entity `id` at transaction time `tx`.
    ///
    /// Semantics (D1): the row whose `tx_from ≤ tx` AND (`tx_to` is `None` OR `tx < tx_to`).
    /// Searches the current-state map first (current row iff `tx_to = None`), then the
    /// history map (frozen rows with an explicit `tx_to`).
    ///
    /// Returns `None` if no row existed for this id at `tx` (i.e. `tx` is before `tx_from`
    /// of the earliest known row, or the id was never inserted).
    pub fn get_entity_as_of_tx(&self, id: EntityId, tx: DateTime<Utc>) -> Option<Entity> {
        // Current-state map: only rows with `tx_to = None`.
        if let Some(current) = self.entities.get(&id) {
            let c = current.value();
            if c.tx_from <= tx {
                // `tx_to` is `None` on current rows by construction, so the range is open.
                return Some(c.clone());
            }
            // `tx` predates the current row — fall through to history search.
        }
        // Scan history for the frozen row containing `tx`.
        self.entities_history
            .iter()
            .filter(|e| e.key().0 == id)
            .find_map(|e| {
                let row = e.value();
                let in_range = row.tx_from <= tx && row.tx_to.map_or(true, |end| tx < end);
                if in_range {
                    Some(row.clone())
                } else {
                    None
                }
            })
    }

    /// Return the row visible for relation `id` at transaction time `tx`.
    /// See [`CtxInfEngine::get_entity_as_of_tx`].
    pub fn get_relation_as_of_tx(&self, id: RelationId, tx: DateTime<Utc>) -> Option<Relation> {
        if let Some(current) = self.relations.get(&id) {
            let c = current.value();
            if c.tx_from <= tx {
                return Some(c.clone());
            }
        }
        self.relations_history
            .iter()
            .filter(|e| e.key().0 == id)
            .find_map(|e| {
                let row = e.value();
                let in_range = row.tx_from <= tx && row.tx_to.map_or(true, |end| tx < end);
                if in_range {
                    Some(row.clone())
                } else {
                    None
                }
            })
    }

    // ── Query ────────────────────────────────────────────────────────

    pub fn find_entities(&self, filter: EntityFilter) -> Vec<Entity> {
        // Current-state query: the `entities` map holds only `tx_to = None` rows by
        // construction (freeze-on-mutate / freeze-on-delete). The explicit filter below
        // is defensive — cheap and makes the contract local.
        self.entities
            .iter()
            .filter(|e| e.value().tx_to.is_none())
            .filter(|e| filter.matches(e.value()))
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn find_relations(&self, filter: RelationFilter) -> Vec<Relation> {
        // Current-state query — see `find_entities`.
        self.relations
            .iter()
            .filter(|r| r.value().tx_to.is_none())
            .filter(|r| filter.matches(r.value()))
            .map(|r| r.value().clone())
            .collect()
    }

    /// Get neighbors of an entity in the given direction.
    pub fn neighbors(
        &self,
        id: EntityId,
        direction: Direction,
        filter: Option<&NeighborFilter>,
    ) -> Result<Vec<(Relation, Entity)>> {
        if !self.entities.contains_key(&id) {
            return Err(CtxInfError::EntityNotFound(id));
        }

        let mut results = Vec::new();
        let mut seen = HashSet::new();

        // Current-state neighbor traversal (D1): both the relation and the neighbor
        // entity must have `tx_to = None`. The `entities` / `relations` maps already
        // hold only current rows, so the `tx_to.is_none()` checks are defensive.
        if matches!(direction, Direction::Outgoing | Direction::Both) {
            if let Some(adj) = self.adj_out.get(&id) {
                for (rid, target) in adj.iter() {
                    if !seen.insert(*rid) {
                        continue;
                    }
                    if let (Some(rel), Some(ent)) =
                        (self.relations.get(rid), self.entities.get(target))
                    {
                        if rel.tx_to.is_none()
                            && ent.tx_to.is_none()
                            && filter.map_or(true, |f| f.matches_relation(&rel))
                        {
                            results.push((rel.clone(), ent.clone()));
                        }
                    }
                }
            }
        }

        if matches!(direction, Direction::Incoming | Direction::Both) {
            if let Some(adj) = self.adj_in.get(&id) {
                for (rid, source) in adj.iter() {
                    if !seen.insert(*rid) {
                        continue;
                    }
                    if let (Some(rel), Some(ent)) =
                        (self.relations.get(rid), self.entities.get(source))
                    {
                        if rel.tx_to.is_none()
                            && ent.tx_to.is_none()
                            && filter.map_or(true, |f| f.matches_relation(&rel))
                        {
                            results.push((rel.clone(), ent.clone()));
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Get entities by type.
    pub fn entities_by_type(&self, entity_type: &EntityType) -> Vec<Entity> {
        self.type_index
            .get(entity_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.entities.get(id).map(|e| e.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Snapshot: return all entities active at a given point in valid-time `t`.
    ///
    /// Current-state query by default (D1): only rows with `tx_to = None`. To reconstruct
    /// valid-time activity at an earlier transaction time, combine with
    /// [`CtxInfEngine::get_entity_as_of_tx`] (future work: a joint valid-time × tx-time API).
    pub fn active_at(&self, t: DateTime<Utc>) -> Vec<Entity> {
        self.entities
            .iter()
            .filter(|e| e.value().tx_to.is_none())
            .filter(|e| e.value().temporal.contains(t))
            .map(|e| e.value().clone())
            .collect()
    }

    /// Stats for debugging / monitoring.
    pub fn stats(&self) -> EngineStats {
        EngineStats {
            entity_count: self.entities.len(),
            relation_count: self.relations.len(),
            type_counts: self
                .type_index
                .iter()
                .map(|e| (e.key().clone(), e.value().len()))
                .collect(),
        }
    }
}

impl Default for CtxInfEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ── Filter types ─────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct EntityFilter {
    pub entity_type: Option<EntityType>,
    pub name_contains: Option<String>,
    pub active_at: Option<DateTime<Utc>>,
    pub min_version: Option<u64>,
}

impl EntityFilter {
    pub fn matches(&self, entity: &Entity) -> bool {
        if let Some(ref t) = self.entity_type {
            if &entity.entity_type != t {
                return false;
            }
        }
        if let Some(ref s) = self.name_contains {
            if !entity.name.to_lowercase().contains(&s.to_lowercase()) {
                return false;
            }
        }
        if let Some(t) = self.active_at {
            if !entity.temporal.contains(t) {
                return false;
            }
        }
        if let Some(v) = self.min_version {
            if entity.version < v {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Default)]
pub struct RelationFilter {
    pub relation_type: Option<RelationType>,
    pub source: Option<EntityId>,
    pub target: Option<EntityId>,
    pub min_confidence: Option<f64>,
    pub active_at: Option<DateTime<Utc>>,
}

impl RelationFilter {
    pub fn matches(&self, rel: &Relation) -> bool {
        if let Some(ref t) = self.relation_type {
            if &rel.relation_type != t {
                return false;
            }
        }
        if let Some(s) = self.source {
            if rel.source != s {
                return false;
            }
        }
        if let Some(t) = self.target {
            if rel.target != t {
                return false;
            }
        }
        if let Some(c) = self.min_confidence {
            if rel.confidence < c {
                return false;
            }
        }
        if let Some(t) = self.active_at {
            if !rel.temporal.contains(t) {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Default)]
pub struct NeighborFilter {
    pub relation_type: Option<RelationType>,
    pub min_confidence: Option<f64>,
    pub active_at: Option<DateTime<Utc>>,
}

impl NeighborFilter {
    fn matches_relation(&self, rel: &Relation) -> bool {
        if let Some(ref t) = self.relation_type {
            if &rel.relation_type != t {
                return false;
            }
        }
        if let Some(c) = self.min_confidence {
            if rel.confidence < c {
                return false;
            }
        }
        if let Some(t) = self.active_at {
            if !rel.temporal.contains(t) {
                return false;
            }
        }
        true
    }
}

// ── Result types ─────────────────────────────────────────────────────

#[derive(Debug)]
pub struct DeleteResult {
    pub entity_deleted: bool,
    pub relations_deleted: usize,
}

#[derive(Debug)]
pub struct EngineStats {
    pub entity_count: usize,
    pub relation_count: usize,
    pub type_counts: HashMap<EntityType, usize>,
}
