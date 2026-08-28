// CODEGEN-BEGIN
//! Black-box oracle for issue #3953.
//!
//! `Engine::create_collection` treats an EXISTING entry in `state.collections`
//! as an update target unconditionally: it calls `coll.check_live(...)`
//! before touching the schema, and `check_live` returns `StorageError::Gone`
//! (HTTP 410) for any collection whose `deleted_at` is set. That is correct
//! for reads and writes against the tombstoned id — but it also means a
//! `PUT /collections/{id}` for a soft-deleted id can never succeed again: the
//! id is permanently wedged at 410 until the periodic `sweep_deleted` grace
//! window physically removes the entry, which is an operator-timescale
//! background job, not something a caller can invoke to reclaim a name they
//! just deleted on purpose.
//!
//! The frozen contract this oracle encodes:
//! (a) a `PUT` for a soft-deleted id SUCCEEDS — it supersedes the tombstone
//!     and yields a fresh, empty collection (the old docs become invisible).
//! (b) the tombstone must not outlive a restart: after DELETE + recovery
//!     (AOF replay with no checkpoint in between is sufficient), a `PUT` for
//!     the same id succeeds.

use std::sync::{Arc, Mutex};

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{json, Value};

use lumen::aof::{replay_aof_into, AofWriter};
use lumen::api::{router, AppState};
use lumen::auth::AuthConfig;
use lumen::coordinator::{SharedAof, WriteCoordinator, WriteSink};
use lumen::storage::Engine;
use lumen::wal::{MemWal, SharedWal};

const COLLECTION: &str = "notes";

fn keyword_schema() -> Value {
    json!({ "fields": { "kw": { "type": "keyword" } } })
}

fn server_with_engine() -> (TestServer, Arc<Engine>) {
    let engine = Arc::new(Engine::new());
    let app = router(AppState::open(engine.clone()));
    (TestServer::new(app).expect("test server"), engine)
}

#[tokio::test]
async fn put_after_delete_supersedes_tombstone_and_starts_empty() {
    let (s, _engine) = server_with_engine();

    s.put(&format!("/collections/{COLLECTION}"))
        .json(&keyword_schema())
        .await
        .assert_status_ok();
    s.post(&format!("/collections/{COLLECTION}/index"))
        .json(&json!({ "items": [
            { "external_id": "old1", "field": "kw", "value": "before" }
        ]}))
        .await
        .assert_status_ok();

    let del = s.delete(&format!("/collections/{COLLECTION}")).await;
    del.assert_status(StatusCode::ACCEPTED);

    // #3953(a): PUT for the just-deleted id must succeed, not stay wedged at
    // 410 Gone.
    let recreate = s
        .put(&format!("/collections/{COLLECTION}"))
        .json(&keyword_schema())
        .await;
    assert!(
        recreate.status_code().is_success(),
        "PUT for a soft-deleted collection id must supersede the tombstone \
         and succeed (issue #3953); got {} body={}",
        recreate.status_code(),
        recreate.text()
    );

    // The recreated collection must be genuinely fresh: the doc indexed
    // before the delete must not be searchable in it.
    let search = s
        .post(&format!("/collections/{COLLECTION}/search"))
        .json(&json!({
            "query": { "term": { "field": "kw", "value": "before" } },
            "limit": 10
        }))
        .await;
    search.assert_status_ok();
    assert_eq!(
        search.json::<Value>()["total"],
        0,
        "the recreated collection must start empty; the pre-delete doc must not survive"
    );
}

struct DurableFixture {
    _dir: tempfile::TempDir,
    server: TestServer,
    aof: SharedAof,
    aof_path: std::path::PathBuf,
}

fn durable_fixture() -> DurableFixture {
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
    DurableFixture {
        _dir: dir,
        server,
        aof,
        aof_path,
    }
}

#[tokio::test]
async fn tombstone_does_not_survive_restart() {
    let fixture = durable_fixture();

    fixture
        .server
        .put(&format!("/collections/{COLLECTION}"))
        .json(&keyword_schema())
        .await
        .assert_status_ok();

    let del = fixture.server.delete(&format!("/collections/{COLLECTION}")).await;
    del.assert_status(StatusCode::ACCEPTED);

    fixture
        .aof
        .lock()
        .expect("aof lock")
        .sync_strict()
        .expect("strict-sync AOF");

    // Simulate a cold restart WITHOUT any checkpoint: replay the entire AOF
    // (the collection create + the soft-delete) into a brand new engine, then
    // wire a brand new server on top of it — exactly what a restarted process
    // does before serving its first request.
    let recovered = Arc::new(Engine::new());
    let replayed = replay_aof_into(&recovered, &fixture.aof_path, 0).expect("replay AOF");
    assert!(
        replayed > 0,
        "the AOF must have logged the collection create and the soft-delete"
    );

    // A fresh `MemWal::new()` would restart its own sequence domain at 0,
    // which the apply loop's redelivery-dedup guard would then discard as
    // stale against `applied` (seeded to `replayed`) -- see
    // `MemWal::starting_at`'s doc comment (#1486). `starting_at(replayed)`
    // is what a real restart's reconnect to its durable broker gives for
    // free; a bare `MemWal::new()` here is a fixture bug, not #3953.
    let wal2: SharedWal = Arc::new(MemWal::starting_at(replayed));
    let writer2 = WriteCoordinator::start_from(wal2, recovered.clone(), replayed);
    let state2 = AppState::with_components(
        recovered,
        Arc::new(AuthConfig::open()),
        writer2 as Arc<dyn WriteSink>,
    );
    let restarted_server = TestServer::new(router(state2)).expect("restarted test server");

    // #3953(b): the tombstone must not outlive the restart — PUT for the same
    // id on the restarted server must succeed.
    let recreate = restarted_server
        .put(&format!("/collections/{COLLECTION}"))
        .json(&keyword_schema())
        .await;
    assert!(
        recreate.status_code().is_success(),
        "PUT for a soft-deleted collection id must succeed after a restart \
         (issue #3953); got {} body={}",
        recreate.status_code(),
        recreate.text()
    );
}
