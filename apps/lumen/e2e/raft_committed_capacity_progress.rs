//! Black-box contract for a fitting generic committed Raft record at a full
//! pending-change budget.
//!
//! The parent creates a schema-only durable segment checkpoint. Its isolated
//! child cold-opens that checkpoint, makes real local `docs:replace` changes
//! until at most 1 MiB remains in the Engine-owned pending budget, then calls
//! public `RaftStateMachine::apply` with one valid 2 MiB generic-CBOR record.
//! The record is below the 32 MiB borrowed mapped-record route, so it reaches
//! the normal generic Raft decoder. The child starts no `PendingChangeSpill`,
//! `SegmentCheckpointDriver`, or caller checkpoint before that apply returns.
//!
//! A correct state machine starts independent capacity maintenance before its
//! pre-decode reservation waits. It returns with the retained peer command
//! intact, advances one Raft index, exposes both rows, and leaves a checkpoint
//! that a later cold open can read. The short observation and child watchdog
//! only bound test cleanup. They are not a product latency promise.
//!
//! # Facets
//!
//! - Behavior: assertions at `raft_committed_capacity_progress.rs:224-237`
//!   construct the fitting peer record and `:694-731` makes the schema-only
//!   durable base. Assertions at `:393-471` require the generic `EngineSm::apply`
//!   callback to finish before a caller checkpoint and retain
//!   its source bytes. Assertions at `:560-596` require one committed
//!   watermark, maintenance, both rows, and a final cold open at that watermark.
//!   They cover `apps/lumen/src/raft_sm.rs:107-160` and `:223-302`, plus
//!   `apps/lumen/src/segment_capacity.rs:410-443`.
//! - Security: assertions at `:604-617` corrupt the external peer command
//!   after the valid result and require the generic decoder to refuse it without
//!   moving the Raft watermark or changing indexed rows. They cover the same
//!   `apps/lumen/src/raft_sm.rs:107-160` byte boundary and
//!   `apps/lumen/src/wal.rs:256-267` `WalRecord::decode` input validation. The capacity
//!   owner only adds maintenance before a wait; it does not widen the accepted
//!   peer format.
//! - Performance: `apps/lumen/docs/indexing.md:264-276` says, verbatim,
//!   "Pending active, frozen, and reserved changes have a 256 MiB budget."
//!   Assertions at `:309-317`, `:322-326`, and `:570-596` use the public
//!   pending total and high-water gauges and require both to remain within that
//!   budget. Fixture source and query bytes are outside those gauges. This
//!   contract names no latency budget.
//!
//! # Root negative control
//!
//! Remove the generic capacity-owner ensure before a Full wait in
//! `apps/lumen/src/raft_sm.rs:107-160`. The behavior assertion at `:433-436`
//! must fail after its bounded cleanup checkpoint proves that the valid retained
//! peer command had been waiting for caller-driven progress. Restore every
//! changed production file by SHA-256 before another gate.
//!
//! Target gate: `cargo test -p lumen --features raft-wal --test
//! raft_committed_capacity_progress -- --nocapture`.
//! Full declared behavior gate: `cargo test -p lumen --features raft-wal`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use axum_test::TestServer;

use lumen::api::{router, AppState};
use lumen::auth::AuthConfig;
use lumen::coordinator::WriteSink;
use lumen::log_entry::RaftLogEntry;
use lumen::raft_sm::EngineSm;
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::{ApplyOutcome, Engine};
use lumen::types::{
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, QueryNode, ReplaceDocItem,
    ReplaceDocsRequest, SearchRequest, TermQuery,
};
use lumen::wal::WalRecord;
use raft_runtime::{Index, RaftStateMachine};

