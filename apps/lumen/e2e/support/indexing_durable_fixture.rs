//! Shared HTTP and persistence fixture for durable-indexing E2E targets.
//!
//! The fixture owns its temporary directory, in-process writer, AOF, segment
//! store, and public HTTP server.  Tests observe durable state through the
//! server or a cold `SegmentRdbStore` reopen.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;
use axum_test::TestServer;
use serde_json::{json, Value};

use lumen::aof::{replay_aof_into, AofWriter};
use lumen::api::{router, AppState, CheckpointSink};
use lumen::auth::AuthConfig;
use lumen::coordinator::{SharedAof, WriteCoordinator, WriteSink};
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::Engine;
use lumen::types::SearchResponse;
use lumen::wal::{MemWal, SharedWal};

pub(crate) const COLLECTION: &str = "docs";

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

pub(crate) fn local_checkpoint_sink(
    engine: Arc<Engine>,
    store: Arc<SegmentRdbStore>,
    writer: Arc<WriteCoordinator>,
    aof: SharedAof,
) -> Arc<dyn CheckpointSink> {
    Arc::new(LocalCheckpointSink {
        engine,
        store,
        writer,
        aof,
    })
}

pub(crate) struct Fixture {
    _dir: tempfile::TempDir,
    pub(crate) checkpoint_root: PathBuf,
    pub(crate) server: TestServer,
    pub(crate) engine: Arc<Engine>,
    pub(crate) writer: Arc<WriteCoordinator>,
    pub(crate) aof: SharedAof,
    pub(crate) aof_path: PathBuf,
    pub(crate) store: Arc<SegmentRdbStore>,
}

pub(crate) fn fixture() -> Fixture {
    let dir = tempfile::tempdir().expect("fixture directory");
    let checkpoint_root = dir.path().join("segments");
    let aof_path = dir.path().join("aof.log");
    let store = Arc::new(SegmentRdbStore::new(&checkpoint_root).expect("segment store"));
    let aof: SharedAof = Arc::new(Mutex::new(AofWriter::open(&aof_path).expect("aof")));
    let engine = Arc::new(Engine::new());
    let wal: SharedWal = Arc::new(MemWal::new());
    let writer = WriteCoordinator::start_from_with_aof(wal, engine.clone(), 0, aof.clone());
    let checkpoint =
        local_checkpoint_sink(engine.clone(), store.clone(), writer.clone(), aof.clone());
    let state = AppState::with_components(
        engine.clone(),
        Arc::new(AuthConfig::open()),
        writer.clone() as Arc<dyn WriteSink>,
    )
    .with_checkpoint(checkpoint);
    let server = TestServer::new(router(state)).expect("test server");
    Fixture {
        _dir: dir,
        checkpoint_root,
        server,
        engine,
        writer,
        aof,
        aof_path,
        store,
    }
}

pub(crate) async fn create_schema(server: &TestServer) {
    server
        .put("/collections/docs")
        .json(&json!({
            "fields": {
                "kw": { "type": "keyword" },
                "num": { "type": "number" },
                "body": { "type": "text", "analyzer": "whitespace_lower" }
            }
        }))
        .await
        .assert_status_ok();
}

pub(crate) async fn post_index(server: &TestServer, items: Vec<Value>) {
    for chunk in items.chunks(1000) {
        server
            .post("/collections/docs/index")
            .json(&json!({ "items": chunk }))
            .await
            .assert_status_ok();
    }
}

pub(crate) async fn checkpoint(server: &TestServer) {
    let response = server.post("/admin/checkpoint").await;
    response.assert_status_ok();
    assert_eq!(response.json::<Value>()["persisted"], true);
}

/// Drive a search through the public HTTP route.  The case deliberately does
/// not inspect a recovered index's private postings: callers only observe the
/// recovered value through the same search request they used before the
/// checkpoint.
pub(crate) async fn http_search(server: &TestServer, query: Value, context: &str) -> Value {
    let response = server
        .post(&format!("/collections/{COLLECTION}/search"))
        .json(&json!({ "query": query, "limit": 500 }))
        .await;
    response.assert_status_ok();
    let body = response.json::<Value>();
    assert!(
        body["total"].is_u64(),
        "{context}: search response must expose a numeric total: {body}"
    );
    body
}

pub(crate) fn hit_ids(body: &Value) -> Vec<&str> {
    body["hits"]
        .as_array()
        .expect("search hits are an array")
        .iter()
        .map(|hit| hit["external_id"].as_str().expect("hit external id"))
        .collect()
}

pub(crate) fn sorted_hit_ids(response: SearchResponse) -> Vec<String> {
    let mut ids: Vec<_> = response
        .hits
        .into_iter()
        .map(|hit| hit.external_id)
        .collect();
    ids.sort();
    ids
}

pub(crate) fn recover_from_checkpoint(fixture: &Fixture) -> (Arc<Engine>, u64, u64) {
    let loaded = fixture
        .store
        .load_current_generation()
        .expect("load CURRENT")
        .expect("checkpoint generation");
    let checkpoint_sequence = loaded.sequence;
    let replayed = replay_aof_into(&loaded.engine, &fixture.aof_path, checkpoint_sequence)
        .expect("replay AOF tail");
    (loaded.engine, checkpoint_sequence, replayed)
}
