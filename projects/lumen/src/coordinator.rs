//! Write coordinator — the seam between the HTTP write handlers and the
//! log-driven apply loop.
//!
//! Design (see `wal` for the why): a write handler does **not** touch
//! the index. It calls [`WriteCoordinator::submit`], which:
//!
//! 1. publishes the mutation to the [`WalLog`] (the log assigns a global
//!    sequence — the total order),
//! 2. waits until **this node's** apply loop has folded the stream up to
//!    that sequence (read-your-write), and
//! 3. returns the [`ApplyOutcome`] the apply loop computed for it.
//!
//! Apply happens in exactly one place — the background loop subscribed to
//! the log — so every node converges by applying the same ordered
//! stream, and the node that received the write holds no special state.
//! For [`MemWal`](crate::wal::MemWal) the loop runs in-process and the
//! round-trip is sub-millisecond, so single-node writes feel synchronous
//! and existing tests see their writes immediately.
//!
//! Apply errors (e.g. a type mismatch caught at apply time) are routed
//! back as the original `anyhow::Error` — carrying the `StorageError` —
//! so the handler still maps them to the right HTTP status.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use anyhow::{bail, Result};
use futures::StreamExt;
use tokio::sync::watch;

use crate::log_entry::RaftLogEntry;
use crate::storage::{ApplyOutcome, Engine};
use crate::wal::{SharedWal, WalRecord};

/// How many recent outcomes to retain. A publisher reads its outcome
/// within microseconds of the apply loop reaching its sequence, far
/// inside this window; outcomes for sequences no local handler is
/// waiting on (writes that originated on other nodes) age out.
const OUTCOME_WINDOW: u64 = 8192;

pub struct WriteCoordinator {
    wal: SharedWal,
    applied: watch::Sender<u64>,
    outcomes: Mutex<BTreeMap<u64, Result<ApplyOutcome>>>,
}

impl WriteCoordinator {
    /// Spawn the apply loop and return the coordinator. The loop tails
    /// the log from the beginning and folds it into `engine`.
    pub fn start(wal: SharedWal, engine: Arc<Engine>) -> Arc<Self> {
        Self::start_from(wal, engine, 0)
    }

    /// Like [`start`](Self::start) but begins applying after `from_seq`
    /// — used when a snapshot (RDB) already seeded the engine up to that
    /// sequence.
    pub fn start_from(wal: SharedWal, engine: Arc<Engine>, from_seq: u64) -> Arc<Self> {
        let (applied_tx, _rx) = watch::channel(from_seq);
        let coord = Arc::new(Self {
            wal: wal.clone(),
            applied: applied_tx,
            outcomes: Mutex::new(BTreeMap::new()),
        });
        let loop_coord = coord.clone();
        tokio::spawn(async move {
            let mut backoff = std::time::Duration::from_millis(100);
            // Outer loop: re-subscribe from the last-applied sequence whenever
            // the stream ends or the subscribe fails. A NATS broker restart
            // tears down our ephemeral consumer, so the apply loop MUST
            // recreate it and resume tailing — otherwise writes silently stop
            // applying after a broker blip. Resuming from `applied` is safe:
            // redelivery is skipped idempotently below.
            loop {
                let from = *loop_coord.applied.borrow();
                let mut sub = match wal.subscribe(from).await {
                    Ok(s) => {
                        backoff = std::time::Duration::from_millis(100);
                        s
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, from, "apply loop: subscribe failed; retrying");
                        tokio::time::sleep(backoff).await;
                        backoff = (backoff * 2).min(std::time::Duration::from_secs(5));
                        continue;
                    }
                };
                while let Some(item) = sub.next().await {
                    match item {
                        Ok((seq, rec)) => {
                            // Idempotent under redelivery: skip anything at or
                            // below what we've already applied.
                            if seq <= *loop_coord.applied.borrow() {
                                continue;
                            }
                            // apply_raft_entry is synchronous and CPU-bound (a
                            // bulk index folds thousands of items + BM25 stats).
                            // Running it directly on the async worker would block
                            // the tokio runtime and starve the NATS client's I/O
                            // (missed pings → reconnect → the consumer stream
                            // stalls), wedging the apply loop under tight CPU.
                            // Offload to a blocking thread so I/O keeps flowing.
                            let eng = engine.clone();
                            let outcome = match tokio::task::spawn_blocking(move || {
                                eng.apply_raft_entry(rec.entry)
                            })
                            .await
                            {
                                Ok(o) => o,
                                Err(e) => Err(anyhow::anyhow!("apply task panicked: {e}")),
                            };
                            if let Err(e) = &outcome {
                                tracing::warn!(seq, error = %e, "apply error (entry no-ops)");
                            }
                            loop_coord.complete(seq, outcome);
                        }
                        Err(e) => tracing::warn!(error = %e, "apply loop: stream item error"),
                    }
                }
                // Stream ended (e.g. broker restart killed the ephemeral
                // consumer). Re-subscribe from the applied head after a short
                // pause so we don't tight-spin if the broker is flapping.
                tracing::warn!("apply loop: stream ended; re-subscribing from applied seq");
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
        });
        coord
    }

