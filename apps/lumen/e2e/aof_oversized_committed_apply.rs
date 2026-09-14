//! Black-box contract for oversized committed scalar AOF replay.
//!
//! This fixture writes one valid legacy AOF frame containing a real fast LWAL
//! Index record with 1,000 distinct Keyword values of 270 KiB each. It uses
//! AofWriter for the normal create frame. The oversized frame uses the
//! documented 16-byte AOF header directly because the current public writer
//! caps one frame at 64 MiB, while this retained record is larger than the
//! 256 MiB pending-change budget. The frame still has the production sequence,
//! length, and CRC layout and its payload comes from WalRecord::encode().
//!
//! The fixture owns the large encoded Vec<u8> only while writing the file and
//! drops it before replay. The public pending-change metrics below do not claim
//! to measure that fixture allocation or the mapped AOF source bytes.
//!
//! # Facets
//!
//! - Behavior: aof_oversized_committed_apply.rs:514-527 requires the valid
//!   oversized prefix to replay completely. :337-352, :541-580, :355-385,
//!   and :591-626 require all rows, a strict AOF suffix after the checkpoint,
//!   the later update and delete, a final checkpoint, and cold reopen. These
//!   assertions
//!   cover apps/lumen/src/aof.rs:248-377 and
//!   apps/lumen/src/segment_rdb.rs:489-506,939-960.
//! - Security: aof_oversized_committed_apply.rs:445-448,469-503 feeds a CRC-valid,
//!   complete AOF frame whose fast LWAL payload is truncated. It requires the
//!   replay to refuse the frame, leave its indexed value absent, and save only
//!   the prior watermark. It covers persisted bytes at
//!   apps/lumen/src/aof.rs:266-325.
//! - Performance: apps/lumen/docs/indexing.md:264-272 says, verbatim,
//!   "Pending active, frozen, and reserved changes have a 256 MiB budget."
//!   aof_oversized_committed_apply.rs:415-422, called at :529-534, :545-550,
//!   :581-586, and :619-624, reads public /metrics total and high-water gauges
//!   and requires both to stay within that budget. The fixture source allocation
//!   and mapped AOF bytes are outside this pending-change metric scope.
//!
//! Gate: cargo test -p lumen --test aof_oversized_committed_apply -- --nocapture.

use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Arc;

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
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest, QueryNode,
    SearchRequest, TermQuery,
};
use lumen::wal::WalRecord;

const COLLECTION: &str = "oversized-aof-committed-scalar";
const FIELD: &str = "keyword";
const ITEM_COUNT: usize = 1_000;
const VALUE_BYTES: usize = 270 * 1024;
const PENDING_HARD_LIMIT_BYTES: usize = 256 * 1024 * 1024;
const CURRENT_AOF_FRAME_LIMIT_BYTES: usize = 64 * 1024 * 1024;
const AOF_FRAME_HEADER_BYTES: u64 = 16;
const CREATE_SEQUENCE: u64 = 1;
const INDEX_SEQUENCE: u64 = 2;
const UPDATE_SEQUENCE: u64 = 3;
const DELETE_SEQUENCE: u64 = 4;
const UPDATED_ORDINAL: usize = 0;
const DELETED_ORDINAL: usize = 1;
const UPDATED_VALUE: &str = "later-AOF-scalar-update";

#[derive(Clone, Copy)]
struct FrameSpan {
    start: u64,
    payload_bytes: usize,
}

/// Supplies the replayed sequence to the read-only metrics application.
///
/// A metrics request must never write. Keeping `submit` closed catches an
/// accidental route change while avoiding a second coordinator that would
/// reset the Engine capture barrier to sequence zero.
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
    format!("oversized-aof-scalar-{ordinal:04}")
}