const COLLECTION: &str = "raft-committed-capacity-progress";
const FILLER_FIELD: &str = "filler";
const TARGET_FIELD: &str = "target";
const FILLER_ID: &str = "checkpointable-local-predecessor";
const TARGET_ID: &str = "committed-generic-raft-target";
const CREATE_SEQUENCE: Index = 1;
const TARGET_SEQUENCE: Index = 2;
const MALFORMED_SEQUENCE: Index = 3;
const PENDING_HARD_LIMIT_BYTES: u64 = 256 * 1024 * 1024;
const MAPPED_ROUTE_MINIMUM_BYTES: usize = 32 * 1024 * 1024;
const TARGET_TOKEN_BYTES: usize = 2 * 1024 * 1024;
const FILLER_VALUE_BYTES: usize = 64 * 1024;
const INITIAL_MAX_FREE_BYTES: u64 = 1024 * 1024;
const MAX_FILLER_WRITES: usize = 6_000;
const PROGRESS_OBSERVATION: Duration = Duration::from_secs(5);
const CLEANUP_WAIT: Duration = Duration::from_secs(30);
const CHILD_WATCHDOG: Duration = Duration::from_secs(120);
const POLL_INTERVAL: Duration = Duration::from_millis(25);
const CHILD_MODE_ENV: &str = "LUMEN_RAFT_CAPACITY_PROGRESS_CHILD";
const CHILD_ROOT_ENV: &str = "LUMEN_RAFT_CAPACITY_PROGRESS_ROOT";
const CHILD_HANDSHAKE_ENV: &str = "LUMEN_RAFT_CAPACITY_PROGRESS_HANDSHAKE";
const CHILD_CASE: &str = "generic-predecode-capacity";
const TEST_NAME: &str =
    "raft_committed_generic_apply_makes_capacity_progress_before_any_caller_checkpoint";

/// Supplies the actual committed Raft watermark without constructing a fresh
/// coordinator, which would initialize this Engine's capture barrier at zero.
struct ReadOnlyMetricsSink {
    applied_sequence: u64,
}

#[async_trait]
impl WriteSink for ReadOnlyMetricsSink {
    async fn submit(&self, _entry: RaftLogEntry) -> anyhow::Result<ApplyOutcome> {
        anyhow::bail!("read-only metrics server must not submit a mutation")
    }

    fn applied_seq(&self) -> u64 {
        self.applied_sequence
    }
}

/// Reap a child before its parent-owned temporary directory disappears.
struct ChildCleanup(Option<Child>);

impl Drop for ChildCleanup {
    fn drop(&mut self) {
        let Some(child) = self.0.as_mut() else {
            return;
        };
        if !matches!(child.try_wait(), Ok(Some(_))) {
            let _ = child.kill();
        }
        let _ = child.wait();
    }
}

fn keyword_spec() -> FieldSpec {
    FieldSpec {
        field_type: FieldType::Keyword,
        analyzer: None,
        multi: None,
        dim: None,
        metric: None,
        backend: None,
        quantize: None,
    }
}

fn schema() -> CreateCollectionRequest {
    CreateCollectionRequest {
        fields: BTreeMap::from([
            (FILLER_FIELD.to_owned(), keyword_spec()),
            (TARGET_FIELD.to_owned(), keyword_spec()),
        ]),
    }
}

fn encode_record(record: WalRecord, phase: &str) -> Vec<u8> {
    record
        .encode()
        .unwrap_or_else(|error| panic!("{phase}: encode valid committed Raft record: {error}"))
}

fn create_command() -> Vec<u8> {
    encode_record(
        WalRecord::new(RaftLogEntry::CreateCollection {
            collection_id: COLLECTION.to_owned(),
            req: schema(),
        }),
        "schema-only collection",
    )
}

fn marked_ascii(prefix: &str, ordinal: usize, bytes: usize) -> String {
    let head = format!("{prefix}-{ordinal:04}:");
    let tail = format!(":end-{prefix}-{ordinal:04}");
    assert!(
        head.len() + tail.len() < bytes,
        "fixture markers must leave a real valid Keyword value",
    );
    let mut value = vec![b'x'; bytes];
    value[..head.len()].copy_from_slice(head.as_bytes());
    value[bytes - tail.len()..].copy_from_slice(tail.as_bytes());
    String::from_utf8(value).expect("capacity fixture Keyword must be valid ASCII")
}

