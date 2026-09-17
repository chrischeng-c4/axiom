//! Black-box contract for one borrowed oversized Unicode Text token in AOF.
//!
//! One committed fast `Index` value contains exactly one whitespace-delimited
//! token. It has more than 256 MiB of ASCII uppercase bytes and ends with
//! `İΟΣ`. The suffix requires an expanding lowercase mapping for `İ` and a
//! context-sensitive Greek final sigma for the terminal `Σ`. The fixture uses
//! full `str::to_lowercase()` only to construct its public query oracle after
//! the source record is dropped. Query memory is separate from the Engine's
//! pending-change accounting.
//!
//! A small Text base is checkpointed before the giant suffix. The contract
//! then cold-opens that suffix, checks the exact normalized token through the
//! public Match query, applies a versioned replacement plus a stale retry and
//! a delete, and cold-opens the completed tail. The child process owns all
//! behavior work so a baseline capacity wait cannot leak its process-wide
//! budget. Its temporary directory is parent-owned through `TMPDIR`, `TEMP`,
//! and `TMP`; the parent kills and reaps it only for cleanup, not as a latency
//! promise.
//!
//! # Facets
//!
//! - Behavior: aof_oversized_committed_unicode_token.rs:202-391, :393-480,
//!   :627-786, and :854-861
//!   establish a small committed Text base, then replay and cold-open one valid >256 MiB
//!   Unicode token, queries its exact full-string lowercase result, and applies
//!   later version/delete records. These assertions cover
//!   apps/lumen/src/aof.rs:285-315, apps/lumen/src/storage/committed_text_apply.rs:49-81,
//!   apps/lumen/src/storage/text_preparation.rs:119-141 and :323-336, and
//!   libs/index-text/src/lib.rs:510-535.
//! - Security: aof_oversized_committed_unicode_token.rs:347-358, :393-412,
//!   and :521-604 corrupts the
//!   real >256 MiB CRC-valid persisted Text frame after a valid prefix and
//!   requires refusal before its row or watermark become visible. It covers
//!   the AOF byte boundary in apps/lumen/src/aof.rs:285-315.
//! - Performance: apps/lumen/docs/indexing.md:264-276 says, verbatim,
//!   "Pending active, frozen, and reserved changes have a 256 MiB budget."
//!   aof_oversized_committed_unicode_token.rs:482-519 reads public pending total
//!   and high-water gauges after replay and checkpoints. The fixture source,
//!   full-string query oracle, and mapped AOF bytes are outside those gauges.
//!   This contract sets no latency budget.
//!
//! # Root negative control
//!
//! Restore the current full-token lowercase workspace pricing in
//! `apps/lumen/src/storage/text_preparation.rs`. The valid oversized replay
//! assertion below must fail through its capacity refusal or the child cleanup
//! watchdog. Restore the correct source hash before any other gate. Never lower
//! the fixture threshold or replace its full-string oracle with per-character
//! lowercasing.
//!
//! Gate: cargo test -p lumen --test aof_oversized_committed_unicode_token -- --nocapture.

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
    Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    MatchOp, MatchQuery, QueryNode, SearchRequest,
};
use lumen::wal::WalRecord;

