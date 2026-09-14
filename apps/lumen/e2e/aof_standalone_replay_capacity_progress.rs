//! Black-box contract for standalone generic AOF replay under a full pending
//! change budget.
//!
//! Each child cold-opens a schema-only segment checkpoint and then creates real
//! checkpointable local ReplaceDocs changes until the shared pending budget has
//! little free space. It calls the public replay_aof_into helper without a
//! coordinator, PendingChangeSpill, or serving checkpoint driver.
//!
//! The initial case has one valid 2 MiB Keyword token and leaves at most 1 MiB
//! free. The scanner itself must therefore ask for capacity progress. The
//! decoded-growth case has eight 256 KiB Keyword values and leaves 2 to 4 MiB
//! free. Its scanner fits, but its owned generic decode does not. Each AOF
//! record fits by itself and stays below the mapped-record route threshold.
//!
//! A correct replay starts independent maintenance before either wait. It
//! returns its committed watermark without a caller checkpoint, keeps both the
//! pre-existing local row and AOF row queryable, and survives a later caller
//! checkpoint and cold open. The short observation and child watchdog only
//! bound test cleanup. They are not a product latency promise.
//!
//! # Facets
//!
//! - Behavior: aof_standalone_replay_capacity_progress.rs:346-410 constructs
//!   real generic AOF frames and :878-908 creates a schema-only cold base.
//!   :635-689 requires replay to return before caller cleanup. :725-792
//!   requires the exact source watermark, prior local value, AOF value,
//!   maintenance checkpoint, final checkpoint, cold open, and covered-suffix
//!   skip. These assertions
//!   cover apps/lumen/src/aof.rs:333-357 and
//!   apps/lumen/src/segment_capacity.rs:312-348.
//! - Security: :571-579 pre-creates the predictable first temporary spill
//!   directory name and :747-753 requires its sentinel to remain unchanged
//!   after fallback maintenance. This closes the filesystem collision boundary
//!   introduced by apps/lumen/src/segment_checkpoint.rs:349-386. The unchanged
//!   AOF format boundary remains covered by
//!   apps/lumen/e2e/aof_oversized_committed_replace.rs:797-868, which refuses a
//!   CRC-valid corrupt generic ReplaceDocs frame before rows or watermark
//!   publication.
//! - Performance: apps/lumen/docs/indexing.md:264-276 says, verbatim,
//!   "Pending active, frozen, and reserved changes have a 256 MiB budget."
//!   :471-490 reads public pending total and high-water gauges after replay
//!   and cold open, at :741-746 and :780-785, and requires both to stay
//!   within that budget.
//!   Fixture AOF and query bytes are outside these gauges. This contract names
//!   no latency budget.
//!
//! # Root negative control
//!
//! Remove the generic replay capacity-owner ensure call before either Full wait
//! in apps/lumen/src/aof.rs:333-357. The child assertion at :660-665 must fail
//! after its cleanup checkpoint proves that the valid committed record had been
//! waiting. Restore every changed production file by SHA-256 before another
//! gate.
//!
//! Target gate: cargo test -p lumen --test aof_standalone_replay_capacity_progress -- --nocapture.
//! Full declared behavior gate: cargo test -p lumen.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use axum_test::TestServer;

use lumen::aof::{replay_aof_into, AofReader, AofWriter};
use lumen::api::{router, AppState};
use lumen::auth::AuthConfig;
use lumen::coordinator::WriteSink;
use lumen::log_entry::RaftLogEntry;
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::{ApplyOutcome, Engine};
use lumen::types::{
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, QueryNode, ReplaceDocItem,
    ReplaceDocsRequest, SearchRequest, TermQuery,
};
use lumen::wal::WalRecord;

