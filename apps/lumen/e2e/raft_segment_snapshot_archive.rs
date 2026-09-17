//! Black-box contract for the Lumen 0.6.1 segment-backed Raft snapshot.
//!
//! A segment-mode Raft state machine must capture one exact applied prefix,
//! then write a self-contained `LSEGRAFT` v1 archive without holding that
//! capture boundary. A fresh segment-mode receiver must restore the archive
//! without reaching the sender's segment root. It must also retain the old RDB
//! restore path for a rolling upgrade.
//!
//! # Facets
//!
//! - Behavior: exact-cut, compatibility, durable-CURRENT, and blocked-output assertions are at
//!   `apps/lumen/e2e/raft_segment_snapshot_archive.rs:213`, `apps/lumen/e2e/raft_segment_snapshot_archive.rs:218`,
//!   `apps/lumen/e2e/raft_segment_snapshot_archive.rs:223`, `apps/lumen/e2e/raft_segment_snapshot_archive.rs:228`,
//!   `apps/lumen/e2e/raft_segment_snapshot_archive.rs:236`, `apps/lumen/e2e/raft_segment_snapshot_archive.rs:241`,
//!   `apps/lumen/e2e/raft_segment_snapshot_archive.rs:246`, `apps/lumen/e2e/raft_segment_snapshot_archive.rs:251`,
//!   `apps/lumen/e2e/raft_segment_snapshot_archive.rs:326`, `apps/lumen/e2e/raft_segment_snapshot_archive.rs:429`,
//!   `apps/lumen/e2e/raft_segment_snapshot_archive.rs:443`, `apps/lumen/e2e/raft_segment_snapshot_archive.rs:454`,
//!   `apps/lumen/e2e/raft_segment_snapshot_archive.rs:464`, `apps/lumen/e2e/raft_segment_snapshot_archive.rs:472`,
//!   `apps/lumen/e2e/raft_segment_snapshot_archive.rs:482`, `apps/lumen/e2e/raft_segment_snapshot_archive.rs:662`,
//!   `apps/lumen/e2e/raft_segment_snapshot_archive.rs:666`, `apps/lumen/e2e/raft_segment_snapshot_archive.rs:670`,
//!   `apps/lumen/e2e/raft_segment_snapshot_archive.rs:693`, `apps/lumen/e2e/raft_segment_snapshot_archive.rs:702`,
//!   `apps/lumen/e2e/raft_segment_snapshot_archive.rs:712`, `apps/lumen/e2e/raft_segment_snapshot_archive.rs:738`, `apps/lumen/e2e/raft_segment_snapshot_archive.rs:751`, and `apps/lumen/e2e/raft_segment_snapshot_archive.rs:761`. These exercise `apps/lumen/src/raft_sm.rs:229` and
//!   `apps/lumen/src/segment_rdb.rs:455`.
//! - Security: `apps/lumen/e2e/raft_segment_snapshot_archive.rs:295`, `apps/lumen/e2e/raft_segment_snapshot_archive.rs:299`,
//!   `apps/lumen/e2e/raft_segment_snapshot_archive.rs:303`, `apps/lumen/e2e/raft_segment_snapshot_archive.rs:310`, and
//!   `apps/lumen/e2e/raft_segment_snapshot_archive.rs:314` reject each malformed archive and retain the Raft watermark and CURRENT.
//!   `apps/lumen/e2e/raft_segment_snapshot_archive.rs:259`, `apps/lumen/e2e/raft_segment_snapshot_archive.rs:264`,
//!   `apps/lumen/e2e/raft_segment_snapshot_archive.rs:269`, and `apps/lumen/e2e/raft_segment_snapshot_archive.rs:274` retain live queries.
//!   The peer/file archive boundary changes at `apps/lumen/src/raft_sm.rs:247` with segment archive files from
//!   `apps/lumen/src/segment_rdb.rs:455`.
//! - Performance: the approved work-item workload is a separate pending gate: 30 min on 2.5 CPU/16 GiB with checkpoint and merge,
//!   10 QPS, 100 doc ops/s, p99 <= 1 s, each query <= 5 s, zero errors/timeouts, drain <= 60 s, and RSS <= 12 GiB.
//!   `SNAPSHOT_OUTPUT_WATCHDOG` is test cleanup only, not an archive SLA; periodic wiring remains pending at
//!   `apps/lumen/src/raft_sm.rs:229` and `apps/lumen/src/segment_rdb.rs:455`.

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use lumen::log_entry::RaftLogEntry;
use lumen::raft_sm::EngineSm;
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::Engine;
use lumen::types::{
    Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    KnnQuery, MatchOp, MatchQuery, QueryNode, SearchRequest, TermQuery, VectorBackend,
    VectorMetric,
};
use lumen::wal::WalRecord;
use raft_runtime::{PreparedSnapshot, RaftStateMachine, SnapshotPreparation};
use tempfile::TempDir;