fn original_value(ordinal: usize) -> String {
    let prefix = format!("oversized-aof-value-{ordinal:04}:begin:");
    let suffix = format!(":end:{ordinal:04}:oversized-aof-value");
    assert!(
        prefix.len() + suffix.len() < VALUE_BYTES,
        "fixture markers must leave a true 270 KiB Keyword value",
    );

    let mut bytes = vec![b'x'; VALUE_BYTES];
    bytes[..prefix.len()].copy_from_slice(prefix.as_bytes());
    bytes[VALUE_BYTES - suffix.len()..].copy_from_slice(suffix.as_bytes());
    String::from_utf8(bytes).expect("fixture Keyword values are valid UTF-8")
}

fn create_entry() -> RaftLogEntry {
    RaftLogEntry::CreateCollection {
        collection_id: COLLECTION.to_owned(),
        req: keyword_schema(),
    }
}

fn oversized_index_payload() -> Vec<u8> {
    let mut items = Vec::with_capacity(ITEM_COUNT);
    for ordinal in 0..ITEM_COUNT {
        items.push(IndexItem {
            external_id: external_id(ordinal),
            field: FIELD.to_owned(),
            value: FieldValue::String(original_value(ordinal)),
            version: None,
        });
    }

    let record = WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items,
            request_id: None,
        },
    });
    let payload = record
        .encode()
        .expect("encode the authentic fast Index AOF payload");
    drop(record);

    assert!(
        payload.starts_with(b"LWAL"),
        "the oversized AOF record must use the authentic fast Index codec",
    );
    assert!(
        payload.len() > PENDING_HARD_LIMIT_BYTES,
        "fixture must exceed the 256 MiB pending-change budget: payload_bytes={}",
        payload.len(),
    );
    assert!(
        payload.len() > CURRENT_AOF_FRAME_LIMIT_BYTES,
        "fixture must also cross the current AOF single-frame limit: payload_bytes={}",
        payload.len(),
    );
    payload
}

/// Write exactly the documented AOF frame layout. This is needed only for the
/// legacy oversized frame because AofWriter currently enforces its 64 MiB
/// writer limit before the replay contract can exercise it.
fn append_complete_aof_frame(path: &Path, sequence: u64, payload: &[u8]) -> FrameSpan {
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .expect("open real AOF file for complete frame append");
    let start = file
        .metadata()
        .expect("read AOF size before frame append")
        .len();
    let payload_bytes = u32::try_from(payload.len())
        .expect("fixture AOF payload must fit the documented u32 frame length");
    let mut header = [0_u8; AOF_FRAME_HEADER_BYTES as usize];
    header[0..8].copy_from_slice(&sequence.to_le_bytes());
    header[8..12].copy_from_slice(&payload_bytes.to_le_bytes());
    header[12..16].copy_from_slice(&crc32fast::hash(payload).to_le_bytes());
    file.write_all(&header)
        .expect("write complete AOF frame header");
    file.write_all(payload)
        .expect("write complete AOF frame payload");
    file.sync_all().expect("sync complete AOF frame");
    FrameSpan {
        start,
        payload_bytes: payload.len(),
    }
}

fn append_record_frame(path: &Path, sequence: u64, entry: RaftLogEntry) {
    let payload = WalRecord::new(entry)
        .encode()
        .expect("encode valid AOF tail record");
    append_complete_aof_frame(path, sequence, &payload);
}

fn write_oversized_prefix(path: &Path) -> FrameSpan {
    let mut writer = AofWriter::open(path).expect("open normal AOF writer for schema frame");
    writer
        .append(CREATE_SEQUENCE, &WalRecord::new(create_entry()))
        .expect("append collection schema through AofWriter");
    writer
        .sync_strict()
        .expect("strict-sync normal schema AOF frame");
    drop(writer);

    let payload = oversized_index_payload();
    let span = append_complete_aof_frame(path, INDEX_SEQUENCE, &payload);
    drop(payload);
    span
}

fn append_later_update_and_delete(path: &Path) {
    append_record_frame(
        path,
        UPDATE_SEQUENCE,
        RaftLogEntry::Index {
            collection_id: COLLECTION.to_owned(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: external_id(UPDATED_ORDINAL),
                    field: FIELD.to_owned(),
                    value: FieldValue::String(UPDATED_VALUE.to_owned()),
                    version: None,
                }],
                request_id: None,
            },
        },
    );
    append_record_frame(
        path,
        DELETE_SEQUENCE,
        RaftLogEntry::Delete {
            collection_id: COLLECTION.to_owned(),
            external_id: external_id(DELETED_ORDINAL),
            field: None,
        },
    );
}

