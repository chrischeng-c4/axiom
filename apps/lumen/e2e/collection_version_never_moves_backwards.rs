//! Black-box oracle: a collection id's `version` is monotonic for the life of
//! the id, including across the #3953 tombstone supersede.
//!
//! `Engine::create_collection`'s supersede path builds a brand-new
//! `Collection` and `insert`s it over the tombstone. `Collection::new` starts
//! at version 1, so an id that had climbed to 3 through online field additions
//! comes back as 1 after a soft delete and a re-`PUT`. `version` is the value
//! `PUT /collections/{id}` hands the caller to key cached schema state on, so a
//! provisioning script that re-`PUT`s an idempotent schema sees the number go
//! backwards and concludes nothing has changed — while every document under
//! that id is in fact gone. That is an ABA, not a fresh start.
//!
//! What each case pins:
//!
//! | Case | Path | Contract |
//! |---|---|---|
//! | supersede | soft `DELETE` then `PUT` | version STRICTLY EXCEEDS the tombstone's last version |
//! | fresh id | `PUT` an id never used | version is 1 |
//! | force delete | `DELETE ?force=true` then `PUT` | version is 1 — the id was physically removed, so it really is new |
//!
//! The two controls are what make the first case a statement about the
//! tombstone path specifically rather than about `Collection::new`: both are
//! green before the fix and must stay green after it.

use axum_test::TestServer;
use serde_json::{json, Value};

use lumen::api::{router, AppState};
use lumen::storage::Engine;
use std::sync::Arc;

const COLLECTION: &str = "notes";

fn server() -> TestServer {
    let engine = Arc::new(Engine::new());
    TestServer::new(router(AppState::open(engine))).expect("test server")
}

/// A schema declaring `n` keyword fields `kw0..kw{n-1}`. Re-`PUT`ting with a
/// larger `n` is the online field-addition path, which bumps the version by 1.
fn schema(n: usize) -> Value {
    let mut fields = serde_json::Map::new();
    for i in 0..n {
        fields.insert(format!("kw{i}"), json!({ "type": "keyword" }));
    }
    json!({ "fields": fields })
}

/// `PUT /collections/{id}` with `n` fields; returns the reported version.
async fn put(s: &TestServer, id: &str, n: usize) -> u32 {
    let resp = s.put(&format!("/collections/{id}")).json(&schema(n)).await;
    resp.assert_status_ok();
    resp.json::<Value>()["version"]
        .as_u64()
        .expect("version is a number") as u32
}

#[tokio::test]
async fn a_superseded_tombstone_continues_the_versions_it_left_behind() {
    let s = server();

    assert_eq!(put(&s, COLLECTION, 1).await, 1, "first PUT opens at 1");
    assert_eq!(put(&s, COLLECTION, 2).await, 2, "adding a field bumps");
    let last_live = put(&s, COLLECTION, 3).await;
    assert_eq!(last_live, 3, "adding a second field bumps again");

    // Write a doc so the supersede has something to discard — the point is
    // that the id's CONTENT resets while its version must not.
    s.post(&format!("/collections/{COLLECTION}/index"))
        .json(&json!({ "items": [
            { "external_id": "before", "field": "kw0", "value": "old" }
        ]}))
        .await
        .assert_status_ok();

    s.delete(&format!("/collections/{COLLECTION}"))
        .await
        .assert_status_success();

    let superseded = put(&s, COLLECTION, 1).await;
    assert!(
        superseded > last_live,
        "the id reached version {last_live} before the tombstone, so the collection \
         that supersedes it must report a version STRICTLY ABOVE that; reporting \
         {superseded} moves a monotonic identifier backwards and a caller keying \
         cached schema state on it cannot tell the supersede from a stale read"
    );

    // The supersede still starts EMPTY — #3953's contract is unchanged.
    let stats = s
        .get(&format!("/collections/{COLLECTION}/stats"))
        .await
        .json::<Value>();
    assert_eq!(
        stats["documents_indexed"].as_u64(),
        Some(0),
        "the superseded collection inherits no documents"
    );
}

#[tokio::test]
async fn a_fresh_collection_id_still_starts_at_version_one() {
    let s = server();
    assert_eq!(
        put(&s, "never-used", 2).await,
        1,
        "an id with no history — tombstoned or live — opens at 1"
    );
}

#[tokio::test]
async fn a_force_deleted_id_is_a_genuinely_new_collection() {
    let s = server();

    assert_eq!(put(&s, COLLECTION, 1).await, 1);
    assert_eq!(put(&s, COLLECTION, 2).await, 2);

    // `force=true` physically removes the entry rather than tombstoning it, so
    // nothing is left to carry a version forward and the id really is new.
    s.delete(&format!("/collections/{COLLECTION}?force=true"))
        .await
        .assert_status_success();

    assert_eq!(
        put(&s, COLLECTION, 1).await,
        1,
        "a physically removed id has no predecessor to continue from"
    );
}