const ARCHIVE_MAGIC: &[u8] = b"LSEGRAFT";
const SNAPSHOT_INDEX: u64 = 2;
const POST_CAPTURE_INDEX: u64 = SNAPSHOT_INDEX + 1;
const SNAPSHOT_OUTPUT_WATCHDOG: Duration = Duration::from_secs(2);

const COLLECTION: &str = "archive-contract";
const KEYWORD_FIELD: &str = "keyword";
const TEXT_FIELD: &str = "body";
const FLAT_FIELD: &str = "flat";
const HNSW_FIELD: &str = "hnsw";

fn ordinary_field(field_type: FieldType) -> FieldSpec {
    FieldSpec {
        field_type,
        analyzer: None,
        multi: None,
        dim: None,
        metric: None,
        backend: None,
        quantize: None,
    }
}

fn vector_field(backend: VectorBackend) -> FieldSpec {
    FieldSpec {
        field_type: FieldType::Vector,
        analyzer: None,
        multi: None,
        dim: Some(2),
        metric: Some(VectorMetric::L2),
        backend: Some(backend),
        quantize: None,
    }
}

fn schema() -> CreateCollectionRequest {
    let mut fields = BTreeMap::new();
    fields.insert(KEYWORD_FIELD.to_owned(), ordinary_field(FieldType::Keyword));
    fields.insert(
        TEXT_FIELD.to_owned(),
        FieldSpec {
            field_type: FieldType::Text,
            analyzer: Some(Analyzer::WhitespaceLower),
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        },
    );
    fields.insert(FLAT_FIELD.to_owned(), vector_field(VectorBackend::FlatCpu));
    fields.insert(HNSW_FIELD.to_owned(), vector_field(VectorBackend::HnswCpu));
    CreateCollectionRequest { fields }
}

fn create_collection() -> RaftLogEntry {
    RaftLogEntry::CreateCollection {
        collection_id: COLLECTION.to_owned(),
        req: schema(),
    }
}

fn index_document(
    external_id: &str,
    keyword: &str,
    body: &str,
    flat: [f32; 2],
    hnsw: [f32; 2],
) -> RaftLogEntry {
    RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items: vec![
                IndexItem {
                    external_id: external_id.to_owned(),
                    field: KEYWORD_FIELD.to_owned(),
                    value: FieldValue::String(keyword.to_owned()),
                    version: None,
                },
                IndexItem {
                    external_id: external_id.to_owned(),
                    field: TEXT_FIELD.to_owned(),
                    value: FieldValue::String(body.to_owned()),
                    version: None,
                },
                IndexItem {
                    external_id: external_id.to_owned(),
                    field: FLAT_FIELD.to_owned(),
                    value: FieldValue::Vector(flat.into()),
                    version: None,
                },
                IndexItem {
                    external_id: external_id.to_owned(),
                    field: HNSW_FIELD.to_owned(),
                    value: FieldValue::Vector(hnsw.into()),
                    version: None,
                },
            ],
            request_id: None,
        },
    }
}

fn apply(sm: &EngineSm, index: u64, entry: RaftLogEntry) {
    let command = WalRecord::new(entry)
        .encode()
        .expect("encode a valid Raft WAL record");
    sm.apply(index, &command)
        .expect("apply a valid Raft WAL record");
}

