//! Black-box contract for borrowed oversized committed Hash AOF replay.
//!
//! The fixture first checkpoints a small Hash base. It then writes one valid
//! fast Index suffix through `AofWriter`. Its Hash value is `0x`, more than
//! 256 MiB of leading zeroes, and `42`. The value is legal 64-bit hex and must
//! become the small numeric hash `0x42`, not an owned 256 MiB change. The same
//! committed record also carries a small Keyword row. The test drops the
//! source record after `AofWriter` encodes it and never builds a second giant
//! input. Queries use `0x42` and small Keyword prefixes only.
//!
//! The behavior replay runs in a child process. A baseline owned decode may
//! refuse or wait behind admission. The parent kills that child at a bounded
//! cleanup deadline. This is cleanup, not a latency promise.
//!
//! # Facets
//!
//! - Behavior: aof_oversized_committed_hash.rs:493-621 and :810-1015 establish
//!   a Hash base, apply the valid oversized AOF suffix, check exact Hamming and
//!   Keyword-prefix state, incrementally checkpoint it, apply a versioned Hash
//!   update and a delete, then cold-open both later cuts. The state checks
//!   include 12 bytes while a live Hash row is materialized and zero after each
//!   cold mmap reopen. :1049-1084 makes a stuck or failed baseline red without
//!   leaving a child alive. These assertions cover
//!   apps/lumen/src/aof.rs:274-315 and the borrowed Hash planning and
//!   attachment path in apps/lumen/src/storage/committed_index_apply.rs:251-322,753-771.
//! - Security: aof_oversized_committed_hash.rs:752-789 writes a complete,
//!   CRC-valid but format-corrupt Hash AOF frame and requires refusal before
//!   its watermark or row becomes visible. It covers the persisted-byte
//!   boundary at apps/lumen/src/aof.rs:285-315. The Hash grammar itself is not
//!   widened: existing apps/lumen/e2e/hash_hamming.rs:163-179 rejects invalid
//!   hex at the public Hash input boundary.
//! - Performance: apps/lumen/docs/indexing.md:264-276 says, verbatim,
//!   "Pending active, frozen, and reserved changes have a 256 MiB budget."
//!   aof_oversized_committed_hash.rs:651-660 reads real public pending total and
//!   high-water metrics after each replay and checkpoint. Fixture and mapped AOF
//!   source bytes are outside those gauges. This contract sets no latency budget.
//!
//! # Root negative control
//!
//! After the borrowed Hash route is implemented, remove Hash planner routing
//! from the production committed-Index adapter. The child oversized-suffix
//! behavior assertion below must fail through the old owned decode/refusal or
//! its cleanup watchdog. Restore the correct source hash before any other
//! gate. Do not lower the fixture threshold or change expected searches.
//!
//! Gate: cargo test -p lumen --test aof_oversized_committed_hash -- --nocapture.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use axum_test::TestServer;

use lumen::aof::{replay_aof_into, AofWriter};
use lumen::api::{router, AppState};
use lumen::auth::AuthConfig;
use lumen::coordinator::WriteSink;
use lumen::log_entry::RaftLogEntry;
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::{ApplyOutcome, Engine};
use lumen::types::{
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, HammingQuery, IndexItem,
    IndexRequest, PrefixQuery, QueryNode, SearchRequest, MAX_INDEX_BATCH_SIZE,
};
use lumen::wal::WalRecord;

const COLLECTION: &str = "oversized-aof-committed-hash";
const HASH_FIELD: &str = "sig";
const KEYWORD_FIELD: &str = "kind";
const PRIMARY_ID: &str = "oversized-hash-primary";
const DELETED_ID: &str = "oversized-hash-delete";
const BASE_KEYWORD_PREFIX: &str = "oversized-hash-base-";
const GIANT_KEYWORD_PREFIX: &str = "oversized-hash-giant-";
const UPDATED_KEYWORD_PREFIX: &str = "oversized-hash-updated-";
const DELETED_KEYWORD_PREFIX: &str = "oversized-hash-delete-";
const HUGE_ZERO_BYTES: usize = 256 * 1024 * 1024 + 64;
const PENDING_HARD_LIMIT_BYTES: u64 = 256 * 1024 * 1024;
const FRAME_HEADER_BYTES: u64 = 16;
const CREATE_SEQUENCE: u64 = 1;
const BASE_SEQUENCE: u64 = 2;
const OVERSIZED_SEQUENCE: u64 = 3;
const UPDATE_SEQUENCE: u64 = 4;
const DELETE_SEQUENCE: u64 = 5;
const BASE_VERSION: u64 = 10;
const OVERSIZED_VERSION: u64 = BASE_VERSION + 1;
const UPDATED_VERSION: u64 = OVERSIZED_VERSION + 1;
const REPLAY_WATCHDOG: Duration = Duration::from_secs(90);
const POLL_INTERVAL: Duration = Duration::from_millis(25);
const CHILD_MODE_ENV: &str = "LUMEN_AOF_OVERSIZED_HASH_CHILD";
const CHILD_ROOT_ENV: &str = "LUMEN_AOF_OVERSIZED_HASH_ROOT";
const CHILD_AOF_ENV: &str = "LUMEN_AOF_OVERSIZED_HASH_AOF";
const CHILD_HANDSHAKE_ENV: &str = "LUMEN_AOF_OVERSIZED_HASH_HANDSHAKE";
const CHILD_CASE: &str = "replay";
const TEST_NAME: &str = "oversized_committed_hash_aof_replays_suffixes_checkpoints_and_cold_opens";

