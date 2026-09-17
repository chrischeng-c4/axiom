//! Focused field and query correctness contract.

use std::sync::{Arc, Mutex};
use axum_test::TestServer;
use lumen::aof::AofWriter;
use lumen::api::{router, AppState, CheckpointSink};
use lumen::auth::AuthConfig;
use lumen::coordinator::{SharedAof, WriteCoordinator, WriteSink};
use lumen::segment_checkpoint::SegmentCheckpointSink;
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::Engine;
use lumen::wal::{MemWal, SharedWal};
use serde_json::json;

#[tokio::test]
async fn scalar_and_text_queries_return_the_indexed_document() {
    let dir = tempfile::tempdir().expect("checkpoint root");
    let engine = Arc::new(Engine::new());
    let aof: SharedAof = Arc::new(Mutex::new(AofWriter::open(dir.path().join("aof.log")).expect("AOF")));
    let wal: SharedWal = Arc::new(MemWal::default());
    let writer = WriteCoordinator::start_from_with_aof(wal, engine.clone(), 0, aof.clone());
    let sink_writer: Arc<dyn WriteSink> = writer;
    let store = Arc::new(SegmentRdbStore::new(dir.path().join("segments")).expect("store"));
    let checkpoint: Arc<dyn CheckpointSink> = Arc::new(SegmentCheckpointSink { engine: engine.clone(), store: store.clone(), writer: sink_writer.clone(), aof: Some(aof) });
    let state = AppState::with_components(engine, Arc::new(AuthConfig::open()), sink_writer).with_checkpoint(checkpoint);
    let server = TestServer::new(router(state)).expect("server");
    server.put("/collections/query").json(&json!({"fields":{"kw":{"type":"keyword"},"body":{"type":"text"},"num":{"type":"number"},"v":{"type":"vector","dim":3,"metric":"cosine"}}})).await.assert_status_ok();
    server.post("/collections/query/index").json(&json!({"items":[{"external_id":"q1","field":"kw","value":"stable"},{"external_id":"q1","field":"body","value":"stable text"},{"external_id":"q1","field":"num","value":7},{"external_id":"q1","field":"v","value":[0.1,0.2,0.3]}]})).await.assert_status_ok();
    for query in [json!({"term":{"field":"kw","value":"stable"}}), json!({"match":{"field":"body","text":"stable"}}), json!({"range":{"field":"num","gte":7,"lte":7}}), json!({"knn":{"field":"v","vector":[0.1,0.2,0.3],"k":1}})] {
        let response = server.post("/collections/query/search").json(&json!({"query":query})).await;
        response.assert_status_ok();
        assert_eq!(response.json::<serde_json::Value>()["total"], 1);
    }
    server.post("/admin/checkpoint").json(&json!({})).await.assert_status_ok();
    let cold = SegmentRdbStore::new(dir.path().join("segments")).expect("cold store").load_current_generation().expect("CURRENT").expect("generation");
    let cold_server = TestServer::new(router(AppState::open(cold.engine))).expect("cold server");
    let response = cold_server.post("/collections/query/search").json(&json!({"query":{"knn":{"field":"v","vector":[0.1,0.2,0.3],"k":1}}})).await;
    response.assert_status_ok();
    assert_eq!(response.json::<serde_json::Value>()["total"], 1);
}
