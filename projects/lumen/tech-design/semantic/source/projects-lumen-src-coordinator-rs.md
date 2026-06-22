---
id: projects-lumen-src-coordinator-rs
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/coordinator.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/coordinator.rs` captured as a per-file rust-source-unit (td_ast) during lumen standardization onto the per-file codegen ladder.

### Symbols

| Name | Target | Kind | Visibility |
|------|--------|------|------------|
| `SharedAof` | projects/lumen/src/coordinator.rs | type | pub |
| `WriteCoordinator` | projects/lumen/src/coordinator.rs | struct | pub |
| `start` | projects/lumen/src/coordinator.rs | function | pub |
| `start_from` | projects/lumen/src/coordinator.rs | function | pub |
| `start_from_with_aof` | projects/lumen/src/coordinator.rs | function | pub |
| `submit` | projects/lumen/src/coordinator.rs | function | pub |
| `applied_seq` | projects/lumen/src/coordinator.rs | function | pub |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-coordinator-rs.md#rust-source-unit
// CODEGEN-BEGIN
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
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{bail, Result};
use futures::{FutureExt, StreamExt};
use rustc_hash::FxHashMap;
use tokio::sync::oneshot;

use crate::log_entry::RaftLogEntry;
use crate::storage::{ApplyOutcome, Engine};
use crate::wal::{SharedWal, WalRecord};

/// How many recent outcomes to retain. A publisher reads its outcome
/// within microseconds of the apply loop reaching its sequence, far
/// inside this window; outcomes for sequences no local handler is
/// waiting on (writes that originated on other nodes) age out.
const OUTCOME_WINDOW: u64 = 8192;
const APPLY_LOOP_BATCH: usize = 128;

struct PendingApply {
    seq: u64,
    rec: WalRecord,
    aof_rec: Option<WalRecord>,
}

struct CompletionState {
    outcomes: BTreeMap<u64, Result<ApplyOutcome>>,
    waiters: FxHashMap<u64, oneshot::Sender<Result<ApplyOutcome>>>,
}

/// The optional local AOF the apply loop appends every applied record to (Stage
/// 2 Phase 2f-3). Wrapped in a `Mutex` because the apply loop appends from the
/// async task while the periodic checkpoint snapshotter calls `truncate_through`
/// from another task; the lock is held only for the (buffered) append/truncate.
/// `None` on the default / non-AOF path, so `start_from` is byte-identical to
/// today. The everysec fsync runs off the hot path via `maybe_sync` (driven by
/// the loop after each batch), so an append never blocks on the disk.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-coordinator-rs.md#source
pub type SharedAof = Arc<Mutex<crate::aof::AofWriter>>;

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-coordinator-rs.md#source
pub struct WriteCoordinator {
    wal: SharedWal,
    applied: AtomicU64,
    completions: Mutex<CompletionState>,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-coordinator-rs.md#source
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
        // The default / non-AOF path. Delegates with no AOF, so the apply loop is
        // byte-identical to today.
        Self::start_from_inner(wal, engine, from_seq, None)
    }

    /// Like [`start_from`](Self::start_from) but also appends every APPLIED
    /// `(seq, record)` to a local AOF (Stage 2 Phase 2f-3), AFTER the apply
    /// succeeds and `applied` advances. The default / non-AOF path is unchanged —
    /// this is the only entry point that wires an AOF in.
    pub fn start_from_with_aof(
        wal: SharedWal,
        engine: Arc<Engine>,
        from_seq: u64,
        aof: SharedAof,
    ) -> Arc<Self> {
        Self::start_from_inner(wal, engine, from_seq, Some(aof))
    }

    /// The apply-loop spawner. Identical structure regardless of the AOF; the
    /// only AOF-specific work is the append after `complete`, conditioned on the
    /// `aof` being `Some` (the default `start_from` passes `None`).
    fn start_from_inner(
        wal: SharedWal,
        engine: Arc<Engine>,
        from_seq: u64,
        aof: Option<SharedAof>,
    ) -> Arc<Self> {
        let coord = Arc::new(Self {
            wal: wal.clone(),
            applied: AtomicU64::new(from_seq),
            completions: Mutex::new(CompletionState {
                outcomes: BTreeMap::new(),
                waiters: FxHashMap::default(),
            }),
        });
        let loop_coord = coord.clone();
        tokio::spawn(async move {
            let mut backoff = std::time::Duration::from_millis(100);
            // Outer loop: re-subscribe from the last-applied sequence whenever
            // the stream ends or the subscribe fails. A broker restart can tear
            // down our ephemeral subscription, so the apply loop MUST recreate
            // it and resume tailing — otherwise writes silently stop applying
            // after a broker blip. Resuming from `applied` is safe:
            // redelivery is skipped idempotently below.
            loop {
                let from = loop_coord.applied.load(Ordering::Acquire);
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
                    let mut stream_ended = false;
                    match item {
                        Ok((seq, rec)) => {
                            // Idempotent under redelivery: skip anything at or
                            // below what we've already applied.
                            if seq <= loop_coord.applied.load(Ordering::Acquire) {
                                continue;
                            }
                            let mut batch = Vec::with_capacity(APPLY_LOOP_BATCH);
                            batch.push(PendingApply {
                                seq,
                                aof_rec: aof.as_ref().map(|_| rec.clone()),
                                rec,
                            });
                            while batch.len() < APPLY_LOOP_BATCH {
                                match sub.next().now_or_never() {
                                    Some(Some(Ok((seq, rec)))) => {
                                        if seq <= loop_coord.applied.load(Ordering::Acquire) {
                                            continue;
                                        }
                                        batch.push(PendingApply {
                                            seq,
                                            aof_rec: aof.as_ref().map(|_| rec.clone()),
                                            rec,
                                        });
                                    }
                                    Some(Some(Err(e))) => {
                                        tracing::warn!(
                                            error = %e,
                                            "apply loop: stream item error"
                                        );
                                    }
                                    Some(None) => {
                                        stream_ended = true;
                                        break;
                                    }
                                    None => break,
                                }
                            }
                            // apply_raft_entry is synchronous and CPU-bound (a bulk index folds
                            // thousands of items + BM25 stats). Run ready records in one blocking
                            // task to keep broker I/O moving without paying a thread handoff per WAL
                            // record.
                            let eng = engine.clone();
                            let seqs: Vec<u64> = batch.iter().map(|pending| pending.seq).collect();
                            let results = match tokio::task::spawn_blocking(move || {
                                let mut results = Vec::with_capacity(batch.len());
                                for pending in batch {
                                    let outcome = eng.apply_raft_entry(pending.rec.entry);
                                    results.push((pending.seq, outcome, pending.aof_rec));
                                }
                                results
                            })
                            .await
                            {
                                Ok(results) => results,
                                Err(e) => {
                                    let err = e.to_string();
                                    seqs.into_iter()
                                        .map(|seq| {
                                            (
                                                seq,
                                                Err(anyhow::anyhow!(
                                                    "apply batch task panicked: {err}"
                                                )),
                                                None,
                                            )
                                        })
                                        .collect()
                                }
                            };

                            for (seq, outcome, aof_rec) in results {
                                let applied_ok = outcome.is_ok();
                                if let Err(e) = &outcome {
                                    tracing::warn!(seq, error = %e, "apply error (entry no-ops)");
                                }
                                loop_coord.complete(seq, outcome);
                                // AOF append AFTER apply + advancing `applied`: the log mirrors
                                // exactly what the engine folded in. A failed apply no-ops the
                                // engine, so it is NOT appended.
                                if applied_ok {
                                    if let (Some(aof), Some(rec)) = (aof.as_ref(), aof_rec) {
                                        let mut w = aof.lock().expect("aof writer poisoned");
                                        if let Err(e) = w.append(seq, &rec) {
                                            tracing::warn!(seq, error = %e, "AOF append failed");
                                        }
                                        // everysec fsync, off the hot path (no-op unless dirty AND
                                        // ≥1s elapsed; Always already synced).
                                        let _ = w.maybe_sync();
                                    }
                                }
                            }
                        }
                        Err(e) => tracing::warn!(error = %e, "apply loop: stream item error"),
                    }
                    if stream_ended {
                        break;
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
        let mut direct = None;
        {
            let mut m = self.completions.lock().expect("completions poisoned");
            if let Some(tx) = m.waiters.remove(&seq) {
                direct = Some((tx, outcome));
            } else {
                m.outcomes.insert(seq, outcome);
            }
            // Prune everything older than the retention window.
            let cutoff = seq.saturating_sub(OUTCOME_WINDOW);
            while let Some((&k, _)) = m.outcomes.iter().next() {
                if k < cutoff {
                    m.outcomes.remove(&k);
                } else {
                    break;
                }
            }
        }
        // Publish the new applied head AFTER the outcome is stored, so checkpoint
        // readers never see a seq before its engine mutation has been applied.
        self.applied.store(seq, Ordering::Release);
        if let Some((tx, outcome)) = direct {
            let _ = tx.send(outcome);
        }
    }

    fn register_waiter(&self, seq: u64) -> Result<oneshot::Receiver<Result<ApplyOutcome>>> {
        let mut m = self.completions.lock().expect("completions poisoned");
        if let Some(result) = m.outcomes.remove(&seq) {
            let (tx, rx) = oneshot::channel();
            let _ = tx.send(result);
            return Ok(rx);
        }
        let (tx, rx) = oneshot::channel();
        if m.waiters.insert(seq, tx).is_some() {
            bail!("duplicate waiter for sequence {seq}");
        }
        Ok(rx)
    }

    /// Publish `entry`, wait for local apply, and return its outcome.
    pub async fn submit(&self, entry: RaftLogEntry) -> Result<ApplyOutcome> {
        let seq = self.wal.publish(WalRecord::new(entry)).await?;
        let rx = self.register_waiter(seq)?;
        rx.await
            .map_err(|_| anyhow::anyhow!("apply loop stopped before sequence {seq} was applied"))?
    }

    /// Highest sequence this node has applied.
    pub fn applied_seq(&self) -> u64 {
        self.applied.load(Ordering::Acquire)
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
// CODEGEN-END

````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/coordinator.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/coordinator.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```