/// Supplies the actual replay watermark without constructing a fresh
/// coordinator, which would initialize this Engine at sequence zero.
struct ReadOnlyMetricsSink {
    applied_sequence: u64,
}

#[async_trait]
impl WriteSink for ReadOnlyMetricsSink {
    async fn submit(&self, _entry: RaftLogEntry) -> anyhow::Result<ApplyOutcome> {
        anyhow::bail!("the read-only metrics server must not submit a mutation");
    }

    fn applied_seq(&self) -> u64 {
        self.applied_sequence
    }
}

/// Owns an isolated replay process until it has exited. A baseline admission
/// loop cannot leak a live child into a later test.
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
}

fn field_spec(field_type: FieldType) -> FieldSpec {
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

fn schema() -> CreateCollectionRequest {
    CreateCollectionRequest {
        fields: BTreeMap::from([
            (HASH_FIELD.to_owned(), field_spec(FieldType::Hash)),
            (KEYWORD_FIELD.to_owned(), field_spec(FieldType::Keyword)),
        ]),
    }
}

fn create_entry() -> RaftLogEntry {
    RaftLogEntry::CreateCollection {
        collection_id: COLLECTION.to_owned(),
        req: schema(),
    }
}

fn item(external_id: &str, field: &str, value: FieldValue, version: Option<u64>) -> IndexItem {
    IndexItem {
        external_id: external_id.to_owned(),
        field: field.to_owned(),
        value,
        version,
    }
}

fn giant_leading_zero_hash() -> String {
    assert!(
        HUGE_ZERO_BYTES as u64 > PENDING_HARD_LIMIT_BYTES,
        "the fixture must contain more than 256 MiB of leading zeroes",
    );
    let mut bytes = Vec::with_capacity(2 + HUGE_ZERO_BYTES + 2);
    bytes.extend_from_slice(b"0x");
    bytes.resize(2 + HUGE_ZERO_BYTES, b'0');
    bytes.extend_from_slice(b"42");
    String::from_utf8(bytes).expect("the giant leading-zero Hash fixture is valid ASCII")
}

fn base_hash_record() -> WalRecord {
    let items = vec![
        item(
            PRIMARY_ID,
            HASH_FIELD,
            FieldValue::String("0x41".to_owned()),
            Some(BASE_VERSION),
        ),
        item(
            PRIMARY_ID,
            KEYWORD_FIELD,
            FieldValue::String(format!("{BASE_KEYWORD_PREFIX}primary")),
            Some(BASE_VERSION),
        ),
        item(
            DELETED_ID,
            KEYWORD_FIELD,
            FieldValue::String(format!("{DELETED_KEYWORD_PREFIX}row")),
            Some(BASE_VERSION),
        ),
    ];
    assert!(
        items.len() <= MAX_INDEX_BATCH_SIZE,
        "the pre-existing Hash base must stay within the public 1,000-item Index limit",
    );
    WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items,
            request_id: None,
        },
    })
}

fn oversized_hash_record() -> WalRecord {
    let items = vec![
        item(
            PRIMARY_ID,
            HASH_FIELD,
            FieldValue::String(giant_leading_zero_hash()),
            Some(OVERSIZED_VERSION),
        ),
        item(
            PRIMARY_ID,
            KEYWORD_FIELD,
            FieldValue::String(format!("{GIANT_KEYWORD_PREFIX}primary")),
            Some(OVERSIZED_VERSION),
        ),
    ];
    assert!(
        items.len() <= MAX_INDEX_BATCH_SIZE,
        "the oversized Hash suffix must stay within the public 1,000-item Index limit",
    );
    WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items,
            request_id: None,
        },
    })
}

/// Write a small committed Hash base first. The oversized Hash arrives only
/// after this state has been published, so the next save must preserve it as
/// an incremental change rather than merely proving an initial full save.
fn append_base_prefix(path: &Path) {
    let mut writer = AofWriter::open(path).expect("open real Hash AOF writer");
    writer
        .append(CREATE_SEQUENCE, &WalRecord::new(create_entry()))
        .expect("append Hash schema through AofWriter");
    writer
        .sync_strict()
        .expect("strict-sync real Hash schema AOF frame");
    writer
        .append(BASE_SEQUENCE, &base_hash_record())
        .expect("append committed Hash base through AofWriter");
    writer
        .sync_strict()
        .expect("strict-sync real committed Hash base AOF prefix");
}