fn filler_value(ordinal: usize) -> String {
    marked_ascii("checkpointable-local-filler", ordinal, FILLER_VALUE_BYTES)
}

fn target_value() -> String {
    marked_ascii("committed-generic-raft-token", 0, TARGET_TOKEN_BYTES)
}

fn local_filler_request(ordinal: usize) -> ReplaceDocsRequest {
    ReplaceDocsRequest {
        docs: vec![ReplaceDocItem {
            external_id: FILLER_ID.to_owned(),
            version: Some(ordinal as u64 + 1),
            fields: BTreeMap::from([(
                FILLER_FIELD.to_owned(),
                FieldValue::String(filler_value(ordinal)),
            )]),
        }],
    }
}

fn target_command() -> Vec<u8> {
    let command = encode_record(
        WalRecord::new(RaftLogEntry::ReplaceDocs {
            collection_id: COLLECTION.to_owned(),
            req: ReplaceDocsRequest {
                docs: vec![ReplaceDocItem {
                    external_id: TARGET_ID.to_owned(),
                    version: Some(100),
                    fields: BTreeMap::from([(
                        TARGET_FIELD.to_owned(),
                        FieldValue::String(target_value()),
                    )]),
                }],
            },
        }),
        "fitting generic committed target",
    );
    assert!(
        !command.starts_with(b"LWAL"),
        "target must use its real generic-CBOR Raft form, not the fast Index wire",
    );
    assert!(
        command.len() >= TARGET_TOKEN_BYTES,
        "target must retain its real 2 MiB Keyword token: command_bytes={}",
        command.len(),
    );
    assert!(
        command.len() < MAPPED_ROUTE_MINIMUM_BYTES,
        "target must stay below the 32 MiB borrowed mapped-record route: command_bytes={}",
        command.len(),
    );
    command
}

fn search_request(query: QueryNode) -> SearchRequest {
    SearchRequest {
        query,
        limit: 16,
        offset: 0,
        cursor: None,
        routing_key: None,
        sort: None,
        track_total: true,
        collapse: None,
    }
}

fn term_ids(engine: &Engine, field: &str, value: String, phase: &str) -> BTreeSet<String> {
    engine
        .search(
            COLLECTION,
            search_request(QueryNode::Term(TermQuery {
                field: field.to_owned(),
                value: FieldValue::String(value),
            })),
        )
        .unwrap_or_else(|error| panic!("{phase}: public exact Keyword query failed: {error:#}"))
        .hits
        .into_iter()
        .map(|hit| hit.external_id)
        .collect()
}

