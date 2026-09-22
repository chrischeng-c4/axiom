//! # Facets
//!
//! - Behavior: segment_startup_replay_budget.rs:58, :63, :72, :77, :90, and
//!   :95 require a real lumen serve process in segment mode to finish a valid
//!   AOF replay, publish at least one replay-phase checkpoint, and answer
//!   live and cold exact Keyword queries.
//!   Change points: apps/lumen/src/bin/lumen.rs:3585-3610 starts replay, and
//!   apps/lumen/src/segment_checkpoint.rs:221-255 publishes and trims a
//!   checkpoint.
//! - Security: segment_startup_replay_budget.rs:86 invokes
//!   support/serve_budget_support.rs:476, :483, and :488 over locally stored
//!   aof.log: each missing caller-record sequence must be covered by CURRENT,
//!   and every sequence later than CURRENT must remain.
//!   This closed result protects the read/trim boundary in
//!   apps/lumen/src/aof.rs:212-270 and
//!   apps/lumen/src/segment_checkpoint.rs:221-255. Existing malformed-root
//!   refusal remains covered by
//!   apps/lumen/e2e/segment_startup_fail_closed_e2e.rs:714-740.
//! - Performance: apps/lumen/ROADMAP.md:60-70 promises, verbatim, "Pending
//!   active, frozen, and reserved changes have a 256 MiB total budget" and
//!   "At 128 MiB, the runtime requests an early checkpoint."
//!   segment_startup_replay_budget.rs:45 and
//!   support/serve_budget_support.rs:411, :415, and :419, invoked at :71,
//!   check the real replay reaches the threshold and never reports pending
//!   accounting above 256 MiB.
//!   This is a structural budget assertion, not a latency claim.
//!
//! Gate: cargo test -p lumen --test segment_startup_replay_budget -- --nocapture.

#[path = "support/serve_budget_support.rs"]
mod serve_budget_support;

use serve_budget_support::{
    aof_sequences, assert_keyword_hit, assert_pending_metrics, assert_safe_aof_tail,
    checkpoint_sequence, raw_payload_bytes, replay_event_u64, stats_documents,
    write_large_aof_tail, LumenProcess, ServeMode, LARGE_VALUE_COUNT, PENDING_HARD_BYTES,
};
use std::time::Duration;

#[test]
fn idle_segment_sigterm_drains_before_default_grace_and_cold_restarts() {
    let root = tempfile::tempdir().expect("segment graceful-shutdown root");
    let original_sequences = write_large_aof_tail(root.path());
    let mut process = LumenProcess::spawn(Some(root.path()), ServeMode::Segment, 300);
    process.wait_until_ready(300);
    assert_eq!(stats_documents(&process), LARGE_VALUE_COUNT as u64);
    assert_keyword_hit(&process, 0);
    assert_keyword_hit(&process, LARGE_VALUE_COUNT - 1);

    // Do not shorten the production default. The server must finish an idle
    // HTTP drain before this 30-second upper bound, then retain its durable
    // segment data for a cold restart.
    let default_grace = Duration::from_secs(30);
    process.send_sigterm();
    let (shutdown_elapsed, shutdown_logs) = process.wait_for_exit(default_grace);
    assert!(
        shutdown_logs.contains("http server drained; shutting down"),
        "SIGTERM must report normal HTTP drain completion, not only early process exit; logs:\n{shutdown_logs}",
    );
    assert!(
        shutdown_elapsed < default_grace,
        "idle SIGTERM shutdown must not sleep through the full default grace: {shutdown_elapsed:?}"
    );

    let current_sequence = checkpoint_sequence(root.path());
    let surviving_sequences = aof_sequences(root.path());
    assert_safe_aof_tail(&original_sequences, &surviving_sequences, current_sequence);

    let mut cold = LumenProcess::spawn(Some(root.path()), ServeMode::Segment, 300);
    cold.wait_until_ready(300);
    assert_eq!(
        stats_documents(&cold),
        LARGE_VALUE_COUNT as u64,
        "cold segment startup must recover all durable records after graceful shutdown",
    );
    assert_keyword_hit(&cold, 0);
    assert_keyword_hit(&cold, LARGE_VALUE_COUNT - 1);
}