const COLLECTION: &str = "aof-standalone-capacity-progress";
const FILLER_FIELD: &str = "filler";
const INITIAL_FIELD: &str = "initial";
const GROWTH_FIELD_PREFIX: &str = "growth-";
const FILLER_ID: &str = "checkpointable-local-predecessor";
const TARGET_ID: &str = "committed-aof-target";
const CREATE_SEQUENCE: u64 = 1;
const TARGET_SEQUENCE: u64 = 2;
const PENDING_HARD_LIMIT_BYTES: u64 = 256 * 1024 * 1024;
const MAPPED_ROUTE_MINIMUM_BYTES: u64 = 32 * 1024 * 1024;
const FRAME_HEADER_BYTES: u64 = 16;
const INITIAL_TOKEN_BYTES: usize = 2 * 1024 * 1024;
const GROWTH_FIELD_BYTES: usize = 256 * 1024;
const GROWTH_FIELD_COUNT: usize = 8;
const FILLER_VALUE_BYTES: usize = 64 * 1024;
const INITIAL_MAX_FREE_BYTES: u64 = 1024 * 1024;
const GROWTH_MIN_FREE_BYTES: u64 = 2 * 1024 * 1024;
const GROWTH_MAX_FREE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_FILLER_WRITES: usize = 6_000;
const PROGRESS_OBSERVATION: Duration = Duration::from_secs(5);
const CLEANUP_WAIT: Duration = Duration::from_secs(30);
const CHILD_WATCHDOG: Duration = Duration::from_secs(120);
const POLL_INTERVAL: Duration = Duration::from_millis(25);
const CHILD_MODE_ENV: &str = "LUMEN_AOF_CAPACITY_PROGRESS_CHILD";
const CHILD_CASE_ENV: &str = "LUMEN_AOF_CAPACITY_PROGRESS_CASE";
const CHILD_ROOT_ENV: &str = "LUMEN_AOF_CAPACITY_PROGRESS_ROOT";
const CHILD_AOF_ENV: &str = "LUMEN_AOF_CAPACITY_PROGRESS_AOF";
const CHILD_HANDSHAKE_ENV: &str = "LUMEN_AOF_CAPACITY_PROGRESS_HANDSHAKE";
const TEST_NAME: &str = "standalone_aof_replay_makes_capacity_progress_before_any_caller_checkpoint";
const SPILL_SENTINEL: &str = "do-not-reuse-preexisting-temp-path";

#[derive(Clone, Copy, Debug)]
enum CapacityCase {
    InitialScanner,
    DecodedGrowth,
}

impl CapacityCase {
    const ALL: [Self; 2] = [Self::DecodedGrowth, Self::InitialScanner];

    fn name(self) -> &'static str {
        match self {
            Self::InitialScanner => "initial-scanner",
            Self::DecodedGrowth => "decoded-growth",
        }
    }

    fn from_name(name: &str) -> Self {
        match name {
            "initial-scanner" => Self::InitialScanner,
            "decoded-growth" => Self::DecodedGrowth,
            other => panic!("unknown isolated capacity case: {other}"),
        }
    }

    fn minimum_free(self) -> u64 {
        match self {
            Self::InitialScanner => 0,
            Self::DecodedGrowth => GROWTH_MIN_FREE_BYTES,
        }
    }

    fn maximum_free(self) -> u64 {
        match self {
            Self::InitialScanner => INITIAL_MAX_FREE_BYTES,
            Self::DecodedGrowth => GROWTH_MAX_FREE_BYTES,
        }
    }
}

/// Supplies only the replay watermark to public metrics. It cannot submit a
/// write, so creating this server never resets the cold Engine barrier.
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