fn singleton(id: &str) -> BTreeSet<String> {
    BTreeSet::from([id.to_owned()])
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

fn pending_total_from_engine(engine: &Engine) -> u64 {
    metric_u64(
        &engine.metrics().render(),
        "lumen_pending_change_total_bytes",
    )
}

async fn assert_public_pending_budget(engine: Arc<Engine>, sequence: u64, phase: &str) {
    let sink: Arc<dyn WriteSink> = Arc::new(ReadOnlyMetricsSink {
        applied_sequence: sequence,
    });
    let state = AppState::with_components(engine, Arc::new(AuthConfig::open()), sink);
    let server = TestServer::new(router(state)).expect("open read-only public metrics server");
    let response = server.get("/metrics").await;
    response.assert_status_ok();
    let metrics = response.text();
    let total = metric_u64(&metrics, "lumen_pending_change_total_bytes");
    let high_water = metric_u64(&metrics, "lumen_pending_change_high_water_bytes");
    assert!(
        total <= PENDING_HARD_LIMIT_BYTES,
        "{phase}: public pending total must stay within the documented 256 MiB budget: total={total}",
    );
    assert!(
        high_water <= PENDING_HARD_LIMIT_BYTES,
        "{phase}: public pending high water must stay within the documented 256 MiB budget: high_water={high_water}",
    );
}

fn fill_checkpointable_local_work(engine: &Arc<Engine>) -> usize {
    for ordinal in 0..MAX_FILLER_WRITES {
        let total = pending_total_from_engine(engine);
        assert!(
            total <= PENDING_HARD_LIMIT_BYTES,
            "direct local setup must never exceed the documented pending budget: total={total}",
        );
        let free = PENDING_HARD_LIMIT_BYTES - total;
        if free <= INITIAL_MAX_FREE_BYTES {
            return ordinal
                .checked_sub(1)
                .expect("schema-only base must not start within the selected free-space interval");
        }
        let response = engine
            .replace_docs(COLLECTION, local_filler_request(ordinal))
            .unwrap_or_else(|error| {
                panic!(
                    "direct local setup must create checkpointable work before capacity becomes full: {error:#}",
                )
            });
        assert_eq!(
            response.results.len(),
            1,
            "each direct local setup replacement must produce one public result",
        );
    }
    panic!("setup did not leave at most 1 MiB free after {MAX_FILLER_WRITES} real local records",);
}

fn assert_live_state(engine: &Engine, filler_ordinal: usize, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("capacity Raft collection stats")
            .documents_indexed,
        2,
        "{phase}: one checkpointable local predecessor and one committed target must be live",
    );
    assert_eq!(
        term_ids(engine, FILLER_FIELD, filler_value(filler_ordinal), phase),
        singleton(FILLER_ID),
        "{phase}: the checkpointable local predecessor must survive capacity maintenance",
    );
    if filler_ordinal != 0 {
        assert_eq!(
            term_ids(engine, FILLER_FIELD, filler_value(0), phase),
            BTreeSet::new(),
            "{phase}: superseded local filler values must not revive",
        );
    }
    assert_eq!(
        term_ids(engine, TARGET_FIELD, target_value(), phase),
        singleton(TARGET_ID),
        "{phase}: the whole valid committed generic Raft Keyword must be query-visible",
    );
}