#[test]
fn segment_aof_replay_over_budget_charges_during_bootstrap_and_cold_restarts() {
    let root = tempfile::tempdir().expect("segment replay budget root");
    let original_sequences = write_large_aof_tail(root.path());
    assert_eq!(
        raw_payload_bytes(),
        269_123_584,
        "fixture must keep its approved 43 x legal 6 MiB raw input premise stable",
    );
    assert!(
        raw_payload_bytes() > PENDING_HARD_BYTES as usize,
        "fixture must cross the approved 256 MiB pending-change budget with distinct AOF values",
    );

    // 300 seconds prevents the regular serving driver from being the evidence
    // for synchronous bootstrap replay. The startup event is emitted before
    // that driver is constructed.
    let mut process = LumenProcess::spawn(Some(root.path()), ServeMode::Segment, 300);
    process.wait_until_ready(300);
    let startup_logs = process.logs();

    let replay_checkpoints = replay_event_u64(&startup_logs, "replay_checkpoints");
    assert!(
        replay_checkpoints.is_some(),
        "AOF startup decision must expose numeric replay_checkpoints for replay-phase checkpoint evidence; logs:\n{startup_logs}",
    );
    assert!(
        replay_checkpoints.expect("prior assertion established replay_checkpoints") >= 1,
        "AOF startup must complete at least one durable checkpoint during replay, before normal serving handoff; logs:\n{startup_logs}",
    );
    assert!(
        replay_event_u64(&startup_logs, "replay_ms").is_some(),
        "AOF startup decision must expose numeric replay_ms beside replay checkpoint evidence; logs:\n{startup_logs}",
    );

    assert_pending_metrics(&process.get_text("/metrics"));
    assert_eq!(
        stats_documents(&process),
        LARGE_VALUE_COUNT as u64,
        "all valid AOF records must be visible after startup replay",
    );
    for ordinal in [0, LARGE_VALUE_COUNT / 2, LARGE_VALUE_COUNT - 1] {
        assert_keyword_hit(&process, ordinal);
    }

    // Reap the process before reading disk state. Drop still kills and reaps
    // it on every earlier panic path.
    process.stop_and_logs();
    let current_sequence = checkpoint_sequence(root.path());
    let surviving_sequences = aof_sequences(root.path());
    assert_safe_aof_tail(&original_sequences, &surviving_sequences, current_sequence);

    let mut cold = LumenProcess::spawn(Some(root.path()), ServeMode::Segment, 300);
    cold.wait_until_ready(300);
    assert_eq!(
        stats_documents(&cold),
        LARGE_VALUE_COUNT as u64,
        "cold segment startup must recover every checkpointed and retained-tail record",
    );
    for ordinal in [0, LARGE_VALUE_COUNT / 2, LARGE_VALUE_COUNT - 1] {
        assert_keyword_hit(&cold, ordinal);
    }
}

mod large_aof_codec_compatibility {
    //! # Facets
    //!
    //! - Behavior: `segment_startup_replay_budget.rs:294`, `:319`, `:323`,
    //!   `:327`, `:402`, `:499`, `:505`, `:518`, and `:524` require authentic
    //!   fast, generic CBOR, and legacy JSON frames, exact decoder output, full
    //!   live and cold Keyword matches, and all three applied documents. Change
    //!   points are `apps/lumen/src/aof.rs:264-315` and
    //!   `apps/lumen/src/wal.rs:142-166`.
    //! - Security: `segment_startup_replay_budget.rs:339`, `:428`, `:509`, and
    //!   `:527` keep every file-controlled payload below the 64 MiB framed-log
    //!   maximum and retain each AOF sequence until CURRENT covers it through
    //!   `apps/lumen/e2e/support/serve_budget_support.rs:516` and `:522`.
    //!   Existing hostile-frame refusal remains covered by
    //!   `apps/lumen/e2e/segment_startup_fail_closed_e2e.rs:923`, `:939`, and
    //!   `:943`.
    //! - Performance: no new number. `apps/lumen/e2e/support/serve_budget_support.rs:32`
    //!   gives each local 60 MiB HTTP request the existing bounded 30-second
    //!   fixture deadline. The release command in `apps/lumen/README.md:253`
    //!   and durable workload in `apps/lumen/e2e/perf_gate.rs:722-725` retain
    //!   the latency and RSS evidence for this AOF path.

    use std::collections::BTreeMap;
    use std::path::Path;

