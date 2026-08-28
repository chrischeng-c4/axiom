// CODEGEN-BEGIN
//! Black-box oracle for issue #3957.
//!
//! A checkpoint SEALS every field of the LIVE collection in place
//! (`Engine::flush_to_segments` takes the write lock and calls
//! `Collection::seal_to_segments` on the in-memory collection itself). The
//! seal moves the field's inverted index onto disk and then FREES the in-RAM
//! driver — in `seal_to_segment`, the Keyword arm runs
//! `k.terms = BTreeMap::new()` and the Set arm runs
//! `s.forward = FastHashMap::default(); s.elements = BTreeMap::new()`.
//!
//! From that moment the sealed base is reachable only through the
//! segment-aware accessors: `term_postings` / `live_terms` for keyword,
//! `element_postings` / `live_elements` / `set_members` for set. Every read
//! path already goes through them, so the LIVE engine stays correct — which
//! is why this is invisible until a restore.
//!
//! `Collection::to_snapshot` does not go through them:
//!
//! | arm | inverted index gathered from | forward gathered from | result |
//! |---|---|---|---|
//! | Text | `tok_postings` + `text_tokens_all` | — | correct |
//! | Number | (none persisted) | `live_number_at` | correct |
//! | Keyword | RAW `k.terms` | `keyword_at` | inverted index truncated to the post-seal tail |
//! | Set | RAW `s.elements` | RAW `s.forward` | field erased outright |
//!
//! `FieldIndex::from_snapshot` then takes the persisted inverted index
//! verbatim (`dup_values: dup_values_of(&t)` over that same truncated map)
//! and sets `segment: None`, so the sealed postings left on disk are
//! unreachable afterwards too. Nothing errors anywhere along the way.
//!
//! The Number arm is the shape the other two should have had: it persists no
//! inverted index at all and REBUILDS `values` from a segment-aware `forward`
//! on restore. Keyword's `forward` is likewise complete, so a keyword field
//! can be repaired on the read side; Set's is not, so Set has to be fixed on
//! the write side.
//!
//! Reported consequence, confirmed at the source: `term`/`terms`/`exists`/
//! `duplicated` read the inverted index (`eval_term` -> `term_postings`,
//! `eval_field_doc_union` -> `live_terms`) and miss every sealed doc, while
//! the per-doc predicate path (`clause_matches` -> `keyword_at`) still answers
//! correctly from the forward column the snapshot did persist. Which of the
//! two a query reaches is a planner decision, so this file asserts only on
//! the posting-driven surfaces, which are decidable without pinning the
//! planner.
//!
//! The round trip is driven through the real HTTP surface — `POST
//! /admin/checkpoint` to seal, `GET /admin/backup` (`Engine::snapshot`) and
//! `POST /admin/restore` (`Engine::restore`) to round-trip — because that is
//! the shape a production backup/restore, an RDB baseline, and the Raft state
//! machine's `snapshot`/`restore` all share.

use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;
use axum_test::TestServer;
use serde_json::json;

use lumen::aof::AofWriter;
use lumen::api::{router, AppState, CheckpointSink};
use lumen::auth::AuthConfig;
use lumen::coordinator::{SharedAof, WriteCoordinator, WriteSink};
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::Engine;
use lumen::wal::{MemWal, SharedWal};

/// Docs indexed BEFORE the checkpoint. The seal moves these onto disk and the
/// snapshot then drops them.
const SEALED_DOCS: usize = 6;
/// Docs indexed AFTER the checkpoint. These live in the post-seal RAM tail,
/// the only part the truncated snapshot preserves — so the fault leaves
/// EXACTLY this many hits behind rather than a round zero, and that number is
/// its signature.
const TAIL_DOCS: usize = 2;
const TOTAL_DOCS: usize = SEALED_DOCS + TAIL_DOCS;

struct LocalCheckpointSink {
    engine: Arc<Engine>,
    store: Arc<SegmentRdbStore>,
    writer: Arc<WriteCoordinator>,
    aof: SharedAof,
}

#[async_trait]
impl CheckpointSink for LocalCheckpointSink {
    async fn checkpoint_now(&self) -> Result<bool> {
        let _permit = self.writer.mutation_gate().shared().await?;
        let sequence = self.writer.applied_seq();
        self.store.save(&self.engine, sequence)?;
        self.store.prune(3)?;
        self.aof
            .lock()
            .map_err(|_| anyhow::anyhow!("aof writer poisoned"))?
            .truncate_through(sequence)?;
        Ok(true)
    }
}

