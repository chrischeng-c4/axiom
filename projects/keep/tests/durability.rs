//! Proves durable-before-ack: once a write returns 200, its data is already
//! fsynced to the WAL and survives a cold recovery.

use std::sync::Arc;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use keep::persistence::recovery::RecoveryManager;
use keep::persistence::{PersistenceConfig, PersistenceHandle};
use keep::{router, AppState, KvEngine, KvKey};
use serde_json::{json, Value};
use tower::ServiceExt;

async fn put_json(app: &Router, method: &str, path: &str, body: Value) -> StatusCode {
    let req = Request::builder()
        .method(method)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    app.clone().oneshot(req).await.unwrap().status()
}

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

#[tokio::test]
async fn collections_survive_recovery() {
    let dir = tempfile::tempdir().unwrap();
    let shards = 16;

    let engine = Arc::new(KvEngine::with_shards(shards));
    let cfg = PersistenceConfig::new(dir.path()).with_fsync_interval_ms(100);
    let persistence = Arc::new(PersistenceHandle::new(cfg, engine.clone()).unwrap());
    engine.enable_persistence(persistence);
    let app = router(AppState::new(engine));

    // Write one of each collection type + a TTL, all durably acked.
    assert_eq!(put_json(&app, "POST", "/v1/hashes/h", json!({"fields": {"a": 1, "b": "x"}})).await, StatusCode::OK);
    assert_eq!(put_json(&app, "POST", "/v1/sets/s", json!({"members": ["m1", "m2"]})).await, StatusCode::OK);
    assert_eq!(put_json(&app, "POST", "/v1/zsets/z", json!({"members": [{"member": "a", "score": 2.0}]})).await, StatusCode::OK);
    assert_eq!(put_json(&app, "POST", "/v1/lists/l/rpush", json!({"values": [1, 2, 3]})).await, StatusCode::OK);
    assert_eq!(put_json(&app, "PUT", "/v1/kv/scalar", json!({"value": "v"})).await, StatusCode::OK);
    assert_eq!(put_json(&app, "POST", "/v1/kv/scalar/expire", json!({"seconds": 1000})).await, StatusCode::OK);

    // Cold recovery: every collection write must be on disk (the WalOp fix).
    let (rec, _stats) = RecoveryManager::recover(dir.path(), shards).unwrap();
    assert_eq!(rec.hgetall(&KvKey::new("h").unwrap()).unwrap().len(), 2, "hash lost");
    assert_eq!(rec.scard(&KvKey::new("s").unwrap()).unwrap(), 2, "set lost");
    assert_eq!(rec.zcard(&KvKey::new("z").unwrap()).unwrap(), 1, "zset lost");
    assert_eq!(rec.llen(&KvKey::new("l").unwrap()).unwrap(), 3, "list lost");
    assert!(rec.ttl(&KvKey::new("scalar").unwrap()) > 0, "TTL lost");
}
