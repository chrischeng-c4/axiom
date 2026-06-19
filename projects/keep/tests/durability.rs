//! Proves durable-before-ack: once a write returns 200, its data is already
//! fsynced to the WAL and survives a cold recovery.

use std::sync::Arc;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use keep::persistence::recovery::RecoveryManager;
use keep::persistence::{PersistenceConfig, PersistenceHandle};
use keep::{router, AppState, KvEngine, KvKey};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn durable_before_ack_survives_recovery() {
    let dir = tempfile::tempdir().unwrap();
    let shards = 16;

    // Engine + WAL persistence.
    let engine = Arc::new(KvEngine::with_shards(shards));
    let cfg = PersistenceConfig::new(dir.path()).with_fsync_interval_ms(100);
    let persistence = Arc::new(PersistenceHandle::new(cfg, engine.clone()).unwrap());
    engine.enable_persistence(persistence);
    let app = router(AppState::new(engine.clone()));

    // A durable PUT: the handler awaits the WAL fsync barrier before replying,
    // so a 200 means the op is already on disk.
    let req = Request::builder()
        .method("PUT")
        .uri("/v1/kv/result:job-7")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!({"value": {"rows": 1000}})).unwrap(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Cold recovery from the same dir must see the durably-acked write.
    let (recovered, _stats) = RecoveryManager::recover(dir.path(), shards).unwrap();
    let got = recovered.get(&KvKey::new("result:job-7").unwrap());
    assert!(
        got.is_some(),
        "a write that returned 200 must survive recovery (durable-before-ack)"
    );
}

#[tokio::test]
async fn many_concurrent_durable_writes_all_persist() {
    let dir = tempfile::tempdir().unwrap();
    let shards = 16;

    let engine = Arc::new(KvEngine::with_shards(shards));
    let cfg = PersistenceConfig::new(dir.path()).with_fsync_interval_ms(100);
    let persistence = Arc::new(PersistenceHandle::new(cfg, engine.clone()).unwrap());
    engine.enable_persistence(persistence);
    let app = router(AppState::new(engine.clone()));

    // Fire many durable writes concurrently — they group-commit but each must
    // be durable when its own 200 returns. None may be dropped.
    let n = 500;
    let mut tasks = Vec::new();
    for i in 0..n {
        let app = app.clone();
        tasks.push(tokio::spawn(async move {
            let req = Request::builder()
                .method("PUT")
                .uri(format!("/v1/kv/k:{i}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(format!("{{\"value\":{i}}}")))
                .unwrap();
            app.oneshot(req).await.unwrap().status()
        }));
    }
    for t in tasks {
        assert_eq!(t.await.unwrap(), StatusCode::OK);
    }

    // Every acked key must be recoverable from disk — no silent WAL drops.
    let (recovered, _stats) = RecoveryManager::recover(dir.path(), shards).unwrap();
    for i in 0..n {
        assert!(
            recovered.get(&KvKey::new(format!("k:{i}")).unwrap()).is_some(),
            "durably-acked key k:{i} missing after recovery"
        );
    }
}