struct Fixture {
    _dir: tempfile::TempDir,
    server: TestServer,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().expect("fixture directory");
    let store =
        Arc::new(SegmentRdbStore::new(&dir.path().join("segments")).expect("segment store"));
    let aof: SharedAof = Arc::new(Mutex::new(
        AofWriter::open(&dir.path().join("aof.log")).expect("aof"),
    ));
    let engine = Arc::new(Engine::new());
    let wal: SharedWal = Arc::new(MemWal::new());
    let writer = WriteCoordinator::start_from_with_aof(wal, engine.clone(), 0, aof.clone());
    let checkpoint = Arc::new(LocalCheckpointSink {
        engine: engine.clone(),
        store,
        writer: writer.clone(),
        aof,
    });
    let state = AppState::with_components(
        engine,
        Arc::new(AuthConfig::open()),
        writer as Arc<dyn WriteSink>,
    )
    .with_checkpoint(checkpoint);
    let server = TestServer::new(router(state)).expect("test server");
    Fixture { _dir: dir, server }
}

/// `total` for `query`. `track_total` is on, so this is the exact
/// matching-set size rather than a page count.
async fn total_for(server: &TestServer, query: serde_json::Value) -> u64 {
    let response = server
        .post("/collections/docs/search")
        .json(&json!({ "query": query, "limit": 100, "track_total": true }))
        .await;
    response.assert_status_ok();
    response.json::<serde_json::Value>()["total"]
        .as_u64()
        .expect("search response carries a numeric total")
}

async fn checkpoint(server: &TestServer) {
    let response = server.post("/admin/checkpoint").await;
    response.assert_status_ok();
    assert_eq!(
        response.json::<serde_json::Value>()["persisted"],
        true,
        "the fixture must wire a real checkpoint sink; a sink that never seals \
         a field would leave this oracle measuring nothing"
    );
}

/// One doc per call, writing every field type this oracle covers in the SAME
/// batch — the shape the issue reports, where the keyword half is lost and
/// the number half is not.
async fn index_doc(server: &TestServer, n: usize) {
    let group = if n <= SEALED_DOCS { "sealed" } else { "tail" };
    server
        .post("/collections/docs/index")
        .json(&json!({ "items": [
            { "external_id": format!("d{n}"), "field": "kw",   "value": format!("v{n}") },
            { "external_id": format!("d{n}"), "field": "grp",  "value": group },
            { "external_id": format!("d{n}"), "field": "tags", "value": [format!("t{n}"), "shared"] },
            { "external_id": format!("d{n}"), "field": "num",  "value": n as f64 },
            { "external_id": format!("d{n}"), "field": "body", "value": format!("doc number {n}") },
        ]}))
        .await
        .assert_status_ok();
}

/// Seal `SEALED_DOCS` docs into a segment, add a `TAIL_DOCS` live tail, then
/// round-trip the whole engine through backup/restore.
async fn sealed_then_round_tripped() -> Fixture {
    let fixture = fixture();
    fixture
        .server
        .put("/collections/docs")
        .json(&json!({
            "fields": {
                "kw":   { "type": "keyword" },
                "grp":  { "type": "keyword" },
                "tags": { "type": "set" },
                "num":  { "type": "number" },
                "body": { "type": "text" }
            }
        }))
        .await
        .assert_status_ok();

    for n in 1..=SEALED_DOCS {
        index_doc(&fixture.server, n).await;
    }

    // The seal: every field's inverted index moves onto disk and the in-RAM
    // driver is freed. The live engine stays correct from here, which the
    // preconditions below pin — the seal itself is not the defect.
    checkpoint(&fixture.server).await;

    for n in SEALED_DOCS + 1..=TOTAL_DOCS {
        index_doc(&fixture.server, n).await;
    }

    for field in ["kw", "grp", "tags", "num"] {
        assert_eq!(
            total_for(&fixture.server, json!({ "exists": { "field": field } })).await,
            TOTAL_DOCS as u64,
            "precondition: the sealed LIVE engine must see every doc on `{field}` \
             before the round trip"
        );
    }
    assert_eq!(
        total_for(
            &fixture.server,
            json!({ "term": { "field": "kw", "value": "v1" } })
        )
        .await,
        1,
        "precondition: a sealed base doc is reachable by `term` on the live engine"
    );

    let backup = fixture.server.get("/admin/backup").await;
    backup.assert_status_ok();
    let snapshot = backup.json::<serde_json::Value>();
    let restore = fixture.server.post("/admin/restore").json(&snapshot).await;
    restore.assert_status(axum::http::StatusCode::NO_CONTENT);

    fixture
}

