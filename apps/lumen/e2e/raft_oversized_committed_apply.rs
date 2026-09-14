//! Black-box contract for Lumen #4246 committed oversized fast-Index replay.
//!
//! The fixture stores a valid `LWAL` v1 Index record in a real single-voter
//! Raft store with a recording state machine. It then cold-starts that same
//! retained store with Lumen's `EngineSm`, so this case enters the
//! already-committed callback instead of leader-side proposal admission.
//! The oracle measures only public Lumen pending-change metrics. Cold replay
//! borrows a resident `RaftEntry::command` `Vec<u8>`; the live ordered host
//! pins and maps a command. It does not claim either Raft source owner is free,
//! mapped, or charged as Lumen pending-change memory.
//!
//! # Facets
//!
//! - Behavior: `apps/lumen/e2e/raft_oversized_committed_apply.rs:409-416`
//!   owns the current cold-host refusal red. `:428-444` detects partial
//!   visibility and a later stall. `:497-502`, `:516-520`, and helper
//!   assertions `:196-209` require a 1,000-item,
//!   270 KiB-per-Keyword fast Index record already retained by Raft to publish
//!   every exact Keyword, advance its Raft index, checkpoint, and cold reopen.
//!   This covers committed apply at `apps/lumen/src/raft_sm.rs:104-158` and
//!   `:221-249`, with retained-store replay at
//!   `libs/raft-runtime/src/host.rs:568-584`.
//! - Security: `apps/lumen/e2e/raft_oversized_committed_apply.rs:459-480`
//!   feeds a truncated caller/peer-controlled `LWAL` fast Index into the
//!   committed `EngineSm::apply` callback and requires refusal, no document,
//!   and the old watermark. It covers the fast-record byte boundary at
//!   `apps/lumen/src/wal.rs:320-419` as reached by
//!   `apps/lumen/src/raft_sm.rs:221-249`. It runs after the behavior oracle,
//!   so the current red remains the oversized committed-record behavior red.
//! - Performance: `apps/lumen/docs/indexing.md:264-272` says, verbatim,
//!   "Pending active, frozen, and reserved changes have a 256 MiB budget."
//!   `apps/lumen/e2e/raft_oversized_committed_apply.rs:238-245`, called at
//!   `:503` and `:511`, reads the
//!   public `/metrics` pending high-water and total gauges and keeps both at or
//!   below that budget. `REPLAY_WATCHDOG` is test cleanup that turns a stalled
//!   committed replay into an assertion; it is not a latency promise.
//!
//! Gate: `cargo test -p lumen --features raft-wal --test
//! raft_oversized_committed_apply -- --nocapture`.

use std::any::Any;
use std::collections::{BTreeMap, HashMap};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::Request;
use axum_test::TestServer;
use lumen::api::{router, AppState};
use lumen::log_entry::RaftLogEntry;
use lumen::raft_sm::{EngineSm, RaftWriteSink};
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::Engine;
use lumen::types::{
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest, QueryNode,
    SearchRequest, TermQuery,
};
use lumen::wal::WalRecord;
use raft_runtime::{
    FsyncPolicy, HostConfig, Index, Membership, RaftHost, RaftStateMachine, RaftStatus, RaftStore,
};
use tower::ServiceExt as _;

const COLLECTION: &str = "oversized-committed-fast-index";
const FIELD: &str = "keyword";
const ITEM_COUNT: usize = 1_000;
const VALUE_BYTES: usize = 270 * 1024;
const PENDING_HARD_LIMIT_BYTES: u64 = 256 * 1024 * 1024;
const CREATE_INDEX: Index = 1;
const INDEX_RECORD: Index = 2;
const REPLAY_WATCHDOG: Duration = Duration::from_secs(30);
// This only avoids treating a 264 MiB durable fixture write as a product
// latency assertion. The contract's observable watchdog is below.
const FIXTURE_PROPOSE_TIMEOUT: Duration = Duration::from_secs(60);
const POLL_INTERVAL: Duration = Duration::from_millis(5);
const READINESS_STATUS_MAX_BYTES: usize = 8 * 1024;

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

fn create_command() -> Vec<u8> {
    WalRecord::new(RaftLogEntry::CreateCollection {
        collection_id: COLLECTION.to_owned(),
        req: keyword_schema(),
    })
    .encode()
    .expect("encode retained collection schema")
}

fn external_id(ordinal: usize) -> String {
    format!("committed-fast-index-{ordinal:04}")
}

