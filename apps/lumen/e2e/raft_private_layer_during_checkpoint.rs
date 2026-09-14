//! Black-box contract for private scalar layers that arrive during checkpoint
//! file I/O.
//!
//! A valid committed fast Index has 1,000 Keyword values of 270 KiB. Its
//! encoded LWAL bytes exceed the documented 256 MiB boundary. The case pauses
//! a real `SyncFile` before `CURRENT`, then applies a newer oversized value, a
//! normal scalar update, and a full-document tombstone through `EngineSm`.
//!
//! # Facets
//!
//! - Behavior: assertions at `raft_private_layer_during_checkpoint.rs:581-595`
//!   require both an initial sparse cut and a reused catalog cut to accept the
//!   post-cut records and publish after release. Helper assertions at
//!   `:380-412` retain the newer private value, ordinary overlay, and tombstone
//!   live. Assertions at `:602-608` require the first cold `CURRENT` to remain
//!   the captured cut, and `:625-634` requires the next cold `CURRENT` to
//!   retain the later state and final Raft watermark. The changed seam is
//!   `apps/lumen/src/storage.rs:13944-14004` after a staged checkpoint from
//!   `apps/lumen/src/segment_rdb.rs:643-824`.
//! - Security: this change accepts no new caller, peer, or disk format. The
//!   committed-LWAL byte boundary already has a closed malformed-input case in
//!   `apps/lumen/e2e/raft_oversized_committed_apply.rs:459-480`, which feeds a
//!   truncated record to `EngineSm::apply` and requires no mutation or watermark
//!   advance. The process-written checkpoint reader boundary remains validated
//!   before cold open at `apps/lumen/src/segment_rdb.rs:1910-2070`.
//! - Performance: `apps/lumen/docs/indexing.md:264-272` says, verbatim,
//!   "Pending active, frozen, and reserved changes have a 256 MiB budget."
//!   This case asserts its one reused command really exceeds that boundary at
//!   `:229-237`. Existing
//!   `apps/lumen/e2e/raft_oversized_committed_apply.rs:238-245` measures the
//!   public pending gauges against that same limit. The pause timeouts only
//!   bound test cleanup; they are not a latency promise.
//!
//! Gate: `cargo test -p lumen --features raft-wal --test
//! raft_private_layer_during_checkpoint -- --nocapture`.

use std::collections::BTreeMap;
use std::io;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use lumen::log_entry::RaftLogEntry;
use lumen::raft_sm::EngineSm;
use lumen::segment_rdb::{MergeObserver, MergePhase, SegmentRdbStore};
use lumen::storage::Engine;
use lumen::types::{
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest, QueryNode,
    SearchRequest, TermQuery,
};
use lumen::wal::WalRecord;
use raft_runtime::{Index, RaftStateMachine};
use storage_durable::{CommitStep, FailureInjector, FailurePoint};

const COLLECTION: &str = "private-layer-during-checkpoint";
const FIELD: &str = "keyword";
const ITEM_COUNT: usize = 1_000;
const VALUE_BYTES: usize = 270 * 1024;
const PENDING_HARD_LIMIT_BYTES: usize = 256 * 1024 * 1024;
const QUERY_ORDINAL: usize = 0;
const DELETED_ORDINAL: usize = 1;
const ORDINARY_ORDINAL: usize = 2;
const CAPTURED_PHASE: u8 = b'A';
const NEWER_PHASE: u8 = b'B';
const VALUE_MARKER_PREFIX: &str = "lumen061-private-layer-phase=";
const SEED_ZERO: &str = "private-layer-empty-cut-seed-zero";
const SEED_DELETED: &str = "private-layer-empty-cut-seed-deleted";
const ORDINARY_VALUE: &str = "private-layer-ordinary-overlay-after-cut";
const READY_TIMEOUT: Duration = Duration::from_secs(60);
const CHECKPOINT_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Clone, Copy, Debug)]
enum Setup {
    EmptyAtCapture,
    ReusedCatalogAtCapture,
}

impl Setup {
    fn name(self) -> &'static str {
        match self {
            Self::EmptyAtCapture => "empty-at-capture",
            Self::ReusedCatalogAtCapture => "reused-catalog-at-capture",
        }
    }
}

#[derive(Default)]
struct NoopMergeObserver;

impl MergeObserver for NoopMergeObserver {
    fn observe(&self, _: MergePhase) -> io::Result<()> {
        Ok(())
    }
}