#[tokio::test]
async fn sealed_keyword_inverted_index_survives_a_snapshot_round_trip() {
    let fixture = sealed_then_round_tripped().await;

    // The defect as a caller meets it: a value that demonstrably exists
    // returns nothing. `eval_term` reads `term_postings`, which after restore
    // holds only what the truncated `terms` map carried.
    assert_eq!(
        total_for(
            &fixture.server,
            json!({ "term": { "field": "kw", "value": "v1" } })
        )
        .await,
        1,
        "`term` on a SEALED base doc's keyword value must still hit after a \
         backup/restore round trip; today `to_snapshot` persists the raw \
         post-seal `k.terms` tail, so this returns 0"
    );

    // `terms` shares the posting source with `term`, so asserting both is what
    // says a fix landed in the shared inverted index rather than in one
    // query's special case.
    assert_eq!(
        total_for(
            &fixture.server,
            json!({ "terms": { "field": "kw", "values": ["v1", "v2"] } })
        )
        .await,
        2,
        "`terms` reads the same postings as `term` and must agree with it"
    );

    // `exists` (`eval_field_doc_union`) shows the truncation's exact size.
    assert_eq!(
        total_for(&fixture.server, json!({ "exists": { "field": "kw" } })).await,
        TOTAL_DOCS as u64,
        "`exists` on a restored keyword field must count every live doc; today \
         it counts only the post-seal tail"
    );

    // `duplicated` reads `dup_values`, which `from_snapshot` derives from the
    // same truncated map — so the sealed group vanishes instead of being
    // reported as duplicated.
    assert_eq!(
        total_for(
            &fixture.server,
            json!({ "duplicated": { "field": "grp", "min_group_size": 2 } })
        )
        .await,
        TOTAL_DOCS as u64,
        "both `grp` groups have >= 2 members, so `duplicated` must return every \
         doc; today the sealed group is gone and only the tail group remains"
    );
    assert_eq!(
        total_for(
            &fixture.server,
            json!({ "term": { "field": "grp", "value": "sealed" } })
        )
        .await,
        SEALED_DOCS as u64,
        "every sealed doc shares one `grp` value, so losing the sealed base \
         costs the whole group at once"
    );
}

#[tokio::test]
async fn sealed_set_inverted_index_survives_a_snapshot_round_trip() {
    let fixture = sealed_then_round_tripped().await;

    // Set is worse than keyword: its snapshot arm reads BOTH `s.elements` and
    // `s.forward` raw and the seal empties both, so a sealed set field is
    // erased rather than merely de-indexed. No forward column survives to
    // repair it from — which is why the fix has to gather through
    // `set_members` / `live_elements` on the WRITE side, not only rebuild on
    // the read side the way keyword can.
    assert_eq!(
        total_for(
            &fixture.server,
            json!({ "term": { "field": "tags", "value": "t1" } })
        )
        .await,
        1,
        "`term` on a SEALED base doc's set member must still hit after a \
         backup/restore round trip"
    );
    assert_eq!(
        total_for(
            &fixture.server,
            json!({ "term": { "field": "tags", "value": "shared" } })
        )
        .await,
        TOTAL_DOCS as u64,
        "every doc carries the `shared` member, so an element spanning the \
         sealed base and the live tail must return both halves"
    );
    assert_eq!(
        total_for(&fixture.server, json!({ "exists": { "field": "tags" } })).await,
        TOTAL_DOCS as u64,
        "`exists` on a restored set field must count every live doc; today it \
         counts only the post-seal tail"
    );
}