#[derive(Clone, Copy)]
struct FrameSpan {
    start: u64,
    payload_bytes: u64,
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

fn field_name(index: usize) -> String {
    format!("{GROWTH_FIELD_PREFIX}{index:02}")
}

fn schema() -> CreateCollectionRequest {
    let mut fields = BTreeMap::from([
        (FILLER_FIELD.to_owned(), keyword_spec()),
        (INITIAL_FIELD.to_owned(), keyword_spec()),
    ]);
    for index in 0..GROWTH_FIELD_COUNT {
        fields.insert(field_name(index), keyword_spec());
    }
    CreateCollectionRequest { fields }
}

fn create_record() -> WalRecord {
    WalRecord::new(RaftLogEntry::CreateCollection {
        collection_id: COLLECTION.to_owned(),
        req: schema(),
    })
}

fn marked_ascii(prefix: &str, ordinal: usize, bytes: usize) -> String {
    let head = format!("{prefix}-{ordinal:04}:");
    let tail = format!(":end-{prefix}-{ordinal:04}");
    assert!(
        head.len() + tail.len() < bytes,
        "fixture markers must leave a real bounded Keyword value",
    );
    let mut value = vec![b'x'; bytes];
    value[..head.len()].copy_from_slice(head.as_bytes());
    value[bytes - tail.len()..].copy_from_slice(tail.as_bytes());
    String::from_utf8(value).expect("capacity fixture Keyword must be valid ASCII")
}

fn filler_value(ordinal: usize) -> String {
    marked_ascii("checkpointable-local-filler", ordinal, FILLER_VALUE_BYTES)
}

fn target_value(case: CapacityCase, index: usize) -> String {
    match case {
        CapacityCase::InitialScanner => {
            assert_eq!(index, 0, "initial target has exactly one value");
            marked_ascii("committed-initial-token", index, INITIAL_TOKEN_BYTES)
        }
        CapacityCase::DecodedGrowth => marked_ascii(
            "committed-decoded-growth",
            index,
            GROWTH_FIELD_BYTES,
        ),
    }
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

fn target_record(case: CapacityCase) -> WalRecord {
    let fields = match case {
        CapacityCase::InitialScanner => BTreeMap::from([(
            INITIAL_FIELD.to_owned(),
            FieldValue::String(target_value(case, 0)),
        )]),
        CapacityCase::DecodedGrowth => (0..GROWTH_FIELD_COUNT)
            .map(|index| {
                (
                    field_name(index),
                    FieldValue::String(target_value(case, index)),
                )
            })
            .collect(),
    };
    WalRecord::new(RaftLogEntry::ReplaceDocs {
        collection_id: COLLECTION.to_owned(),
        req: ReplaceDocsRequest {
            docs: vec![ReplaceDocItem {
                external_id: TARGET_ID.to_owned(),
                version: Some(100),
                fields,
            }],
        },
    })
}

fn frame_payload_bytes(path: &Path, start: u64) -> u64 {
    let mut file = File::open(path).expect("open AOF frame");
    file.seek(SeekFrom::Start(start + 8))
        .expect("seek AOF frame length");
    let mut length = [0_u8; 4];
    file.read_exact(&mut length).expect("read AOF frame length");
    u64::from(u32::from_le_bytes(length))
}

fn frame_payload_prefix(path: &Path, start: u64) -> [u8; 4] {
    let mut file = File::open(path).expect("open AOF payload");
    file.seek(SeekFrom::Start(start + FRAME_HEADER_BYTES))
        .expect("seek AOF payload prefix");
    let mut prefix = [0_u8; 4];
    file.read_exact(&mut prefix).expect("read AOF payload prefix");
    prefix
}

fn append_frame(
    writer: &mut AofWriter,
    path: &Path,
    sequence: u64,
    record: &WalRecord,
    phase: &str,
) -> FrameSpan {
    writer
        .flush()
        .unwrap_or_else(|error| panic!("{phase}: flush earlier AOF frames: {error:#}"));
    let start = fs::metadata(path)
        .unwrap_or_else(|error| panic!("{phase}: measure AOF before append: {error:#}"))
        .len();
    writer
        .append(sequence, record)
        .unwrap_or_else(|error| panic!("{phase}: append AOF frame: {error:#}"));
    writer
        .flush()
        .unwrap_or_else(|error| panic!("{phase}: flush appended AOF frame: {error:#}"));
    let payload_bytes = frame_payload_bytes(path, start);
    let after = fs::metadata(path)
        .unwrap_or_else(|error| panic!("{phase}: measure AOF after append: {error:#}"))
        .len();
    assert_eq!(
        after,
        start + FRAME_HEADER_BYTES + payload_bytes,
        "{phase}: AofWriter must persist one complete frame",
    );
    FrameSpan {
        start,
        payload_bytes,
    }
}

fn write_schema_aof(path: &Path) {
    let mut writer = AofWriter::open(path).expect("open schema AOF writer");
    let record = create_record();
    append_frame(
        &mut writer,
        path,
        CREATE_SEQUENCE,
        &record,
        "append schema AOF frame",
    );
    drop(record);
    writer
        .sync_strict()
        .expect("strict-sync schema AOF frame");
}

fn write_target_aof(path: &Path, case: CapacityCase) -> u64 {
    let mut writer = AofWriter::open(path).expect("open target AOF writer");
    let create = create_record();
    append_frame(
        &mut writer,
        path,
        CREATE_SEQUENCE,
        &create,
        "append target AOF schema frame",
    );
    drop(create);
    let target = target_record(case);
    let target_frame = append_frame(
        &mut writer,
        path,
        TARGET_SEQUENCE,
        &target,
        "append fitting committed generic AOF target",
    );
    drop(target);
    writer
        .sync_strict()
        .expect("strict-sync fitting committed generic AOF target");
    assert!(
        target_frame.payload_bytes < MAPPED_ROUTE_MINIMUM_BYTES,
        "{}: fitting committed record must stay below the mapped-record route: payload_bytes={}",
        case.name(),
        target_frame.payload_bytes,
    );
    assert_ne!(
        frame_payload_prefix(path, target_frame.start),
        *b"LWAL",
        "{}: target must exercise generic CBOR replay rather than fast LWAL",
        case.name(),
    );
    match case {
        CapacityCase::InitialScanner => assert!(
            target_frame.payload_bytes >= INITIAL_TOKEN_BYTES as u64,
            "initial scanner target must contain its real 2 MiB Keyword token",
        ),
        CapacityCase::DecodedGrowth => assert!(
            target_frame.payload_bytes
                >= (GROWTH_FIELD_BYTES * GROWTH_FIELD_COUNT) as u64,
            "growth target must retain all eight 256 KiB Keyword values",
        ),
    }
    fs::metadata(path)
        .expect("measure strict-synced fitting AOF source")
        .len()
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
        "public metrics must publish exactly one {name} sample: {metrics}",
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

fn fill_checkpointable_local_work(engine: &Arc<Engine>, case: CapacityCase) -> usize {
    for ordinal in 0..MAX_FILLER_WRITES {
        let total = pending_total_from_engine(engine);
        assert!(
            total <= PENDING_HARD_LIMIT_BYTES,
            "{}: direct local setup must never exceed the documented pending budget: total={total}",
            case.name(),
        );
        let free = PENDING_HARD_LIMIT_BYTES - total;
        if (case.minimum_free()..=case.maximum_free()).contains(&free) {
            return ordinal.saturating_sub(1);
        }
        assert!(
            free > case.maximum_free(),
            "{}: 64 KiB local setup step overshot the required free-space interval {}..={}: free={free}",
            case.name(),
            case.minimum_free(),
            case.maximum_free(),
        );
        let response = engine
            .replace_docs(COLLECTION, local_filler_request(ordinal))
            .unwrap_or_else(|error| {
                panic!(
                    "{}: direct local setup must create checkpointable work before capacity becomes full: {error:#}",
                    case.name(),
                )
            });
        assert_eq!(
            response.results.len(),
            1,
            "{}: each direct local setup replacement must produce one public result",
            case.name(),
        );
    }
    panic!(
        "{}: setup did not reach its required free-space interval after {MAX_FILLER_WRITES} real local records",
        case.name(),
    );
}

fn assert_live_state(engine: &Engine, case: CapacityCase, filler_ordinal: usize, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("capacity replay collection stats")
            .documents_indexed,
        2,
        "{phase}: one checkpointable local predecessor and one committed AOF target must be live",
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
    match case {
        CapacityCase::InitialScanner => assert_eq!(
            term_ids(engine, INITIAL_FIELD, target_value(case, 0), phase),
            singleton(TARGET_ID),
            "{phase}: the valid committed 2 MiB AOF Keyword must be indexed whole",
        ),
        CapacityCase::DecodedGrowth => {
            for index in 0..GROWTH_FIELD_COUNT {
                assert_eq!(
                    term_ids(engine, &field_name(index), target_value(case, index), phase),
                    singleton(TARGET_ID),
                    "{phase}: generic decoded growth field {index} must keep its exact committed bytes",
                );
            }
        }
    }
}

fn reserve_spill_collision() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "lumen-pending-spill-{}-0",
        std::process::id()
    ));
    fs::write(&path, SPILL_SENTINEL)
        .unwrap_or_else(|error| panic!("pre-create fallback collision sentinel {}: {error}", path.display()));
    path
}