/// Append the only oversized source after the small Hash base has a durable
/// generation. The record is dropped before replay reads its retained AOF
/// frame, which proves the AOF source rather than an owned fixture drives it.
fn append_oversized_suffix(path: &Path) -> FrameSpan {
    let mut writer = AofWriter::open(path).expect("reopen real Hash AOF writer");
    let start = fs::metadata(path)
        .expect("measure pre-oversized Hash AOF prefix")
        .len();
    let record = oversized_hash_record();
    writer
        .append(OVERSIZED_SEQUENCE, &record)
        .expect("append oversized Hash Index through AofWriter large-frame path");
    drop(record);
    writer
        .sync_strict()
        .expect("strict-sync real oversized Hash AOF suffix");
    drop(writer);
    assert!(
        fs::metadata(path)
            .expect("read real oversized Hash AOF size")
            .len()
            > PENDING_HARD_LIMIT_BYTES,
        "the actual AofWriter Hash frame must exceed the 256 MiB pending budget",
    );
    assert_fast_index_magic(path, start);
    FrameSpan { start }
}

fn append_later_update_and_delete(path: &Path) {
    let mut writer = AofWriter::open(path).expect("reopen real Hash AOF for suffix");
    let update = WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items: vec![
                item(
                    PRIMARY_ID,
                    HASH_FIELD,
                    FieldValue::String("0x43".to_owned()),
                    Some(UPDATED_VERSION),
                ),
                item(
                    PRIMARY_ID,
                    HASH_FIELD,
                    FieldValue::String("0x99".to_owned()),
                    Some(OVERSIZED_VERSION),
                ),
                item(
                    PRIMARY_ID,
                    KEYWORD_FIELD,
                    FieldValue::String(format!("{UPDATED_KEYWORD_PREFIX}primary")),
                    Some(UPDATED_VERSION),
                ),
            ],
            request_id: None,
        },
    });
    writer
        .append(UPDATE_SEQUENCE, &update)
        .expect("append real versioned Hash update AOF suffix");
    drop(update);
    let delete = WalRecord::new(RaftLogEntry::Delete {
        collection_id: COLLECTION.to_owned(),
        external_id: DELETED_ID.to_owned(),
        field: None,
    });
    writer
        .append(DELETE_SEQUENCE, &delete)
        .expect("append real Hash companion delete AOF suffix");
    drop(delete);
    writer
        .sync_strict()
        .expect("strict-sync real Hash AOF suffix");
}

fn assert_fast_index_magic(path: &Path, frame_start: u64) {
    let mut file = OpenOptions::new()
        .read(true)
        .open(path)
        .expect("open real Hash AOF for fast-codec check");
    file.seek(SeekFrom::Start(frame_start + FRAME_HEADER_BYTES))
        .expect("seek real Hash Index payload magic");
    let mut magic = [0_u8; 4];
    file.read_exact(&mut magic)
        .expect("read real Hash Index payload magic");
    assert_eq!(
        &magic, b"LWAL",
        "the oversized AOF fixture must use the real fast Index wire codec",
    );
}