const COLLECTION: &str = "oversized-aof-unicode-token";
const FIELD: &str = "body";
const BASE_ID: &str = "unicode-base";
const DELETE_ID: &str = "unicode-delete";
const GIANT_ID: &str = "unicode-giant";
const BASE_TERM: &str = "basealpha";
const DELETE_TERM: &str = "deleteme";
const UPDATED_TERM: &str = "updatedterm";
const STALE_TERM: &str = "staleterm";
const SOURCE_UNICODE_SUFFIX: &str = "İΟΣ";
const EXPECTED_NORMALIZED_SUFFIX: &str = "i\u{307}ος";
const HUGE_ASCII_BYTES: usize = 256 * 1024 * 1024 + 128;
const PENDING_HARD_LIMIT_BYTES: u64 = 256 * 1024 * 1024;
const FRAME_HEADER_BYTES: u64 = 16;
const CREATE_SEQUENCE: u64 = 1;
const BASE_SEQUENCE: u64 = 2;
const GIANT_SEQUENCE: u64 = 3;
const UPDATE_SEQUENCE: u64 = 4;
const DELETE_SEQUENCE: u64 = 5;
const BASE_VERSION: u64 = 10;
const GIANT_VERSION: u64 = 20;
const UPDATED_VERSION: u64 = 30;
const REPLAY_WATCHDOG: Duration = Duration::from_secs(240);
const POLL_INTERVAL: Duration = Duration::from_millis(25);
const CHILD_MODE_ENV: &str = "LUMEN_AOF_OVERSIZED_UNICODE_TOKEN_CHILD";
const CHILD_ROOT_ENV: &str = "LUMEN_AOF_OVERSIZED_UNICODE_TOKEN_ROOT";
const CHILD_AOF_ENV: &str = "LUMEN_AOF_OVERSIZED_UNICODE_TOKEN_AOF";
const CHILD_HANDSHAKE_ENV: &str = "LUMEN_AOF_OVERSIZED_UNICODE_TOKEN_HANDSHAKE";
const CHILD_CASE: &str = "replay";
const TEST_NAME: &str = "oversized_committed_unicode_token_aof_replays_normalizes_and_cold_opens";

/// Gives `/metrics` the true replay sequence without constructing a fresh
/// coordinator that would reset this Engine's capture barrier to zero.
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

/// Owns the spawned process on every panic path. A killed baseline child is
/// reaped before its parent scratch workspace is dropped.
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

fn text_schema() -> CreateCollectionRequest {
    CreateCollectionRequest {
        fields: BTreeMap::from([(
            FIELD.to_owned(),
            FieldSpec {
                field_type: FieldType::Text,
                analyzer: Some(Analyzer::WhitespaceLower),
                multi: None,
                dim: None,
                metric: None,
                backend: None,
                quantize: None,
            },
        )]),
    }
}

fn create_entry() -> RaftLogEntry {
    RaftLogEntry::CreateCollection {
        collection_id: COLLECTION.to_owned(),
        req: text_schema(),
    }
}

fn item(external_id: &str, value: FieldValue, version: Option<u64>) -> IndexItem {
    IndexItem {
        external_id: external_id.to_owned(),
        field: FIELD.to_owned(),
        value,
        version,
    }
}

fn base_record() -> WalRecord {
    WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items: vec![
                item(
                    BASE_ID,
                    FieldValue::String(format!("{BASE_TERM} present")),
                    Some(BASE_VERSION),
                ),
                item(
                    DELETE_ID,
                    FieldValue::String(format!("{DELETE_TERM} present")),
                    Some(BASE_VERSION),
                ),
            ],
            request_id: None,
        },
    })
}

/// Produce exactly one whitespace-delimited source token. The source has no
/// allocated normalized twin while the AOF writer owns it.
fn giant_unicode_token() -> String {
    assert!(
        HUGE_ASCII_BYTES as u64 > PENDING_HARD_LIMIT_BYTES,
        "the giant token's ASCII body must exceed the 256 MiB pending budget",
    );
    let mut token = "A".repeat(HUGE_ASCII_BYTES);
    token.push_str(SOURCE_UNICODE_SUFFIX);
    assert_eq!(
        token.split_whitespace().count(),
        1,
        "the fixture must be one whitespace-delimited token",
    );
    assert!(
        token.ends_with(SOURCE_UNICODE_SUFFIX),
        "the fixture must retain the Unicode expansion and final-sigma suffix",
    );
    token
}

