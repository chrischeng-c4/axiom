// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-raft_sm-rs.md#rust-source-unit
// HANDWRITE-BEGIN gap="missing-generator:logic:d11d8be6" tracker="standardize-gap-projects-lumen-src-raft-sm-rs" reason="EngineSm: lumen's Engine as a raft_host::RaftStateMachine (epic #524 convergence). apply folds a committed command into the engine and records ApplyOutcome in a window for read-your-write; snapshot/restore bridge to the engine RDB checkpoint; the raft log index is the WAL seq. No semantic TD captured yet; aw claim_code/fillback adoption hangs."
//! `EngineSm` — lumen's [`Engine`] as a [`raft_host::RaftStateMachine`].
//!
//! This is lumen's convergence onto the shared raft host (epic #524): the host
//! is the sole applier, so the `WriteCoordinator`/`WalLog` seam (a NATS-era
//! leftover) is no longer needed for the raft path. `apply` folds a committed
//! command into the engine and records the rich [`ApplyOutcome`] in a small
//! window so the write handler can return it (read-your-write); `snapshot`/
//! `restore` bridge to the engine's RDB checkpoint (the "backup layer").
//!
//! The raft log index **is** the WAL seq (both 1-based), so `apply_raft_entry`,
//! the RDB `up_to_seq` tag, and the outcome key all share the same `Index`.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::Result;
use raft_host::{Index, RaftStateMachine};

use crate::coordinator::WriteSink;
use crate::log_entry::RaftLogEntry;
use crate::rdb::RdbSnapshot;
use crate::storage::{ApplyOutcome, Engine};
use crate::wal::WalRecord;
use raft_host::RaftHost;

/// How many recent apply outcomes to retain for the write handler to claim.
const OUTCOME_WINDOW: u64 = 8192;

/// lumen's engine driven as a raft state machine.
pub struct EngineSm {
    engine: Arc<Engine>,
    applied: AtomicU64,
    outcomes: Mutex<BTreeMap<u64, Result<ApplyOutcome>>>,
}

impl EngineSm {
    /// Wrap `engine`, seeded at `from_seq` (the seq the engine was cold-started
    /// to, e.g. from an RDB checkpoint — `0` for a fresh engine).
    pub fn new(engine: Arc<Engine>, from_seq: u64) -> Arc<Self> {
        Arc::new(EngineSm {
            engine,
            applied: AtomicU64::new(from_seq),
            outcomes: Mutex::new(BTreeMap::new()),
        })
    }

    /// Claim the outcome for `index` (the host's `propose` returns the index;
    /// the write handler then takes the rich outcome the local apply produced).
    pub fn take_outcome(&self, index: u64) -> Result<ApplyOutcome> {
        self.outcomes
            .lock()
            .expect("outcomes poisoned")
            .remove(&index)
            .unwrap_or_else(|| Err(anyhow::anyhow!("outcome for seq {index} unavailable")))
    }
}

impl RaftStateMachine for EngineSm {
    fn apply(&self, index: Index, command: &[u8]) -> Result<()> {
        let outcome =
            WalRecord::decode(command).and_then(|rec| self.engine.apply_raft_entry(rec.entry));
        // Record the outcome (Ok or Err) for the write handler, evicting old
        // entries, then advance `applied` AFTER — so a reader never sees a seq
        // whose engine mutation hasn't landed.
        {
            let mut m = self.outcomes.lock().expect("outcomes poisoned");
            m.insert(index, outcome);
            let cutoff = index.saturating_sub(OUTCOME_WINDOW);
            while let Some((&k, _)) = m.iter().next() {
                if k < cutoff {
                    m.remove(&k);
                } else {
                    break;
                }
            }
        }
        self.applied.store(index, Ordering::Release);
        Ok(()) // the entry is "applied" (a failed apply no-ops the engine + is surfaced via the outcome)
    }

