//! Exact segment checkpoint cuts for the Raft snapshot adapter.

use super::*;

pub(crate) struct SegmentRaftPreparation {
    store: Arc<SegmentRdbStore>,
    engine: Arc<Engine>,
    permit: save_gate::SavePermit,
    current: Option<GenerationRecord>,
    floor: u64,
    epoch: u64,
    prior: Option<SegmentGenerationManifest>,
}

pub(crate) struct SegmentRaftCapture {
    store: Arc<SegmentRdbStore>,
    engine: Arc<Engine>,
    // Drop order returns the frozen payload before releasing root serialization.
    pending: PendingFrozenLease,
    permit: save_gate::SavePermit,
}

impl SegmentRdbStore {
    pub(crate) fn raft_snapshot_preflight(
        self: &Arc<Self>,
        engine: Arc<Engine>,
    ) -> Result<SegmentRaftPreparation> {
        loop {
            let permit = self.save_gate.lock_owned();
            self.inventory_root()?;
            self.sweep_abandoned_staging()?;
            let current = self.current_record()?;
            let prior = current
                .as_ref()
                .filter(|record| !record.legacy)
                .map(|record| read_generation_manifest(&record.path))
                .transpose()?;
            let predecessor = PendingPredecessor::from_current(current.as_ref());
            if let Some(pending) = self.take_matching_pending(&engine, &predecessor)? {
                let sequence = pending.pending().sequence;
                // Return ownership to the common publish path, under this permit.
                drop(pending);
                self.save_inner_permitted(
                    &engine,
                    sequence,
                    true,
                    permit,
                    SaveIntent::ExactRaft,
                    None,
                )?;
                continue;
            }
            if let Some((record, manifest)) = current.as_ref().zip(prior.as_ref()) {
                self.verify_predecessor_catalog(record, manifest)?;
                if self.needs_delta_capacity(&engine, manifest, &record.path)? {
                    drop(permit);
                    self.request_merge(&engine)?;
                    self.wait_for_merges(std::time::Duration::from_secs(60))?;
                    continue;
                }
            }
            let floor = prior
                .as_ref()
                .map_or(1, |manifest| manifest.next_collection_generation);
            let epoch = engine.capture_barrier.epoch();
            return Ok(SegmentRaftPreparation {
                store: self.clone(),
                engine,
                permit,
                current,
                floor,
                epoch,
                prior,
            });
        }
    }
}

impl SegmentRaftPreparation {
    pub(crate) fn capture_at(self, index: u64) -> Result<SegmentRaftCapture> {
        let SegmentRaftPreparation {
            store,
            engine,
            permit,
            current,
            floor,
            epoch,
            prior,
        } = self;
        let capture = engine
            .capture_barrier
            .capture(index)
            .map_err(anyhow::Error::msg)?;
        let capture_started = std::time::Instant::now();
        let stamp = capture.stamp();
        if stamp.sequence != index {
            bail!(
                "raft snapshot cut {index} is stale; applied sequence is {}",
                stamp.sequence
            );
        }
        if epoch != stamp.epoch {
            bail!("raft snapshot capture epoch changed");
        }
        if let Some((record, manifest)) = current.as_ref().zip(prior.as_ref()) {
            if store.needs_delta_capacity(&engine, manifest, &record.path)? {
                bail!("Raft snapshot preflight became stale: incremental segment capacity is full");
            }
        }
        engine.prepare_checkpoint_namespace(&store.root, floor)?;
        let frozen = engine
            .freeze_checkpoint_collections(current.as_ref().map(|record| record.path.as_path()))?;
        let detached_capture_ns =
            u64::try_from(capture_started.elapsed().as_nanos()).unwrap_or(u64::MAX);
        let layer_window = engine.layer_maintenance.freeze();
        drop(capture);
        Ok(SegmentRaftCapture {
            store: store.clone(),
            engine: engine.clone(),
            pending: PendingFrozenLease {
                slot: store.pending_frozen.clone(),
                pending: Some(PendingFrozenCheckpoint {
                    engine: Arc::downgrade(&engine),
                    stamp,
                    sequence: index,
                    predecessor: PendingPredecessor::from_current(current.as_ref()),
                    frozen,
                    detached_capture_ns,
                    _layer_window: layer_window,
                }),
            },
            permit,
        })
    }
}