    fn complete(&self, seq: u64, outcome: Result<ApplyOutcome>) {
        {
            let mut m = self.outcomes.lock().expect("outcomes poisoned");
            m.insert(seq, outcome);
            // Prune everything older than the retention window.
            let cutoff = seq.saturating_sub(OUTCOME_WINDOW);
            while let Some((&k, _)) = m.iter().next() {
                if k < cutoff {
                    m.remove(&k);
                } else {
                    break;
                }
            }
        }
        // Publish the new applied head AFTER the outcome is stored, so a
        // waiter that observes `applied >= seq` always finds its outcome.
        let _ = self.applied.send(seq);
    }

    /// Publish `entry`, wait for local apply, and return its outcome.
    pub async fn submit(&self, entry: RaftLogEntry) -> Result<ApplyOutcome> {
        let seq = self.wal.publish(&WalRecord::new(entry)).await?;
        let mut rx = self.applied.subscribe();
        // `watch` always exposes the latest value on borrow and coalesces
        // without losing it, so this converges even if the loop races
        // ahead between checks.
        while *rx.borrow() < seq {
            if rx.changed().await.is_err() {
                bail!("apply loop stopped before sequence {seq} was applied");
            }
        }
        let out = self
            .outcomes
            .lock()
            .expect("outcomes poisoned")
            .remove(&seq);
        match out {
            Some(result) => result,
            None => bail!("outcome for sequence {seq} was pruned before retrieval"),
        }
    }

    /// Highest sequence this node has applied.
    pub fn applied_seq(&self) -> u64 {
        *self.applied.borrow()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    };
    use crate::wal::MemWal;
    use std::collections::BTreeMap as Map;

    fn keyword_schema() -> CreateCollectionRequest {
        let mut fields = Map::new();
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

    #[tokio::test]
    async fn submit_creates_then_indexes_and_outcome_is_routed_back() {
        let engine = Arc::new(Engine::new());
        let wal = Arc::new(MemWal::new());
        let coord = WriteCoordinator::start(wal, engine.clone());

        let created = coord
            .submit(RaftLogEntry::CreateCollection {
                collection_id: "u".into(),
                req: keyword_schema(),
            })
            .await
            .unwrap();
        match created {
            ApplyOutcome::Created(r) => {
                assert_eq!(r.version, 1);
                assert_eq!(r.fields_count, 1);
            }
            other => panic!("expected Created, got {other:?}"),
        }

        let indexed = coord
            .submit(RaftLogEntry::Index {
                collection_id: "u".into(),
                req: IndexRequest {
                    items: vec![IndexItem {
                        external_id: "u1".into(),
                        field: "email".into(),
                        value: FieldValue::String("a@x.com".into()),
                    }],
                    request_id: None,
                },
            })
            .await
            .unwrap();
        match indexed {
            ApplyOutcome::Indexed(r) => assert_eq!(r.indexed, 1),
            other => panic!("expected Indexed, got {other:?}"),
        }

        // The write is visible via a direct engine read (read-your-write).
        assert_eq!(engine.stats("u").unwrap().documents_indexed, 1);
    }

    #[tokio::test]
    async fn submit_propagates_apply_error_with_type() {
        use crate::storage::StorageError;
        let engine = Arc::new(Engine::new());
        let wal = Arc::new(MemWal::new());
        let coord = WriteCoordinator::start(wal, engine.clone());

        // Index into a collection that doesn't exist → CollectionNotFound,
        // and the error must survive routing (downcast still works).
        let err = coord
            .submit(RaftLogEntry::Index {
                collection_id: "ghost".into(),
                req: IndexRequest {
                    items: vec![IndexItem {
                        external_id: "x".into(),
                        field: "email".into(),
                        value: FieldValue::String("a@x.com".into()),
                    }],
                    request_id: None,
                },
            })
            .await
            .unwrap_err();
        assert!(
            err.downcast_ref::<StorageError>()
                .map(|e| matches!(e, StorageError::CollectionNotFound(_)))
                .unwrap_or(false),
            "StorageError must survive coordinator routing, got: {err}"
        );
    }
}