fn value(ordinal: usize) -> String {
    let prefix = format!("committed-fast-index-value-{ordinal:04}:begin:");
    let suffix = format!(":end:{ordinal:04}:committed-fast-index-value");
    assert!(
        prefix.len() + suffix.len() < VALUE_BYTES,
        "fixture markers must leave an actual 270 KiB Keyword value"
    );
    let mut bytes = vec![b'x'; VALUE_BYTES];
    bytes[..prefix.len()].copy_from_slice(prefix.as_bytes());
    bytes[VALUE_BYTES - suffix.len()..].copy_from_slice(suffix.as_bytes());
    String::from_utf8(bytes).expect("fixture Keywords stay valid UTF-8")
}

fn oversized_fast_index_command() -> Vec<u8> {
    let mut items = Vec::with_capacity(ITEM_COUNT);
    for ordinal in 0..ITEM_COUNT {
        items.push(IndexItem {
            external_id: external_id(ordinal),
            field: FIELD.to_owned(),
            value: FieldValue::String(value(ordinal)),
            version: None,
        });
    }
    // This command is retained through Raft by the recording producer below.
    // It does not enter Lumen through an HTTP or local leader-admission API.
    let command = WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items,
            request_id: None,
        },
    })
    .encode()
    .expect("encode a valid fast Index record");
    assert!(
        command.starts_with(b"LWAL"),
        "fixture must enter the authentic fast Index codec"
    );
    assert!(
        command.len() > PENDING_HARD_LIMIT_BYTES as usize,
        "fixture must exceed the 256 MiB pending-change budget without widening an HTTP batch: command_bytes={}",
        command.len(),
    );
    command
}

fn small_fast_index_command() -> Vec<u8> {
    WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items: vec![IndexItem {
                external_id: "truncated-fast-record".to_owned(),
                field: FIELD.to_owned(),
                value: FieldValue::String("must-not-publish".to_owned()),
                version: None,
            }],
            request_id: None,
        },
    })
    .encode()
    .expect("encode a small fast Index control record")
}

fn search_ids(engine: &Engine, exact_keyword: &str) -> Vec<String> {
    engine
        .search(
            COLLECTION,
            SearchRequest {
                query: QueryNode::Term(TermQuery {
                    field: FIELD.to_owned(),
                    value: FieldValue::String(exact_keyword.to_owned()),
                }),
                limit: 2,
                offset: 0,
                cursor: None,
                routing_key: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .expect("exact Keyword query")
        .hits
        .into_iter()
        .map(|hit| hit.external_id)
        .collect()
}

fn assert_every_keyword_is_present(engine: &Engine, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("oversized collection stats")
            .documents_indexed,
        ITEM_COUNT as u64,
        "{phase}: every distinct external ID must be present",
    );
    for ordinal in 0..ITEM_COUNT {
        assert_eq!(
            search_ids(engine, &value(ordinal)),
            vec![external_id(ordinal)],
            "{phase}: exact Keyword item {ordinal} was lost, changed, or swapped",
        );
    }
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

async fn assert_public_pending_budget(engine: Arc<Engine>, writer: Arc<RaftWriteSink>) {
    // Reuse the actual Raft writer. Starting a new MemWal coordinator here
    // would incorrectly initialize this already-replayed Engine to sequence 0.
    let state =
        AppState::with_components(engine, Arc::new(lumen::auth::AuthConfig::open()), writer);
    let server = TestServer::new(router(state)).expect("metrics test server");
    let response = server.get("/metrics").await;
    response.assert_status_ok();
    let metrics = response.text();
    let total = metric_u64(&metrics, "lumen_pending_change_total_bytes");
    let high_water = metric_u64(&metrics, "lumen_pending_change_high_water_bytes");
    assert!(
        total <= PENDING_HARD_LIMIT_BYTES,
        "public pending-change total must remain within the approved 256 MiB budget: total={total}",
    );
    assert!(
        high_water <= PENDING_HARD_LIMIT_BYTES,
        "public pending-change high water must remain within the approved 256 MiB budget: high_water={high_water}",
    );
}

/// This writer deliberately accepts any validly retained command. It is only
/// the prior owner used to place the exact two commands in a durable Raft log;
/// the Lumen `EngineSm` is the cold replayer under test.
struct RecordingStateMachine {
    applied: AtomicU64,
}

impl RecordingStateMachine {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            applied: AtomicU64::new(0),
        })
    }
}

impl RaftStateMachine for RecordingStateMachine {
    fn apply(&self, index: Index, _: &[u8]) -> anyhow::Result<()> {
        self.applied.store(index, Ordering::Release);
        Ok(())
    }

    fn snapshot(&self, writer: &mut dyn Write) -> anyhow::Result<()> {
        writer.write_all(&self.applied.load(Ordering::Acquire).to_le_bytes())?;
        Ok(())
    }

