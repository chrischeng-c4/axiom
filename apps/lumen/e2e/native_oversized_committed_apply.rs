//! Black-box contract for a native committed oversized scalar Index.
//!
//! The fixture publishes a valid typed WalRecord straight to the real MemWal.
//! It does not call WriteCoordinator::submit and it does not stage the source.
//! The coordinator must own admission, source staging, apply, AOF persistence,
//! and the committed watermark after the foreign publisher already has a
//! sequence.
//!
//! The record has 1,000 distinct Keyword values of 270 KiB each. The fixture
//! owns those source strings only while it transfers the record to MemWal.
//! That source owner, and the byte-for-byte AOF copy kept before checkpoint
//! trimming, are outside the Lumen pending-change metrics below.
//! MemWal publish assigns and returns its sequence in one direct call.
//! Cancellation after that return needs a test-only delivery blocker, so this
//! contract instead asserts the reachable no-error commit acknowledgement.
//!
//! # Facets
//!
//! - Behavior: native_oversized_committed_apply.rs:325-342 requires the native
//!   publisher to receive a committed sequence without an error. :344-364 is
//!   the current behavior red: the coordinator must reach that committed
//!   sequence within the bounded cleanup watchdog. :250-287 and :436-504
//!   require every item, real AOF replay, an on-demand checkpoint, a real
//!   post-checkpoint AOF suffix, and cold reopen to retain the committed head.
//!   It covers apps/lumen/src/coordinator.rs:382-479,
//!   apps/lumen/src/aof.rs:266-387, and
//!   apps/lumen/src/segment_checkpoint.rs:220-268.
//! - Security: the new foreign delivery at apps/lumen/src/coordinator.rs:382-475
//!   receives a typed WalDelivery from apps/lumen/src/wal.rs:677-699; this
//!   contract adds no byte parser, path, or authorization input. The persisted
//!   AOF byte boundary remains covered by
//!   apps/lumen/e2e/aof_oversized_committed_apply.rs:445-503, which feeds a
//!   complete malformed frame and requires refusal with the old watermark.
//! - Performance: apps/lumen/docs/indexing.md:264-272 says, verbatim,
//!   "Pending active, frozen, and reserved changes have a 256 MiB budget."
//!   native_oversized_committed_apply.rs:309-323, called at :418, :437, and
//!   :477,
//!   reads the real public /metrics total and high-water gauges and requires
//!   both to stay within that budget. APPLY_WATCHDOG and CHECKPOINT_WATCHDOG
//!   bound test cleanup only. They make no product latency claim.
//!
//! # Root negative controls
//!
//! - Keep the oversized foreign-delivery restart/refusal branch in
//!   apps/lumen/src/coordinator.rs:456-479. The behavior assertion at :359
//!   must fail because the committed watermark stays at the schema sequence.
//! - Remove the post-apply AOF append in apps/lumen/src/coordinator.rs:605-614.
//!   The pre-checkpoint real-AOF replay assertion at :441-455 must fail.
//! - Do not change the fixture threshold or an assertion to create either red.
//!
//! Gate: cargo test -p lumen --test native_oversized_committed_apply -- --nocapture.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum_test::TestServer;

use lumen::aof::{replay_aof_into, AofWriter};
use lumen::api::{router, AppState, CheckpointSink};
use lumen::auth::AuthConfig;
use lumen::coordinator::{SharedAof, WriteCoordinator, WriteSink};
use lumen::log_entry::RaftLogEntry;
use lumen::segment_checkpoint::SegmentCheckpointSink;
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::Engine;
use lumen::types::{
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest, QueryNode,
    SearchRequest, TermQuery,
};
use lumen::wal::{MemWal, SharedWal, WalLog, WalRecord};

const COLLECTION: &str = "native-oversized-committed-scalar";
const FIELD: &str = "keyword";
const ITEM_COUNT: usize = 1_000;
const VALUE_BYTES: usize = 270 * 1024;
const RAW_SCALAR_BYTES: usize = ITEM_COUNT * VALUE_BYTES;
const PENDING_HARD_LIMIT_BYTES: u64 = 256 * 1024 * 1024;
const CREATE_SEQUENCE: u64 = 1;
const INDEX_SEQUENCE: u64 = 2;
const TAIL_SEQUENCE: u64 = 3;
const TAIL_EXTERNAL_ID: &str = "native-committed-tail";
const TAIL_VALUE: &str = "native-committed-tail-value";
const COMMIT_WATCHDOG: Duration = Duration::from_secs(15);
const APPLY_WATCHDOG: Duration = Duration::from_secs(30);
const CHECKPOINT_WATCHDOG: Duration = Duration::from_secs(60);
const POLL_INTERVAL: Duration = Duration::from_millis(5);