/// The incremental checkpoint covers the huge suffix. A CRC-valid but
/// semantically bad covered frame must be skipped without parsing when strict
/// suffix replay begins at the covered sequence.
fn poison_covered_prefix_frame(path: &Path, span: FrameSpan) {
    let payload_start = span
        .start
        .checked_add(FRAME_HEADER_BYTES)
        .expect("covered Hash payload offset fits u64");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("open Hash AOF to poison its covered prefix");
    file.seek(SeekFrom::Start(span.start + 8))
        .expect("seek covered Hash AOF payload length");
    let mut length = [0_u8; 4];
    file.read_exact(&mut length)
        .expect("read covered Hash AOF payload length");
    let payload_bytes = u64::from(u32::from_le_bytes(length));
    assert!(
        payload_bytes > PENDING_HARD_LIMIT_BYTES,
        "the covered Hash payload must still exceed the 256 MiB fixture threshold",
    );
    file.seek(SeekFrom::Start(payload_start))
        .expect("seek covered Hash AOF payload");
    let mut first = [0_u8; 1];
    file.read_exact(&mut first)
        .expect("read first covered Hash AOF payload byte");
    first[0] ^= 0xff;
    file.seek(SeekFrom::Start(payload_start))
        .expect("rewind covered Hash AOF payload");
    file.write_all(&first)
        .expect("poison one covered Hash AOF payload byte");

    let mut hasher = crc32fast::Hasher::new();
    let mut remaining = payload_bytes;
    let mut buffer = [0_u8; 64 * 1024];
    file.seek(SeekFrom::Start(payload_start))
        .expect("rewind to stream covered Hash AOF payload");
    while remaining != 0 {
        let read_len = remaining.min(buffer.len() as u64) as usize;
        file.read_exact(&mut buffer[..read_len])
            .expect("stream covered Hash AOF payload for CRC");
        hasher.update(&buffer[..read_len]);
        remaining -= read_len as u64;
    }
    file.seek(SeekFrom::Start(span.start + 12))
        .expect("seek covered Hash AOF CRC");
    file.write_all(&hasher.finalize().to_le_bytes())
        .expect("rewrite CRC for poisoned covered Hash AOF frame");
    file.sync_all()
        .expect("sync poisoned checkpoint-covered Hash AOF prefix");
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

fn search_ids(engine: &Engine, query: QueryNode, label: &str) -> Vec<String> {
    let response = engine
        .search(COLLECTION, search_request(query))
        .unwrap_or_else(|error| panic!("{label} query failed: {error}"));
    let mut ids = BTreeSet::new();
    for hit in response.hits {
        assert!(
            ids.insert(hit.external_id.clone()),
            "{label} query must not return duplicate external ID {}",
            hit.external_id,
        );
    }
    ids.into_iter().collect()
}

fn hamming_ids(engine: &Engine, hash: &str) -> Vec<String> {
    search_ids(
        engine,
        QueryNode::Hamming(HammingQuery {
            field: HASH_FIELD.to_owned(),
            hash: hash.to_owned(),
            max_distance: 0,
        }),
        "exact Hash Hamming",
    )
}

fn prefix_ids(engine: &Engine, prefix: &str) -> Vec<String> {
    search_ids(
        engine,
        QueryNode::Prefix(PrefixQuery {
            field: KEYWORD_FIELD.to_owned(),
            value: prefix.to_owned(),
        }),
        "Keyword prefix",
    )
}

/// `FieldStats::bytes` is live index attribution. A materialized Hash row is
/// 12 bytes while active; a cold mmap reader exposes its query value without a
/// live Hash allocation and reports zero here.
fn assert_hash_residency_bytes(engine: &Engine, expected: u64, phase: &str) {
    let stats = engine
        .stats(COLLECTION)
        .expect("oversized Hash collection stats");
    let hash = stats
        .fields
        .get(HASH_FIELD)
        .expect("oversized Hash field stats");
    assert_eq!(
        hash.field_type,
        FieldType::Hash,
        "{phase}: the committed Hash field must remain Hash",
    );
    assert_eq!(
        hash.bytes, expected,
        "{phase}: Hash FieldStats bytes must match active-or-cold index residency",
    );
}

fn assert_base_state(engine: &Engine, expected_hash_bytes: u64, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("oversized Hash base stats")
            .documents_indexed,
        2,
        "{phase}: the committed Hash base and its Keyword-only companion must be visible",
    );
    assert_eq!(
        hamming_ids(engine, "0x41"),
        vec![PRIMARY_ID.to_owned()],
        "{phase}: the established Hash base must be exact-Hamming searchable as 0x41",
    );
    assert_eq!(
        hamming_ids(engine, "0x42"),
        Vec::<String>::new(),
        "{phase}: the future oversized Hash value must not appear in the base",
    );
    assert_eq!(
        prefix_ids(engine, BASE_KEYWORD_PREFIX),
        vec![PRIMARY_ID.to_owned()],
        "{phase}: the mixed committed base Keyword prefix must remain visible",
    );
    assert_eq!(
        prefix_ids(engine, GIANT_KEYWORD_PREFIX),
        Vec::<String>::new(),
        "{phase}: the future oversized mixed Keyword value must not appear in the base",
    );
    assert_eq!(
        prefix_ids(engine, DELETED_KEYWORD_PREFIX),
        vec![DELETED_ID.to_owned()],
        "{phase}: the later-delete companion must be visible before its delete",
    );
    assert_hash_residency_bytes(engine, expected_hash_bytes, phase);
}