fn apply_before_caller_checkpoint(
    state_machine: Arc<EngineSm>,
    engine: Arc<Engine>,
    store: &SegmentRdbStore,
    command: Arc<Vec<u8>>,
) {
    let source_len = command.len();
    let source_digest = crc32fast::hash(command.as_slice());
    // The outer child retains one owner while this helper receives another.
    // Only the temporary worker owner must disappear when `apply` returns.
    let command_owners_before_worker = Arc::strong_count(&command);
    let (sender, receiver) = mpsc::sync_channel(1);
    let worker_state_machine = state_machine.clone();
    let worker_command = command.clone();
    let worker = thread::spawn(move || {
        let _ = sender.send(worker_state_machine.apply(TARGET_SEQUENCE, worker_command.as_slice()));
    });

    let completed_before_cleanup = match receiver.recv_timeout(PROGRESS_OBSERVATION) {
        Ok(result) => {
            worker
                .join()
                .expect("committed Raft apply worker must not panic");
            result
        }
        Err(mpsc::RecvTimeoutError::Timeout) => match receiver.try_recv() {
            Ok(result) => {
                worker
                    .join()
                    .expect("committed Raft apply worker must not panic");
                result
            }
            Err(mpsc::TryRecvError::Empty) => {
                let cleanup_sequence = store
                    .save_with_sequence(&engine, CREATE_SEQUENCE)
                    .expect("caller cleanup checkpoint must release retained local work");
                assert_eq!(
                    cleanup_sequence, CREATE_SEQUENCE,
                    "cleanup checkpoint must retain the schema watermark while it only releases local setup work",
                );
                let resumed = receiver.recv_timeout(CLEANUP_WAIT).unwrap_or_else(|error| {
                    panic!(
                        "valid retained committed Raft record did not resume after bounded cleanup checkpoint: {error}",
                    )
                });
                worker
                    .join()
                    .expect("resumed committed Raft apply worker must not panic");
                assert_eq!(
                    Arc::strong_count(&command),
                    command_owners_before_worker,
                    "the worker must release its temporary borrowed peer-command owner after the cleanup-resumed apply returns",
                );
                assert_eq!(
                    command.len(),
                    source_len,
                    "caller-owned peer command must remain retained through the capacity wait",
                );
                assert_eq!(
                    crc32fast::hash(command.as_slice()),
                    source_digest,
                    "capacity waiting must not alter the caller-owned peer command bytes",
                );
                assert!(
                    resumed.is_ok(),
                    "cleanup checkpoint must leave the valid retained peer command able to apply: {resumed:?}",
                );
                assert!(
                    false,
                    "valid fitting generic committed Raft apply waited for a caller checkpoint instead of starting independent capacity maintenance; cleanup checkpoint released the retained source and apply result was {resumed:?}",
                );
                unreachable!("the behavior assertion above always panics");
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                worker
                    .join()
                    .expect("disconnected committed Raft apply worker must not panic");
                panic!("committed Raft apply worker disconnected before returning a public result");
            }
        },
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            worker
                .join()
                .expect("disconnected committed Raft apply worker must not panic");
            panic!("committed Raft apply worker disconnected before returning a public result");
        }
    };

    completed_before_cleanup.unwrap_or_else(|error| {
        panic!("valid fitting generic committed Raft apply returned an error before caller cleanup: {error:#}")
    });
    assert_eq!(
        command.len(),
        source_len,
        "caller-owned peer command must remain retained until committed apply returns",
    );
    assert_eq!(
        crc32fast::hash(command.as_slice()),
        source_digest,
        "successful committed apply must not alter the caller-owned peer command bytes",
    );
    assert_eq!(
        Arc::strong_count(&command),
        command_owners_before_worker,
        "the worker must release its temporary borrowed peer-command owner after apply returns",
    );
}

fn checkpoint_and_cold(
    store: &SegmentRdbStore,
    engine: &Arc<Engine>,
    sequence: Index,
    phase: &str,
) -> Arc<Engine> {
    let saved = store
        .save_with_sequence(engine, sequence)
        .unwrap_or_else(|error| panic!("{phase}: real Raft checkpoint failed: {error:#}"));
    assert_eq!(
        saved, sequence,
        "{phase}: checkpoint must retain the exact committed Raft watermark",
    );
    let cold = store
        .load_current_generation()
        .unwrap_or_else(|error| panic!("{phase}: cold-open CURRENT failed: {error:#}"))
        .unwrap_or_else(|| panic!("{phase}: checkpoint must publish CURRENT"));
    assert_eq!(
        cold.sequence, sequence,
        "{phase}: cold CURRENT must retain the exact committed Raft watermark",
    );
    cold.engine
}

fn child_paths() -> Option<(PathBuf, PathBuf)> {
    if std::env::var(CHILD_MODE_ENV).ok().as_deref() != Some(CHILD_CASE) {
        return None;
    }
    let root = std::env::var_os(CHILD_ROOT_ENV)
        .map(PathBuf::from)
        .expect("committed Raft capacity child needs a parent-owned durable root");
    let handshake = std::env::var_os(CHILD_HANDSHAKE_ENV)
        .map(PathBuf::from)
        .expect("committed Raft capacity child needs a handshake path");
    Some((root, handshake))
}

