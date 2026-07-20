// SPEC-MANAGED: apps/defer/tech-design/logic/core-scheduler-priority-rate-dispatch.md#e2e-test
// HANDWRITE-BEGIN gap="missing-generator:e2e-test:defer-http-dispatch" tracker="#766" reason="Real HTTP target proof for signed delivery, retry, stable idempotency, and terminal committed ack."
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, Method, StatusCode};
use axum::routing::{any, post};
use axum::Router;
use chrono::Utc;
use defer::{
    AuthConfig, CreateTask, DeferRaft, DeferScheduler, DispatchDisposition, HttpDispatcher,
    QueuePolicy, Target, TargetSigningKey, TaskStatus,
};
use raft_runtime::Membership;
use tokio::sync::Notify;

#[derive(Clone)]
struct Received {
    method: Method,
    target_header: String,
    idempotency_key: String,
    attempt_id: String,
    key_id: String,
    signature: String,
    body: serde_json::Value,
}

async fn bounded_target(
    State((active, peak)): State<(Arc<AtomicUsize>, Arc<AtomicUsize>)>,
) -> StatusCode {
    let now = active.fetch_add(1, Ordering::SeqCst) + 1;
    peak.fetch_max(now, Ordering::SeqCst);
    tokio::time::sleep(Duration::from_millis(40)).await;
    active.fetch_sub(1, Ordering::SeqCst);
    StatusCode::NO_CONTENT
}

async fn flaky_target(
    State(received): State<Arc<Mutex<Vec<Received>>>>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    let record = Received {
        method,
        target_header: headers["x-defer-tenant"].to_str().unwrap().into(),
        idempotency_key: headers["idempotency-key"].to_str().unwrap().into(),
        attempt_id: headers["x-defer-attempt-id"].to_str().unwrap().into(),
        key_id: headers["x-defer-key-id"].to_str().unwrap().into(),
        signature: headers["x-defer-signature"].to_str().unwrap().into(),
        body: serde_json::from_slice(&body).unwrap(),
    };
    let mut received = received.lock().unwrap();
    received.push(record);
    if received.len() == 1 {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::NO_CONTENT
    }
}

#[derive(Default)]
struct AmbiguousTarget {
    requests: AtomicUsize,
    accepted_first: Notify,
    release_first: Notify,
    idempotency_keys: Mutex<Vec<String>>,
}