fn assert_oversized_state(engine: &Engine, expected_hash_bytes: u64, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("oversized Hash incremental stats")
            .documents_indexed,
        2,
        "{phase}: the oversized Hash update must retain the pre-existing companion row",
    );
    assert_eq!(
        hamming_ids(engine, "0x42"),
        vec![PRIMARY_ID.to_owned()],
        "{phase}: the giant leading-zero Hash must be exact-Hamming searchable as 0x42",
    );
    assert_eq!(
        hamming_ids(engine, "0x41"),
        Vec::<String>::new(),
        "{phase}: the older Hash base must not revive after the oversized version wins",
    );
    assert_eq!(
        hamming_ids(engine, "0x43"),
        Vec::<String>::new(),
        "{phase}: the later Hash version must not appear before its AOF record",
    );
    assert_eq!(
        prefix_ids(engine, BASE_KEYWORD_PREFIX),
        Vec::<String>::new(),
        "{phase}: the older mixed Keyword base must not revive after the oversized version wins",
    );
    assert_eq!(
        prefix_ids(engine, GIANT_KEYWORD_PREFIX),
        vec![PRIMARY_ID.to_owned()],
        "{phase}: the oversized record's mixed Keyword prefix must remain visible",
    );
    assert_eq!(
        prefix_ids(engine, DELETED_KEYWORD_PREFIX),
        vec![DELETED_ID.to_owned()],
        "{phase}: the later-delete companion must survive the oversized update",
    );
    assert_hash_residency_bytes(engine, expected_hash_bytes, phase);
}

fn assert_tail_state(engine: &Engine, expected_hash_bytes: u64, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("oversized Hash tail stats")
            .documents_indexed,
        1,
        "{phase}: the complete companion delete must leave only the primary row",
    );
    assert_eq!(
        hamming_ids(engine, "0x43"),
        vec![PRIMARY_ID.to_owned()],
        "{phase}: the higher-version Hash must be exact-Hamming searchable",
    );
    assert_eq!(
        hamming_ids(engine, "0x42"),
        Vec::<String>::new(),
        "{phase}: the old giant-source Hash must not revive after the update",
    );
    assert_eq!(
        hamming_ids(engine, "0x41"),
        Vec::<String>::new(),
        "{phase}: the original Hash base must not revive after later versions",
    );
    assert_eq!(
        hamming_ids(engine, "0x99"),
        Vec::<String>::new(),
        "{phase}: the stale lower-version Hash must not replace the winner",
    );
    assert_eq!(
        prefix_ids(engine, BASE_KEYWORD_PREFIX),
        Vec::<String>::new(),
        "{phase}: the old mixed Keyword base must not revive after later updates",
    );
    assert_eq!(
        prefix_ids(engine, GIANT_KEYWORD_PREFIX),
        Vec::<String>::new(),
        "{phase}: the oversized mixed Keyword value must not revive after its update",
    );
    assert_eq!(
        prefix_ids(engine, UPDATED_KEYWORD_PREFIX),
        vec![PRIMARY_ID.to_owned()],
        "{phase}: the updated mixed Keyword prefix must remain visible",
    );
    assert_eq!(
        prefix_ids(engine, DELETED_KEYWORD_PREFIX),
        Vec::<String>::new(),
        "{phase}: the deleted companion Keyword must not revive",
    );
    assert_hash_residency_bytes(engine, expected_hash_bytes, phase);
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
        "{phase}: public pending-change total must stay within the documented 256 MiB budget: total={total}",
    );
    assert!(
        high_water <= PENDING_HARD_LIMIT_BYTES,
        "{phase}: public pending-change high water must stay within the documented 256 MiB budget: high_water={high_water}",
    );
}

fn corrupt_complete_frame_payload(path: &Path, frame_start: u64) {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("open authentic Hash AOF frame for corruption");
    file.seek(SeekFrom::Start(frame_start + 8))
        .expect("seek authentic Hash AOF frame length");
    let mut length = [0_u8; 4];
    file.read_exact(&mut length)
        .expect("read authentic Hash AOF frame length");
    let payload_bytes = u64::from(u32::from_le_bytes(length));
    assert!(
        payload_bytes > 0,
        "security Hash payload must have a byte to corrupt"
    );
    let payload_start = frame_start + FRAME_HEADER_BYTES;
    file.seek(SeekFrom::Start(payload_start))
        .expect("seek authentic Hash AOF payload byte");
    let mut byte = [0_u8; 1];
    file.read_exact(&mut byte)
        .expect("read authentic Hash AOF payload byte");
    byte[0] ^= 0xff;
    file.seek(SeekFrom::Start(payload_start))
        .expect("rewind authentic Hash AOF payload byte");
    file.write_all(&byte)
        .expect("corrupt one authentic Hash AOF payload byte");

    let mut hasher = crc32fast::Hasher::new();
    let mut remaining = payload_bytes;
    let mut buffer = [0_u8; 64 * 1024];
    file.seek(SeekFrom::Start(payload_start))
        .expect("rewind authentic Hash AOF payload for CRC");
    while remaining != 0 {
        let read_len = remaining.min(buffer.len() as u64) as usize;
        file.read_exact(&mut buffer[..read_len])
            .expect("stream authentic Hash payload for CRC");
        hasher.update(&buffer[..read_len]);
        remaining -= read_len as u64;
    }
    file.seek(SeekFrom::Start(frame_start + 12))
        .expect("seek authentic Hash AOF frame CRC");
    file.write_all(&hasher.finalize().to_le_bytes())
        .expect("rewrite authentic Hash AOF frame CRC");
    file.sync_all()
        .expect("sync complete corrupt Hash AOF frame");
}