/// The checkpoint covers the large prefix. A CRC-valid but semantically bad
/// covered frame must be skipped without typed decoding when replay starts at
/// INDEX_SEQUENCE; its following suffix must still apply.
fn poison_covered_prefix_frame(path: &Path, span: FrameSpan) {
    let payload_start = span
        .start
        .checked_add(AOF_FRAME_HEADER_BYTES)
        .expect("covered payload offset fits u64");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("open AOF to poison its checkpoint-covered prefix");
    file.seek(SeekFrom::Start(payload_start))
        .expect("seek to covered AOF payload");
    let mut first = [0_u8; 1];
    file.read_exact(&mut first)
        .expect("read first byte of covered AOF payload");
    first[0] ^= 0xff;
    file.seek(SeekFrom::Start(payload_start))
        .expect("rewind to covered AOF payload");
    file.write_all(&first)
        .expect("poison one covered AOF payload byte");

    let mut hasher = crc32fast::Hasher::new();
    let mut remaining = span.payload_bytes;
    let mut buffer = [0_u8; 64 * 1024];
    file.seek(SeekFrom::Start(payload_start))
        .expect("rewind to stream the covered AOF payload");
    while remaining != 0 {
        let read_len = remaining.min(buffer.len());
        file.read_exact(&mut buffer[..read_len])
            .expect("stream covered AOF payload for CRC");
        hasher.update(&buffer[..read_len]);
        remaining -= read_len;
    }
    file.seek(SeekFrom::Start(
        span.start
            .checked_add(12)
            .expect("covered AOF CRC offset fits u64"),
    ))
    .expect("seek to covered AOF CRC");
    file.write_all(&hasher.finalize().to_le_bytes())
        .expect("rewrite CRC for poisoned covered AOF frame");
    file.sync_all()
        .expect("sync poisoned checkpoint-covered AOF prefix");
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
        .unwrap_or_else(|error| panic!("exact Keyword query failed: {error}"))
        .hits
        .into_iter()
        .map(|hit| hit.external_id)
        .collect()
}

fn assert_every_original_keyword(engine: &Engine, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("oversized AOF collection stats")
            .documents_indexed,
        ITEM_COUNT as u64,
        "{phase}: all 1,000 original external IDs must be present",
    );
    for ordinal in 0..ITEM_COUNT {
        assert_eq!(
            search_ids(engine, &original_value(ordinal)),
            vec![external_id(ordinal)],
            "{phase}: original Keyword row {ordinal} was lost, changed, or swapped",
        );
    }
}