    use super::serve_budget_support::{
        aof_sequences, assert_safe_aof_tail, LumenProcess, ServeMode,
    };
    use lumen::aof::AofReader;
    use lumen::log_entry::RaftLogEntry;
    use lumen::segment_rdb::SegmentRdbStore;
    use lumen::types::{
        CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    };
    use lumen::wal::WalRecord;
    use serde_json::{json, Value};
    use storage_durable::{FramedLogWriter, FsyncPolicy, MAX_FRAME_PAYLOAD_BYTES};

    const COLLECTION: &str = "large-aof-codec-compat";
    const FIELD: &str = "keyword";
    const LARGE_VALUE_BYTES: usize = 60 * 1024 * 1024;
    const QUERY_BODY_LIMIT_BYTES: usize = 64 * 1024 * 1024;
    const SNAPSHOT_SECS: u64 = 300;
    const LAST_SEQUENCE: u64 = 4;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum LargeAofCodec {
        Fast,
        Cbor,
        LegacyJson,
    }

    impl LargeAofCodec {
        const fn label(self) -> &'static str {
            match self {
                Self::Fast => "fast",
                Self::Cbor => "cbor",
                Self::LegacyJson => "legacy-json",
            }
        }
    }

    #[derive(Clone, Copy)]
    struct LargeAofCase {
        codec: LargeAofCodec,
        sequence: u64,
        external_id: &'static str,
    }

    const CASES: [LargeAofCase; 3] = [
        LargeAofCase {
            codec: LargeAofCodec::Fast,
            sequence: 2,
            external_id: "large-fast",
        },
        LargeAofCase {
            codec: LargeAofCodec::Cbor,
            sequence: 3,
            external_id: "large-cbor",
        },
        LargeAofCase {
            codec: LargeAofCodec::LegacyJson,
            sequence: 4,
            external_id: "large-legacy-json",
        },
    ];

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

    fn prefix(case: LargeAofCase) -> String {
        format!("aof-compat-{}-prefix:", case.codec.label())
    }

    fn suffix(case: LargeAofCase) -> String {
        format!(":aof-compat-{}-suffix", case.codec.label())
    }

    /// Exactly 60 MiB. Both ends differ by codec so a shortened value, a swapped
    /// record, or a decoder that preserves only one end cannot satisfy the query.
    fn full_value(case: LargeAofCase) -> String {
        let prefix = prefix(case);
        let suffix = suffix(case);
        assert!(
            prefix.len() + suffix.len() < LARGE_VALUE_BYTES,
            "compatibility markers must leave a real 60 MiB payload body"
        );
        let mut bytes = vec![b'x'; LARGE_VALUE_BYTES];
        bytes[..prefix.len()].copy_from_slice(prefix.as_bytes());
        bytes[LARGE_VALUE_BYTES - suffix.len()..].copy_from_slice(suffix.as_bytes());
        String::from_utf8(bytes).expect("large compatibility Keyword must remain ASCII")
    }

    fn index_record(case: LargeAofCase) -> WalRecord {
        WalRecord::new(RaftLogEntry::Index {
            collection_id: COLLECTION.into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: case.external_id.into(),
                    field: FIELD.into(),
                    value: FieldValue::String(full_value(case)),
                    version: None,
                }],
                request_id: None,
            },
        })
    }

    fn assert_decoded_exact_value(record: &WalRecord, case: LargeAofCase, stage: &str) {
        let RaftLogEntry::Index { collection_id, req } = &record.entry else {
            panic!(
                "{stage}: {} payload did not decode as an Index record",
                case.codec.label()
            );
        };
        assert_eq!(
            collection_id,
            COLLECTION,
            "{stage}: {} payload changed collection identity",
            case.codec.label()
        );
        assert_eq!(
            req.items.len(),
            1,
            "{stage}: {} payload changed item count",
            case.codec.label()
        );
        let item = &req.items[0];
        assert_eq!(
            item.external_id,
            case.external_id,
            "{stage}: {} payload changed external ID",
            case.codec.label()
        );
        assert_eq!(
            item.field,
            FIELD,
            "{stage}: {} payload changed Keyword field",
            case.codec.label()
        );
        let FieldValue::String(value) = &item.value else {
            panic!(
                "{stage}: {} payload changed Keyword value type",
                case.codec.label()
            );
        };
        let expected = full_value(case);
        assert!(
            value.starts_with(&prefix(case)),
            "{stage}: {} decoder lost the distinguishing value prefix",
            case.codec.label()
        );
        assert!(
            value.ends_with(&suffix(case)),
            "{stage}: {} decoder lost the distinguishing value suffix",
            case.codec.label()
        );
        assert!(
            value == &expected,
            "{stage}: {} decoder changed the exact {}-byte Keyword value (observed {} bytes)",
            case.codec.label(),
            LARGE_VALUE_BYTES,
            value.len(),
        );
    }

    fn encode_case_payload(case: LargeAofCase) -> Vec<u8> {
        let record = index_record(case);
        let payload = match case.codec {
            LargeAofCodec::Fast => record.encode().expect("encode fast compatibility record"),
            LargeAofCodec::Cbor => {
                let mut bytes = Vec::new();
                ciborium::ser::into_writer(&record, &mut bytes)
                    .expect("force generic CBOR compatibility record");
                bytes
            }
            LargeAofCodec::LegacyJson => {
                serde_json::to_vec(&record).expect("force legacy JSON compatibility record")
            }
        };
        drop(record);
        match case.codec {
            LargeAofCodec::Fast => assert!(
                payload.starts_with(b"LWAL"),
                "fast compatibility record must use the authentic fast WAL encoder"
            ),
            LargeAofCodec::Cbor => assert!(
                !payload.starts_with(b"LWAL"),
                "forced generic CBOR compatibility record must not use the fast WAL marker"
            ),
            LargeAofCodec::LegacyJson => assert!(
                payload.starts_with(b"{"),
                "forced legacy JSON compatibility record must retain JSON bytes"
            ),
        }
        assert!(
            payload.len() >= LARGE_VALUE_BYTES,
            "{} payload must include the complete {}-byte Keyword value; payload_bytes={}",
            case.codec.label(),
            LARGE_VALUE_BYTES,
            payload.len(),
        );
        assert!(
            payload.len() < MAX_FRAME_PAYLOAD_BYTES,
            "{} payload must remain below storage-durable's {}-byte frame maximum; payload_bytes={}",
            case.codec.label(),
            MAX_FRAME_PAYLOAD_BYTES,
            payload.len(),
        );
        let decoded = WalRecord::decode(&payload).unwrap_or_else(|error| {
            panic!(
                "preexisting decoder must accept the complete {} payload ({} bytes): {error}",
                case.codec.label(),
                payload.len(),
            )
        });
        assert_decoded_exact_value(&decoded, case, "preflight decoder");
        payload
    }

    fn write_large_compatibility_aof(root: &Path) -> Vec<u64> {
        let path = root.join("aof.log");
        let schema = WalRecord::new(RaftLogEntry::CreateCollection {
            collection_id: COLLECTION.into(),
            req: keyword_schema(),
        });
        let schema_payload = schema.encode().expect("encode schema AOF frame");
        assert!(
            WalRecord::decode(&schema_payload).is_ok(),
            "preexisting decoder must accept the schema AOF payload"
        );
        let mut writer = FramedLogWriter::open(&path, FsyncPolicy::Always)
            .expect("open CRC-framed compatibility AOF");
        writer
            .append(1, &schema_payload)
            .expect("append schema CRC frame");
        for case in CASES {
            let payload = encode_case_payload(case);
            writer
                .append(case.sequence, &payload)
                .unwrap_or_else(|error| panic!("append {} CRC frame: {error}", case.codec.label()));
        }
        writer
            .sync_strict()
            .expect("strict-sync complete compatibility AOF");
        drop(writer);

        let mut replayed = Vec::new();
        let max_sequence = AofReader::replay(&path, 0, |sequence, record| {
            if sequence == 1 {
                assert!(
                    matches!(record.entry, RaftLogEntry::CreateCollection { .. }),
                    "first compatibility frame must remain the schema record"
                );
            } else {
                let case = CASES
                    .iter()
                    .copied()
                    .find(|case| case.sequence == sequence)
                    .unwrap_or_else(|| panic!("unexpected compatibility sequence {sequence}"));
                assert_decoded_exact_value(&record, case, "CRC-framed AOF reader");
            }
            replayed.push(sequence);
        })
        .expect("AOF reader must accept every complete compatibility frame");
        assert_eq!(
            replayed,
            vec![1, 2, 3, 4],
            "AOF reader must reach every complete compatibility frame in order"
        );
        assert_eq!(
            max_sequence, LAST_SEQUENCE,
            "AOF reader must report the last compatibility sequence"
        );
        replayed
    }

    fn current_checkpoint_sequence(root: &Path) -> u64 {
        SegmentRdbStore::new(root)
            .expect("open segment root after compatibility replay")
            .load_current_generation()
            .expect("read compatibility CURRENT")
            .map(|generation| generation.sequence)
            .unwrap_or(0)
    }

    /// A restart may compact a frame only after `CURRENT` covers that sequence.
    /// Check this after each child, because a correct live replay cannot excuse an
    /// unsafe trim performed while the cold child reconstructs the same values.
    fn assert_checkpoint_and_aof_safety(root: &Path, original_sequences: &[u64], stage: &str) {
        let current_sequence = current_checkpoint_sequence(root);
        assert!(
            current_sequence <= LAST_SEQUENCE,
            "{stage}: CURRENT cannot advance past the last complete compatibility AOF sequence"
        );
        let surviving_sequences = aof_sequences(root);
        assert_safe_aof_tail(original_sequences, &surviving_sequences, current_sequence);
    }

    fn documents_indexed(process: &LumenProcess) -> u64 {
        let body: Value =
            serde_json::from_str(&process.get_text(&format!("/collections/{COLLECTION}/stats")))
                .expect("decode compatibility stats response");
        body["documents_indexed"]
            .as_u64()
            .expect("compatibility stats must report documents_indexed")
    }

    fn assert_exact_keyword_hit(process: &LumenProcess, case: LargeAofCase, stage: &str) {
        let value = full_value(case);
        assert!(
            value.starts_with(&prefix(case)) && value.ends_with(&suffix(case)),
            "{stage}: test fixture must send both distinguishing Keyword ends"
        );
        let request = json!({
            "query": { "term": { "field": FIELD, "value": value } },
            "limit": 2,
        });
        let body_bytes = serde_json::to_vec(&request).expect("serialize full compatibility query");
        assert!(
            body_bytes.len() < QUERY_BODY_LIMIT_BYTES,
            "{stage}: full {} query must fit inside the child-only {}-byte body limit; bytes={}",
            case.codec.label(),
            QUERY_BODY_LIMIT_BYTES,
            body_bytes.len(),
        );
        let response = process.post_json(&format!("/collections/{COLLECTION}/search"), &request);
        assert_eq!(
            response.status,
            200,
            "{stage}: full {} Keyword query must succeed: {}",
            case.codec.label(),
            response.body
        );
        assert_eq!(
            response.body["total"],
            1,
            "{stage}: full {} Keyword value must match exactly: {}",
            case.codec.label(),
            response.body
        );
        assert_eq!(
            response.body["hits"][0]["external_id"],
            case.external_id,
            "{stage}: full {} Keyword value must select its own document: {}",
            case.codec.label(),
            response.body
        );
    }

    #[test]
    fn complete_large_fast_cbor_and_legacy_json_aof_frames_replay_live_and_cold() {
        let root = tempfile::tempdir().expect("large AOF compatibility root");
        let original_sequences = write_large_compatibility_aof(root.path());

        let mut live = LumenProcess::spawn_with_body_limit(
            Some(root.path()),
            ServeMode::Segment,
            SNAPSHOT_SECS,
            QUERY_BODY_LIMIT_BYTES,
        );
        live.wait_until_ready(SNAPSHOT_SECS);
        assert_eq!(
            documents_indexed(&live),
            CASES.len() as u64,
            "live replay must apply all three large compatibility records"
        );
        for case in CASES {
            assert_exact_keyword_hit(&live, case, "live replay");
        }
        let _ = live.stop_and_logs();

        assert_checkpoint_and_aof_safety(root.path(), &original_sequences, "live replay");

        let mut cold = LumenProcess::spawn_with_body_limit(
            Some(root.path()),
            ServeMode::Segment,
            SNAPSHOT_SECS,
            QUERY_BODY_LIMIT_BYTES,
        );
        cold.wait_until_ready(SNAPSHOT_SECS);
        assert_eq!(
            documents_indexed(&cold),
            CASES.len() as u64,
            "cold replay must recover all three large compatibility records"
        );
        for case in CASES {
            assert_exact_keyword_hit(&cold, case, "cold replay");
        }
        let _ = cold.stop_and_logs();
        assert_checkpoint_and_aof_safety(root.path(), &original_sequences, "cold replay");
    }
}