/// Pause one real generation-file sync after the checkpoint capture has
/// detached. `SegmentRdbStore::save` runs on a dedicated thread below, so this
/// receiver never holds the test thread or an apply lease.
#[derive(Default)]
struct HoldNextSyncFile {
    hold: Mutex<Option<(mpsc::SyncSender<()>, mpsc::Receiver<()>)>>,
}

impl HoldNextSyncFile {
    fn arm(&self, entered: mpsc::SyncSender<()>, release: mpsc::Receiver<()>) {
        assert!(
            self.hold
                .lock()
                .expect("private-layer SyncFile hold mutex")
                .replace((entered, release))
                .is_none(),
            "private-layer fixture arms exactly one checkpoint SyncFile hold",
        );
    }
}

impl FailureInjector for HoldNextSyncFile {
    fn check(&self, point: &FailurePoint) -> io::Result<()> {
        if point.step != CommitStep::SyncFile {
            return Ok(());
        }
        let Some((entered, release)) = self
            .hold
            .lock()
            .expect("private-layer SyncFile hold mutex")
            .take()
        else {
            return Ok(());
        };
        entered.send(()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::BrokenPipe,
                "private-layer checkpoint readiness receiver dropped",
            )
        })?;
        release.recv().map_err(|_| {
            io::Error::new(
                io::ErrorKind::BrokenPipe,
                "private-layer checkpoint release sender dropped",
            )
        })?;
        Ok(())
    }
}

/// Releases the checkpoint even if a later behavior assertion unwinds.
struct SyncRelease(Option<mpsc::SyncSender<()>>);

impl SyncRelease {
    fn release(&mut self) {
        if let Some(sender) = self.0.take() {
            let _ = sender.send(());
        }
    }
}

impl Drop for SyncRelease {
    fn drop(&mut self) {
        self.release();
    }
}

fn keyword_schema() -> CreateCollectionRequest {
    CreateCollectionRequest {
        fields: BTreeMap::from([(
            FIELD.to_owned(),
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
    }
}

fn external_id(ordinal: usize) -> String {
    format!("private-layer-{ordinal:04}")
}

fn dictionary_marker(ordinal: usize, phase: u8) -> String {
    format!(
        "{VALUE_MARKER_PREFIX}{};ordinal={ordinal:04};",
        phase as char
    )
}

fn oversized_value(ordinal: usize, phase: u8) -> String {
    let marker = dictionary_marker(ordinal, phase);
    let suffix = format!(";end-lumen061-private-layer-{ordinal:04}");
    assert!(
        marker.len() + suffix.len() < VALUE_BYTES,
        "fixture markers must leave a full 270 KiB Keyword value",
    );
    let mut bytes = vec![b'x'; VALUE_BYTES];
    bytes[..marker.len()].copy_from_slice(marker.as_bytes());
    bytes[VALUE_BYTES - suffix.len()..].copy_from_slice(suffix.as_bytes());
    String::from_utf8(bytes).expect("private-layer fixture values are ASCII")
}

fn encoded_index(items: Vec<IndexItem>) -> Vec<u8> {
    WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items,
            request_id: None,
        },
    })
    .encode()
    .expect("encode valid committed fast Index")
}

/// Build once per setup. The second oversized apply mutates one query-point
/// byte in this already-retained command instead of cloning another 256+ MiB
/// buffer. Fast LWAL records have no checksum, and the marker length is fixed.
fn oversized_fast_index_command() -> Vec<u8> {
    let mut items = Vec::with_capacity(ITEM_COUNT);
    for ordinal in 0..ITEM_COUNT {
        items.push(IndexItem {
            external_id: external_id(ordinal),
            field: FIELD.to_owned(),
            value: FieldValue::String(oversized_value(ordinal, CAPTURED_PHASE)),
            version: None,
        });
    }
    let command = encoded_index(items);
    assert!(
        command.starts_with(b"LWAL"),
        "fixture must use the real committed fast Index wire format",
    );
    assert!(
        command.len() > PENDING_HARD_LIMIT_BYTES,
        "fixture must exceed the documented 256 MiB pending-change boundary: command_bytes={}",
        command.len(),
    );
    command
}