/// This is the only expected-token oracle. It deliberately uses full
/// `str::to_lowercase`, not `char::to_lowercase`, so final sigma uses its token
/// context and `İ` expands to `i` plus a combining dot.
fn normalized_giant_token() -> String {
    let normalized = giant_unicode_token().to_lowercase();
    assert!(
        normalized.ends_with(EXPECTED_NORMALIZED_SUFFIX),
        "the full-string oracle must expand İ and use final Greek sigma: suffix={EXPECTED_NORMALIZED_SUFFIX:?}",
    );
    assert_eq!(
        normalized.len(),
        HUGE_ASCII_BYTES + EXPECTED_NORMALIZED_SUFFIX.len(),
        "the full-string oracle must have its exact ASCII-plus-Unicode expansion length",
    );
    assert_eq!(
        normalized.split_whitespace().count(),
        1,
        "the normalized public query must remain one token",
    );
    normalized
}

fn oversized_unicode_token_record() -> WalRecord {
    WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items: vec![item(
                GIANT_ID,
                FieldValue::String(giant_unicode_token()),
                Some(GIANT_VERSION),
            )],
            request_id: None,
        },
    })
}

fn append_base_prefix(path: &Path) {
    let mut writer = AofWriter::open(path).expect("open real Unicode Text AOF writer");
    writer
        .append(CREATE_SEQUENCE, &WalRecord::new(create_entry()))
        .expect("append Unicode Text schema through AofWriter");
    writer
        .sync_strict()
        .expect("strict-sync real Unicode Text schema AOF frame");
    let base = base_record();
    writer
        .append(BASE_SEQUENCE, &base)
        .expect("append committed Unicode Text base through AofWriter");
    drop(base);
    writer
        .sync_strict()
        .expect("strict-sync committed Unicode Text base AOF prefix");
}

/// The only giant source exists inside this append. Replay reads the retained
/// AOF frame after the `WalRecord` and encoder buffer are dropped.
fn append_oversized_suffix(path: &Path) {
    let mut writer = AofWriter::open(path).expect("reopen real Unicode Text AOF writer");
    let frame_start = fs::metadata(path)
        .expect("measure pre-oversized Unicode Text AOF prefix")
        .len();
    let record = oversized_unicode_token_record();
    writer
        .append(GIANT_SEQUENCE, &record)
        .expect("append oversized Unicode Text Index through AofWriter");
    drop(record);
    writer
        .sync_strict()
        .expect("strict-sync real oversized Unicode Text AOF suffix");
    drop(writer);
    let payload_bytes = frame_payload_bytes(path, frame_start);
    assert!(
        payload_bytes > PENDING_HARD_LIMIT_BYTES,
        "the real AofWriter Unicode Text frame must exceed 256 MiB: payload_bytes={payload_bytes}",
    );
    assert_fast_index_magic(path, frame_start);
}

fn append_tail(path: &Path) {
    let mut writer = AofWriter::open(path).expect("reopen Unicode Text AOF for tail");
    let update = WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items: vec![
                item(
                    GIANT_ID,
                    FieldValue::String(format!("{UPDATED_TERM} present")),
                    Some(UPDATED_VERSION),
                ),
                item(
                    GIANT_ID,
                    FieldValue::String(format!("{STALE_TERM} present")),
                    Some(GIANT_VERSION),
                ),
            ],
            request_id: None,
        },
    });
    writer
        .append(UPDATE_SEQUENCE, &update)
        .expect("append versioned Unicode Text update AOF suffix");
    drop(update);
    let delete = WalRecord::new(RaftLogEntry::Delete {
        collection_id: COLLECTION.to_owned(),
        external_id: DELETE_ID.to_owned(),
        field: None,
    });
    writer
        .append(DELETE_SEQUENCE, &delete)
        .expect("append Unicode Text delete AOF suffix");
    drop(delete);
    writer
        .sync_strict()
        .expect("strict-sync Unicode Text update/delete AOF suffix");
}

