// HANDWRITE-BEGIN gap="missing-generator:e2e-test:tape-retention-backfill" tracker="#768" reason="Real HTTP proof for protected retention compaction, stable offsets, and timestamp/offset backfill windows."
use std::net::SocketAddr;

use serde_json::json;
use tape::server::{router, AppState};
use tape::TapeJournal;

async fn start_server() -> SocketAddr {
    let app = router(AppState::new(TapeJournal::default(), None));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(service_http::serve(
        listener,
        app,
        std::future::pending::<()>(),
    ));
    addr
}

fn url(addr: SocketAddr, path: &str) -> String {
    format!("http://{addr}{path}")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn protected_retention_preserves_consumer_window_and_backfill_offsets() {
    let addr = start_server().await;
    let client = reqwest::Client::new();

    for offset in 0..5_u64 {
        let response: serde_json::Value = client
            .post(url(addr, "/topics/orders/append"))
            .json(&json!({
                "payload": {"offset": offset},
                "timestamp_ms": 1_000 + offset * 1_000
            }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(response["offset"], offset);
    }

    let checkpoint = client
        .put(url(addr, "/topics/orders/consumers/audit/checkpoint"))
        .json(&json!({"offset": 2}))
        .send()
        .await
        .unwrap();
    assert_eq!(checkpoint.status(), 200);

    let retention: serde_json::Value = client
        .put(url(addr, "/topics/orders/retention"))
        .json(&json!({
            "min_offset": 4,
            "max_age_seconds": null,
            "protected_consumers": ["audit"]
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(retention["removed"], 2);
    assert_eq!(retention["earliest_offset"], 2);
    assert_eq!(retention["end_offset"], 5);

    let by_offset: serde_json::Value = client
        .get(url(addr, "/topics/orders/replay?from_offset=2&limit=10"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(by_offset["events"].as_array().unwrap().len(), 3);
    assert_eq!(by_offset["events"][0]["offset"], 2);
    assert_eq!(by_offset["events"][2]["offset"], 4);

    let by_timestamp: serde_json::Value = client
        .get(url(
            addr,
            "/topics/orders/replay?from_timestamp_ms=3500&limit=10",
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(by_timestamp["events"].as_array().unwrap().len(), 2);
    assert_eq!(by_timestamp["events"][0]["offset"], 3);
    assert_eq!(by_timestamp["events"][1]["offset"], 4);

    let appended: serde_json::Value = client
        .post(url(addr, "/topics/orders/append"))
        .json(&json!({"payload": {"offset": 5}, "timestamp_ms": 6_000}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(appended["offset"], 5, "compaction must not reuse offsets");

    let policy: serde_json::Value = client
        .get(url(addr, "/topics/orders/retention"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(policy["policy"]["min_offset"], 4);
    assert_eq!(policy["policy"]["protected_consumers"][0], "audit");
}
// HANDWRITE-END
