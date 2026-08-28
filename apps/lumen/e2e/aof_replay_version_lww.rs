//! Black-box oracle for issue #3952.
//!
//! `Engine::index_collection` enforces external-version last-write-wins
//! (#184): a write whose `version` is not strictly greater than the stored
//! `cell_versions` ceiling for that `(external_id, field)` is dropped
//! (`indexed: 0`), and does not move the ceiling.
//!
//! But `WalRecord::encode()` prefers a fast binary codec for every `Index`
//! entry (`encode_fast_index`), and that codec never writes `IndexItem.version`
//! to the wire at all; `decode_fast_record` always reconstructs each replayed
//! item with `version: None`. So while the LWW ceiling holds correctly for the
//! live in-memory apply path, ANY replay of the AOF (or of the underlying WAL,
//! which uses the identical codec) loses every version and folds back to
//! plain arrival-order apply: a write the server already, correctly rejected
//! as stale gets silently applied on replay, and the version ceiling itself
//! is lost (not merely reduced) for the cell it touched.

use std::sync::{Arc, Mutex};

use axum_test::TestServer;
use serde_json::json;

use lumen::aof::{replay_aof_into, AofWriter};
use lumen::api::{router, AppState};
use lumen::auth::AuthConfig;
use lumen::coordinator::{SharedAof, WriteCoordinator, WriteSink};
use lumen::storage::Engine;
use lumen::types::{
    FieldValue, IndexItem, IndexRequest, QueryNode, SearchRequest, TermQuery,
};
use lumen::wal::{MemWal, SharedWal};

const COLLECTION: &str = "docs";

struct Fixture {
    _dir: tempfile::TempDir,
    server: TestServer,
    aof: SharedAof,
    aof_path: std::path::PathBuf,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().expect("fixture directory");
    let aof_path = dir.path().join("aof.log");
    let aof: SharedAof = Arc::new(Mutex::new(AofWriter::open(&aof_path).expect("aof")));
    let engine = Arc::new(Engine::new());
    let wal: SharedWal = Arc::new(MemWal::new());
    let writer = WriteCoordinator::start_from_with_aof(wal, engine.clone(), 0, aof.clone());
    let state = AppState::with_components(
        engine,
        Arc::new(AuthConfig::open()),
        writer as Arc<dyn WriteSink>,
    );
    let server = TestServer::new(router(state)).expect("test server");
    Fixture {
        _dir: dir,
        server,
        aof,
        aof_path,
    }
}

fn term_query(field: &str, value: &str) -> SearchRequest {
    SearchRequest {
        query: QueryNode::Term(TermQuery {
            field: field.into(),
            value: FieldValue::String(value.into()),
        }),
        limit: 20,
        offset: 0,
        cursor: None,
        routing_key: None,
        sort: None,
        track_total: true,
        collapse: None,
    }
}

fn assert_hits_only(engine: &Arc<Engine>, value: &str, expect_hit: bool, context: &str) {
    let response = engine
        .search(COLLECTION, term_query("kw", value))
        .unwrap_or_else(|e| panic!("term query for `{value}` ({context}) failed: {e}"));
    if expect_hit {
        assert_eq!(
            response.total, 1,
            "expected exactly one hit for kw=`{value}` ({context}), got {}",
            response.total
        );
        assert_eq!(
            response.hits.first().map(|h| h.external_id.as_str()),
            Some("d1"),
            "expected d1 to be the hit for kw=`{value}` ({context})"
        );
    } else {
        assert_eq!(
            response.total, 0,
            "expected NO hit for kw=`{value}` ({context}); the stale write must not be visible, got {}",
            response.total
        );
    }
}

#[tokio::test]
async fn version_ceiling_survives_aof_replay() {
    let fixture = fixture();

    fixture
        .server
        .put(&format!("/collections/{COLLECTION}"))
        .json(&json!({ "fields": { "kw": { "type": "keyword" } } }))
        .await
        .assert_status_ok();

    // Write #1: version 5. Must apply.
    let resp = fixture
        .server
        .post(&format!("/collections/{COLLECTION}/index"))
        .json(&json!({ "items": [
            { "external_id": "d1", "field": "kw", "value": "v5", "version": 5 }
        ]}))
        .await;
    resp.assert_status_ok();
    assert_eq!(
        resp.json::<serde_json::Value>()["indexed"],
        1,
        "version 5 write must be applied (first write for this cell)"
    );

    // Write #2: version 3, strictly older than the stored ceiling (5). The
    // live in-memory LWW check must drop it.
    let resp = fixture
        .server
        .post(&format!("/collections/{COLLECTION}/index"))
        .json(&json!({ "items": [
            { "external_id": "d1", "field": "kw", "value": "v3", "version": 3 }
        ]}))
        .await;
    resp.assert_status_ok();
    assert_eq!(
        resp.json::<serde_json::Value>()["indexed"],
        0,
        "version 3 write must be dropped as stale against ceiling 5"
    );

    // Confirm live in-memory state: v5 visible, v3 is not. Routed through the
    // same HTTP surface a caller sees, to prove the live behavior end to end.
    let resp = fixture
        .server
        .post(&format!("/collections/{COLLECTION}/search"))
        .json(&json!({ "query": { "term": { "field": "kw", "value": "v5" } }, "limit": 10 }))
        .await;
    resp.assert_status_ok();
    assert_eq!(resp.json::<serde_json::Value>()["total"], 1, "live: v5 must be visible");

    let resp = fixture
        .server
        .post(&format!("/collections/{COLLECTION}/search"))
        .json(&json!({ "query": { "term": { "field": "kw", "value": "v3" } }, "limit": 10 }))
        .await;
    resp.assert_status_ok();
    assert_eq!(resp.json::<serde_json::Value>()["total"], 0, "live: v3 must NOT be visible");

    fixture
        .aof
        .lock()
        .expect("aof lock")
        .sync_strict()
        .expect("strict-sync AOF");

    // Simulate a cold restart WITHOUT any checkpoint: replay the entire AOF
    // (from seq 0) into a brand new, empty engine.
    let recovered = Arc::new(Engine::new());
    let replayed = replay_aof_into(&recovered, &fixture.aof_path, 0).expect("replay AOF");
    assert!(
        replayed > 0,
        "the AOF must have logged at least the collection create and both index writes"
    );

    // #3952: the version ceiling must survive replay byte-for-byte. Today it
    // does not — the fast WAL codec never writes `version` to the wire, so
    // both replayed items look unversioned and apply in arrival order: v3
    // (logged second) clobbers v5.
    assert_hits_only(&recovered, "v5", true, "after replay");
    assert_hits_only(&recovered, "v3", false, "after replay");

    // The strongest signal: the ceiling itself, not merely the value. Submit
    // a version:4 write against the recovered state — it is strictly BELOW
    // the original ceiling (5) and strictly ABOVE the corrupted one a
    // lost-version replay would leave behind (absent, or 3). It must be
    // rejected.
    let rejected = recovered
        .index(
            COLLECTION,
            IndexRequest {
                items: vec![IndexItem {
                    external_id: "d1".into(),
                    field: "kw".into(),
                    value: FieldValue::String("v4".into()),
                    version: Some(4),
                }],
                request_id: None,
            },
        )
        .expect("index call against recovered engine");
    assert_eq!(
        rejected.indexed, 0,
        "version 4 write must be rejected: the recovered ceiling must still be 5 (issue #3952)"
    );
    assert_hits_only(&recovered, "v4", false, "after rejected version-4 write");
    assert_hits_only(&recovered, "v5", true, "after rejected version-4 write");
}