    fn snapshot(&self) -> Result<Vec<u8>> {
        RdbSnapshot {
            up_to_seq: self.applied_index(),
            snapshot: self.engine.snapshot()?,
        }
        .encode()
    }

    fn restore(&self, snapshot: &[u8]) -> Result<()> {
        let rdb = RdbSnapshot::decode(snapshot)?;
        self.engine.restore(rdb.snapshot)?;
        self.applied.store(rdb.up_to_seq, Ordering::Release);
        Ok(())
    }

    fn applied_index(&self) -> Index {
        self.applied.load(Ordering::Acquire)
    }
}

/// The [`WriteSink`] for `--wal raft`: a write proposes through the shared
/// [`RaftHost`] (which handles leader-redirect + read-your-write), and the rich
/// [`ApplyOutcome`] is claimed from the local [`EngineSm`] apply (the host
/// applies on every node, so a follower has its own outcome).
pub struct RaftWriteSink {
    host: Arc<RaftHost>,
    sm: Arc<EngineSm>,
}

impl RaftWriteSink {
    pub fn new(host: Arc<RaftHost>, sm: Arc<EngineSm>) -> Self {
        Self { host, sm }
    }
}

#[async_trait::async_trait]
impl WriteSink for RaftWriteSink {
    async fn submit(&self, entry: RaftLogEntry) -> Result<ApplyOutcome> {
        let index = self.host.propose(WalRecord::new(entry).encode()?).await?;
        self.sm.take_outcome(index)
    }
    fn applied_seq(&self) -> u64 {
        self.sm.applied_index()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log_entry::RaftLogEntry;
    use crate::types::{
        CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    };
    use raft_host::{HostConfig, Membership, RaftHost, RaftStore};
    use std::collections::{BTreeMap, HashMap};

    fn number_field() -> FieldSpec {
        FieldSpec {
            field_type: FieldType::Number,
            analyzer: None,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        }
    }

    /// lumen's real `Engine`, driven through the shared `RaftHost`, applies
    /// committed commands and returns the rich `ApplyOutcome` (read-your-write).
    #[tokio::test]
    async fn engine_applies_through_the_shared_host() {
        let tmp = std::env::temp_dir().join(format!("lumen-enginesm-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&tmp);
        let engine = Arc::new(Engine::new());
        let sm = EngineSm::new(engine.clone(), 0);
        let host = RaftHost::spawn(
            0,
            Membership {
                voters: vec![0],
                learners: vec![],
            },
            HashMap::new(),
            RaftStore::open(tmp.to_str().unwrap(), 0, raft_host::FsyncPolicy::Os).unwrap(),
            sm.clone() as Arc<dyn RaftStateMachine>,
            HostConfig::default(),
        );

        // create a collection through consensus → rich Created outcome.
        let mut fields = BTreeMap::new();
        fields.insert("n".to_string(), number_field());
        let cmd = WalRecord::new(RaftLogEntry::CreateCollection {
            collection_id: "docs".into(),
            req: CreateCollectionRequest { fields },
        })
        .encode()
        .unwrap();
        let idx = host.propose(cmd).await.unwrap();
        assert_eq!(idx, 1);
        assert!(matches!(sm.take_outcome(1), Ok(ApplyOutcome::Created(_))));

        // index a doc → rich Indexed outcome, applied to the real engine.
        let cmd = WalRecord::new(RaftLogEntry::Index {
            collection_id: "docs".into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: "d1".into(),
                    field: "n".into(),
                    value: FieldValue::Number(42.0),
                    version: None,
                }],
                request_id: None,
            },
        })
        .encode()
        .unwrap();
        let idx = host.propose(cmd).await.unwrap();
        assert_eq!(idx, 2);
        match sm.take_outcome(2) {
            Ok(ApplyOutcome::Indexed(r)) => assert_eq!(r.indexed, 1),
            other => panic!("expected Indexed, got {other:?}"),
        }
        // RYW: the engine reflects the applied write immediately.
        assert_eq!(sm.applied_index(), 2);
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
// HANDWRITE-END
