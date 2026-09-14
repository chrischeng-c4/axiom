//! Borrow a staged native WAL record through apply and AOF persistence.
//!
//! The subscription keeps the committed head pinned. Staging, validation and
//! capacity waits run before apply. A single apply lease covers the mutation,
//! AOF flush and applied watermark, including the uncertain-failure boundary.

use super::*;
use crate::wal::fast_index_scanner::FastIndexScanner;

pub(super) enum MappedApply {
    Applied,
    Unresolved,
    Fallback(WalDelivery),
}

impl WriteCoordinator {
    /// Called only for a committed delivery without a local reservation.
    pub(super) async fn try_apply_mapped_delivery(
        self: &Arc<Self>,
        seq: u64,
        delivery: WalDelivery,
        aof: Option<SharedAof>,
    ) -> Result<MappedApply> {
        if delivery.decoded_owned_bytes()? <= crate::change_budget::HARD_LIMIT / 8 {
            return Ok(MappedApply::Fallback(delivery));
        }
        let Some(proof) = self.wal.stage_source(seq).await? else {
            return Ok(MappedApply::Fallback(delivery));
        };
        anyhow::ensure!(
            proof.sequence() == seq,
            "staged WAL proof identifies another sequence"
        );
        let coord = self.clone();
        tokio::task::spawn_blocking(move || {
            // Mapping includes integrity validation and must not block an
            // async worker or retain the native slot mutex during apply.
            let fast = delivery.mapped_fast_index()?;
            let generic = if fast.is_none() { delivery.mapped_generic_cbor()? } else { None };
            let Some(payload) = fast.as_ref().map(|source| source.payload())
                .or_else(|| generic.as_ref().map(|source| source.bytes())) else {
                return Ok(MappedApply::Fallback(delivery));
            };
            let mut completed = false;
            let mut ensure_owner = || {
                let mut owner = coord.layer_capacity_owner.lock().map_err(|_| anyhow::anyhow!("capacity owner poisoned"))?;
                crate::segment_capacity::Fallback::ensure(&mut owner, &coord.engine, None)
            };
            let complete = |apply: &crate::capture_barrier::ApplyLease<'_>, mut outcome: Result<ApplyOutcome>| {
                if coord.mutation_gate.is_restart_required() {
                    apply.mark_uncertain();
                    coord.fail_unresolved(seq, Err(anyhow::Error::new(RestartRequired(
                        format!("sequence {seq} cannot complete after an earlier local AOF gap")
                    ))), None);
                    return;
                }
                if let Err(error) = &outcome {
                    tracing::warn!(seq, %error, "committed scalar apply returned a business error");
                }
                let persisted = aof.as_ref().map_or(Ok(()), |aof| {
                    let mut writer = aof.lock().expect("aof writer poisoned");
                    writer.append_raw_payload(seq, payload)
                        .and_then(|()| writer.flush())
                        .and_then(|()| writer.maybe_sync())
                });
                if let Err(error) = persisted {
                    coord.mutation_gate.require_restart();
                    apply.mark_uncertain();
                    outcome = if is_storage_full(&error) {
                        coord.engine.metrics().mark_storage_degraded();
                        Err(anyhow::Error::new(StorageFullError(format!(
                            "local storage is full (ENOSPC) persisting sequence {seq}; node entered degraded read-only mode"
                        ))))
                    } else {
                        Err(anyhow::Error::new(RestartRequired(format!(
                            "persisted state is uncertain after local AOF failure at sequence {seq}: {error}; restart this Lumen process"
                        ))))
                    };
                    coord.fail_unresolved(seq, outcome, None);
                    return;
                }
                apply.advance_sequence(seq);
                coord.complete(seq, outcome);
                completed = true;
            };
            let handled = if fast.is_some() {
                let scanner = FastIndexScanner::parse(payload)?;
                coord.engine.try_apply_committed_index_with_capacity_owner(&scanner, seq, &mut ensure_owner, complete)?
            } else {
                coord.engine.try_apply_committed_replace_with_capacity_owner(payload, seq, &mut ensure_owner, complete)?
            };
            Ok(if handled {
                if completed { MappedApply::Applied } else { MappedApply::Unresolved }
            } else {
                MappedApply::Fallback(delivery)
            })
        }).await.map_err(|error| anyhow::anyhow!("mapped WAL apply task failed: {error}"))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aof::{AofReader, AofWriter};
    use crate::types::{
        CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    };
    use crate::wal::{MemWal, WalLog};
    use std::collections::BTreeMap;
    use std::time::Duration;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn mapped_native_aof_failure_keeps_the_watermark_and_committed_source() {
        for kind in [std::io::ErrorKind::Other, std::io::ErrorKind::StorageFull] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("aof.log");
            let aof = Arc::new(Mutex::new(AofWriter::open(&path).unwrap()));
            let wal = Arc::new(MemWal::new());
            let engine = Arc::new(Engine::new());
            let coord =
                WriteCoordinator::start_from_with_aof(wal.clone(), engine.clone(), 0, aof.clone());
            coord
                .submit(RaftLogEntry::CreateCollection {
                    collection_id: "mapped-gap".into(),
                    req: CreateCollectionRequest {
                        fields: BTreeMap::from([(
                            "keyword".into(),
                            FieldSpec {
                                field_type: FieldType::Keyword,
                                analyzer: None,
                                multi: None,
                                dim: None,
                                metric: None,
                                backend: None,
                                quantize: None,
                            },
                        )]),
                    },
                })
                .await
                .unwrap();
            assert_eq!(coord.applied_seq(), 1);
            aof.lock().unwrap().inject_failure_once(kind);
            wal.publish(WalRecord::new(RaftLogEntry::Index {
                collection_id: "mapped-gap".into(),
                req: IndexRequest {
                    request_id: None,
                    items: vec![IndexItem {
                        external_id: "retained".into(),
                        field: "keyword".into(),
                        version: None,
                        value: FieldValue::String("x".repeat(40 * 1024 * 1024)),
                    }],
                },
            }))
            .await
            .unwrap();
            tokio::time::timeout(Duration::from_secs(30), async {
                while !coord
                    .completions
                    .lock()
                    .unwrap()
                    .unresolved
                    .contains_key(&2)
                {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            })
            .await
            .expect("mapped native apply must observe the injected AOF failure");
            assert!(coord.mutation_gate.is_restart_required());
            assert_eq!(
                engine.stats("mapped-gap").unwrap().documents_indexed,
                1,
                "fault must occur after the real scalar mutation"
            );
            assert_eq!(
                coord.applied_seq(),
                1,
                "AOF failure must not advance the applied prefix"
            );
            assert_eq!(
                engine.metrics().is_storage_degraded(),
                kind == std::io::ErrorKind::StorageFull
            );
            let mut sequences = Vec::new();
            AofReader::replay(&path, 0, |seq, _| sequences.push(seq)).unwrap();
            assert_eq!(
                sequences,
                [1],
                "failed raw append must leave only the valid AOF prefix"
            );
            let mut retained = wal.subscribe_admitted(1).await.unwrap();
            let (seq, source) = tokio::time::timeout(Duration::from_secs(5), retained.next())
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            assert_eq!(seq, 2);
            assert!(
                source.mapped_fast_index().unwrap().is_some(),
                "failed committed head must retain its staged payload"
            );
            let store =
                crate::segment_rdb::SegmentRdbStore::new(dir.path().join("segments")).unwrap();
            assert!(
                store.save_with_sequence(&engine, 1).is_err(),
                "uncertain apply must refuse checkpoint publication"
            );
        }
    }
}