fn assert_source_complete(path: &Path, expected_len: u64, phase: &str) {
    assert_eq!(
        fs::metadata(path)
            .unwrap_or_else(|error| panic!("{phase}: inspect source AOF after replay: {error:#}"))
            .len(),
        expected_len,
        "{phase}: replay must not rewrite or truncate committed AOF source",
    );
    let mut sequences = Vec::new();
    let maximum = AofReader::replay(path, 0, |sequence, _| sequences.push(sequence))
        .unwrap_or_else(|error| panic!("{phase}: AOF reader must accept every committed frame: {error:#}"));
    assert_eq!(
        sequences,
        vec![CREATE_SEQUENCE, TARGET_SEQUENCE],
        "{phase}: source AOF must retain exactly one schema and one committed target in order",
    );
    assert_eq!(
        maximum, TARGET_SEQUENCE,
        "{phase}: source AOF must retain the exact committed head",
    );
}

fn child_inputs() -> Option<(CapacityCase, PathBuf, PathBuf, PathBuf)> {
    if std::env::var(CHILD_MODE_ENV).ok().as_deref() != Some("1") {
        return None;
    }
    let case = std::env::var(CHILD_CASE_ENV)
        .map(|name| CapacityCase::from_name(&name))
        .expect("isolated capacity child needs a valid case name");
    let root = std::env::var_os(CHILD_ROOT_ENV)
        .map(PathBuf::from)
        .expect("isolated capacity child needs durable root");
    let aof = std::env::var_os(CHILD_AOF_ENV)
        .map(PathBuf::from)
        .expect("isolated capacity child needs AOF path");
    let handshake = std::env::var_os(CHILD_HANDSHAKE_ENV)
        .map(PathBuf::from)
        .expect("isolated capacity child needs handshake path");
    Some((case, root, aof, handshake))
}

