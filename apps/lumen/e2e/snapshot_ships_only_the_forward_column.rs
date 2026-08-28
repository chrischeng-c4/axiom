//! Black-box oracle: a snapshot carries ONE representation of a `keyword` or
//! `set` field — the `forward` column — and the reader rebuilds the inverted
//! index from it.
//!
//! `Collection::from_snapshot` already stopped trusting the persisted `terms` /
//! `elements` maps: both arms bind them to `_terms` / `_elements` and rebuild
//! from `forward`, so an on-disk snapshot with a corrupt inverted index
//! self-heals on restore. `to_snapshot` was not changed to match. It still
//! materialises the whole dictionary through `live_terms()` / `live_elements()`
//! — which, on a sealed field, decodes it out of the segment — and serialises
//! it. Every consumer pays: `raft_sm.rs` ships it to a follower on another
//! host during catch-up, `reshard.rs` moves it between shards, `rdb.rs` writes
//! it to disk. The receiver throws it away.
//!
//! `reshard.rs` compounds it. `merge_field_index_delta` unions the two shards'
//! `terms` while `forward` collides last-writer-wins, so a doc whose keyword
//! differs between shards leaves a posting in `terms` that its own `forward`
//! no longer holds — two representations that disagree, with the second one
//! read by nobody.
//!
//! | Case | What it pins |
//! |---|---|
//! | ships no inverted index | the serialised document has no `terms` / `elements` key |
//! | a v1 snapshot still restores | an older document that DOES carry them is still read, and its `forward` still wins |
//! | round trip preserves answers | the rebuild from `forward` alone is complete, sparse fields and explicit empties included |
//! | merge leaves no stale posting | `forward`'s last-writer-wins is the only surviving authority after a shard merge |
//!
//! The last three are green before the change and must stay green after it:
//! they are what makes the first case a statement about the wire payload
//! rather than about the index.

use std::sync::Arc;

use axum_test::TestServer;
use serde_json::{json, Value};

use lumen::api::{router, AppState};
use lumen::storage::{Engine, SnapshotV1};

const COLLECTION: &str = "docs";

fn server() -> (TestServer, Arc<Engine>) {
    let engine = Arc::new(Engine::new());
    let app = router(AppState::open(engine.clone()));
    (TestServer::new(app).expect("test server"), engine)
}

fn schema() -> Value {
    json!({ "fields": {
        "kw": { "type": "keyword" },
        "tags": { "type": "set" },
    }})
}

/// `n` docs, of which only every third writes `kw`/`tags` — the sparse shape
/// the forward gather has to reproduce without probing every interned id.
async fn seed(s: &TestServer, n: usize, kw_of: impl Fn(usize) -> String) {
    s.put(&format!("/collections/{COLLECTION}"))
        .json(&schema())
        .await
        .assert_status_ok();
    let mut items = Vec::new();
    for i in 0..n {
        // Every doc exists (it writes `other`), but only every third carries
        // the two fields under test.
        items.push(json!({ "external_id": format!("d{i}"), "field": "kw", "value": "filler" }));
        if i % 3 == 0 {
            items.push(json!({ "external_id": format!("d{i}"), "field": "kw", "value": kw_of(i) }));
            items.push(json!({
                "external_id": format!("d{i}"),
                "field": "tags",
                "value": [format!("t{}", i % 5), "shared"]
            }));
        }
    }
    s.post(&format!("/collections/{COLLECTION}/index"))
        .json(&json!({ "items": items }))
        .await
        .assert_status_ok();
}

/// Hits for `query`, as a sorted list of external_ids.
async fn hits(s: &TestServer, query: Value) -> Vec<String> {
    let resp = s
        .post(&format!("/collections/{COLLECTION}/search"))
        .json(&json!({ "query": query, "limit": 1000 }))
        .await;
    resp.assert_status_ok();
    let mut ids: Vec<String> = resp.json::<Value>()["hits"]
        .as_array()
        .expect("hits array")
        .iter()
        .map(|h| h["external_id"].as_str().expect("external_id").to_string())
        .collect();
    ids.sort();
    ids
}

/// A fresh server serving `snap`.
fn restored(snap: SnapshotV1) -> TestServer {
    let engine = Arc::new(Engine::new());
    engine.restore(snap).expect("restore");
    TestServer::new(router(AppState::open(engine))).expect("test server")
}