fn assert_complete_corrupt_hash_frame_is_refused(root: &Path) {
    let path = root.join("complete-corrupt-hash-frame.aof");
    let mut writer = AofWriter::open(&path).expect("open security Hash AOF writer");
    writer
        .append(CREATE_SEQUENCE, &WalRecord::new(create_entry()))
        .expect("append valid security Hash schema");
    writer
        .sync_strict()
        .expect("strict-sync valid security Hash schema");
    let corrupt_frame_start = fs::metadata(&path)
        .expect("measure authentic Hash AOF prefix")
        .len();
    let record = WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items: vec![
                item(
                    PRIMARY_ID,
                    HASH_FIELD,
                    FieldValue::String("0x42".to_owned()),
                    Some(BASE_VERSION),
                ),
                item(
                    PRIMARY_ID,
                    KEYWORD_FIELD,
                    FieldValue::String(format!("{BASE_KEYWORD_PREFIX}security")),
                    Some(BASE_VERSION),
                ),
            ],
            request_id: None,
        },
    });
    writer
        .append(BASE_SEQUENCE, &record)
        .expect("append authentic security Hash frame through AofWriter");
    drop(record);
    writer
        .sync_strict()
        .expect("strict-sync authentic security Hash frame");
    drop(writer);
    assert_fast_index_magic(&path, corrupt_frame_start);
    corrupt_complete_frame_payload(&path, corrupt_frame_start);

    let engine = Arc::new(Engine::new());
    let refusal = replay_aof_into(&engine, &path, 0);
    assert!(
        refusal.is_err(),
        "a CRC-valid but format-corrupt complete Hash AOF frame must be refused before publication",
    );
    assert_eq!(
        hamming_ids(&engine, "0x42"),
        Vec::<String>::new(),
        "a refused complete Hash frame must not become Hamming-query-visible",
    );
    assert_eq!(
        prefix_ids(&engine, BASE_KEYWORD_PREFIX),
        Vec::<String>::new(),
        "a refused complete Hash frame must not become Keyword-query-visible",
    );
    let store = SegmentRdbStore::new(root.join("complete-corrupt-hash-frame-segments"))
        .expect("open security Hash segment store");
    let saved = store
        .save_with_sequence(&engine, 0)
        .expect("save only the replayed valid Hash prefix");
    assert_eq!(
        saved, CREATE_SEQUENCE,
        "a refused complete Hash frame must not advance the durable checkpoint watermark",
    );
    let cold = store
        .load_current_generation()
        .expect("cold-open valid security Hash prefix")
        .expect("valid security Hash prefix checkpoint exists");
    assert_eq!(
        cold.sequence, CREATE_SEQUENCE,
        "cold-open must retain only the valid Hash prefix watermark",
    );
    assert_eq!(
        cold.engine
            .stats(COLLECTION)
            .expect("security Hash schema survives replay")
            .documents_indexed,
        0,
        "the refused Hash frame's rows must stay absent after cold reopen",
    );
}

fn child_paths() -> Option<(PathBuf, PathBuf, PathBuf)> {
    if std::env::var(CHILD_MODE_ENV).ok().as_deref() != Some(CHILD_CASE) {
        return None;
    }
    let root = std::env::var_os(CHILD_ROOT_ENV)
        .map(PathBuf::from)
        .expect("isolated Hash replay child needs its durable root");
    let aof = std::env::var_os(CHILD_AOF_ENV)
        .map(PathBuf::from)
        .expect("isolated Hash replay child needs its AOF path");
    let handshake = std::env::var_os(CHILD_HANDSHAKE_ENV)
        .map(PathBuf::from)
        .expect("isolated Hash replay child needs a handshake path");
    Some((root, aof, handshake))
}

