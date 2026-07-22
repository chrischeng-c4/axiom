// SPEC-MANAGED: apps/defer/tech-design/logic/core-scheduler-priority-rate-dispatch.md#e2e-test
// HANDWRITE-BEGIN gap="missing-generator:e2e-test:defer-http-api" tracker="#766" reason="Real h2c service shell, OpenAPI, metrics, auth, and domain-route integration."
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use chrono::Utc;
use defer::{AuthConfig, DeferRaft, DeferScheduler, HttpDispatcher, QueuePolicy};
use raft_runtime::Membership;

async fn raft() -> (tempfile::TempDir, Arc<DeferRaft>) {
    let dir = tempfile::tempdir().unwrap();
    let raft = Arc::new(
        DeferRaft::spawn(
            Arc::new(Mutex::new(DeferScheduler::new())),
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
        assert!(
            Instant::now() < deadline,
            "single node did not elect itself"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    (dir, raft)
}

async fn start(raft: Arc<DeferRaft>, auth: AuthConfig) -> (String, tokio::task::JoinHandle<()>) {
    let state = defer::server::AppState::new(
        raft,
        HttpDispatcher::new(Duration::from_secs(2), None).unwrap(),
        auth,
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let serve = tokio::spawn(service_http::serve(
        listener,
        defer::server::router(state),
        std::future::pending(),
    ));
    (url, serve)
}

fn h2c_client() -> reqwest::Client {
    reqwest::Client::builder()
        .http2_prior_knowledge()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap()
}

fn http1_client() -> reqwest::Client {
    reqwest::Client::builder()
        .http1_only()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap()
}

const EXPECTED_DOMAIN_OPERATIONS: &[&str] = &[
    "DELETE /v1/queues/{queue}/tasks/{task_id}",
    "GET /admin/backup",
    "GET /v1/queues/{queue}",
    "GET /v1/queues/{queue}/tasks/{task_id}",
    "POST /v1/queues/{queue}/control",
    "POST /v1/queues/{queue}/dispatch",
    "POST /v1/queues/{queue}/tasks",
    "POST /v1/queues/{queue}/tasks:batch",
    "PUT /v1/queues/{queue}",
];

fn assert_exact_domain_operations(spec: &serde_json::Value) {
    let paths = spec["paths"].as_object().expect("OpenAPI paths object");
    let mut actual = Vec::new();
    for (path, item) in paths {
        let item = item.as_object().expect("OpenAPI path item");
        for method in [
            "delete", "get", "head", "options", "patch", "post", "put", "trace",
        ] {
            if item.contains_key(method) {
                actual.push(format!("{} {path}", method.to_ascii_uppercase()));
            }
        }
    }
    actual.sort();
    assert_eq!(
        actual,
        EXPECTED_DOMAIN_OPERATIONS
            .iter()
            .map(|operation| (*operation).to_string())
            .collect::<Vec<_>>()
    );
}

// <HANDWRITE gap="missing-generator:unit-test" tracker="#2215" reason="Own the required-auth h2c oracle for tokenless operational routes, protected task/admin routes, queue-scoped RBAC, and cross-queue tenant denial.">
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn h2c_routes_probes_openapi_metrics_dispatch_and_auth_are_live() {
    let target_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_url = format!("http://{}/target", target_listener.local_addr().unwrap());
    let target = tokio::spawn(async move {
        axum::serve(
            target_listener,
            Router::new().route("/target", post(|| async { StatusCode::NO_CONTENT })),
        )
        .await
        .unwrap();
    });

    let (_dir, raft) = raft().await;
    let (url, open_server) = start(raft.clone(), AuthConfig::open()).await;
    let client = h2c_client();
    let http1 = http1_client();
    for (protocol, protocol_client) in [("h2c", &client), ("http/1.1", &http1)] {
        for path in ["/healthz", "/readyz", "/docs", "/openapi.json", "/metrics"] {
            assert_eq!(
                protocol_client
                    .get(format!("{url}{path}"))
                    .send()
                    .await
                    .unwrap()
                    .status(),
                StatusCode::OK,
                "{protocol} {path}"
            );
        }
    }
    let h2c_spec = client
        .get(format!("{url}/openapi.json"))
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let http1_spec = http1
        .get(format!("{url}/openapi.json"))
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    assert_eq!(
        h2c_spec, http1_spec,
        "the one-port HTTP/1.1 and h2c OpenAPI responses must be byte-identical"
    );
    let served_spec: serde_json::Value = serde_json::from_slice(&h2c_spec).unwrap();
    assert_eq!(
        served_spec,
        serde_json::to_value(defer::openapi::openapi()).unwrap(),
        "served OpenAPI must equal the canonical IR"
    );
    assert_exact_domain_operations(&served_spec);

    assert_eq!(
        client
            .put(format!("{url}/v1/queues/jobs"))
            .json(&QueuePolicy::default())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        http1
            .get(format!("{url}/v1/queues/jobs"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK,
        "domain routes share the same HTTP/1.1+h2c listener"
    );
    for state in ["Paused", "Running"] {
        let response = http1
            .post(format!("{url}/v1/queues/jobs/control"))
            .json(&serde_json::json!({"state": state}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.json::<serde_json::Value>().await.unwrap()["control_state"],
            state
        );
    }
    assert_eq!(
        client
            .post(format!("{url}/v1/queues/jobs/tasks"))
            .json(&serde_json::json!({
                "task_id": "hello",
                "target": {"url": target_url, "method": "POST", "headers": {}},
                "payload": {"hello": "world"},
                "schedule_at": Utc::now(),
                "priority": 10,
                "max_attempts": 2
            }))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::CREATED
    );
    let batch = http1
        .post(format!("{url}/v1/queues/jobs/tasks:batch"))
        .json(&serde_json::json!({
            "tasks": [{
                "task_id": "batch-through-http1",
                "target": {"url": "http://127.0.0.1/", "method": "POST", "headers": {}},
                "payload": {"contract": "http1-batch-route"},
                "schedule_at": Utc::now() + chrono::Duration::minutes(5),
                "priority": 10,
                "max_attempts": 1
            }]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(batch.status(), StatusCode::CREATED);
    assert_eq!(
        batch.json::<serde_json::Value>().await.unwrap()["created"],
        1
    );
    let dispatched: serde_json::Value = client
        .post(format!("{url}/v1/queues/jobs/dispatch"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(dispatched["target_status"], 204);
    let status: serde_json::Value = client
        .get(format!("{url}/v1/queues/jobs/tasks/hello"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(status["status"], "Succeeded");
    let metrics = client
        .get(format!("{url}/metrics"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(metrics.contains("defer_dispatch_acked_total 1"));

    let snapshot = http1
        .get(format!("{url}/admin/backup"))
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let seed = tempfile::tempdir().unwrap();
    defer::raft::prepare_bootstrap_seed(seed.path(), 0, &snapshot).unwrap();
    let recovered = Arc::new(Mutex::new(DeferScheduler::new()));
    let recovered_raft = DeferRaft::spawn(
        recovered.clone(),
        &seed.path().join("raft"),
        0,
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        DeferRaft::host_config(8),
    )
    .unwrap();
    assert!(recovered_raft.applied_index() > 0);
    assert!(matches!(
        recovered.lock().unwrap().status("jobs", "hello").unwrap(),
        Some(defer::TaskStatus::Succeeded)
    ));
    open_server.abort();

    let registry = serde_json::json!({
        "reader": {"subject": "reader", "roles": {"jobs": "read"}},
        "admin": {"subject": "admin", "roles": {"*": "admin"}}
    })
    .to_string();
    let auth = AuthConfig::resolve("required", None, Some(&registry)).unwrap();
    let (auth_url, auth_server) = start(raft, auth).await;
    for path in ["/healthz", "/readyz", "/docs", "/openapi.json", "/metrics"] {
        assert_eq!(
            client
                .get(format!("{auth_url}{path}"))
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::OK,
            "required-auth service must leave probe/spec route {path} tokenless"
        );
    }
    assert_eq!(
        client
            .get(format!("{auth_url}/v1/queues/jobs"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        client
            .get(format!("{auth_url}/v1/queues/jobs"))
            .bearer_auth("reader")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        client
            .post(format!("{auth_url}/v1/queues/jobs/tasks"))
            .json(&serde_json::json!({
                "task_id": "unauthenticated",
                "target": {"url": "http://127.0.0.1/", "method": "POST", "headers": {}},
                "payload": {},
                "schedule_at": Utc::now(),
                "priority": 10,
                "max_attempts": 1
            }))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        client
            .get(format!("{auth_url}/admin/backup"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        client
            .get(format!("{auth_url}/admin/backup"))
            .bearer_auth("reader")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        client
            .put(format!("{auth_url}/v1/queues/jobs"))
            .bearer_auth("reader")
            .json(&QueuePolicy::default())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        client
            .put(format!("{auth_url}/v1/queues/jobs"))
            .bearer_auth("admin")
            .json(&QueuePolicy::default())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        client
            .put(format!("{auth_url}/v1/queues/other-tenant"))
            .bearer_auth("admin")
            .json(&QueuePolicy::default())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        client
            .get(format!("{auth_url}/v1/queues/other-tenant"))
            .bearer_auth("reader")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN,
        "a queue-scoped credential must not cross a tenant boundary"
    );
    assert_eq!(
        client
            .post(format!("{auth_url}/v1/queues/jobs/tasks"))
            .bearer_auth("admin")
            .json(&serde_json::json!({
                "task_id": "cancel-through-api",
                "target": {"url": "http://127.0.0.1/", "method": "POST", "headers": {}},
                "payload": {"contract": "public-cancel-inspect"},
                "schedule_at": Utc::now() + chrono::Duration::minutes(5),
                "priority": 10,
                "max_attempts": 1
            }))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::CREATED
    );
    assert_eq!(
        client
            .delete(format!(
                "{auth_url}/v1/queues/jobs/tasks/cancel-through-api"
            ))
            .bearer_auth("reader")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN,
        "read access must not grant task cancellation"
    );
    let still_scheduled: serde_json::Value = client
        .get(format!(
            "{auth_url}/v1/queues/jobs/tasks/cancel-through-api"
        ))
        .bearer_auth("reader")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        still_scheduled["status"], "Scheduled",
        "a rejected cancellation must not mutate task state"
    );
    assert_eq!(
        client
            .delete(format!(
                "{auth_url}/v1/queues/jobs/tasks/cancel-through-api"
            ))
            .bearer_auth("admin")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::NO_CONTENT
    );
    let canceled: serde_json::Value = client
        .get(format!(
            "{auth_url}/v1/queues/jobs/tasks/cancel-through-api"
        ))
        .bearer_auth("reader")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(canceled["status"], "Canceled");

    auth_server.abort();
    target.abort();
}
// </HANDWRITE>
// HANDWRITE-END