fn mutate_query_point_phase(command: &mut [u8], from: u8, to: u8) {
    let marker = dictionary_marker(QUERY_ORDINAL, from);
    let start = command
        .windows(marker.len())
        .position(|window| window == marker.as_bytes())
        .expect("the one retained oversized command contains its query-point marker");
    let phase = start + VALUE_MARKER_PREFIX.len();
    assert_eq!(
        command[phase], from,
        "the retained command mutation targets only its fixed-width phase byte",
    );
    command[phase] = to;
}

fn seed_command() -> Vec<u8> {
    encoded_index(vec![
        IndexItem {
            external_id: external_id(QUERY_ORDINAL),
            field: FIELD.to_owned(),
            value: FieldValue::String(SEED_ZERO.to_owned()),
            version: None,
        },
        IndexItem {
            external_id: external_id(DELETED_ORDINAL),
            field: FIELD.to_owned(),
            value: FieldValue::String(SEED_DELETED.to_owned()),
            version: None,
        },
    ])
}

fn ordinary_overlay_command() -> Vec<u8> {
    encoded_index(vec![IndexItem {
        external_id: external_id(ORDINARY_ORDINAL),
        field: FIELD.to_owned(),
        value: FieldValue::String(ORDINARY_VALUE.to_owned()),
        version: None,
    }])
}

fn delete_command() -> Vec<u8> {
    WalRecord::new(RaftLogEntry::Delete {
        collection_id: COLLECTION.to_owned(),
        external_id: external_id(DELETED_ORDINAL),
        field: None,
    })
    .encode()
    .expect("encode committed full-document tombstone")
}

fn create_command() -> Vec<u8> {
    WalRecord::new(RaftLogEntry::CreateCollection {
        collection_id: COLLECTION.to_owned(),
        req: keyword_schema(),
    })
    .encode()
    .expect("encode committed collection schema")
}