fn replay_before_caller_cleanup(
    engine: Arc<Engine>,
    store: &SegmentRdbStore,
    aof_path: &Path,
    case: CapacityCase,
) -> u64 {
    let (sender, receiver) = mpsc::sync_channel(1);
    let replay_engine = engine.clone();
    let replay_path = aof_path.to_owned();
    let worker = thread::spawn(move || {
        let _ = sender.send(replay_aof_into(&replay_engine, replay_path, CREATE_SEQUENCE));
    });

    let completed_before_cleanup = match receiver.recv_timeout(PROGRESS_OBSERVATION) {
        Ok(replay) => {
            worker.join().expect("replay worker must not panic");
            replay
        }
        Err(mpsc::RecvTimeoutError::Timeout) => match receiver.try_recv() {
            Ok(replay) => {
                worker.join().expect("replay worker must not panic");
                replay
            }
            Err(mpsc::TryRecvError::Empty) => {
                let cleanup_sequence = store
                    .save_with_sequence(&engine, CREATE_SEQUENCE)
                    .expect("caller cleanup checkpoint must release retained baseline work");
                assert_eq!(
                    cleanup_sequence, CREATE_SEQUENCE,
                    "cleanup checkpoint must retain the schema watermark while it only releases local setup work",
                );
                let resumed = receiver.recv_timeout(CLEANUP_WAIT).unwrap_or_else(|error| {
                    panic!(
                        "{}: retained replay did not resume after bounded cleanup checkpoint: {error}",
                        case.name(),
                    )
                });
                worker.join().expect("resumed replay worker must not panic");
                assert!(
                    false,
                    "{}: valid committed generic AOF replay waited for a caller checkpoint instead of starting independent capacity maintenance; cleanup checkpoint released the retained source and replay result was {resumed:?}",
                    case.name(),
                );
                unreachable!("the behavior assertion above always panics");
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                worker.join().expect("disconnected replay worker must not panic");
                panic!(
                    "{}: replay worker disconnected before returning a public result",
                    case.name(),
                );
            }
        },
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            worker.join().expect("disconnected replay worker must not panic");
            panic!(
                "{}: replay worker disconnected before returning a public result",
                case.name(),
            );
        }
    };

    completed_before_cleanup.unwrap_or_else(|error| {
        panic!(
            "{}: valid committed generic AOF replay returned an error before caller cleanup: {error:#}",
            case.name(),
        )
    })
}