impl SegmentRaftCapture {
    pub(crate) fn publish(self) -> Result<SegmentArchivePin> {
        let SegmentRaftCapture {
            store,
            engine,
            mut pending,
            permit,
        } = self;
        let sequence = pending.pending().sequence;
        {
            let mut slot = store
                .pending_frozen
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            if slot.is_some() {
                bail!("raft snapshot publish found another pending frozen checkpoint");
            }
            *slot = pending.pending.take();
        }
        let mut pin = None;
        let _name = store.save_inner_permitted(
            &engine,
            sequence,
            true,
            permit,
            SaveIntent::ExactRaft,
            Some(&mut pin),
        )?;
        pin.ok_or_else(|| anyhow!("raft snapshot publish did not create archive pin"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CreateCollectionRequest, IndexRequest};
    use serde_json::json;

    fn seeded() -> (tempfile::TempDir, Arc<SegmentRdbStore>, Arc<Engine>) {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SegmentRdbStore::new(dir.path()).unwrap());
        let engine = Arc::new(Engine::new());
        let schema: CreateCollectionRequest =
            serde_json::from_value(json!({"fields": {"kind": {"type": "keyword"}}})).unwrap();
        engine.create_collection("docs", schema).unwrap();
        let request: IndexRequest = serde_json::from_value(
            json!({"items": [{"external_id":"one", "field":"kind", "value":"before"}]}),
        )
        .unwrap();
        engine.index("docs", request).unwrap();
        engine.capture_barrier.apply().initialize_sequence(2);
        (dir, store, engine)
    }

    #[test]
    fn cancelled_capture_retains_the_frozen_cut_for_retry() {
        let (_dir, store, engine) = seeded();
        let preparation = store.raft_snapshot_preflight(engine.clone()).unwrap();
        drop(preparation.capture_at(2).unwrap());
        assert_eq!(
            store
                .pending_frozen
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .sequence,
            2
        );
        // Preflight must publish the retained cut and release its slot first.
        let preparation = store.raft_snapshot_preflight(engine.clone()).unwrap();
        assert!(store.pending_frozen.lock().unwrap().is_none());
        let pin = preparation.capture_at(2).unwrap().publish().unwrap();
        assert_eq!(pin.sequence(), 2);
        assert_eq!(
            store.load_current_generation().unwrap().unwrap().sequence,
            2
        );
    }

    #[test]
    fn stale_applied_cut_refuses_before_freezing() {
        let (_dir, store, engine) = seeded();
        let preparation = store.raft_snapshot_preflight(engine.clone()).unwrap();
        engine.capture_barrier.apply().advance_sequence(3);
        assert!(preparation.capture_at(2).is_err());
        assert!(store.pending_frozen.lock().unwrap().is_none());
    }

    #[test]
    fn restore_epoch_invalidates_preflight_before_freezing() {
        let (_dir, store, engine) = seeded();
        let preparation = store.raft_snapshot_preflight(engine.clone()).unwrap();
        let inhibition = engine
            .capture_barrier
            .apply()
            .inhibit_checkpoints_for_restore();
        drop(inhibition);
        assert!(preparation.capture_at(2).is_err());
        assert!(store.pending_frozen.lock().unwrap().is_none());
    }

    #[test]
    fn captured_raft_cut_cannot_recapture_after_restore_changes_epoch() {
        let (dir, store, engine) = seeded();
        let captured = store
            .raft_snapshot_preflight(engine.clone())
            .unwrap()
            .capture_at(2)
            .unwrap();
        let before = std::fs::read(dir.path().join("CURRENT")).unwrap();
        engine.restore(Engine::new().snapshot().unwrap()).unwrap();
        assert!(
            captured.publish().is_err(),
            "a pre-restore Raft cut must not recapture replacement state during publish"
        );
        assert_eq!(std::fs::read(dir.path().join("CURRENT")).unwrap(), before);
    }

    #[test]
    fn published_pin_survives_a_newer_checkpoint_and_prune() {
        let (_dir, store, engine) = seeded();
        let pin = store
            .raft_snapshot_preflight(engine.clone())
            .unwrap()
            .capture_at(2)
            .unwrap()
            .publish()
            .unwrap();
        let path = pin.path().to_owned();
        engine.capture_barrier.apply().advance_sequence(3);
        store.save_required(&engine, 3).unwrap();
        store.prune(0).unwrap();
        assert!(
            path.is_dir(),
            "live archive pin must retain the exact old generation"
        );
        drop(pin);
        store.prune(0).unwrap();
        assert!(
            !path.exists(),
            "unreferenced old generation should be reclaimed"
        );
    }
}