async fn run_replay_child(root: PathBuf, aof_path: PathBuf, handshake: PathBuf) {
    fs::write(&handshake, CHILD_CASE).expect("record exact isolated Hash replay child entry");
    let base_engine = Arc::new(Engine::new());
    let base_replay = replay_aof_into(&base_engine, &aof_path, 0);
    assert!(
        base_replay.is_ok(),
        "the valid committed Hash base AOF prefix must replay before the oversized suffix: {}",
        base_replay
            .as_ref()
            .err()
            .map(|error| format!("{error:#}"))
            .unwrap_or_default(),
    );
    assert_eq!(
        base_replay.expect("behavior assertion above checked Hash base replay"),
        BASE_SEQUENCE,
        "the committed Hash base replay must advance through its exact sequence",
    );
    assert_base_state(&base_engine, 12, "committed Hash base AOF replay");
    assert_public_pending_budget(
        base_engine.clone(),
        BASE_SEQUENCE,
        "committed Hash base AOF replay",
    )
    .await;

    let store = SegmentRdbStore::new(root.join("segments"))
        .expect("open segment store for oversized Hash AOF checkpoint");
    let base_checkpoint = store
        .save_with_sequence(&base_engine, BASE_SEQUENCE)
        .expect("checkpoint the committed Hash base before the oversized suffix");
    assert_eq!(
        base_checkpoint, BASE_SEQUENCE,
        "the initial Hash checkpoint must retain the established base watermark",
    );
    assert_public_pending_budget(
        base_engine.clone(),
        BASE_SEQUENCE,
        "committed Hash base checkpoint",
    )
    .await;

    let base_cold = store
        .load_current_generation()
        .expect("cold-open committed Hash base checkpoint")
        .expect("committed Hash base checkpoint must publish CURRENT");
    assert_eq!(
        base_cold.sequence, BASE_SEQUENCE,
        "cold Hash base checkpoint must keep its exact watermark",
    );
    assert_base_state(&base_cold.engine, 0, "cold committed Hash base checkpoint");

    let span = append_oversized_suffix(&aof_path);
    let oversized_replay = replay_aof_into(&base_cold.engine, &aof_path, base_cold.sequence);
    assert!(
        oversized_replay.is_ok(),
        "a valid committed >256 MiB leading-zero Hash AOF suffix must replay without an owned decode refusal: {}",
        oversized_replay
            .as_ref()
            .err()
            .map(|error| format!("{error:#}"))
            .unwrap_or_default(),
    );
    assert_eq!(
        oversized_replay.expect("behavior assertion above checked oversized Hash replay"),
        OVERSIZED_SEQUENCE,
        "valid oversized Hash replay must advance through its committed suffix sequence",
    );
    assert_oversized_state(&base_cold.engine, 12, "oversized Hash AOF suffix replay");
    assert_public_pending_budget(
        base_cold.engine.clone(),
        OVERSIZED_SEQUENCE,
        "oversized Hash AOF suffix replay",
    )
    .await;

    let incremental_checkpoint = store
        .save_with_sequence(&base_cold.engine, OVERSIZED_SEQUENCE)
        .expect("incrementally checkpoint the oversized Hash AOF suffix over its base");
    assert_eq!(
        incremental_checkpoint, OVERSIZED_SEQUENCE,
        "incremental Hash checkpoint must retain the oversized suffix watermark",
    );
    assert_public_pending_budget(
        base_cold.engine.clone(),
        OVERSIZED_SEQUENCE,
        "incremental oversized Hash checkpoint",
    )
    .await;

    let incremental_cold = store
        .load_current_generation()
        .expect("cold-open incremental oversized Hash checkpoint")
        .expect("incremental oversized Hash checkpoint must publish CURRENT");
    assert_eq!(
        incremental_cold.sequence, OVERSIZED_SEQUENCE,
        "cold incremental Hash checkpoint must keep its exact suffix watermark",
    );
    assert_oversized_state(
        &incremental_cold.engine,
        0,
        "cold incremental oversized Hash checkpoint",
    );
    assert_public_pending_budget(
        incremental_cold.engine.clone(),
        OVERSIZED_SEQUENCE,
        "cold incremental oversized Hash checkpoint",
    )
    .await;

    append_later_update_and_delete(&aof_path);
    poison_covered_prefix_frame(&aof_path, span);
    let suffix_replay = replay_aof_into(
        &incremental_cold.engine,
        &aof_path,
        incremental_cold.sequence,
    );
    assert!(
        suffix_replay.is_ok(),
        "AOF replay from the Hash checkpoint watermark must skip its CRC-valid malformed covered prefix and apply later records: {}",
        suffix_replay
            .as_ref()
            .err()
            .map(|error| format!("{error:#}"))
            .unwrap_or_default(),
    );
    assert_eq!(
        suffix_replay.expect("behavior assertion above checked Hash suffix replay"),
        DELETE_SEQUENCE,
        "AOF replay must apply only sequences strictly greater than the Hash checkpoint watermark",
    );
    assert_tail_state(
        &incremental_cold.engine,
        12,
        "strict Hash AOF suffix after incremental checkpoint",
    );
    assert_public_pending_budget(
        incremental_cold.engine.clone(),
        DELETE_SEQUENCE,
        "strict Hash AOF suffix replay",
    )
    .await;

    let final_checkpoint = store
        .save_with_sequence(&incremental_cold.engine, DELETE_SEQUENCE)
        .expect("checkpoint Hash update and delete after oversized AOF replay");
    assert_eq!(
        final_checkpoint, DELETE_SEQUENCE,
        "final Hash checkpoint must retain the completed AOF suffix watermark",
    );
    assert_public_pending_budget(
        incremental_cold.engine.clone(),
        DELETE_SEQUENCE,
        "final Hash checkpoint",
    )
    .await;
    let final_cold = store
        .load_current_generation()
        .expect("cold-open final oversized Hash AOF checkpoint")
        .expect("final oversized Hash checkpoint must publish CURRENT");
    assert_eq!(
        final_cold.sequence, DELETE_SEQUENCE,
        "final cold Hash checkpoint must retain the full AOF watermark",
    );
    assert_tail_state(&final_cold.engine, 0, "final Hash cold reopen");
    assert_public_pending_budget(
        final_cold.engine.clone(),
        DELETE_SEQUENCE,
        "final oversized Hash AOF cold reopen",
    )
    .await;

    let covered_replay = replay_aof_into(&final_cold.engine, &aof_path, final_cold.sequence);
    assert!(
        covered_replay.is_ok(),
        "fully checkpoint-covered Hash AOF frames must be skipped without replay failure: {}",
        covered_replay
            .as_ref()
            .err()
            .map(|error| format!("{error:#}"))
            .unwrap_or_default(),
    );
    assert_eq!(
        covered_replay.expect("behavior assertion above checked covered Hash replay"),
        0,
        "fully checkpoint-covered Hash AOF frames must not advance the watermark",
    );
    assert_tail_state(&final_cold.engine, 0, "fully covered Hash AOF replay");
    assert_public_pending_budget(
        final_cold.engine.clone(),
        DELETE_SEQUENCE,
        "fully checkpoint-covered Hash AOF replay",
    )
    .await;
}