#[tokio::test]
async fn a_keyword_and_set_snapshot_ship_no_inverted_index() {
    let (s, engine) = server();
    seed(&s, 12, |i| format!("v{i}")).await;

    let doc = serde_json::to_value(engine.snapshot().expect("snapshot")).expect("serialise");
    let fields = &doc["collections"][COLLECTION]["fields"];

    assert!(
        fields["kw"].get("terms").is_none(),
        "the Keyword arm still ships its inverted index; `from_snapshot` rebuilds \
         `terms` from `forward` and never reads this map, so every byte of it is \
         written, shipped to a Raft follower, and discarded. Got: {}",
        fields["kw"]
    );
    assert!(
        fields["tags"].get("elements").is_none(),
        "the Set arm still ships its inverted index, same as Keyword above. Got: {}",
        fields["tags"]
    );
    // The forward column is what the reader needs, so it must still be there.
    assert!(
        fields["kw"]["forward"].is_object() && fields["tags"]["forward"]["d0"].is_array(),
        "the forward column is the surviving representation and must be complete"
    );
}

#[tokio::test]
async fn a_v1_snapshot_that_still_carries_terms_is_still_restorable() {
    let (s, engine) = server();
    seed(&s, 6, |i| format!("v{i}")).await;

    // Rebuild the document an older writer would have produced: format version
    // 1, and both inverted maps present. The `terms` map is deliberately WRONG
    // — it claims `d1` under `ghost` — because `forward` is the authority and
    // the reader is supposed to ignore this map entirely.
    let mut doc = serde_json::to_value(engine.snapshot().expect("snapshot")).expect("serialise");
    doc["version"] = json!(1);
    doc["collections"][COLLECTION]["fields"]["kw"]["terms"] = json!({ "ghost": ["d1"] });
    doc["collections"][COLLECTION]["fields"]["tags"]["elements"] = json!({ "ghost": ["d1"] });

    let snap: SnapshotV1 = serde_json::from_value(doc).expect(
        "a format-version-1 document still deserialises — dropping a field from the \
         wire type must not make previously written snapshots unreadable",
    );
    let s2 = restored(snap);

    assert_eq!(
        hits(&s2, json!({ "term": { "field": "kw", "value": "v0" } })).await,
        vec!["d0".to_string()],
        "the restored index answers from `forward`"
    );
    assert!(
        hits(&s2, json!({ "term": { "field": "kw", "value": "ghost" } }))
            .await
            .is_empty(),
        "the persisted `terms` map claimed `d1` under `ghost`; the reader must not \
         trust it — `forward` never held that value"
    );
    assert!(
        hits(&s2, json!({ "term": { "field": "tags", "value": "ghost" } }))
            .await
            .is_empty(),
        "same for the Set arm's `elements`"
    );
}

#[tokio::test]
async fn a_round_trip_preserves_every_keyword_and_set_answer() {
    let (s, engine) = server();
    seed(&s, 30, |i| format!("v{i}")).await;

    let probes = [
        json!({ "term": { "field": "kw", "value": "v0" } }),
        json!({ "term": { "field": "kw", "value": "v9" } }),
        json!({ "term": { "field": "kw", "value": "filler" } }),
        json!({ "terms": { "field": "kw", "values": ["v0", "v3", "v6"] } }),
        json!({ "term": { "field": "tags", "value": "shared" } }),
        json!({ "term": { "field": "tags", "value": "t0" } }),
        json!({ "exists": { "field": "tags" } }),
    ];

    let mut before = Vec::new();
    for p in &probes {
        before.push(hits(&s, p.clone()).await);
    }

    let s2 = restored(engine.snapshot().expect("snapshot"));

    for (p, expected) in probes.iter().zip(&before) {
        assert!(
            !expected.is_empty(),
            "probe {p} matched nothing before the round trip, so it proves nothing about it"
        );
        assert_eq!(
            &hits(&s2, p.clone()).await,
            expected,
            "the index rebuilt from `forward` alone must answer {p} exactly as the live one did"
        );
    }
}

#[tokio::test]
async fn a_merge_of_disagreeing_shards_leaves_no_stale_posting() {
    // Two shards hold the SAME external_id under different values. `forward`
    // resolves the collision last-writer-wins; the inverted index must agree
    // with whatever `forward` ended up holding, not accumulate both.
    let (a, engine_a) = server();
    seed(&a, 3, |_| "from-a".to_string()).await;
    let (b, engine_b) = server();
    seed(&b, 3, |_| "from-b".to_string()).await;

    let merged = lumen::reshard::merge_snapshot_delta(
        engine_a.snapshot().expect("snapshot a"),
        engine_b.snapshot().expect("snapshot b"),
    )
    .expect("merge");

    let s2 = restored(merged);
    let winner = hits(&s2, json!({ "term": { "field": "kw", "value": "from-b" } })).await;
    let loser = hits(&s2, json!({ "term": { "field": "kw", "value": "from-a" } })).await;

    assert_eq!(
        winner,
        vec!["d0".to_string()],
        "the delta shard's value wins the `forward` collision"
    );
    assert!(
        loser.is_empty(),
        "`from-a` lost the `forward` collision, so no posting may still name d0 under \
         it; got {loser:?}"
    );
}