fn search_ids(engine: &Engine, value: &str) -> Vec<String> {
    let mut ids: Vec<_> = engine
        .search(
            COLLECTION,
            SearchRequest {
                query: QueryNode::Term(TermQuery {
                    field: FIELD.to_owned(),
                    value: FieldValue::String(value.to_owned()),
                }),
                limit: 4,
                offset: 0,
                cursor: None,
                routing_key: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .expect("exact private-layer Keyword query")
        .hits
        .into_iter()
        .map(|hit| hit.external_id)
        .collect();
    ids.sort();
    ids
}

fn assert_term(engine: &Engine, value: &str, expected: &[String], context: &str) {
    let mut expected = expected.to_vec();
    expected.sort();
    assert_eq!(
        search_ids(engine, value),
        expected,
        "{context}: exact Keyword query must return the selected IDs",
    );
}

fn assert_committed_apply(state_machine: &EngineSm, index: Index, command: &[u8], context: &str) {
    let result = state_machine
        .apply(index, command)
        .map_err(|error| format!("{error:#}"));
    let error = result
        .as_ref()
        .err()
        .map(String::as_str)
        .unwrap_or("no refusal text");
    assert!(
        result.is_ok(),
        "{context}: valid committed record must apply; refusal: {error}",
    );
    assert_eq!(
        state_machine.applied_index(),
        index,
        "{context}: successful committed record must advance its Raft watermark",
    );
}

fn apply_during_pause(
    state_machine: &EngineSm,
    capture_index: Index,
    oversized: &[u8],
) -> Result<(), String> {
    let ordinary = ordinary_overlay_command();
    let tombstone = delete_command();
    for (index, command, label) in [
        (capture_index + 1, oversized, "newer oversized scalar layer"),
        (
            capture_index + 2,
            ordinary.as_slice(),
            "ordinary scalar overlay",
        ),
        (
            capture_index + 3,
            tombstone.as_slice(),
            "full-document tombstone",
        ),
    ] {
        state_machine
            .apply(index, command)
            .map_err(|error| format!("{label} at Raft index {index} was refused: {error:#}"))?;
        state_machine.take_outcome(index).map_err(|error| {
            format!("{label} at Raft index {index} had a failed business outcome: {error:#}")
        })?;
        if state_machine.applied_index() != index {
            return Err(format!(
                "{label} at Raft index {index} returned success without advancing the watermark"
            ));
        }
    }
    Ok(())
}

fn assert_post_cut_state(engine: &Engine, context: &str) {
    assert_term(
        engine,
        &oversized_value(QUERY_ORDINAL, NEWER_PHASE),
        &[external_id(QUERY_ORDINAL)],
        context,
    );
    assert_term(
        engine,
        &oversized_value(QUERY_ORDINAL, CAPTURED_PHASE),
        &[],
        context,
    );
    assert_term(
        engine,
        ORDINARY_VALUE,
        &[external_id(ORDINARY_ORDINAL)],
        context,
    );
    assert_term(
        engine,
        &oversized_value(DELETED_ORDINAL, CAPTURED_PHASE),
        &[],
        context,
    );
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("private-layer collection stats")
            .documents_indexed,
        (ITEM_COUNT - 1) as u64,
        "{context}: the post-cut tombstone removes exactly one large-record document",
    );
}

fn assert_first_cold_cut(engine: &Engine, setup: Setup) {
    match setup {
        Setup::EmptyAtCapture => {
            assert_term(
                engine,
                SEED_ZERO,
                &[external_id(QUERY_ORDINAL)],
                "empty capture cold CURRENT",
            );
            assert_term(
                engine,
                SEED_DELETED,
                &[external_id(DELETED_ORDINAL)],
                "empty capture cold CURRENT",
            );
            assert_term(
                engine,
                &oversized_value(QUERY_ORDINAL, NEWER_PHASE),
                &[],
                "empty capture cold CURRENT excludes the post-cut private layer",
            );
            assert_eq!(
                engine
                    .stats(COLLECTION)
                    .expect("empty capture cold collection stats")
                    .documents_indexed,
                2,
                "empty capture cold CURRENT contains only the pre-pause cut",
            );
        }
        Setup::ReusedCatalogAtCapture => {
            assert_term(
                engine,
                &oversized_value(QUERY_ORDINAL, CAPTURED_PHASE),
                &[external_id(QUERY_ORDINAL)],
                "reused capture cold CURRENT",
            );
            assert_term(
                engine,
                &oversized_value(QUERY_ORDINAL, NEWER_PHASE),
                &[],
                "reused capture cold CURRENT excludes the later private layer",
            );
            assert_term(
                engine,
                &oversized_value(DELETED_ORDINAL, CAPTURED_PHASE),
                &[external_id(DELETED_ORDINAL)],
                "reused capture cold CURRENT predates the later tombstone",
            );
            assert_eq!(
                engine
                    .stats(COLLECTION)
                    .expect("reused capture cold collection stats")
                    .documents_indexed,
                ITEM_COUNT as u64,
                "reused capture cold CURRENT retains the captured oversized document set",
            );
        }
    }
}

fn run_case(setup: Setup) {
    let dir = tempfile::tempdir().expect("private-layer fixture directory");
    let hold = Arc::new(HoldNextSyncFile::default());
    let store = Arc::new(
        SegmentRdbStore::with_failure_injector_and_merge_observer(
            dir.path().join("segments"),
            hold.clone(),
            Arc::new(NoopMergeObserver),
        )
        .expect("open observed private-layer segment store"),
    );
    let engine = Arc::new(Engine::new());
    let state_machine = EngineSm::new_with_segment_store(engine.clone(), 0, store.clone());

    assert_committed_apply(state_machine.as_ref(), 1, &create_command(), setup.name());
    assert_committed_apply(state_machine.as_ref(), 2, &seed_command(), setup.name());

    let mut oversized = oversized_fast_index_command();
    let capture_index = match setup {
        Setup::EmptyAtCapture => 2,
        Setup::ReusedCatalogAtCapture => {
            store
                .save(&engine, 2)
                .expect("publish reusable catalog base before oversized committed apply");
            assert_eq!(
                store
                    .load_current_generation()
                    .expect("load reusable base CURRENT")
                    .expect("reusable base generation")
                    .sequence,
                2,
                "reused setup begins from its durable catalog base",
            );
            assert_committed_apply(
                state_machine.as_ref(),
                3,
                &oversized,
                "reused setup captured oversized committed apply",
            );
            3
        }
    };

    // Only one fixed-width byte changes. This avoids a second giant command
    // allocation while making the post-cut winner observable at one query point.
    mutate_query_point_phase(&mut oversized, CAPTURED_PHASE, NEWER_PHASE);

    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    hold.arm(entered_tx, release_rx);
    let mut release = SyncRelease(Some(release_tx));
    let (save_done_tx, save_done_rx) = mpsc::sync_channel(1);
    let save_store = store.clone();
    let save_engine = engine.clone();
    let save_thread = thread::Builder::new()
        .name(format!("lumen-private-layer-save-{}", setup.name()))
        .spawn(move || {
            let _ = save_done_tx.send(save_store.save(&save_engine, capture_index));
        })
        .expect("start paused checkpoint thread");

    if let Err(error) = entered_rx.recv_timeout(READY_TIMEOUT) {
        release.release();
        let completed = save_done_rx.recv_timeout(CHECKPOINT_TIMEOUT);
        if completed.is_ok() {
            save_thread
                .join()
                .expect("checkpoint thread must not panic after readiness failure");
        }
        panic!(
            "{} checkpoint never reached the real SyncFile pause: {error}; completion={completed:?}",
            setup.name(),
        );
    }

    let (apply_done_tx, apply_done_rx) = mpsc::sync_channel(1);
    let apply_sm = state_machine.clone();
    let apply_thread = thread::Builder::new()
        .name(format!("lumen-private-layer-apply-{}", setup.name()))
        .spawn(move || {
            let result = apply_during_pause(apply_sm.as_ref(), capture_index, &oversized);
            let _ = apply_done_tx.send(result);
        })
        .expect("start post-cut committed apply worker");
    let during_pause = apply_done_rx.recv_timeout(READY_TIMEOUT);
    // Always release I/O, including when a regressed apply path waits on it.
    release.release();
    let during_pause = match during_pause {
        Ok(result) => {
            apply_thread
                .join()
                .expect("post-cut apply worker must not panic");
            result
        }
        Err(error) => {
            // A wait caused by checkpoint locking can finish after release.
            // Reap it, but retain the timeout as a contract failure.
            if apply_done_rx.recv_timeout(CHECKPOINT_TIMEOUT).is_ok() {
                apply_thread
                    .join()
                    .expect("timed-out apply worker must not panic");
            }
            Err(format!(
                "committed apply failed to finish while checkpoint I/O was paused: {error}"
            ))
        }
    };
    let first_save = match save_done_rx.recv_timeout(CHECKPOINT_TIMEOUT) {
        Ok(result) => {
            save_thread
                .join()
                .expect("paused checkpoint thread must not panic");
            result
        }
        Err(error) => panic!(
            "{} checkpoint did not finish after SyncFile release: {error}",
            setup.name(),
        ),
    };

    let apply_error = during_pause
        .as_ref()
        .err()
        .map(String::as_str)
        .unwrap_or("no refusal text");
    assert!(
        during_pause.is_ok(),
        "{}: valid committed post-cut scalar records must apply while checkpoint file I/O is paused; refusal: {apply_error}",
        setup.name(),
    );
    assert!(
        first_save.is_ok(),
        "{}: checkpoint must publish after SyncFile release; error: {}",
        setup.name(),
        first_save
            .as_ref()
            .err()
            .map(|error| format!("{error:#}"))
            .unwrap_or_default(),
    );

    assert_post_cut_state(
        &engine,
        &format!("{} live after first checkpoint", setup.name()),
    );
    let first = store
        .load_current_generation()
        .expect("cold-open first paused checkpoint CURRENT")
        .expect("first paused checkpoint publishes CURRENT");
    assert_eq!(
        first.sequence,
        capture_index,
        "{} first cold CURRENT must retain the pre-pause capture watermark",
        setup.name(),
    );
    assert_first_cold_cut(&first.engine, setup);

    let final_index = capture_index + 3;
    assert_eq!(
        state_machine.applied_index(),
        final_index,
        "{} live state machine retains every post-cut committed index",
        setup.name(),
    );
    store
        .save(&engine, final_index)
        .expect("second checkpoint persists post-cut private and ordinary state");
    assert_post_cut_state(
        &engine,
        &format!("{} live after second checkpoint", setup.name()),
    );
    let latest = store
        .load_current_generation()
        .expect("cold-open final private-layer CURRENT")
        .expect("second checkpoint publishes CURRENT");
    assert_eq!(
        latest.sequence,
        final_index,
        "{} final cold CURRENT must carry the final Raft watermark",
        setup.name(),
    );
    assert_post_cut_state(
        &latest.engine,
        &format!("{} final cold CURRENT", setup.name()),
    );
    // Retain the first cold view across a later save. It remains an immutable
    // observable cut and must not be changed by later private-layer publication.
    assert_first_cold_cut(&first.engine, setup);
}

#[test]
fn committed_private_scalar_layers_survive_a_paused_checkpoint_and_cold_reopen() {
    for setup in [Setup::EmptyAtCapture, Setup::ReusedCatalogAtCapture] {
        run_case(setup);
    }
}