fn frame_payload_bytes(path: &Path, frame_start: u64) -> u64 {
    let mut file = File::open(path).expect("open real Unicode Text AOF frame");
    file.seek(SeekFrom::Start(frame_start + 8))
        .expect("seek real Unicode Text AOF frame length");
    let mut length = [0_u8; 4];
    file.read_exact(&mut length)
        .expect("read real Unicode Text AOF frame length");
    u64::from(u32::from_le_bytes(length))
}

fn assert_fast_index_magic(path: &Path, frame_start: u64) {
    let mut file = File::open(path).expect("open Unicode Text AOF for fast-codec check");
    file.seek(SeekFrom::Start(frame_start + FRAME_HEADER_BYTES))
        .expect("seek Unicode Text fast Index payload magic");
    let mut magic = [0_u8; 4];
    file.read_exact(&mut magic)
        .expect("read Unicode Text fast Index payload magic");
    assert_eq!(
        &magic, b"LWAL",
        "the oversized Unicode Text fixture must use the real fast Index wire codec",
    );
}

fn search_request(text: String) -> SearchRequest {
    SearchRequest {
        query: QueryNode::Match(MatchQuery {
            field: FIELD.to_owned(),
            text,
            op: MatchOp::And,
        }),
        limit: 16,
        offset: 0,
        cursor: None,
        routing_key: None,
        sort: None,
        track_total: true,
        collapse: None,
    }
}

/// Take query ownership to avoid cloning the >256 MiB normalization oracle.
fn match_ids(engine: &Engine, text: String, label: &str) -> BTreeSet<String> {
    let response = engine
        .search(COLLECTION, search_request(text))
        .unwrap_or_else(|error| panic!("{label} public Match query failed: {error}"));
    let mut ids = BTreeSet::new();
    for hit in response.hits {
        assert!(
            ids.insert(hit.external_id.clone()),
            "{label} public Match query must not return duplicate ID {}",
            hit.external_id,
        );
    }
    ids
}

fn assert_base_state(engine: &Engine, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("Unicode Text base stats")
            .documents_indexed,
        2,
        "{phase}: the committed small Text base must retain two documents",
    );
    assert_eq!(
        match_ids(engine, BASE_TERM.to_owned(), "base Text term"),
        BTreeSet::from([BASE_ID.to_owned()]),
        "{phase}: the committed base Text term must remain query-visible",
    );
    assert_eq!(
        match_ids(engine, DELETE_TERM.to_owned(), "delete Text term"),
        BTreeSet::from([DELETE_ID.to_owned()]),
        "{phase}: the later-delete Text document must be visible before its delete",
    );
}

fn assert_giant_cold_state(engine: &Engine, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("Unicode Text giant stats")
            .documents_indexed,
        3,
        "{phase}: the oversized one-token Text record must retain the two base documents",
    );
    assert_eq!(
        match_ids(engine, BASE_TERM.to_owned(), "base after giant Text replay"),
        BTreeSet::from([BASE_ID.to_owned()]),
        "{phase}: the base Text row must survive the oversized suffix",
    );
    assert_eq!(
        match_ids(
            engine,
            normalized_giant_token(),
            "full-string Unicode lowercase giant token",
        ),
        BTreeSet::from([GIANT_ID.to_owned()]),
        "{phase}: the public Match query must find the full str::to_lowercase Unicode token",
    );
}

fn assert_tail_state(engine: &Engine, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("Unicode Text tail stats")
            .documents_indexed,
        2,
        "{phase}: the later delete must remove exactly its committed base document",
    );
    assert_eq!(
        match_ids(engine, BASE_TERM.to_owned(), "base after Unicode Text tail"),
        BTreeSet::from([BASE_ID.to_owned()]),
        "{phase}: the base Text row must remain after later committed records",
    );
    assert_eq!(
        match_ids(engine, UPDATED_TERM.to_owned(), "updated Unicode Text term"),
        BTreeSet::from([GIANT_ID.to_owned()]),
        "{phase}: the higher-version Text replacement must remain visible",
    );
    assert_eq!(
        match_ids(engine, STALE_TERM.to_owned(), "stale Unicode Text term"),
        BTreeSet::new(),
        "{phase}: the lower-version Text retry must not replace the winner",
    );
    assert_eq!(
        match_ids(engine, DELETE_TERM.to_owned(), "deleted Unicode Text term"),
        BTreeSet::new(),
        "{phase}: the deleted Text document must not revive",
    );
}