/// The other side of the same coin: making `to_snapshot` gather through the
/// segment must not carry a doc the segment still holds but a reader must not
/// see.
///
/// Deleting a SEALED-base doc cannot mutate the on-disk column, so the delete
/// is recorded only in `tombstones`. Every live read path subtracts them —
/// `keyword_at` folds the check inline, `live_terms` / `live_elements` /
/// `live_number_at` all carry it. A gather that reaches the segment WITHOUT
/// subtracting them resurrects the deleted doc into the snapshot's forward
/// column, and `from_snapshot` rebuilds the whole inverted index from exactly
/// that column — so one missed check un-deletes the doc on the far side of the
/// round trip, in every field at once.
///
/// This case is the reason the fix cannot simply be "read through the segment":
/// it has to read through the segment the way a QUERY does.
#[tokio::test]
async fn a_doc_deleted_after_the_seal_stays_deleted_across_the_round_trip() {
    let fixture = fixture();
    fixture
        .server
        .put("/collections/docs")
        .json(&json!({
            "fields": {
                "kw":   { "type": "keyword" },
                "grp":  { "type": "keyword" },
                "tags": { "type": "set" },
                "num":  { "type": "number" },
                "body": { "type": "text" }
            }
        }))
        .await
        .assert_status_ok();

    for n in 1..=SEALED_DOCS {
        index_doc(&fixture.server, n).await;
    }
    checkpoint(&fixture.server).await;
    for n in SEALED_DOCS + 1..=TOTAL_DOCS {
        index_doc(&fixture.server, n).await;
    }

    // `d2` is a sealed-base doc: its postings are in the .lseg and only a
    // tombstone records that it is gone.
    fixture
        .server
        .delete("/collections/docs/index/d2")
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);

    const LIVE: u64 = TOTAL_DOCS as u64 - 1;
    let expectations = [
        (json!({ "term": { "field": "kw", "value": "v2" } }), 0, "kw v2"),
        (json!({ "term": { "field": "tags", "value": "t2" } }), 0, "tags t2"),
        (
            json!({ "term": { "field": "tags", "value": "shared" } }),
            LIVE,
            "tags shared",
        ),
        (
            json!({ "term": { "field": "grp", "value": "sealed" } }),
            SEALED_DOCS as u64 - 1,
            "grp sealed",
        ),
        (json!({ "exists": { "field": "kw" } }), LIVE, "exists kw"),
        (json!({ "exists": { "field": "tags" } }), LIVE, "exists tags"),
    ];

    // Every expectation is first asserted on the LIVE sealed engine, so a
    // failure after the round trip is the round trip's and not the delete's.
    for (query, expected, what) in &expectations {
        assert_eq!(
            total_for(&fixture.server, query.clone()).await,
            *expected,
            "precondition: the LIVE engine must already answer `{what}` without \
             the deleted sealed doc"
        );
    }

    // Exactly ONE round trip. A second would run against an engine whose
    // collection is already `segment: None`, which is no longer the state
    // under test.
    let backup = fixture.server.get("/admin/backup").await;
    backup.assert_status_ok();
    let restore = fixture
        .server
        .post("/admin/restore")
        .json(&backup.json::<serde_json::Value>())
        .await;
    restore.assert_status(axum::http::StatusCode::NO_CONTENT);

    for (query, expected, what) in &expectations {
        assert_eq!(
            total_for(&fixture.server, query.clone()).await,
            *expected,
            "`{what}` must answer identically after the round trip; a gather \
             that reaches the segment without subtracting `tombstones` \
             un-deletes the doc here"
        );
    }
}

#[tokio::test]
async fn number_and_text_fields_are_the_control_for_the_round_trip() {
    let fixture = sealed_then_round_tripped().await;

    // These two arms are already correct and pass BEFORE the fix. If this
    // case ever goes red, the round trip itself broke and the two cases above
    // are measuring a dead fixture rather than the keyword/set defect.
    assert_eq!(
        total_for(&fixture.server, json!({ "exists": { "field": "num" } })).await,
        TOTAL_DOCS as u64,
        "control: the number arm rebuilds its postings from a segment-aware \
         forward column and is unaffected by this defect"
    );
    assert_eq!(
        total_for(
            &fixture.server,
            json!({ "range": { "field": "num", "gte": 1.0, "lte": 6.0 } })
        )
        .await,
        SEALED_DOCS as u64,
        "control: the sealed base docs reached the snapshot and are still \
         range-queryable after the round trip"
    );
    assert_eq!(
        total_for(
            &fixture.server,
            json!({ "match": { "field": "body", "text": "number 1" } })
        )
        .await,
        1,
        "control: the text arm is segment-aware on both halves"
    );
}