fn assert_tail_state(engine: &Engine, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("oversized AOF collection stats after tail")
            .documents_indexed,
        (ITEM_COUNT - 1) as u64,
        "{phase}: one complete-document delete must remove exactly one row",
    );
    assert_eq!(
        search_ids(engine, UPDATED_VALUE),
        vec![external_id(UPDATED_ORDINAL)],
        "{phase}: later scalar update must replace the original Keyword",
    );
    assert_eq!(
        search_ids(engine, &original_value(UPDATED_ORDINAL)),
        Vec::<String>::new(),
        "{phase}: replaced original Keyword must not revive",
    );
    assert_eq!(
        search_ids(engine, &original_value(DELETED_ORDINAL)),
        Vec::<String>::new(),
        "{phase}: later complete-document delete must not revive",
    );
    for ordinal in 2..ITEM_COUNT {
        assert_eq!(
            search_ids(engine, &original_value(ordinal)),
            vec![external_id(ordinal)],
            "{phase}: unchanged Keyword row {ordinal} was lost, changed, or swapped",
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

async fn assert_public_pending_budget(engine: Arc<Engine>, applied_sequence: u64, phase: &str) {
    let writer: Arc<dyn WriteSink> = Arc::new(ReadOnlyMetricsSink { applied_sequence });
    let state = AppState::with_components(engine, Arc::new(AuthConfig::open()), writer);
    let server = TestServer::new(router(state)).expect("open public metrics server");
    let response = server.get("/metrics").await;
    response.assert_status_ok();
    let metrics = response.text();
    let total = metric_u64(&metrics, "lumen_pending_change_total_bytes");
    let high_water = metric_u64(&metrics, "lumen_pending_change_high_water_bytes");
    assert!(
        total <= PENDING_HARD_LIMIT_BYTES as u64,
        "{phase}: public pending-change total must stay within the approved 256 MiB budget: total={total}",
    );
    assert!(
        high_water <= PENDING_HARD_LIMIT_BYTES as u64,
        "{phase}: public pending-change high water must stay within the approved 256 MiB budget: high_water={high_water}",
    );
}

fn truncated_fast_index_payload() -> Vec<u8> {
    let mut payload = WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items: vec![IndexItem {
                external_id: "malformed-aof-row".to_owned(),
                field: FIELD.to_owned(),
                value: FieldValue::String("must-not-publish".to_owned()),
                version: None,
            }],
            request_id: None,
        },
    })
    .encode()
    .expect("encode small fast AOF control payload");
    assert!(
        payload.starts_with(b"LWAL"),
        "malformed fixture begins as an authentic fast Index payload",
    );
    payload.pop();
    assert!(
        WalRecord::decode(&payload).is_err(),
        "fixture must contain a complete AOF frame with a truncated fast payload",
    );
    payload
}