fn assert_giant_absent_after_update(engine: &Engine, phase: &str) {
    assert_eq!(
        match_ids(
            engine,
            normalized_giant_token(),
            "old full-string Unicode lowercase giant token",
        ),
        BTreeSet::new(),
        "{phase}: the replaced giant Unicode token must not revive after the higher version wins",
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
    let payload_bytes = frame_payload_bytes(path, frame_start);
    assert!(
        payload_bytes > 0,
        "security Text payload must have a byte to corrupt"
    );
    let payload_start = frame_start + FRAME_HEADER_BYTES;
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("open authentic Unicode Text AOF frame for corruption");
    file.seek(SeekFrom::Start(payload_start))
        .expect("seek authentic Unicode Text AOF payload byte");
    let mut byte = [0_u8; 1];
    file.read_exact(&mut byte)
        .expect("read authentic Unicode Text AOF payload byte");
    byte[0] ^= 0xff;
    file.seek(SeekFrom::Start(payload_start))
        .expect("rewind authentic Unicode Text AOF payload byte");
    file.write_all(&byte)
        .expect("corrupt one authentic Unicode Text payload byte");

    let mut hasher = crc32fast::Hasher::new();
    let mut remaining = payload_bytes;
    let mut buffer = [0_u8; 64 * 1024];
    file.seek(SeekFrom::Start(payload_start))
        .expect("rewind authentic Unicode Text payload for CRC");
    while remaining != 0 {
        let read_len = remaining.min(buffer.len() as u64) as usize;
        file.read_exact(&mut buffer[..read_len])
            .expect("stream authentic Unicode Text payload for CRC");
        hasher.update(&buffer[..read_len]);
        remaining -= read_len as u64;
    }
    file.seek(SeekFrom::Start(frame_start + 12))
        .expect("seek authentic Unicode Text frame CRC");
    file.write_all(&hasher.finalize().to_le_bytes())
        .expect("rewrite authentic Unicode Text frame CRC");
    file.sync_all()
        .expect("sync complete corrupt Unicode Text AOF frame");
}

fn assert_complete_corrupt_oversized_frame_is_refused(
    root: &Path,
    path: &Path,
    giant_frame_start: u64,
) {
    assert_fast_index_magic(path, giant_frame_start);
    assert!(
        frame_payload_bytes(path, giant_frame_start) > PENDING_HARD_LIMIT_BYTES,
        "the security frame must stay on the real >256 MiB mapped AOF path",
    );
    corrupt_complete_frame_payload(path, giant_frame_start);
    let engine = Arc::new(Engine::new());
    let refusal = replay_aof_into(&engine, path, 0);
    assert!(
        refusal.is_err(),
        "a CRC-valid but format-corrupt >256 MiB Unicode Text AOF frame must be refused before publication",
    );
    assert_base_state(
        &engine,
        "security valid Unicode Text prefix after refused frame",
    );
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("security Unicode Text collection stats")
            .documents_indexed,
        2,
        "a refused complete oversized Unicode Text frame must not add its row after the valid two-document prefix",
    );
    let store =
        SegmentRdbStore::new(root.join("complete-corrupt-oversized-unicode-token-segments"))
            .expect("open security Unicode Text segment store");
    let saved = store
        .save_with_sequence(&engine, 0)
        .expect("save only the valid Unicode Text AOF prefix");
    assert_eq!(
        saved, BASE_SEQUENCE,
        "a refused oversized Unicode Text frame must not advance the durable checkpoint watermark",
    );
    let cold = store
        .load_current_generation()
        .expect("cold-open valid security Unicode Text prefix")
        .expect("valid security Unicode Text prefix checkpoint exists");
    assert_eq!(
        cold.sequence, BASE_SEQUENCE,
        "cold-open must retain only the valid oversized Unicode Text prefix watermark",
    );
    assert_base_state(
        &cold.engine,
        "security cold Unicode Text prefix after refused frame",
    );
}