async fn run_child(root: PathBuf, handshake: PathBuf) {
    fs::write(&handshake, CHILD_CASE).expect("record exact committed Raft capacity child entry");
    let store = Arc::new(
        SegmentRdbStore::new(root.join("segments"))
            .expect("open schema-only committed Raft capacity segment store"),
    );
    let cold = store
        .load_current_generation()
        .expect("cold-open schema-only Raft checkpoint")
        .expect("schema-only Raft checkpoint must publish CURRENT");
    assert_eq!(
        cold.sequence, CREATE_SEQUENCE,
        "child must begin exactly after the schema checkpoint",
    );
    assert_eq!(
        cold.engine
            .stats(COLLECTION)
            .expect("schema-only Raft collection stats")
            .documents_indexed,
        0,
        "schema checkpoint must not carry a hidden document",
    );

    let engine = cold.engine;
    let state_machine =
        EngineSm::new_with_segment_store(engine.clone(), CREATE_SEQUENCE, store.clone());
    let filler_ordinal = fill_checkpointable_local_work(&engine);
    let free_after_setup = PENDING_HARD_LIMIT_BYTES - pending_total_from_engine(&engine);
    assert!(
        free_after_setup <= INITIAL_MAX_FREE_BYTES,
        "public pending accounting must establish the selected pre-decode pressure: free={free_after_setup}",
    );
    assert_eq!(
        state_machine.applied_index(),
        CREATE_SEQUENCE,
        "local pressure must not manufacture a committed Raft watermark",
    );

    let command = Arc::new(target_command());
    let checkpoint_before = engine.metrics().segment_checkpoint_completed_total.get();
    apply_before_caller_checkpoint(
        state_machine.clone(),
        engine.clone(),
        store.as_ref(),
        command.clone(),
    );
    assert_eq!(
        state_machine.applied_index(),
        TARGET_SEQUENCE,
        "one completed fitting foreign command must advance the Raft watermark exactly once",
    );
    assert!(
        engine.metrics().segment_checkpoint_completed_total.get() > checkpoint_before,
        "capacity progress must complete a real maintenance checkpoint before generic Raft apply returns",
    );
    assert_live_state(
        &engine,
        filler_ordinal,
        "live committed Raft capacity progress",
    );
    assert_public_pending_budget(
        engine.clone(),
        TARGET_SEQUENCE,
        "live committed Raft capacity progress",
    )
    .await;

    // This caller-owned checkpoint happens only after generic committed apply
    // completed. It proves durable recovery, but it must never be needed for
    // the capacity progress assertion above.
    let final_cold = checkpoint_and_cold(
        store.as_ref(),
        &engine,
        TARGET_SEQUENCE,
        "final committed Raft capacity checkpoint",
    );
    assert_live_state(
        &final_cold,
        filler_ordinal,
        "cold committed Raft capacity progress",
    );
    assert_public_pending_budget(
        final_cold.clone(),
        TARGET_SEQUENCE,
        "cold committed Raft capacity progress",
    )
    .await;

    // The peer-owned command stays intact while the valid apply waits. A
    // malformed generic root is refused after the successful durable state is
    // checked, so the closed byte boundary cannot hide the behavior oracle.
    let mut malformed = command.as_ref().clone();
    malformed[0] = 0xff;
    let malformed_result = state_machine.apply(MALFORMED_SEQUENCE, &malformed);
    assert!(
        malformed_result.is_err(),
        "a malformed fitting generic foreign Raft command must be refused before publication",
    );
    assert_eq!(
        state_machine.applied_index(),
        TARGET_SEQUENCE,
        "a refused malformed generic foreign command must not advance the Raft watermark",
    );
    assert_live_state(
        &engine,
        filler_ordinal,
        "malformed generic foreign command leaves live state unchanged",
    );
}

