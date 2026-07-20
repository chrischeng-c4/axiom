// HANDWRITE-BEGIN gap="missing-generator:unit-test:d3b272e8" tracker="pending-tracker" reason="scaffold for apps/beam/tests/restart_recovery.rs — fill in by hand and update tracker when codegen is ready"
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::RwLock;
use serde_json::{json, Value};
use beam::collection::Collection;

async fn wait_healthy(client: &reqwest::Client, base: &str) -> bool {
    for _ in 0..100 {
        if let Ok(resp) = client.get(format!("{base}/healthz")).send().await {
            if resp.status().is_success() {
                return true;
            }
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    false
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_restart_recovery_and_reject_corrupt() {
    let data_dir = std::env::temp_dir().join(format!("beam_test_restart_recovery_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data_dir);
    std::fs::create_dir_all(&data_dir).unwrap();

    // --- Phase 1: Start Server 1, create collection, and upsert vectors ---
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base = format!("http://{addr}");

    let gpu = beam::gpu::GpuContext::new().map(Arc::new);
    let registry = Arc::new(RwLock::new(HashMap::new()));
    let app = beam::service::router_with_state(
        registry.clone(),
        gpu.clone(),
        Some(data_dir.clone()),
        Arc::new(service_auth::StaticRoleMapVerifier::open()),
    );

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let server_handle = tokio::spawn(async move {
        let shutdown_fut = async move {
            let _ = shutdown_rx.await;
        };
        beam::service::serve_on(listener, app, shutdown_fut).await;
    });

    let client = reqwest::Client::new();
    assert!(wait_healthy(&client, &base).await);

    // Create a collection
    let create = client
        .post(format!("{base}/v1/collections"))
        .json(&json!({ "name": "persist_test", "dim": 2, "metric": "l2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(create.status(), 201);

    // Upsert vectors
    let add = client
        .post(format!("{base}/v1/collections/persist_test/vectors"))
        .json(&json!({
            "items": [
                { "id": "v1", "vector": [1.0, 2.0], "payload": { "val": 100 } },
                { "id": "v2", "vector": [3.0, 4.0], "payload": { "val": 200 } }
            ]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(add.status(), 200);

    // Shutdown Server 1
    let _ = shutdown_tx.send(());
    let _ = server_handle.await;

    // Verify the file `persist_test.bin` exists in the data directory
    let file_path = data_dir.join("persist_test.bin");
    assert!(file_path.exists());

    // --- Phase 2: Start Server 2 (Restart & Recover!) ---
    // Simulate serve startup scan directory
    let mut registry_map = HashMap::new();
    if let Ok(entries) = std::fs::read_dir(&data_dir) {
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("bin") {
                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                let col = Collection::load(&path).unwrap();
                let mut cs = beam::service::CollectionState::new(col);
                cs.rebuild(&gpu);
                registry_map.insert(name, cs);
            }
        }
    }
    assert_eq!(registry_map.len(), 1);
    assert!(registry_map.contains_key("persist_test"));

    let registry2 = Arc::new(RwLock::new(registry_map));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base = format!("http://{addr}");
    let app = beam::service::router_with_state(
        registry2.clone(),
        gpu.clone(),
        Some(data_dir.clone()),
        Arc::new(service_auth::StaticRoleMapVerifier::open()),
    );

    let (shutdown_tx2, shutdown_rx2) = tokio::sync::oneshot::channel::<()>();
    let server_handle2 = tokio::spawn(async move {
        let shutdown_fut = async move {
            let _ = shutdown_rx2.await;
        };
        beam::service::serve_on(listener, app, shutdown_fut).await;
    });

    assert!(wait_healthy(&client, &base).await);

    // Verify vectors were recovered successfully
    let query: Value = client
        .post(format!("{base}/v1/collections/persist_test/query"))
        .json(&json!({
            "vector": [1.0, 2.0],
            "k": 2
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let hits = query["neighbors"].as_array().unwrap();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0]["id"], "v1");
    assert_eq!(hits[1]["id"], "v2");

    // Shutdown Server 2
    let _ = shutdown_tx2.send(());
    let _ = server_handle2.await;

    // --- Phase 3: Corrupt the snapshot and assert loading fails ---
    // Backup the good file
    let backup_path = data_dir.join("persist_test.bin.bak");
    std::fs::copy(&file_path, &backup_path).unwrap();

    // Corrupt the active snapshot file with garbage
    std::fs::write(&file_path, b"GARBAGE_DATA_THAT_IS_NOT_A_VALID_SNAPSHOT").unwrap();

    // Assert that loading the corrupted file fails, keeping backup/good data untouched
    let load_res = Collection::load(&file_path);
    assert!(load_res.is_err(), "Loading corrupted snapshot must fail!");

    // Restore backup
    std::fs::copy(&backup_path, &file_path).unwrap();
    let recover_res = Collection::load(&file_path);
    assert!(recover_res.is_ok(), "Restoring verified backup should succeed!");

    // Cleanup
    let _ = std::fs::remove_dir_all(&data_dir);
}
// HANDWRITE-END