async fn run_child(case: CapacityCase, root: PathBuf, aof_path: PathBuf, handshake: PathBuf) {
    fs::write(&handshake, case.name()).expect("record exact capacity child case");
    let store = SegmentRdbStore::new(root.join("segments"))
        .expect("open schema-only checkpoint store without configured maintenance");
    let cold = store
        .load_current_generation()
        .expect("cold-open schema-only checkpoint")
        .expect("schema-only checkpoint must publish CURRENT");
    assert_eq!(
        cold.sequence, CREATE_SEQUENCE,
        "{}: child must begin exactly after schema checkpoint",
        case.name(),
    );
    assert_eq!(
        cold.engine
            .stats(COLLECTION)
            .expect("schema-only collection stats")
            .documents_indexed,
        0,
        "{}: schema checkpoint must not carry a hidden document",
        case.name(),
    );

    let filler_ordinal = fill_checkpointable_local_work(&cold.engine, case);
    let free_after_setup = PENDING_HARD_LIMIT_BYTES - pending_total_from_engine(&cold.engine);
    assert!(
        (case.minimum_free()..=case.maximum_free()).contains(&free_after_setup),
        "{}: public pending accounting must establish the selected free-space condition: free={free_after_setup}",
        case.name(),
    );
    let checkpoint_before = cold.engine.metrics().segment_checkpoint_completed_total.get();
    let collision = reserve_spill_collision();
    let replayed = replay_before_caller_cleanup(cold.engine.clone(), &store, &aof_path, case);
    assert_eq!(
        replayed, TARGET_SEQUENCE,
        "{}: standalone replay must advance exactly through its one committed target",
        case.name(),
    );
    assert!(
        cold.engine.metrics().segment_checkpoint_completed_total.get() > checkpoint_before,
        "{}: capacity progress must complete a real maintenance checkpoint before replay returns",
        case.name(),
    );
    assert_live_state(
        &cold.engine,
        case,
        filler_ordinal,
        "live standalone capacity replay",
    );
    assert_public_pending_budget(
        cold.engine.clone(),
        TARGET_SEQUENCE,
        "live standalone capacity replay",
    )
    .await;
    assert_eq!(
        fs::read_to_string(&collision)
            .unwrap_or_else(|error| panic!("read collision sentinel {}: {error}", collision.display())),
        SPILL_SENTINEL,
        "{}: fallback must not reuse or overwrite a pre-existing temporary spill path",
        case.name(),
    );

    // This caller-owned checkpoint happens only after replay returned. It proves
    // durable state but must never be needed for capacity progress above.
    let saved = store
        .save_with_sequence(&cold.engine, TARGET_SEQUENCE)
        .expect("checkpoint only after standalone replay completed");
    assert_eq!(
        saved, TARGET_SEQUENCE,
        "{}: final caller checkpoint must retain completed replay watermark",
        case.name(),
    );
    let final_cold = store
        .load_current_generation()
        .expect("cold-open final standalone replay checkpoint")
        .expect("final standalone replay checkpoint must publish CURRENT");
    assert_eq!(
        final_cold.sequence, TARGET_SEQUENCE,
        "{}: cold-open checkpoint must retain committed target watermark",
        case.name(),
    );
    assert_live_state(
        &final_cold.engine,
        case,
        filler_ordinal,
        "cold standalone capacity replay",
    );
    assert_public_pending_budget(
        final_cold.engine.clone(),
        TARGET_SEQUENCE,
        "cold standalone capacity replay",
    )
    .await;
    let covered = replay_aof_into(&final_cold.engine, &aof_path, final_cold.sequence)
        .expect("checkpoint-covered replay must not wait or fail");
    assert_eq!(
        covered, 0,
        "{}: checkpoint-covered AOF suffix must be skipped without advancing watermark",
        case.name(),
    );
}