fn request(query: QueryNode) -> SearchRequest {
    SearchRequest {
        query,
        limit: 10,
        offset: 0,
        cursor: None,
        routing_key: None,
        sort: None,
        track_total: true,
        collapse: None,
    }
}

fn result_ids(engine: &Engine, query: QueryNode) -> Vec<String> {
    engine
        .search(COLLECTION, request(query))
        .expect("query the restored backend")
        .hits
        .into_iter()
        .map(|hit| hit.external_id)
        .collect()
}

fn term(value: &str) -> QueryNode {
    QueryNode::Term(TermQuery {
        field: KEYWORD_FIELD.to_owned(),
        value: FieldValue::String(value.to_owned()),
    })
}

fn text(text: &str) -> QueryNode {
    QueryNode::Match(MatchQuery {
        field: TEXT_FIELD.to_owned(),
        text: text.to_owned(),
        op: MatchOp::And,
    })
}

fn knn(field: &str, vector: [f32; 2]) -> QueryNode {
    QueryNode::Knn(KnnQuery {
        field: field.to_owned(),
        vector: vector.into(),
        k: 1,
    })
}

fn assert_pre_capture_surface(engine: &Engine) {
    assert_eq!(
        result_ids(engine, term("before")),
        vec!["before"],
        "the archive must retain the keyword result at the captured Raft prefix"
    );
    assert_eq!(
        result_ids(engine, text("orchid")),
        vec!["before"],
        "the archive must retain the text result at the captured Raft prefix"
    );
    assert_eq!(
        result_ids(engine, knn(FLAT_FIELD, [1.0, 1.0])),
        vec!["before"],
        "the archive must retain the Flat vector result at the captured Raft prefix"
    );
    assert_eq!(
        result_ids(engine, knn(HNSW_FIELD, [2.0, 2.0])),
        vec!["before"],
        "the archive must retain the HNSW vector result at the captured Raft prefix"
    );
}

fn assert_post_capture_record_is_absent(engine: &Engine) {
    assert_eq!(
        result_ids(engine, term("after")),
        Vec::<String>::new(),
        "a record applied after capture_at must not enter the archived keyword state"
    );
    assert_eq!(
        result_ids(engine, text("newer")),
        Vec::<String>::new(),
        "a record applied after capture_at must not enter the archived text state"
    );
    assert_eq!(
        result_ids(engine, knn(FLAT_FIELD, [9.0, 9.0])),
        vec!["before"],
        "the Flat result must be the captured record, not the post-capture vector"
    );
    assert_eq!(
        result_ids(engine, knn(HNSW_FIELD, [8.0, 8.0])),
        vec!["before"],
        "the HNSW result must be the captured record, not the post-capture vector"
    );
}

fn assert_safe_receiver_surface(engine: &Engine, case: &str) {
    assert_eq!(
        result_ids(engine, term("safe")),
        vec!["safe"],
        "{case}: refusal must retain the receiver keyword result"
    );
    assert_eq!(
        result_ids(engine, text("safe")),
        vec!["safe"],
        "{case}: refusal must retain the receiver text result"
    );
    assert_eq!(
        result_ids(engine, knn(FLAT_FIELD, [3.0, 3.0])),
        vec!["safe"],
        "{case}: refusal must retain the receiver Flat vector result"
    );
    assert_eq!(
        result_ids(engine, knn(HNSW_FIELD, [4.0, 4.0])),
        vec!["safe"],
        "{case}: refusal must retain the receiver HNSW vector result"
    );
}

fn assert_refused_archive_preserves_receiver(
    receiver: &EngineSm,
    receiver_engine: &Engine,
    current_path: &std::path::Path,
    current_before: &[u8],
    applied_before: u64,
    bytes: &[u8],
    case: &str,
) {
    let mut bytes_for_validation = bytes;
    let validation = receiver.validate_snapshot(&mut bytes_for_validation);
    let mut bytes_for_restore = bytes;
    let restoration = receiver.restore(&mut bytes_for_restore);

    assert!(
        validation.is_err(),
        "{case}: malformed archive must fail validation before installation"
    );
    assert!(
        restoration.is_err(),
        "{case}: malformed archive must be refused by restore"
    );
    assert_eq!(
        receiver.applied_index(),
        applied_before,
        "{case}: refusal must retain the receiver Raft watermark"
    );
    assert_safe_receiver_surface(receiver_engine, case);
    let current_after = std::fs::read(current_path);
    assert!(
        current_after.is_ok(),
        "{case}: refusal must leave CURRENT readable"
    );
    assert_eq!(
        current_after.expect("asserted CURRENT remains readable"),
        current_before.to_vec(),
        "{case}: refusal must leave CURRENT byte-identical"
    );
}

