//! Focused AOF checkpoint and replay contract.

use std::sync::{Arc, Mutex};

use axum_test::TestServer;
use lumen::aof::{replay_aof_into, AofWriter};
use lumen::api::{router, AppState, CheckpointSink};
use lumen::auth::AuthConfig;
use lumen::coordinator::{SharedAof, WriteCoordinator};
use lumen::segment_checkpoint::SegmentCheckpointSink;
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::Engine;
use lumen::wal::{MemWal, SharedWal};
use serde_json::json;
use std::time::{Duration, Instant};

#[tokio::test]
async fn checkpoint_then_cold_replay_keeps_the_aof_tail() {
    let dir = tempfile::tempdir().expect("AOF root");
    let root = dir.path().join("segments");
    let aof_path = dir.path().join("aof.log");
    let engine = Arc::new(Engine::new());
    let wal: SharedWal = Arc::new(MemWal::default());
    let aof: SharedAof = Arc::new(Mutex::new(AofWriter::open(&aof_path).expect("AOF")));
    let writer = WriteCoordinator::start_from_with_aof(wal, engine.clone(), 0, aof.clone());
    let store = Arc::new(SegmentRdbStore::new(&root).expect("store"));
    let sink_writer: Arc<dyn lumen::coordinator::WriteSink> = writer.clone();
    let checkpoint = Arc::new(SegmentCheckpointSink {
        engine: engine.clone(),
        store: store.clone(),
        writer: sink_writer.clone(),
        aof: Some(aof.clone()),
    });
    let checkpoint_api: Arc<dyn CheckpointSink> = checkpoint.clone();
    let state = AppState::with_components(engine.clone(), Arc::new(AuthConfig::open()), sink_writer)
        .with_checkpoint(checkpoint_api);
    let server = TestServer::new(router(state)).expect("server");
    server.put("/collections/aof").json(&json!({"fields":{"kw":{"type":"keyword"}}})).await.assert_status_ok();
    server.post("/collections/aof/index").json(&json!({"items":[{"external_id":"before","field":"kw","value":"old"}]})).await.assert_status_ok();
    let apply_deadline = Instant::now() + Duration::from_secs(5);
    while writer.applied_seq() < 2 {
        assert!(Instant::now() < apply_deadline, "create and pre-checkpoint index must apply");
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let cut = writer.applied_seq();
    let persisted = CheckpointSink::checkpoint_now(checkpoint.as_ref())
        .await
        .expect("checkpoint");
    assert!(persisted, "checkpoint must publish a durable generation");
    let current = SegmentRdbStore::new(&root).expect("store").load_current_generation().expect("CURRENT").expect("generation");
    assert_eq!(current.sequence, cut, "checkpoint must persist the applied cut");
    server.post("/collections/aof/index").json(&json!({"items":[{"external_id":"after","field":"kw","value":"new"}]})).await.assert_status_ok();
    let tail_deadline = Instant::now() + Duration::from_secs(5);
    while writer.applied_seq() <= cut {
        assert!(Instant::now() < tail_deadline, "post-checkpoint record must apply");
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let replayed = replay_aof_into(&current.engine, &aof_path, current.sequence).expect("AOF replay");
    assert_eq!(replayed, cut + 1, "replay must advance exactly one sequence past the checkpoint cut");
    assert_eq!(writer.applied_seq(), replayed);
}