fn child_paths() -> Option<(PathBuf, PathBuf, PathBuf)> {
    if std::env::var(CHILD_MODE_ENV).ok().as_deref() != Some(CHILD_CASE) {
        return None;
    }
    let root = std::env::var_os(CHILD_ROOT_ENV)
        .map(PathBuf::from)
        .expect("isolated Unicode Text replay child needs its durable root");
    let aof = std::env::var_os(CHILD_AOF_ENV)
        .map(PathBuf::from)
        .expect("isolated Unicode Text replay child needs its AOF path");
    let handshake = std::env::var_os(CHILD_HANDSHAKE_ENV)
        .map(PathBuf::from)
        .expect("isolated Unicode Text replay child needs a handshake path");
    Some((root, aof, handshake))
}

async fn run_replay_child(root: PathBuf, aof_path: PathBuf, handshake: PathBuf) {
    fs::write(&handshake, CHILD_CASE).expect("record exact isolated Unicode Text child entry");
    let base_engine = Arc::new(Engine::new());
    let base_replay = replay_aof_into(&base_engine, &aof_path, 0);
    assert!(
        base_replay.is_ok(),
        "the valid committed Unicode Text base must replay before the giant suffix: {}",
        base_replay
            .as_ref()
            .err()
            .map(|error| format!("{error:#}"))
            .unwrap_or_default(),
    );
    assert_eq!(
        base_replay.expect("behavior assertion above checked Unicode Text base replay"),
        BASE_SEQUENCE,
        "the committed Unicode Text base replay must advance through its exact sequence",
    );
    assert_base_state(&base_engine, "committed Unicode Text base AOF replay");
    assert_public_pending_budget(
        base_engine.clone(),
        BASE_SEQUENCE,
        "committed Unicode Text base AOF replay",
    )
    .await;

    let store = SegmentRdbStore::new(root.join("segments"))
        .expect("open segment store for oversized Unicode Text AOF checkpoint");
    let base_checkpoint = store
        .save_with_sequence(&base_engine, BASE_SEQUENCE)
        .expect("checkpoint committed Unicode Text base before giant suffix");
    assert_eq!(
        base_checkpoint, BASE_SEQUENCE,
        "the initial Unicode Text checkpoint must retain its established watermark",
    );
    let base_cold = store
        .load_current_generation()
        .expect("cold-open committed Unicode Text base checkpoint")
        .expect("committed Unicode Text base checkpoint must publish CURRENT");
    assert_eq!(
        base_cold.sequence, BASE_SEQUENCE,
        "cold Unicode Text base checkpoint must keep its exact watermark",
    );
    assert_base_state(
        &base_cold.engine,
        "cold committed Unicode Text base checkpoint",
    );

    append_oversized_suffix(&aof_path);
    let giant_replay = replay_aof_into(&base_cold.engine, &aof_path, base_cold.sequence);
    assert!(
        giant_replay.is_ok(),
        "a valid committed >256 MiB one-token Unicode Text AOF suffix must replay without full-token owned workspace refusal: {}",
        giant_replay
            .as_ref()
            .err()
            .map(|error| format!("{error:#}"))
            .unwrap_or_default(),
    );
    assert_eq!(
        giant_replay.expect("behavior assertion above checked Unicode giant replay"),
        GIANT_SEQUENCE,
        "valid oversized Unicode Text replay must advance through its committed suffix sequence",
    );
    assert_eq!(
        base_cold
            .engine
            .stats(COLLECTION)
            .expect("live oversized Unicode Text stats")
            .documents_indexed,
        3,
        "the giant Unicode token must become a third live document before its checkpoint",
    );
    assert_public_pending_budget(
        base_cold.engine.clone(),
        GIANT_SEQUENCE,
        "oversized Unicode Text AOF suffix replay",
    )
    .await;

    let giant_checkpoint = store
        .save_with_sequence(&base_cold.engine, GIANT_SEQUENCE)
        .expect("incrementally checkpoint oversized Unicode Text suffix over its base");
    assert_eq!(
        giant_checkpoint, GIANT_SEQUENCE,
        "incremental Unicode Text checkpoint must retain the giant suffix watermark",
    );
    let giant_cold = store
        .load_current_generation()
        .expect("cold-open incremental Unicode Text checkpoint")
        .expect("incremental Unicode Text checkpoint must publish CURRENT");
    assert_eq!(
        giant_cold.sequence, GIANT_SEQUENCE,
        "cold incremental Unicode Text checkpoint must retain the giant sequence",
    );
    assert_giant_cold_state(&giant_cold.engine, "cold oversized Unicode Text checkpoint");
    assert_public_pending_budget(
        giant_cold.engine.clone(),
        GIANT_SEQUENCE,
        "cold oversized Unicode Text checkpoint",
    )
    .await;

    append_tail(&aof_path);
    let tail_replay = replay_aof_into(&giant_cold.engine, &aof_path, giant_cold.sequence);
    assert!(
        tail_replay.is_ok(),
        "versioned Unicode Text update and delete suffix must replay after the giant checkpoint: {}",
        tail_replay
            .as_ref()
            .err()
            .map(|error| format!("{error:#}"))
            .unwrap_or_default(),
    );
    assert_eq!(
        tail_replay.expect("behavior assertion above checked Unicode Text tail replay"),
        DELETE_SEQUENCE,
        "Unicode Text tail replay must advance through its final committed sequence",
    );
    assert_tail_state(
        &giant_cold.engine,
        "Unicode Text version/delete suffix replay",
    );
    assert_giant_absent_after_update(
        &giant_cold.engine,
        "Unicode Text version/delete suffix replay",
    );
    assert_public_pending_budget(
        giant_cold.engine.clone(),
        DELETE_SEQUENCE,
        "Unicode Text version/delete suffix replay",
    )
    .await;

    let final_checkpoint = store
        .save_with_sequence(&giant_cold.engine, DELETE_SEQUENCE)
        .expect("checkpoint Unicode Text version/delete suffix");
    assert_eq!(
        final_checkpoint, DELETE_SEQUENCE,
        "final Unicode Text checkpoint must retain the completed AOF watermark",
    );
    let final_cold = store
        .load_current_generation()
        .expect("cold-open final oversized Unicode Text checkpoint")
        .expect("final oversized Unicode Text checkpoint must publish CURRENT");
    assert_eq!(
        final_cold.sequence, DELETE_SEQUENCE,
        "final cold Unicode Text checkpoint must retain the full watermark",
    );
    assert_tail_state(&final_cold.engine, "final Unicode Text cold reopen");
    assert_public_pending_budget(
        final_cold.engine.clone(),
        DELETE_SEQUENCE,
        "final oversized Unicode Text cold reopen",
    )
    .await;

    let covered_replay = replay_aof_into(&final_cold.engine, &aof_path, final_cold.sequence);
    assert!(
        covered_replay.is_ok(),
        "fully checkpoint-covered Unicode Text AOF frames must be skipped without replay failure: {}",
        covered_replay
            .as_ref()
            .err()
            .map(|error| format!("{error:#}"))
            .unwrap_or_default(),
    );
    assert_eq!(
        covered_replay.expect("behavior assertion above checked covered Unicode Text replay"),
        0,
        "fully checkpoint-covered Unicode Text frames must not advance the watermark",
    );
    assert_tail_state(&final_cold.engine, "fully covered Unicode Text AOF replay");
}