fn assert_published_current_matches_capture(store: &SegmentRdbStore) {
    let current = store
        .load_current_generation()
        .expect("load receiver CURRENT after archive restore")
        .expect("archive restore must publish receiver CURRENT");
    assert_eq!(
        current.sequence, SNAPSHOT_INDEX,
        "receiver CURRENT must name the archive's exact Raft cut"
    );
    assert_pre_capture_surface(&current.engine);
    assert_post_capture_record_is_absent(&current.engine);
}

fn segment_sm(root: &std::path::Path) -> (Arc<Engine>, Arc<EngineSm>, Arc<SegmentRdbStore>) {
    let engine = Arc::new(Engine::new());
    let store = Arc::new(SegmentRdbStore::new(root).expect("open segment store"));
    let sm = EngineSm::new_with_segment_store(engine.clone(), 0, store.clone());
    (engine, sm, store)
}

fn archive_at(sm: &EngineSm, index: u64) -> Vec<u8> {
    let preparation = sm
        .preflight_snapshot()
        .expect("preflight a segment Raft snapshot")
        .expect("segment mode must prepare an archive snapshot");
    let prepared = preparation
        .capture_at(index)
        .expect("capture one exact applied Raft prefix");
    let mut archive = Vec::new();
    prepared
        .write_to(&mut archive)
        .expect("write a self-contained segment Raft archive");
    archive
}

fn legacy_rdb_bytes() -> Vec<u8> {
    let engine = Arc::new(Engine::new());
    let sm = EngineSm::new(engine, 0);
    apply(&sm, 1, create_collection());
    apply(
        &sm,
        SNAPSHOT_INDEX,
        index_document("before", "before", "orchid archive", [1.0, 1.0], [2.0, 2.0]),
    );
    let mut bytes = Vec::new();
    sm.snapshot(&mut bytes)
        .expect("write the legacy RDB snapshot bytes");
    bytes
}

/// Blocks the first archive output write. The test always sends `release` and
/// joins the archive thread before it asserts any observed result.
struct BlockingArchiveWriter {
    bytes: Vec<u8>,
    started: mpsc::SyncSender<()>,
    release: mpsc::Receiver<()>,
    blocked_once: bool,
}