struct Fixture {
    root: tempfile::TempDir,
    engine: Arc<Engine>,
    server: TestServer,
    store: Arc<SegmentRdbStore>,
    aof: SharedAof,
    aof_path: PathBuf,
    wal: Arc<MemWal>,
    writer: Arc<WriteCoordinator>,
}

fn fixture() -> Fixture {
    let root = tempfile::tempdir().expect("native committed scalar fixture directory");
    let aof_path = root.path().join("native-committed.aof");
    let checkpoint_root = root.path().join("segments");
    let aof: SharedAof = Arc::new(Mutex::new(
        AofWriter::open(&aof_path).expect("open native committed AOF"),
    ));
    let engine = Arc::new(Engine::new());
    let store = Arc::new(SegmentRdbStore::new(&checkpoint_root).expect("open segment store"));
    let wal = Arc::new(MemWal::new());
    let shared_wal: SharedWal = wal.clone();
    let writer = WriteCoordinator::start_from_with_aof(shared_wal, engine.clone(), 0, aof.clone());
    let sink_writer: Arc<dyn WriteSink> = writer.clone();
    let checkpoint: Arc<dyn CheckpointSink> = Arc::new(SegmentCheckpointSink {
        engine: engine.clone(),
        store: store.clone(),
        writer: sink_writer.clone(),
        aof: Some(aof.clone()),
    });
    let state =
        AppState::with_components(engine.clone(), Arc::new(AuthConfig::open()), sink_writer)
            .with_checkpoint(checkpoint);
    let server = TestServer::new(router(state)).expect("open native committed metrics server");

    Fixture {
        root,
        engine,
        server,
        store,
        aof,
        aof_path,
        wal,
        writer,
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
    format!("native-committed-scalar-{ordinal:04}")
}

fn keyword_value(ordinal: usize) -> String {
    let prefix = format!("native-committed-value-{ordinal:04}:begin:");
    let suffix = format!(":end:{ordinal:04}:native-committed-value");
    assert!(
        prefix.len() + suffix.len() < VALUE_BYTES,
        "fixture markers must leave a real 270 KiB Keyword value",
    );
    let mut bytes = vec![b'x'; VALUE_BYTES];
    bytes[..prefix.len()].copy_from_slice(prefix.as_bytes());
    bytes[VALUE_BYTES - suffix.len()..].copy_from_slice(suffix.as_bytes());
    String::from_utf8(bytes).expect("native committed Keyword values are valid UTF-8")
}

fn create_record() -> WalRecord {
    WalRecord::new(RaftLogEntry::CreateCollection {
        collection_id: COLLECTION.to_owned(),
        req: keyword_schema(),
    })
}

fn oversized_index_record() -> WalRecord {
    assert!(
        RAW_SCALAR_BYTES > PENDING_HARD_LIMIT_BYTES as usize,
        "fixture must exceed the documented 256 MiB pending budget: raw_scalar_bytes={RAW_SCALAR_BYTES}",
    );
    let mut items = Vec::with_capacity(ITEM_COUNT);
    for ordinal in 0..ITEM_COUNT {
        items.push(IndexItem {
            external_id: external_id(ordinal),
            field: FIELD.to_owned(),
            value: FieldValue::String(keyword_value(ordinal)),
            version: None,
        });
    }
    assert_eq!(
        items.len(),
        ITEM_COUNT,
        "fixture must publish all distinct committed Keyword items in one record",
    );
    WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items,
            request_id: None,
        },
    })
}

fn tail_index_record() -> WalRecord {
    WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items: vec![IndexItem {
                external_id: TAIL_EXTERNAL_ID.to_owned(),
                field: FIELD.to_owned(),
                value: FieldValue::String(TAIL_VALUE.to_owned()),
                version: None,
            }],
            request_id: None,
        },
    })
}

fn term_query(value: &str) -> SearchRequest {
    SearchRequest {
        query: QueryNode::Term(TermQuery {
            field: FIELD.to_owned(),
            value: FieldValue::String(value.to_owned()),
        }),
        limit: 2,
        offset: 0,
        cursor: None,
        routing_key: None,
        sort: None,
        track_total: true,
        collapse: None,
    }
}