fn assert_complete_truncated_frame_is_refused_before_watermark(root: &Path) {
    let path = root.join("complete-truncated-frame.aof");
    let mut writer = AofWriter::open(&path).expect("open malformed-frame AOF writer");
    writer
        .append(CREATE_SEQUENCE, &WalRecord::new(create_entry()))
        .expect("append valid predecessor before malformed full frame");
    writer
        .sync_strict()
        .expect("strict-sync valid predecessor AOF frame");
    drop(writer);

    let malformed = truncated_fast_index_payload();
    append_complete_aof_frame(&path, INDEX_SEQUENCE, &malformed);
    drop(malformed);

    let engine = Arc::new(Engine::new());
    let refusal = replay_aof_into(&engine, &path, 0);
    assert!(
        refusal.is_err(),
        "a CRC-valid complete AOF frame with a truncated fast payload must be refused before publication",
    );
    assert_eq!(
        search_ids(&engine, "must-not-publish"),
        Vec::<String>::new(),
        "a refused complete AOF frame must not become query-visible",
    );

    let store =
        SegmentRdbStore::new(root.join("complete-truncated-frame-segments")).expect("open store");
    let saved = store
        .save_with_sequence(&engine, 0)
        .expect("save only the replayed valid prefix");
    assert_eq!(
        saved, CREATE_SEQUENCE,
        "a refused complete AOF frame must not advance the durable checkpoint watermark",
    );
    let cold = store
        .load_current_generation()
        .expect("cold-open valid predecessor checkpoint")
        .expect("valid predecessor checkpoint exists");
    assert_eq!(
        cold.sequence, CREATE_SEQUENCE,
        "cold-open must retain only the valid AOF prefix watermark",
    );
    assert_eq!(
        cold.engine
            .stats(COLLECTION)
            .expect("schema from valid predecessor survives")
            .documents_indexed,
        0,
        "the refused frame's row must stay absent after cold reopen",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn oversized_committed_scalar_aof_replays_with_strict_suffix_checkpoint_and_cold_open() {
    let root = tempfile::tempdir().expect("oversized committed scalar AOF fixture root");
    let aof_path = root.path().join("aof.log");
    let giant_frame = write_oversized_prefix(&aof_path);

    let prefix_engine = Arc::new(Engine::new());
    let prefix_replay = replay_aof_into(&prefix_engine, &aof_path, 0);
    assert!(
        prefix_replay.is_ok(),
        "a valid retained 1,000-by-270-KiB fast Index AOF frame must replay; current refusal: {}",
        prefix_replay
            .as_ref()
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default(),
    );
    assert_eq!(
        prefix_replay.expect("behavior assertion above checked replay result"),
        INDEX_SEQUENCE,
        "valid oversized prefix replay must advance through its committed Index sequence",
    );
    assert_every_original_keyword(&prefix_engine, "initial oversized AOF replay");
    assert_public_pending_budget(
        prefix_engine.clone(),
        INDEX_SEQUENCE,
        "initial oversized AOF replay",
    )
    .await;

    let store = SegmentRdbStore::new(root.path().join("segments"))
        .expect("open segment store for oversized AOF checkpoint");
    let prefix_checkpoint = store
        .save_with_sequence(&prefix_engine, INDEX_SEQUENCE)
        .expect("checkpoint the fully replayed oversized AOF prefix");
    assert_eq!(
        prefix_checkpoint, INDEX_SEQUENCE,
        "prefix checkpoint must retain the committed oversized AOF watermark",
    );
    assert_public_pending_budget(
        prefix_engine.clone(),
        INDEX_SEQUENCE,
        "oversized AOF prefix checkpoint",
    )
    .await;

    append_later_update_and_delete(&aof_path);
    poison_covered_prefix_frame(&aof_path, giant_frame);

    let prefix_cold = store
        .load_current_generation()
        .expect("cold-open oversized AOF prefix checkpoint")
        .expect("oversized AOF prefix checkpoint must publish CURRENT");
    assert_eq!(
        prefix_cold.sequence, INDEX_SEQUENCE,
        "cold prefix checkpoint must keep its exact AOF watermark",
    );
    assert_every_original_keyword(&prefix_cold.engine, "cold oversized AOF prefix checkpoint");

    let suffix_replay = replay_aof_into(&prefix_cold.engine, &aof_path, prefix_cold.sequence);
    assert!(
        suffix_replay.is_ok(),
        "AOF replay from the checkpoint watermark must skip the CRC-valid malformed covered prefix and apply its strict later suffix: {}",
        suffix_replay
            .as_ref()
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default(),
    );
    assert_eq!(
        suffix_replay.expect("behavior assertion above checked suffix replay result"),
        DELETE_SEQUENCE,
        "AOF replay must apply only sequences strictly greater than the checkpoint watermark",
    );
    assert_tail_state(&prefix_cold.engine, "strict AOF suffix after checkpoint");
    assert_public_pending_budget(
        prefix_cold.engine.clone(),
        DELETE_SEQUENCE,
        "strict AOF suffix replay",
    )
    .await;

    let final_checkpoint = store
        .save_with_sequence(&prefix_cold.engine, DELETE_SEQUENCE)
        .expect("checkpoint update and delete after oversized AOF replay");
    assert_eq!(
        final_checkpoint, DELETE_SEQUENCE,
        "final checkpoint must retain the completed AOF suffix watermark",
    );
    let final_cold = store
        .load_current_generation()
        .expect("cold-open final oversized AOF checkpoint")
        .expect("final oversized AOF checkpoint must publish CURRENT");
    assert_eq!(
        final_cold.sequence, DELETE_SEQUENCE,
        "final cold checkpoint must retain the full AOF watermark",
    );
    assert_tail_state(&final_cold.engine, "final cold reopen");
    let fully_covered_replay = replay_aof_into(&final_cold.engine, &aof_path, final_cold.sequence);
    assert!(
        fully_covered_replay.is_ok(),
        "AOF replay at the final watermark must skip every covered frame: {}",
        fully_covered_replay
            .as_ref()
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default(),
    );
    assert_eq!(
        fully_covered_replay.expect("behavior assertion above checked final replay result"),
        0,
        "AOF replay at the final watermark must apply no covered frame",
    );
    assert_public_pending_budget(
        final_cold.engine.clone(),
        DELETE_SEQUENCE,
        "final oversized AOF cold reopen",
    )
    .await;

    assert_complete_truncated_frame_is_refused_before_watermark(root.path());
}