async fn ambiguous_target(
    State(state): State<Arc<AmbiguousTarget>>,
    headers: HeaderMap,
) -> StatusCode {
    state
        .idempotency_keys
        .lock()
        .unwrap()
        .push(headers["idempotency-key"].to_str().unwrap().to_string());
    if state.requests.fetch_add(1, Ordering::SeqCst) == 0 {
        // Model a target that has accepted the effect while the executor has
        // not yet received the response and therefore cannot settle Raft.
        state.accepted_first.notify_one();
        state.release_first.notified().await;
    }
    StatusCode::NO_CONTENT
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn dispatches_real_http_and_retries_with_stable_task_idempotency() {
    let received = Arc::new(Mutex::new(Vec::new()));
    let target_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_url = format!("http://{}/task", target_listener.local_addr().unwrap());
    let target_app = Router::new()
        .route("/task", any(flaky_target))
        .with_state(received.clone());
    let target_server = tokio::spawn(async move {
        axum::serve(target_listener, target_app).await.unwrap();
    });

    let dir = tempfile::tempdir().unwrap();
    let scheduler = Arc::new(Mutex::new(DeferScheduler::new()));
    let raft = DeferRaft::spawn(
        scheduler.clone(),
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
    raft.configure_queue("jobs".into(), QueuePolicy::default())
        .await
        .unwrap();
    raft.create_task(
        "jobs".into(),
        CreateTask {
            task_id: "invoice-42".into(),
            target: Target {
                url: target_url,
                method: "PATCH".into(),
                headers: BTreeMap::from([("x-defer-tenant".into(), "tenant-a".into())]),
            },
            payload: serde_json::json!({"invoice": 42}),
            schedule_at: now,
            priority: 10,
            max_attempts: 2,
        },
    )
    .await
    .unwrap();

    let signing = TargetSigningKey::new("active-2026-07", vec![7; 32]).unwrap();
    let dispatcher = HttpDispatcher::new(Duration::from_secs(2), Some(signing)).unwrap();
    let first = dispatcher
        .dispatch_one(&raft, "jobs", now)
        .await
        .unwrap()
        .unwrap();
    let retry_at = match first.disposition {
        DispatchDisposition::Retried { next_at } => next_at,
        other => panic!("expected retry, got {other:?}"),
    };
    assert_eq!(first.target_status, Some(503));

    let second = dispatcher
        .dispatch_one(&raft, "jobs", retry_at)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(second.target_status, Some(204));
    assert_eq!(second.disposition, DispatchDisposition::Acked);
    assert!(matches!(
        scheduler
            .lock()
            .unwrap()
            .status("jobs", "invoice-42")
            .unwrap(),
        Some(TaskStatus::Succeeded)
    ));

    let received = received.lock().unwrap();
    assert_eq!(received.len(), 2);
    assert!(received
        .iter()
        .all(|request| request.method == Method::PATCH));
    assert!(received
        .iter()
        .all(|request| request.target_header == "tenant-a"));
    assert_eq!(received[0].idempotency_key, "jobs/invoice-42");
    assert_eq!(received[1].idempotency_key, "jobs/invoice-42");
    assert_ne!(received[0].attempt_id, received[1].attempt_id);
    assert_eq!(received[0].key_id, "active-2026-07");
    assert_eq!(received[1].key_id, "active-2026-07");
    assert!(received[0].signature.starts_with("v1="));
    assert!(received[1].signature.starts_with("v1="));
    assert_eq!(received[0].body, serde_json::json!({"invoice": 42}));
    assert_eq!(received[1].body, serde_json::json!({"invoice": 42}));

    target_server.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn accepted_http_with_lost_fence_is_retried_with_the_stable_key() {
    let target_state = Arc::new(AmbiguousTarget::default());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_url = format!("http://{}/task", listener.local_addr().unwrap());
    let target = Router::new()
        .route("/task", post(ambiguous_target))
        .with_state(target_state.clone());
    let target_server = tokio::spawn(async move { axum::serve(listener, target).await.unwrap() });

    let dir = tempfile::tempdir().unwrap();
    let scheduler = Arc::new(Mutex::new(DeferScheduler::new()));
    let raft = Arc::new(
        DeferRaft::spawn(
            scheduler.clone(),
            &dir.path().join("raft"),
            0,
            Membership {
                voters: vec![0],
                learners: vec![],
            },
            HashMap::new(),
            DeferRaft::host_config(8),
        )
        .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(4);
    while !raft.is_leader().await {
        assert!(Instant::now() < deadline, "single node did not elect");
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    let now = Utc::now();
    raft.configure_queue(
        "jobs".into(),
        QueuePolicy {
            lease_ttl_ms: 10,
            retry_backoff_ms: 0,
            ..QueuePolicy::default()
        },
    )
    .await
    .unwrap();
    raft.create_task(
        "jobs".into(),
        CreateTask {
            task_id: "ambiguous".into(),
            target: Target {
                url: target_url,
                method: "POST".into(),
                headers: Default::default(),
            },
            payload: serde_json::json!({"effect": "may-have-succeeded"}),
            schedule_at: now,
            priority: 10,
            max_attempts: 3,
        },
    )
    .await
    .unwrap();

    let dispatcher = Arc::new(HttpDispatcher::new(Duration::from_secs(2), None).unwrap());
    let first = tokio::spawn({
        let dispatcher = dispatcher.clone();
        let raft = raft.clone();
        async move {
            dispatcher
                .dispatch_one(&raft, "jobs", now)
                .await
                .unwrap()
                .unwrap()
        }
    });
    tokio::time::timeout(
        Duration::from_secs(2),
        target_state.accepted_first.notified(),
    )
    .await
    .expect("target accepted the first effect");

    // Ownership expires after the target has accepted but before the old
    // executor can commit its successful settlement.
    let after_expiry = now + chrono::Duration::milliseconds(11);
    assert_eq!(
        raft.reclaim_expired("jobs".into(), after_expiry)
            .await
            .unwrap(),
        vec!["ambiguous".to_string()]
    );
    target_state.release_first.notify_one();
    let first = first.await.unwrap();
    assert_eq!(first.target_status, Some(204));
    assert_eq!(first.disposition, DispatchDisposition::LostOwnership);

    // The 10ms TTL above exists only to deterministically create the
    // ambiguous first attempt; retry under the queue's normal lease budget.
    raft.configure_queue("jobs".into(), QueuePolicy::default())
        .await
        .unwrap();
    let retry = dispatcher
        .dispatch_one(&raft, "jobs", after_expiry)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(retry.target_status, Some(204));
    assert_eq!(retry.disposition, DispatchDisposition::Acked);
    assert!(matches!(
        scheduler
            .lock()
            .unwrap()
            .status("jobs", "ambiguous")
            .unwrap(),
        Some(TaskStatus::Succeeded)
    ));
    let keys = target_state.idempotency_keys.lock().unwrap().clone();
    assert_eq!(keys.as_slice(), ["jobs/ambiguous", "jobs/ambiguous"]);

    raft.shutdown().await.unwrap();
    target_server.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn shared_executor_dispatches_concurrently_with_a_hard_bound() {
    let active = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_url = format!("http://{}/task", listener.local_addr().unwrap());
    let target = Router::new()
        .route("/task", post(bounded_target))
        .with_state((active.clone(), peak.clone()));
    let server = tokio::spawn(async move { axum::serve(listener, target).await.unwrap() });

    let dir = tempfile::tempdir().unwrap();
    let scheduler = Arc::new(Mutex::new(DeferScheduler::new()));
    let raft = Arc::new(
        DeferRaft::spawn(
            scheduler.clone(),
            &dir.path().join("raft"),
            0,
            Membership {
                voters: vec![0],
                learners: vec![],
            },
            HashMap::new(),
            DeferRaft::host_config(32),
        )
        .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(4);
    while !raft.is_leader().await {
        assert!(Instant::now() < deadline, "single node did not elect");
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    let now = Utc::now();
    let policy = QueuePolicy {
        max_in_flight: 16,
        max_dispatch_per_tick: 16,
        max_dispatches_per_second: 10_000,
        max_burst_size: 16,
        ..QueuePolicy::default()
    };
    raft.configure_queue("jobs".into(), policy).await.unwrap();
    for id in 0..8 {
        raft.create_task(
            "jobs".into(),
            CreateTask {
                task_id: format!("task-{id}"),
                target: Target {
                    url: target_url.clone(),
                    method: "POST".into(),
                    headers: Default::default(),
                },
                payload: serde_json::json!({"id": id}),
                schedule_at: now,
                priority: 10,
                max_attempts: 2,
            },
        )
        .await
        .unwrap();
    }
    let app = defer::server::AppState::new(
        raft,
        HttpDispatcher::new(Duration::from_secs(2), None).unwrap(),
        AuthConfig::open(),
    );
    assert_eq!(app.dispatch_tick(8, 3).await.unwrap(), 8);
    assert_eq!(peak.load(Ordering::SeqCst), 3);
    assert_eq!(
        scheduler
            .lock()
            .unwrap()
            .queue_snapshot("jobs")
            .unwrap()
            .terminal_count,
        8
    );
    server.abort();
}
// HANDWRITE-END