async fn run_isolated_replay(root: &Path, aof_path: &Path) {
    let child_root = tempfile::tempdir().expect("isolated Hash replay child directory");
    let handshake = child_root.path().join("entered-case");
    let stdout_path = child_root.path().join("child.stdout");
    let stderr_path = child_root.path().join("child.stderr");
    let executable = std::env::current_exe().expect("current oversized Hash test executable");
    let stdout = File::create(&stdout_path).expect("create isolated Hash replay child stdout");
    let stderr = File::create(&stderr_path).expect("create isolated Hash replay child stderr");
    let child = Command::new(executable)
        .env(CHILD_MODE_ENV, CHILD_CASE)
        .env(CHILD_ROOT_ENV, root)
        .env(CHILD_AOF_ENV, aof_path)
        .env(CHILD_HANDSHAKE_ENV, &handshake)
        .arg(TEST_NAME)
        .arg("--exact")
        .arg("--nocapture")
        .arg("--test-threads=1")
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .expect("spawn isolated oversized Hash replay child");
    let mut child = ChildCleanup(Some(child));
    let deadline = Instant::now() + REPLAY_WATCHDOG;
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
                let mut raw = child.0.take().expect("timed out child remains owned");
                let _ = raw.kill();
                let status = raw
                    .wait()
                    .expect("wait for killed oversized Hash replay child");
                let stdout = fs::read_to_string(&stdout_path)
                    .expect("read killed oversized Hash replay child stdout");
                let stderr = fs::read_to_string(&stderr_path)
                    .expect("read killed oversized Hash replay child stderr");
                panic!(
                    "a valid committed >256 MiB Hash AOF frame must replay and checkpoint before test cleanup; isolated child was killed after {REPLAY_WATCHDOG:?}: status={status}; stdout={stdout}; stderr={stderr}",
                );
            }
            Err(error) => panic!("poll isolated oversized Hash replay child: {error}"),
        }
    }
    let status = child
        .0
        .take()
        .expect("exited child remains owned")
        .wait()
        .expect("wait for exited oversized Hash replay child");
    let stdout = fs::read_to_string(&stdout_path).expect("read oversized Hash replay child stdout");
    let stderr = fs::read_to_string(&stderr_path).expect("read oversized Hash replay child stderr");
    let entered = fs::read_to_string(&handshake).unwrap_or_else(|error| {
        panic!(
            "isolated child did not enter exact {TEST_NAME}: {error}; stdout={stdout}; stderr={stderr}",
        )
    });
    assert_eq!(
        entered, CHILD_CASE,
        "isolated child must enter the intended oversized Hash replay body",
    );
    assert!(
        status.success(),
        "isolated oversized Hash replay child failed: status={status}; stdout={stdout}; stderr={stderr}",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn oversized_committed_hash_aof_replays_suffixes_checkpoints_and_cold_opens() {
    if let Some((root, aof_path, handshake)) = child_paths() {
        run_replay_child(root, aof_path, handshake).await;
        return;
    }

    let root = tempfile::tempdir().expect("oversized committed Hash AOF fixture root");
    let aof_path = root.path().join("oversized-hash.aof");
    append_base_prefix(&aof_path);
    run_isolated_replay(root.path(), &aof_path).await;
    assert_complete_corrupt_hash_frame_is_refused(root.path());
}
