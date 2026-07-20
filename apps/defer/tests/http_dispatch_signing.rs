// SPEC-MANAGED: apps/defer/tech-design/logic/core-scheduler-priority-rate-dispatch.md#e2e-test
// HANDWRITE-BEGIN gap="missing-generator:e2e-test:defer-http-dispatch-signing-oracle" tracker="#766" reason="Independent target-side HMAC oracle and negative cases for signed HTTP retries."
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::Router;
use base64::Engine as _;
use chrono::Utc;
use defer::{
    CreateTask, DeferRaft, DeferScheduler, DispatchDisposition, HttpDispatcher, QueuePolicy,
    Target, TargetSigningKey,
};
use hmac::{Hmac, Mac};
use raft_runtime::Membership;
use sha2::Sha256;

#[derive(Clone, Debug)]
struct SignedRequest {
    idempotency_key: String,
    attempt_id: String,
    timestamp_ms: i64,
    key_id: String,
    signature: String,
    body: Vec<u8>,
}

async fn flaky_signed_target(
    State(received): State<Arc<Mutex<Vec<SignedRequest>>>>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    let request = SignedRequest {
        idempotency_key: headers["idempotency-key"].to_str().unwrap().to_string(),
        attempt_id: headers["x-defer-attempt-id"].to_str().unwrap().to_string(),
        timestamp_ms: headers["x-defer-timestamp-ms"]
            .to_str()
            .unwrap()
            .parse()
            .unwrap(),
        key_id: headers["x-defer-key-id"].to_str().unwrap().to_string(),
        signature: headers["x-defer-signature"].to_str().unwrap().to_string(),
        body: body.to_vec(),
    };
    let mut received = received.lock().unwrap();
    received.push(request);
    if received.len() == 1 {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::NO_CONTENT
    }
}

fn oracle_signature(
    secret: &[u8],
    idempotency_key: &str,
    attempt_id: &str,
    target_url: &str,
    timestamp_ms: i64,
    body: &[u8],
) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret).unwrap();
    let timestamp = timestamp_ms.to_string();
    for field in [
        idempotency_key.as_bytes(),
        attempt_id.as_bytes(),
        target_url.as_bytes(),
        timestamp.as_bytes(),
        body,
    ] {
        mac.update(&(field.len() as u64).to_be_bytes());
        mac.update(field);
    }
    format!(
        "v1={}",
        base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes())
    )
}

fn oracle_accepts(
    request: &SignedRequest,
    target_url: &str,
    expected_key_id: &str,
    secret: &[u8],
) -> bool {
    request.key_id == expected_key_id
        && request.signature
            == oracle_signature(
                secret,
                &request.idempotency_key,
                &request.attempt_id,
                target_url,
                request.timestamp_ms,
                &request.body,
            )
}

// <HANDWRITE gap="missing-generator:unit-test" tracker="pending-tracker" reason="Independently recompute the length-delimited HMAC at the target and reject field/body tampering, wrong key identity, and wrong secrets across retry attempts.">
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn target_oracle_verifies_exact_signature_and_rejects_tampering() {
    let received = Arc::new(Mutex::new(Vec::new()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_url = format!("http://{}/task", listener.local_addr().unwrap());
    let target = Router::new()
        .route("/task", post(flaky_signed_target))
        .with_state(received.clone());
    let target_server = tokio::spawn(async move {
        axum::serve(listener, target).await.unwrap();
    });

    let dir = tempfile::tempdir().unwrap();
    let scheduler = Arc::new(Mutex::new(DeferScheduler::new()));
    let raft = DeferRaft::spawn(
        scheduler,
        &dir.path().join("raft"),
        0,
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        DeferRaft::host_config(8),
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(4);
    while !raft.is_leader().await {
        assert!(
            Instant::now() < deadline,
            "single node did not elect itself"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    let now = Utc::now();
    raft.configure_queue(
        "signed".into(),
        QueuePolicy {
            retry_backoff_ms: 0,
            ..QueuePolicy::default()
        },
    )
    .await
    .unwrap();
    raft.create_task(
        "signed".into(),
        CreateTask {
            task_id: "invoice-42".into(),
            target: Target {
                url: target_url.clone(),
                method: "POST".into(),
                headers: Default::default(),
            },
            payload: serde_json::json!({"invoice": 42, "action": "capture"}),
            schedule_at: now,
            priority: 10,
            max_attempts: 2,
        },
    )
    .await
    .unwrap();

    let key_id = "active-2026-07";
    let secret = vec![7; 32];
    let dispatcher = HttpDispatcher::new(
        Duration::from_secs(2),
        Some(TargetSigningKey::new(key_id, secret.clone()).unwrap()),
    )
    .unwrap();
    let first = dispatcher
        .dispatch_one(&raft, "signed", now)
        .await
        .unwrap()
        .unwrap();
    let retry_at = match first.disposition {
        DispatchDisposition::Retried { next_at } => next_at,
        other => panic!("expected retry, got {other:?}"),
    };
    let second = dispatcher
        .dispatch_one(&raft, "signed", retry_at)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(second.disposition, DispatchDisposition::Acked);

    let received = received.lock().unwrap();
    assert_eq!(received.len(), 2);
    for request in received.iter() {
        assert!(
            oracle_accepts(request, &target_url, key_id, &secret),
            "independent target oracle rejected production signature"
        );
    }
    assert_eq!(received[0].idempotency_key, "signed/invoice-42");
    assert_eq!(received[0].idempotency_key, received[1].idempotency_key);
    assert_ne!(received[0].attempt_id, received[1].attempt_id);
    assert_ne!(received[0].signature, received[1].signature);

    let original = &received[0];
    let mut tampered = original.clone();
    tampered.idempotency_key.push_str("-tampered");
    assert!(!oracle_accepts(&tampered, &target_url, key_id, &secret));

    let mut tampered = original.clone();
    tampered.attempt_id.push_str("-tampered");
    assert!(!oracle_accepts(&tampered, &target_url, key_id, &secret));

    assert!(!oracle_accepts(
        original,
        &format!("{target_url}/tampered"),
        key_id,
        &secret
    ));

    let mut tampered = original.clone();
    tampered.timestamp_ms += 1;
    assert!(!oracle_accepts(&tampered, &target_url, key_id, &secret));

    let mut tampered = original.clone();
    tampered.body.push(b' ');
    assert!(!oracle_accepts(&tampered, &target_url, key_id, &secret));

    assert!(!oracle_accepts(
        original,
        &target_url,
        "retired-key",
        &secret
    ));
    assert!(!oracle_accepts(original, &target_url, key_id, &[9; 32]));

    target_server.abort();
}
// </HANDWRITE>
// HANDWRITE-END