async fn run_isolated_child(root: &Path) {
    let child_workspace =
        tempfile::tempdir().expect("create parent-owned Raft capacity child workspace");
    let child_tmp = child_workspace.path().join("tmp");
    fs::create_dir(&child_tmp).expect("create parent-owned Raft capacity TMPDIR");
    let handshake = child_workspace.path().join("entered-case");
    let stdout_path = child_workspace.path().join("child.stdout");
    let stderr_path = child_workspace.path().join("child.stderr");
    let executable = std::env::current_exe().expect("locate current Raft capacity e2e executable");
    let child = Command::new(executable)
        .env(CHILD_MODE_ENV, CHILD_CASE)
        .env(CHILD_ROOT_ENV, root)
        .env(CHILD_HANDSHAKE_ENV, &handshake)
        .env("TMPDIR", &child_tmp)
        .env("TEMP", &child_tmp)
        .env("TMP", &child_tmp)
        .arg(TEST_NAME)
        .arg("--exact")
        .arg("--nocapture")
        .arg("--test-threads=1")
        .stdout(Stdio::from(
            File::create(&stdout_path).expect("create isolated Raft capacity child stdout"),
        ))
        .stderr(Stdio::from(
            File::create(&stderr_path).expect("create isolated Raft capacity child stderr"),
        ))
        .spawn()
        .expect("spawn isolated committed Raft capacity child");
    let mut child = ChildCleanup(Some(child));
    let deadline = Instant::now() + CHILD_WATCHDOG;
    loop {
        match child
            .0
            .as_mut()
            .expect("child remains owned until it exits")
            .try_wait()
        {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => tokio::time::sleep(POLL_INTERVAL).await,
            Ok(None) => {
                let mut raw = child.0.take().expect("timed-out child remains owned");
                let _ = raw.kill();
                let status = raw.wait().expect("reap killed Raft capacity child");
                let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
                let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
                panic!(
                    "valid fitting committed Raft child did not finish before cleanup watchdog; status={status}; stdout={stdout}; stderr={stderr}",
                );
            }
            Err(error) => panic!("poll isolated committed Raft capacity child: {error}"),
        }
    }
    let status = child
        .0
        .take()
        .expect("exited child remains owned")
        .wait()
        .expect("reap exited committed Raft capacity child");
    let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
    let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
    let entered = fs::read_to_string(&handshake).unwrap_or_else(|error| {
        panic!(
            "isolated committed Raft capacity child never entered its test body: {error}; stdout={stdout}; stderr={stderr}",
        )
    });
    assert_eq!(
        entered, CHILD_CASE,
        "isolated child must run the intended committed Raft capacity case",
    );
    assert!(
        status.success(),
        "isolated committed Raft capacity child failed: status={status}; stdout={stdout}; stderr={stderr}",
    );
}

fn create_schema_checkpoint(root: &Path) {
    let store = Arc::new(
        SegmentRdbStore::new(root.join("segments"))
            .expect("open schema-only committed Raft capacity store"),
    );
    let engine = Arc::new(Engine::new());
    let state_machine = EngineSm::new_with_segment_store(engine.clone(), 0, store.clone());
    let create = create_command();
    let result = state_machine.apply(CREATE_SEQUENCE, &create);
    assert!(
        result.is_ok(),
        "schema-only foreign Raft command must apply before capacity fixture: {}",
        result
            .as_ref()
            .err()
            .map(|error| format!("{error:#}"))
            .unwrap_or_default(),
    );
    assert_eq!(
        state_machine.applied_index(),
        CREATE_SEQUENCE,
        "schema-only foreign Raft command must establish the exact base watermark",
    );
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("schema-only Raft collection stats")
            .documents_indexed,
        0,
        "schema-only checkpoint setup must not retain a document",
    );
    let saved = store
        .save_with_sequence(&engine, CREATE_SEQUENCE)
        .expect("clear schema admission charge before child setup");
    assert_eq!(
        saved, CREATE_SEQUENCE,
        "schema-only checkpoint must retain its exact watermark",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raft_committed_generic_apply_makes_capacity_progress_before_any_caller_checkpoint() {
    if let Some((root, handshake)) = child_paths() {
        run_child(root, handshake).await;
        return;
    }

    let root = tempfile::tempdir().expect("create run-scoped committed Raft capacity root");
    create_schema_checkpoint(root.path());
    run_isolated_child(root.path()).await;
}