    fn restore(&self, reader: &mut dyn Read) -> anyhow::Result<()> {
        let mut bytes = [0_u8; std::mem::size_of::<u64>()];
        reader.read_exact(&mut bytes)?;
        self.applied
            .store(u64::from_le_bytes(bytes), Ordering::Release);
        Ok(())
    }

    fn applied_index(&self) -> Index {
        self.applied.load(Ordering::Acquire)
    }
}

fn host_config() -> HostConfig {
    HostConfig {
        tick: Duration::from_millis(5),
        propose_timeout: FIXTURE_PROPOSE_TIMEOUT,
        ..HostConfig::default()
    }
}

fn panic_payload_text(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_owned();
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    "non-string panic payload".to_owned()
}

async fn cold_host_readiness_diagnostic(host: &RaftHost) -> String {
    let persistence = host.store().persistence_stats();
    let applied_watch = host.applied_watch();
    let applied_index = *applied_watch.borrow();
    let route_timeout = host_config().rpc_timeout;
    let raftz = tokio::time::timeout(route_timeout, async {
        let response = host
            .router()
            .oneshot(
                Request::builder()
                    .uri("/raftz")
                    .body(Body::empty())
                    .expect("build in-process Raft status request"),
            )
            .await
            .map_err(|error| format!("route /raftz: {error:?}"))?;
        let response_status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), READINESS_STATUS_MAX_BYTES)
            .await
            .map_err(|error| format!("read /raftz response: {error}"))?;
        let status = serde_json::from_slice::<RaftStatus>(&bytes)
            .map_err(|error| format!("decode /raftz response: {error}"))?;
        Ok::<_, String>(format!("http={response_status}; status={status:?}"))
    })
    .await;
    let raftz = match raftz {
        Ok(Ok(status)) => status,
        Ok(Err(error)) => format!("error={error}"),
        Err(_) => format!("timed_out_after_ms={}", route_timeout.as_millis()),
    };
    format!(
        "applied_watch={applied_index}; persistence={persistence:?}; raftz={raftz}"
    )
}

async fn wait_for_leader(host: &RaftHost) {
    let deadline = tokio::time::Instant::now() + REPLAY_WATCHDOG;
    loop {
        match tokio::time::timeout_at(deadline, host.is_leader()).await {
            Ok(true) => return,
            Ok(false) if tokio::time::Instant::now() < deadline => {
                tokio::time::sleep(POLL_INTERVAL).await;
            }
            Ok(false) | Err(_) => break,
        }
    }
    let diagnostic = cold_host_readiness_diagnostic(host).await;
    panic!("single-voter fixture did not elect a leader before test cleanup; {diagnostic}");
}

async fn retain_commands_in_real_raft_store(root: &std::path::Path) {
    let recording = RecordingStateMachine::new();
    let host = Arc::new(RaftHost::spawn(
        0,
        Membership {
            voters: vec![0],
            learners: Vec::new(),
        },
        HashMap::new(),
        RaftStore::open(
            root.to_str().expect("UTF-8 retained Raft root"),
            0,
            FsyncPolicy::Always,
        )
        .expect("open retained Raft store"),
        recording.clone() as Arc<dyn RaftStateMachine>,
        host_config(),
    ));
    wait_for_leader(host.as_ref()).await;
    assert_eq!(
        host.propose(create_command())
            .await
            .expect("persist collection schema in Raft"),
        CREATE_INDEX,
        "schema must be the first retained Raft command",
    );
    assert_eq!(
        host.propose(oversized_fast_index_command())
            .await
            .expect("persist oversized fast Index in Raft"),
        INDEX_RECORD,
        "oversized record must be committed in the retained Raft store before Lumen replays it",
    );
    assert_eq!(
        recording.applied_index(),
        INDEX_RECORD,
        "recording producer must durably apply both retained commands",
    );
    host.shutdown()
        .await
        .expect("finish producer host before cold Lumen replay");
}