fn search_ids(engine: &Engine, value: &str) -> Vec<String> {
    engine
        .search(COLLECTION, term_query(value))
        .unwrap_or_else(|error| panic!("exact native committed Keyword query failed: {error}"))
        .hits
        .into_iter()
        .map(|hit| hit.external_id)
        .collect()
}

fn assert_every_keyword(engine: &Engine, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("native committed scalar collection stats")
            .documents_indexed,
        ITEM_COUNT as u64,
        "{phase}: all 1,000 committed external IDs must be present",
    );
    assert_original_keywords(engine, phase);
}

fn assert_original_keywords(engine: &Engine, phase: &str) {
    for ordinal in 0..ITEM_COUNT {
        assert_eq!(
            search_ids(engine, &keyword_value(ordinal)),
            vec![external_id(ordinal)],
            "{phase}: committed Keyword item {ordinal} was lost, changed, or swapped",
        );
    }
}

fn assert_checkpoint_base_plus_aof_tail(engine: &Engine, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("native committed scalar tail collection stats")
            .documents_indexed,
        (ITEM_COUNT + 1) as u64,
        "{phase}: checkpoint base plus one AOF-tail item must contain every row",
    );
    assert_original_keywords(engine, phase);
    assert_eq!(
        search_ids(engine, TAIL_VALUE),
        vec![TAIL_EXTERNAL_ID.to_owned()],
        "{phase}: native committed AOF tail must retain its exact Keyword item",
    );
}