async fn run_isolated_replay(root: &Path, aof_path: &Path) {
    let child_root = tempfile::tempdir().expect("isolated Unicode Text child workspace");
    let child_tmp = child_root.path().join("child-tmp");
    fs::create_dir(&child_tmp).expect("create parent-owned Unicode Text child temporary directory");
    let handshake = child_root.path().join("entered-case");
    let stdout_path = child_root.path().join("child.stdout");
    let stderr_path = child_root.path().join("child.stderr");
    let executable =
        std::env::current_exe().expect("current oversized Unicode Text test executable");
    let stdout = File::create(&stdout_path).expect("create Unicode Text child stdout");
    let stderr = File::create(&stderr_path).expect("create Unicode Text child stderr");
    let child = Command::new(executable)
        .env(CHILD_MODE_ENV, CHILD_CASE)
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
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .expect("spawn isolated oversized Unicode Text replay child");
    let mut child = ChildCleanup(Some(child));
    let deadline = Instant::now() + REPLAY_WATCHDOG;
    loop {
        match child
            .0
            .as_mut()
            .expect("Unicode Text child remains owned until it exits")
            .try_wait()
        {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => tokio::time::sleep(POLL_INTERVAL).await,
            Ok(None) => {
                let mut raw = child
                    .0
                    .take()
                    .expect("timed-out Unicode Text child remains owned");
                let _ = raw.kill();
                let status = raw.wait().expect("wait for killed Unicode Text child");
                let stdout = fs::read_to_string(&stdout_path)
                    .expect("read killed Unicode Text child stdout");
                let stderr = fs::read_to_string(&stderr_path)
                    .expect("read killed Unicode Text child stderr");
                panic!(
                    "a valid committed >256 MiB one-token Unicode Text AOF frame must replay and checkpoint before cleanup; child was killed after {REPLAY_WATCHDOG:?}: status={status}; stdout={stdout}; stderr={stderr}",
                );
            }
            Err(error) => panic!("poll isolated oversized Unicode Text child: {error}"),
        }
    }
    let status = child
        .0
        .take()
        .expect("exited Unicode Text child remains owned")
        .wait()
        .expect("wait for exited Unicode Text child");
    let stdout = fs::read_to_string(&stdout_path).expect("read Unicode Text child stdout");
    let stderr = fs::read_to_string(&stderr_path).expect("read Unicode Text child stderr");
    let entered = fs::read_to_string(&handshake).unwrap_or_else(|error| {
        panic!(
            "isolated child did not enter exact {TEST_NAME}: {error}; stdout={stdout}; stderr={stderr}",
        )
    });
    assert_eq!(
        entered, CHILD_CASE,
        "isolated child must enter the intended oversized Unicode Text replay body",
    );
    assert!(
        status.success(),
        "isolated oversized Unicode Text replay child failed: status={status}; stdout={stdout}; stderr={stderr}",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn oversized_committed_unicode_token_aof_replays_normalizes_and_cold_opens() {
    if let Some((root, aof_path, handshake)) = child_paths() {
        run_replay_child(root, aof_path, handshake).await;
        return;
    }

    let root = tempfile::tempdir().expect("oversized Unicode Text AOF fixture root");
    let aof_path = root.path().join("oversized-unicode-token.aof");
    append_base_prefix(&aof_path);
    let giant_frame_start = fs::metadata(&aof_path)
        .expect("measure Unicode Text base prefix before isolated giant append")
        .len();
    run_isolated_replay(root.path(), &aof_path).await;
    assert_complete_corrupt_oversized_frame_is_refused(root.path(), &aof_path, giant_frame_start);
}