impl Write for BlockingArchiveWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if !self.blocked_once {
            self.blocked_once = true;
            self.started
                .send(())
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "test lost writer start"))?;
            self.release
                .recv()
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "test released writer"))?;
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn segment_raft_archive_restores_an_exact_cut_and_legacy_rdb() {
    let source_dir = TempDir::new().expect("source temporary directory");
    let source_root = source_dir.path().join("source-segments");
    let (source_engine, source, source_store) = segment_sm(&source_root);
    apply(&source, 1, create_collection());
    apply(
        &source,
        SNAPSHOT_INDEX,
        index_document("before", "before", "orchid archive", [1.0, 1.0], [2.0, 2.0]),
    );

    let preparation = source
        .preflight_snapshot()
        .expect("preflight the segment archive")
        .expect("segment mode must use the archive snapshot path");
    let prepared = preparation
        .capture_at(SNAPSHOT_INDEX)
        .expect("capture the requested exact Raft prefix");
    apply(
        &source,
        POST_CAPTURE_INDEX,
        index_document("after", "after", "newer document", [9.0, 9.0], [8.0, 8.0]),
    );
    let mut archive = Vec::new();
    prepared
        .write_to(&mut archive)
        .expect("write the archive after the short capture");

    assert!(
        archive.starts_with(ARCHIVE_MAGIC),
        "a segment Raft snapshot must use the public LSEGRAFT archive magic"
    );

    drop(source);
    drop(source_engine);
    drop(source_store);
    std::fs::remove_dir_all(&source_root).expect("remove the sender segment root");

    let receiver_dir = TempDir::new().expect("receiver temporary directory");
    let (receiver_engine, receiver, receiver_store) =
        segment_sm(&receiver_dir.path().join("receiver-segments"));
    let mut archive_for_validation = archive.as_slice();
    assert!(
        receiver
            .validate_snapshot(&mut archive_for_validation)
            .is_ok(),
        "a fresh segment receiver must validate the self-contained archive"
    );
    let mut archive_for_restore = archive.as_slice();
    receiver
        .restore(&mut archive_for_restore)
        .expect("restore the self-contained archive into a fresh receiver");

    assert_eq!(
        receiver.applied_index(),
        SNAPSHOT_INDEX,
        "restore must retain the archive's exact applied Raft index"
    );
    assert_pre_capture_surface(&receiver_engine);
    assert_post_capture_record_is_absent(&receiver_engine);
    assert_published_current_matches_capture(&receiver_store);

    let legacy = legacy_rdb_bytes();
    assert!(
        !legacy.starts_with(ARCHIVE_MAGIC),
        "the compatibility fixture must use the old RDB byte format"
    );
    let legacy_receiver_dir = TempDir::new().expect("legacy receiver temporary directory");
    let (legacy_engine, legacy_receiver, _legacy_receiver_store) =
        segment_sm(&legacy_receiver_dir.path().join("legacy-receiver-segments"));
    let mut legacy_for_validation = legacy.as_slice();
    assert!(
        legacy_receiver
            .validate_snapshot(&mut legacy_for_validation)
            .is_ok(),
        "a segment receiver must validate a legacy RDB snapshot"
    );
    let mut legacy_for_restore = legacy.as_slice();
    legacy_receiver
        .restore(&mut legacy_for_restore)
        .expect("restore legacy RDB bytes unchanged");
    assert_eq!(
        legacy_receiver.applied_index(),
        SNAPSHOT_INDEX,
        "legacy RDB restore must retain its applied Raft index"
    );
    assert_pre_capture_surface(&legacy_engine);
}

#[test]
fn segment_raft_archive_refuses_corrupt_input_without_mutating_receiver_state() {
    let source_dir = TempDir::new().expect("source temporary directory");
    let (_source_engine, source, _source_store) =
        segment_sm(&source_dir.path().join("source-segments"));
    apply(&source, 1, create_collection());
    apply(
        &source,
        SNAPSHOT_INDEX,
        index_document("before", "before", "orchid archive", [1.0, 1.0], [2.0, 2.0]),
    );
    let archive = archive_at(&source, SNAPSHOT_INDEX);
    assert!(
        archive.starts_with(ARCHIVE_MAGIC),
        "the corruption fixture must start from a documented LSEGRAFT archive"
    );
    let mut corrupt = archive.clone();
    let first = corrupt
        .first_mut()
        .expect("a written archive includes its documented magic");
    *first ^= 0xff;

    let receiver_dir = TempDir::new().expect("receiver temporary directory");
    let receiver_root = receiver_dir.path().join("receiver-segments");
    let (receiver_engine, receiver, receiver_store) = segment_sm(&receiver_root);
    apply(&receiver, 1, create_collection());
    apply(
        &receiver,
        SNAPSHOT_INDEX,
        index_document("safe", "safe", "safe state", [3.0, 3.0], [4.0, 4.0]),
    );
    receiver_store
        .save_required(&receiver_engine, SNAPSHOT_INDEX)
        .expect("save the receiver safe state as CURRENT before malformed restores");
    let current_path = receiver_root.join("CURRENT");
    let current_before = std::fs::read(&current_path).expect("read receiver CURRENT before faults");
    let applied_before = receiver.applied_index();
    assert_safe_receiver_surface(&receiver_engine, "fixture");

    let mut payload_flip = archive.clone();
    let last = payload_flip
        .last_mut()
        .expect("a documented LSEGRAFT archive has a payload byte");
    *last ^= 0xff;
    let truncated = archive[..archive.len() - 1].to_vec();
    let mut trailing = archive.clone();
    trailing.push(0xff);

    for (case, malformed) in [
        ("magic flip", corrupt),
        ("last payload-byte flip", payload_flip),
        ("truncation", truncated),
        ("trailing byte", trailing),
    ] {
        assert_refused_archive_preserves_receiver(
            &receiver,
            &receiver_engine,
            &current_path,
            &current_before,
            applied_before,
            &malformed,
            case,
        );
    }
}

