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
//! | ships no inverted index | the `terms` / `elements` keys are present but EMPTY — no posting travels |
//! | a released 0.4.29 can still parse it | the document deserialises into a replica of 0.4.29's wire type, so that build refuses it by version rather than by serde |
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

    // The key survives as an empty map, and ONLY as an empty map. It is there
    // so a released 0.4.29 — whose `terms` is a required field — can parse this
    // document and refuse it by version instead of failing inside serde on a
    // file that is intact; `a_released_0_4_29_reader_can_still_parse_this`
    // below is the case that pins that. What must not come back is the
    // dictionary: `from_snapshot` rebuilds it from `forward` and never reads
    // this map, so every posting written here is shipped to a Raft follower and
    // discarded.
    assert_eq!(
        fields["kw"]["terms"],
        json!({}),
        "the Keyword arm's inverted index must travel EMPTY — an absent key breaks \
         a released 0.4.29 reader, and a populated one is bytes nobody reads. Got: {}",
        fields["kw"]
    );
    assert_eq!(
        fields["tags"]["elements"],
        json!({}),
        "the Set arm's inverted index, same as Keyword above. Got: {}",
        fields["tags"]
    );
    // The forward column is what the reader needs, so it must still be there.
    assert!(
        fields["kw"]["forward"].is_object() && fields["tags"]["forward"]["d0"].is_array(),
        "the forward column is the surviving representation and must be complete"
    );
}

/// A replica of the wire type `lumen@0.4.29` compiled — `terms` and
/// `elements` are required fields there, with no `#[serde(default)]`. Nothing
/// in this crate can be substituted for it: the point is to fail the way a
/// binary we can no longer edit fails.
mod v0_4_29 {
    use std::collections::{BTreeMap, BTreeSet, HashMap};

    #[derive(serde::Deserialize)]
    pub struct KeywordArm {
        #[allow(dead_code)]
        pub terms: BTreeMap<String, BTreeSet<String>>,
        pub forward: HashMap<String, String>,
    }

    #[derive(serde::Deserialize)]
    pub struct SetArm {
        #[allow(dead_code)]
        pub elements: BTreeMap<String, BTreeSet<String>>,
        pub forward: HashMap<String, BTreeSet<String>>,
    }
}

/// 0.4.29 is released, so a format-2 document WILL meet one: a rollback reads
/// its own `rdb.rs` data directory, a reshard delta lands on an un-upgraded
/// shard, a Raft catch-up reaches a peer mid-rolling-upgrade.
///
/// That reader must refuse by VERSION. It cannot, if it never finishes
/// deserialising — `version` is a field of the same struct, so serde runs
/// first and a missing `terms` reports a missing field on a file that is
/// intact. This case pins that the document still parses as 0.4.29 declares
/// it, which is the whole of what stands between an operator mid-incident and
/// a decode error that reads like corruption.
#[tokio::test]
async fn a_released_0_4_29_reader_can_still_parse_this() {
    let (s, engine) = server();
    seed(&s, 9, |i| format!("v{i}")).await;

    let doc = serde_json::to_value(engine.snapshot().expect("snapshot")).expect("serialise");
    let fields = &doc["collections"][COLLECTION]["fields"];

    let kw: v0_4_29::KeywordArm = serde_json::from_value(fields["kw"].clone()).expect(
        "0.4.29 requires a `terms` key; dropping it from the wire makes that build fail \
         inside serde, before its own version check can name the real problem",
    );
    let tags: v0_4_29::SetArm = serde_json::from_value(fields["tags"].clone())
        .expect("0.4.29 requires an `elements` key, same as `terms` above");

    // And it parses the part that actually matters, so the refusal it goes on
    // to print is a version refusal over a document it read, not a lucky one.
    assert_eq!(
        kw.forward.get("d0").map(String::as_str),
        Some("v0"),
        "the forward column an 0.4.29 reader sees must be the real one"
    );
    assert!(
        tags.forward.get("d0").is_some_and(|s| s.contains("shared")),
        "same for the Set arm"
    );
}

/// The same claim on the format the ROLLBACK actually reads.
///
/// `rdb.rs` writes the data directory as CBOR, not JSON, so the JSON case
/// above proves the placeholder reaches serde's data model but not that it
/// reaches the bytes on disk. This decodes the encoded snapshot with the same
/// decoder 0.4.29 links, into the same replica type.
#[tokio::test]
async fn a_released_0_4_29_reader_can_still_parse_the_on_disk_cbor() {
    let (s, engine) = server();
    seed(&s, 9, |i| format!("v{i}")).await;

    let mut cbor = Vec::new();
    ciborium::into_writer(&engine.snapshot().expect("snapshot"), &mut cbor).expect("cbor encode");
    let doc: Value = ciborium::from_reader(cbor.as_slice()).expect("cbor decode");
    let fields = &doc["collections"][COLLECTION]["fields"];

    let kw: v0_4_29::KeywordArm = serde_json::from_value(fields["kw"].clone()).expect(
        "0.4.29 requires `terms` in the CBOR data directory too; without it the rollback \
         fails to decode its own snapshot file and reads as corruption",
    );
    assert_eq!(
        kw.forward.get("d0").map(String::as_str),
        Some("v0"),
        "the forward column survives the CBOR round trip"
    );
    let tags: v0_4_29::SetArm = serde_json::from_value(fields["tags"].clone())
        .expect("0.4.29 requires `elements` in CBOR, same as `terms` above");
    assert!(
        tags.forward.get("d0").is_some_and(|s| s.contains("shared")),
        "same for the Set arm"
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