fn metric_u64(metrics: &str, name: &str) -> u64 {
    let values = metrics
        .lines()
        .filter_map(|line| {
            let (metric, value) = line.split_once(|character: char| character.is_whitespace())?;
            (metric == name).then_some(value.trim())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        values.len(),
        1,
        "public /metrics must publish exactly one {name} sample: {metrics}",
    );
    values[0]
        .parse::<u64>()
        .unwrap_or_else(|_| panic!("{name} must be an unsigned byte value: {}", values[0]))
}

async fn assert_public_pending_budget(server: &TestServer, phase: &str) {
    let response = server.get("/metrics").await;
    response.assert_status_ok();
    let metrics = response.text();
    let total = metric_u64(&metrics, "lumen_pending_change_total_bytes");
    let high_water = metric_u64(&metrics, "lumen_pending_change_high_water_bytes");
    assert!(
        total <= PENDING_HARD_LIMIT_BYTES,
        "{phase}: public pending-change total must stay within the approved 256 MiB budget: total={total}",
    );
    assert!(
        high_water <= PENDING_HARD_LIMIT_BYTES,
        "{phase}: public pending-change high water must stay within the approved 256 MiB budget: high_water={high_water}",
    );
}

async fn publish_committed(wal: &MemWal, record: WalRecord, phase: &str) -> u64 {
    let published = tokio::time::timeout(COMMIT_WATCHDOG, wal.publish(record)).await;
    assert!(
        published.is_ok(),
        "{phase}: native MemWal publish must return a committed sequence before test cleanup",
    );
    let published = published.expect("behavior assertion above checked publish watchdog");
    let diagnostic = published
        .as_ref()
        .err()
        .map(|error| format!("{error:#}"))
        .unwrap_or_default();
    assert!(
        published.is_ok(),
        "{phase}: a valid native committed record must not receive an error acknowledgement: {diagnostic}",
    );
    published.expect("behavior assertion above checked native MemWal acknowledgement")
}

async fn wait_for_committed_apply(fixture: &Fixture, sequence: u64, phase: &str) {
    let applied = tokio::time::timeout(APPLY_WATCHDOG, async {
        loop {
            if fixture.writer.applied_seq() >= sequence {
                return;
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    })
    .await;
    let wal_head = fixture
        .wal
        .latest_seq()
        .await
        .expect("read native MemWal head after committed publication");
    assert!(
        applied.is_ok(),
        "{phase}: a valid native committed oversized Index must reach sequence {sequence} before test cleanup; applied_sequence={}, wal_head={wal_head}, restart_required={}",
        fixture.writer.applied_seq(),
        fixture.writer.is_restart_required(),
    );
}

async fn checkpoint_now(server: &TestServer) {
    let response = tokio::time::timeout(CHECKPOINT_WATCHDOG, async {
        server.post("/admin/checkpoint").await
    })
    .await;
    assert!(
        response.is_ok(),
        "the real on-demand checkpoint must finish before test cleanup",
    );
    let response = response.expect("behavior assertion above checked checkpoint watchdog");
    response.assert_status_ok();
    assert_eq!(
        response.json::<serde_json::Value>()["persisted"],
        true,
        "the real on-demand checkpoint must publish durable state",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn native_committed_oversized_scalar_index_applies_persists_and_cold_reopens() {
    let fixture = fixture();

    let create_sequence =
        publish_committed(&fixture.wal, create_record(), "collection schema").await;
    assert_eq!(
        create_sequence, CREATE_SEQUENCE,
        "the schema must be the first native committed MemWal record",
    );
    wait_for_committed_apply(&fixture, create_sequence, "collection schema").await;

    let index_sequence = publish_committed(
        &fixture.wal,
        oversized_index_record(),
        "oversized scalar Index",
    )
    .await;
    assert_eq!(
        index_sequence, INDEX_SEQUENCE,
        "the oversized Index must become the next native committed MemWal record",
    );
    wait_for_committed_apply(&fixture, index_sequence, "oversized scalar Index").await;
    assert_eq!(
        fixture.writer.applied_seq(),
        index_sequence,
        "the applied watermark must equal the committed oversized Index sequence",
    );
    assert!(
        !fixture.writer.is_restart_required(),
        "a valid committed oversized Index must not leave the native coordinator restart-required",
    );
    assert_every_keyword(&fixture.engine, "live native committed apply");
    assert_public_pending_budget(&fixture.server, "live native committed apply").await;

    {
        fixture
            .aof
            .lock()
            .expect("native committed AOF lock")
            .sync_strict()
            .expect("strict-sync applied native committed AOF");
    }
    let retained_aof_path = fixture.root.path().join("before-checkpoint.aof");
    let copied = fs::copy(&fixture.aof_path, &retained_aof_path)
        .expect("copy the real AOF before its real checkpoint trim");
    assert!(
        copied > 0,
        "the applied native committed prefix must persist non-empty AOF bytes before checkpoint",
    );

    checkpoint_now(&fixture.server).await;
    assert_public_pending_budget(&fixture.server, "native committed checkpoint").await;

    let aof_recovered = Arc::new(Engine::new());
    let aof_replayed = replay_aof_into(&aof_recovered, &retained_aof_path, 0);
    assert!(
        aof_replayed.is_ok(),
        "the copied real AOF must replay the applied native oversized Index: {}",
        aof_replayed
            .as_ref()
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default(),
    );
    assert_eq!(
        aof_replayed.expect("behavior assertion above checked AOF replay"),
        index_sequence,
        "AOF replay must retain the native committed watermark",
    );
    assert_every_keyword(&aof_recovered, "real AOF replay before checkpoint trim");

    let tail_sequence = publish_committed(
        &fixture.wal,
        tail_index_record(),
        "post-checkpoint AOF tail",
    )
    .await;
    assert_eq!(
        tail_sequence, TAIL_SEQUENCE,
        "the post-checkpoint native AOF tail must use the next committed sequence",
    );
    wait_for_committed_apply(&fixture, tail_sequence, "post-checkpoint AOF tail").await;
    assert_checkpoint_base_plus_aof_tail(&fixture.engine, "live post-checkpoint AOF tail");
    {
        fixture
            .aof
            .lock()
            .expect("native committed AOF tail lock")
            .sync_strict()
            .expect("strict-sync native committed AOF tail");
    }
    assert_public_pending_budget(&fixture.server, "native committed AOF tail").await;

    let cold = fixture
        .store
        .load_current_generation()
        .expect("cold-open native committed checkpoint")
        .expect("native committed checkpoint must publish CURRENT");
    assert_eq!(
        cold.sequence, index_sequence,
        "cold checkpoint must retain the native committed watermark",
    );
    assert_every_keyword(&cold.engine, "cold native committed checkpoint base");
    let tail_replayed = replay_aof_into(&cold.engine, &fixture.aof_path, cold.sequence);
    assert!(
        tail_replayed.is_ok(),
        "cold replay after checkpoint must accept the trimmed AOF tail: {}",
        tail_replayed
            .as_ref()
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default(),
    );
    assert_eq!(
        tail_replayed.expect("behavior assertion above checked cold AOF tail"),
        tail_sequence,
        "cold replay must apply the post-checkpoint native committed AOF suffix",
    );
    assert_checkpoint_base_plus_aof_tail(&cold.engine, "cold native committed AOF suffix");
}