#[test]
fn segment_raft_archive_keeps_the_exact_cut_while_export_is_blocked() {
    let source_dir = TempDir::new().expect("source temporary directory");
    let (source_engine, source, source_store) =
        segment_sm(&source_dir.path().join("source-segments"));
    apply(&source, 1, create_collection());
    apply(
        &source,
        SNAPSHOT_INDEX,
        index_document("before", "before", "orchid archive", [1.0, 1.0], [2.0, 2.0]),
    );
    let preparation = source
        .preflight_snapshot()
        .expect("preflight the segment archive")
        .expect("segment mode must prepare an archive snapshot");
    let prepared = preparation
        .capture_at(SNAPSHOT_INDEX)
        .expect("capture the exact prefix before output begins");

    let (started_tx, started_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let archive_thread = thread::spawn(move || {
        let mut writer = BlockingArchiveWriter {
            bytes: Vec::new(),
            started: started_tx,
            release: release_rx,
            blocked_once: false,
        };
        let written = prepared.write_to(&mut writer);
        (written, writer.bytes)
    });

    let writer_started = started_rx.recv_timeout(SNAPSHOT_OUTPUT_WATCHDOG);
    let (apply_done_tx, apply_done_rx) = mpsc::sync_channel(1);
    let apply_thread = if writer_started.is_ok() {
        let source = source.clone();
        Some(thread::spawn(move || {
            let command = WalRecord::new(index_document(
                "after",
                "after",
                "newer document",
                [9.0, 9.0],
                [8.0, 8.0],
            ))
            .encode()
            .expect("encode post-capture record");
            let result = source.apply(POST_CAPTURE_INDEX, &command);
            let _ = apply_done_tx.send(());
            result
        }))
    } else {
        None
    };
    let apply_finished_before_release = if apply_thread.is_some() {
        apply_done_rx.recv_timeout(SNAPSHOT_OUTPUT_WATCHDOG)
    } else {
        Err(mpsc::RecvTimeoutError::Disconnected)
    };

    let mut checkpoint_thread = None;
    let checkpoint_published_before_release = if apply_finished_before_release.is_ok() {
        let (checkpoint_done_tx, checkpoint_done_rx) = mpsc::sync_channel(1);
        let store = source_store.clone();
        let engine = source_engine.clone();
        checkpoint_thread = Some(thread::spawn(move || {
            let result = store.save_required(&engine, POST_CAPTURE_INDEX);
            let _ = checkpoint_done_tx.send(result.is_ok());
            result
        }));
        matches!(
            checkpoint_done_rx.recv_timeout(SNAPSHOT_OUTPUT_WATCHDOG),
            Ok(true)
        )
    } else {
        false
    };

    let mut prune_thread = None;
    let prune_finished_before_release = if checkpoint_published_before_release {
        let (prune_done_tx, prune_done_rx) = mpsc::sync_channel(1);
        let store = source_store.clone();
        prune_thread = Some(thread::spawn(move || {
            let result = store.prune(0);
            let _ = prune_done_tx.send(result.is_ok());
            result
        }));
        matches!(
            prune_done_rx.recv_timeout(SNAPSHOT_OUTPUT_WATCHDOG),
            Ok(true)
        )
    } else {
        false
    };

    // Cleanup comes before every assertion. Releasing the writer drops any
    // faulty capture lease, and every join keeps this test from leaking a gate.
    let _ = release_tx.send(());
    let archive_join = archive_thread.join();
    let apply_join = apply_thread.map(|worker| worker.join());
    let checkpoint_join = checkpoint_thread.map(|worker| worker.join());
    let prune_join = prune_thread.map(|worker| worker.join());

    assert!(
        writer_started.is_ok(),
        "the prepared archive must reach an external writer"
    );
    assert!(
        apply_finished_before_release.is_ok(),
        "Raft apply must complete while archive output is still blocked"
    );
    assert!(
        checkpoint_published_before_release,
        "a newer segment checkpoint must publish while old archive output is blocked"
    );
    assert!(
        prune_finished_before_release,
        "prune(0) must return while old archive output is blocked"
    );
    assert!(archive_join.is_ok(), "archive output thread must not panic");
    assert!(
        matches!(apply_join, Some(Ok(Ok(())))),
        "the post-capture Raft record must apply successfully"
    );
    assert!(
        matches!(checkpoint_join, Some(Ok(Ok(_)))),
        "the newer segment checkpoint must finish successfully"
    );
    assert!(
        matches!(prune_join, Some(Ok(Ok(_)))),
        "prune(0) must finish successfully"
    );

    let (archive_result, archive) = archive_join.expect("asserted archive output thread");
    assert!(
        archive_result.is_ok(),
        "releasing the external writer must let archive output finish"
    );
    assert!(
        archive.starts_with(ARCHIVE_MAGIC),
        "the externally written archive must keep the LSEGRAFT magic"
    );

    let receiver_dir = TempDir::new().expect("receiver temporary directory");
    let (receiver_engine, receiver, receiver_store) =
        segment_sm(&receiver_dir.path().join("receiver-segments"));
    let mut archive_for_validation = archive.as_slice();
    assert!(
        receiver
            .validate_snapshot(&mut archive_for_validation)
            .is_ok(),
        "the archive must stay valid after CURRENT advances and prune returns"
    );
    let mut archive_for_restore = archive.as_slice();
    receiver
        .restore(&mut archive_for_restore)
        .expect("restore the archive pinned through the blocked export");
    assert_eq!(
        receiver.applied_index(),
        SNAPSHOT_INDEX,
        "the blocked export must still restore its original Raft cut"
    );
    assert_pre_capture_surface(&receiver_engine);
    assert_post_capture_record_is_absent(&receiver_engine);
    assert_published_current_matches_capture(&receiver_store);
}

#[test]
fn segment_raft_snapshot_entry_point_emits_archive_and_publishes_current() {
    let source_dir = TempDir::new().expect("source temporary directory");
    let (source_engine, source, source_store) =
        segment_sm(&source_dir.path().join("source-segments"));
    apply(&source, 1, create_collection());
    apply(
        &source,
        SNAPSHOT_INDEX,
        index_document("before", "before", "orchid archive", [1.0, 1.0], [2.0, 2.0]),
    );

    let mut archive = Vec::new();
    source
        .snapshot(&mut archive)
        .expect("write a segment Raft snapshot through the compatibility entry point");
    assert!(
        archive.starts_with(ARCHIVE_MAGIC),
        "segment-mode snapshot() must emit the public LSEGRAFT archive"
    );

    drop(source);
    drop(source_engine);
    drop(source_store);

    let receiver_dir = TempDir::new().expect("receiver temporary directory");
    let (receiver_engine, receiver, receiver_store) =
        segment_sm(&receiver_dir.path().join("receiver-segments"));
    let mut archive_for_validation = archive.as_slice();
    assert!(
        receiver
            .validate_snapshot(&mut archive_for_validation)
            .is_ok(),
        "the compatibility entry-point archive must validate"
    );
    let mut archive_for_restore = archive.as_slice();
    receiver
        .restore(&mut archive_for_restore)
        .expect("restore the compatibility entry-point archive");
    assert_eq!(
        receiver.applied_index(),
        SNAPSHOT_INDEX,
        "the compatibility entry-point archive must retain its exact Raft cut"
    );
    assert_pre_capture_surface(&receiver_engine);
    assert_post_capture_record_is_absent(&receiver_engine);
    assert_published_current_matches_capture(&receiver_store);
}

// Append to apps/lumen/e2e/raft_segment_snapshot_archive.rs.
//
// This case uses the helpers and imports already defined in that contract.

#[test]
fn segment_raft_legacy_rdb_startup_restore_replaces_an_ahead_current() {
    let legacy = legacy_rdb_bytes();
    assert!(
        !legacy.starts_with(ARCHIVE_MAGIC),
        "the startup fixture must keep the old RDB snapshot format"
    );

    let receiver_dir = TempDir::new().expect("receiver temporary directory");
    let receiver_root = receiver_dir.path().join("receiver-segments");

    // This is an independent capacity-relief checkpoint, not state restored
    // into the fresh Raft engine below. Drop all its owners before startup.
    {
        let (checkpoint_engine, checkpoint, checkpoint_store) = segment_sm(&receiver_root);
        apply(&checkpoint, 1, create_collection());
        apply(
            &checkpoint,
            SNAPSHOT_INDEX,
            index_document("before", "before", "orchid archive", [1.0, 1.0], [2.0, 2.0]),
        );
        apply(
            &checkpoint,
            POST_CAPTURE_INDEX,
            index_document("after", "after", "newer document", [9.0, 9.0], [8.0, 8.0]),
        );
        checkpoint_store
            .save_required(&checkpoint_engine, POST_CAPTURE_INDEX)
            .expect("publish the independent CURRENT at index 3");
        let current = checkpoint_store
            .load_current_generation()
            .expect("load the independent CURRENT")
            .expect("the independent checkpoint must publish CURRENT");
        assert_eq!(
            current.sequence, POST_CAPTURE_INDEX,
            "the independent checkpoint must leave CURRENT ahead of the legacy snapshot"
        );
        assert_eq!(
            result_ids(&current.engine, term("after")),
            vec!["after"],
            "the independent CURRENT fixture must contain its index-3 keyword record"
        );
    }

    let (receiver_engine, receiver, receiver_store) = segment_sm(&receiver_root);
    assert_eq!(
        receiver.applied_index(),
        0,
        "the startup receiver must begin before the legacy Raft snapshot"
    );

    let mut legacy_for_validation = legacy.as_slice();
    assert!(
        receiver
            .validate_snapshot(&mut legacy_for_validation)
            .is_ok(),
        "the unchanged legacy RDB snapshot must validate with an ahead local CURRENT"
    );
    let mut legacy_for_restore = legacy.as_slice();
    receiver
        .restore(&mut legacy_for_restore)
        .expect("startup must restore legacy RDB index 2 before replaying index 3");

    assert_eq!(
        receiver.applied_index(),
        SNAPSHOT_INDEX,
        "legacy startup restore must install the snapshot's Raft index"
    );
    assert_pre_capture_surface(&receiver_engine);
    assert_post_capture_record_is_absent(&receiver_engine);

    let cold_current = receiver_store
        .load_current_generation()
        .expect("cold-open CURRENT after legacy startup restore")
        .expect("legacy startup restore must publish replacement CURRENT");
    assert_eq!(
        cold_current.sequence, SNAPSHOT_INDEX,
        "cold CURRENT must replace the independent index-3 checkpoint with snapshot index 2"
    );
    assert_pre_capture_surface(&cold_current.engine);
    assert_post_capture_record_is_absent(&cold_current.engine);

    apply(
        &receiver,
        POST_CAPTURE_INDEX,
        index_document("after", "after", "newer document", [9.0, 9.0], [8.0, 8.0]),
    );
    assert_eq!(
        receiver.applied_index(),
        POST_CAPTURE_INDEX,
        "the valid next Raft record must apply after legacy startup restore"
    );
    assert_eq!(
        result_ids(&receiver_engine, term("after")),
        vec!["after"],
        "replay must restore the index-3 keyword result"
    );
    assert_eq!(
        result_ids(&receiver_engine, text("newer")),
        vec!["after"],
        "replay must restore the index-3 text result"
    );
    assert_eq!(
        result_ids(&receiver_engine, knn(FLAT_FIELD, [9.0, 9.0])),
        vec!["after"],
        "replay must restore the index-3 Flat vector result"
    );
    assert_eq!(
        result_ids(&receiver_engine, knn(HNSW_FIELD, [8.0, 8.0])),
        vec!["after"],
        "replay must restore the index-3 HNSW vector result"
    );
}