async fn run_isolated_child(case: CapacityCase, root: &Path, aof_path: &Path) {
    let child_workspace = tempfile::tempdir().expect("create parent-owned child workspace");
    let child_tmp = child_workspace.path().join("tmp");
    fs::create_dir(&child_tmp).expect("create parent-owned child TMPDIR");
    let handshake = child_workspace.path().join("entered-case");
    let stdout_path = child_workspace.path().join("child.stdout");
    let stderr_path = child_workspace.path().join("child.stderr");
    let executable = std::env::current_exe().expect("locate current e2e executable");
    let child = Command::new(executable)
        .env(CHILD_MODE_ENV, "1")
        .env(CHILD_CASE_ENV, case.name())
        .env(CHILD_ROOT_ENV, root)
        .env(CHILD_AOF_ENV, aof_path)
        .env(CHILD_HANDSHAKE_ENV, &handshake)
        .env("TMPDIR", &child_tmp)
        .env("TEMP", &child_tmp)
        .env("TMP", &child_tmp)
        .arg(TEST_NAME)
        .arg("--exact")
        .arg("--nocapture")
        .arg("--test-threads=1")
        .stdout(Stdio::from(
            File::create(&stdout_path).expect("create isolated child stdout"),
        ))
        .stderr(Stdio::from(
            File::create(&stderr_path).expect("create isolated child stderr"),
        ))
        .spawn()
        .expect("spawn isolated standalone AOF replay child");
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
                let status = raw.wait().expect("reap killed capacity child");
                let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
                let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
                panic!(
                    "{} child did not finish before cleanup watchdog; status={status}; stdout={stdout}; stderr={stderr}",
                    case.name(),
                );
            }
            Err(error) => panic!("poll isolated {} child: {error}", case.name()),
        }
    }
    let status = child
        .0
        .take()
        .expect("exited child remains owned")
        .wait()
        .expect("reap exited capacity child");
    let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
    let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
    let entered = fs::read_to_string(&handshake).unwrap_or_else(|error| {
        panic!(
            "{} child never entered its test body: {error}; stdout={stdout}; stderr={stderr}",
            case.name(),
        )
    });
    assert_eq!(
        entered,
        case.name(),
        "{} child must run the intended capacity case",
        case.name(),
    );
    assert!(
        status.success(),
        "{} isolated AOF replay child failed: status={status}; stdout={stdout}; stderr={stderr}",
        case.name(),
    );
}

fn create_schema_checkpoint(root: &Path, schema_aof: &Path) {
    let store = SegmentRdbStore::new(root.join("segments"))
        .expect("open schema-only segment store");
    let engine = Arc::new(Engine::new());
    let replayed = replay_aof_into(&engine, schema_aof, 0)
        .expect("schema AOF replay must succeed before capacity fixture");
    assert_eq!(
        replayed, CREATE_SEQUENCE,
        "schema AOF replay must establish exact base sequence",
    );
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("schema collection stats")
            .documents_indexed,
        0,
        "schema checkpoint setup must not retain a document",
    );
    let saved = store
        .save_with_sequence(&engine, CREATE_SEQUENCE)
        .expect("clear schema admission charge before child setup");
    assert_eq!(
        saved, CREATE_SEQUENCE,
        "schema checkpoint must retain its exact watermark",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn standalone_aof_replay_makes_capacity_progress_before_any_caller_checkpoint() {
    if let Some((case, root, aof_path, handshake)) = child_inputs() {
        run_child(case, root, aof_path, handshake).await;
        return;
    }

    for case in CapacityCase::ALL {
        let root = tempfile::tempdir().expect("create run-scoped AOF capacity root");
        let schema_aof = root.path().join("schema.aof");
        let aof_path = root.path().join("committed.aof");
        write_schema_aof(&schema_aof);
        create_schema_checkpoint(root.path(), &schema_aof);
        let source_len = write_target_aof(&aof_path, case);
        run_isolated_child(case, root.path(), &aof_path).await;
        assert_source_complete(
            &aof_path,
            source_len,
            "parent source after isolated standalone replay",
        );
    }
}