async fn cold_replay_retained_store(
    root: std::path::PathBuf,
    engine: Arc<Engine>,
    state_machine: Arc<EngineSm>,
) -> Arc<RaftHost> {
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    let runtime = tokio::runtime::Handle::current();
    let replay_state_machine = state_machine.clone();
    let replay = thread::Builder::new()
        .name("lumen-oversized-committed-cold-replay".to_owned())
        .spawn(move || {
            // `RaftHost::spawn` starts tick and pump tasks after its blocking
            // cold replay. Enter the test runtime so those real host tasks
            // have the same runtime surface as an ordinary Lumen host.
            let _runtime_guard = runtime.enter();
            // `spawn` currently reports a cold apply error by panicking. Turn
            // that product refusal into the test's own behavior assertion
            // rather than accepting a worker-thread panic as the contract red.
            let host = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                Arc::new(RaftHost::spawn(
                    0,
                    Membership {
                        voters: vec![0],
                        learners: Vec::new(),
                    },
                    HashMap::new(),
                    RaftStore::open(
                        root.to_str().expect("UTF-8 cold Raft root"),
                        0,
                        FsyncPolicy::Always,
                    )
                    .expect("open retained Raft store for Lumen cold replay"),
                    replay_state_machine as Arc<dyn RaftStateMachine>,
                    host_config(),
                ))
            }))
            .map_err(panic_payload_text);
            let _ = ready_tx.send(host);
        })
        .expect("start cold Raft replay thread");

    let deadline = Instant::now() + REPLAY_WATCHDOG;
    let mut saw_schema = false;
    loop {
        match ready_rx.try_recv() {
            Ok(host) => {
                replay.join().expect("cold replay thread must not panic");
                assert!(
                    host.is_ok(),
                    "Raft retained replay must apply the valid oversized fast Index; cold host refused it: {}",
                    host.as_ref()
                        .err()
                        .map(String::as_str)
                        .unwrap_or("no refusal message"),
                );
                return host.expect("behavior assertion checked the cold replay result");
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                panic!("cold Raft replay terminated before constructing its host")
            }
        }

        match engine.stats(COLLECTION) {
            Ok(stats) => {
                saw_schema = true;
                assert!(
                    stats.documents_indexed == 0 || stats.documents_indexed == ITEM_COUNT as u64,
                    "a committed fast Index must never expose a partial item prefix while it replays: documents_indexed={}",
                    stats.documents_indexed,
                );
            }
            Err(error) => assert!(
                !saw_schema,
                "a committed fast Index must not withdraw its already published schema while it replays: {error}",
            ),
        }

        assert!(
            Instant::now() < deadline,
            "Raft retained replay must finish the valid oversized fast Index before test cleanup; saw_schema={saw_schema}, applied_index={}",
            state_machine.applied_index(),
        );
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

fn assert_truncated_fast_record_is_refused() {
    let engine = Arc::new(Engine::new());
    let state_machine = EngineSm::new(engine.clone(), 0);
    state_machine
        .apply(CREATE_INDEX, &create_command())
        .expect("apply valid schema before malformed committed record");
    let mut malformed = small_fast_index_command();
    malformed.pop();

    let refusal = state_machine.apply(INDEX_RECORD, &malformed);
    assert!(
        refusal.is_err(),
        "a truncated committed LWAL fast Index must be refused before publication",
    );
    assert_eq!(
        state_machine.applied_index(),
        CREATE_INDEX,
        "a refused committed fast record must not advance the Raft watermark",
    );
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("schema remains readable after malformed record")
            .documents_indexed,
        0,
        "a refused committed fast record must not mutate the document census",
    );
    assert_eq!(
        search_ids(&engine, "must-not-publish"),
        Vec::<String>::new(),
        "a refused committed fast record must not become query-visible",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn retained_oversized_fast_index_replays_atomically_within_budget_and_cold_reopens() {
    let root = tempfile::tempdir().expect("oversized committed Raft fixture root");
    let raft_root = root.path().join("raft");
    retain_commands_in_real_raft_store(&raft_root).await;

    let segment_store = Arc::new(
        SegmentRdbStore::new(root.path().join("segments"))
            .expect("open segment checkpoint store for cold replay"),
    );
    let engine = Arc::new(Engine::new());
    let state_machine = EngineSm::new_with_segment_store(engine.clone(), 0, segment_store.clone());
    let host = cold_replay_retained_store(raft_root, engine.clone(), state_machine.clone()).await;
    // Cold replay completion does not mean first tick/election persistence is finished.
    wait_for_leader(host.as_ref()).await;
    let writer = Arc::new(RaftWriteSink::new(host.clone(), state_machine.clone()));

    assert_eq!(
        state_machine.applied_index(),
        INDEX_RECORD,
        "valid retained replay must advance through the oversized committed index",
    );
    assert_every_keyword_is_present(&engine, "live retained replay");
    assert_public_pending_budget(engine.clone(), writer.clone()).await;

    host.shutdown()
        .await
        .expect("stop cold replay host before standalone checkpoint");
    segment_store
        .save(&engine, INDEX_RECORD)
        .expect("checkpoint the fully applied oversized committed record");
    assert_public_pending_budget(engine.clone(), writer).await;
    let cold = segment_store
        .load_current_generation()
        .expect("open segment CURRENT after oversized committed replay")
        .expect("oversized committed checkpoint must publish CURRENT");
    assert_eq!(
        cold.sequence, INDEX_RECORD,
        "checkpoint must retain the committed Raft watermark",
    );
    assert_every_keyword_is_present(&cold.engine, "cold segment reopen");

    assert_truncated_fast_record_is_refused();
}
