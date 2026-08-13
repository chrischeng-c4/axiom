use axum::routing::get;
use server_lifecycle::{HookStage, LifecycleController, LifecyclePhase};
use service_http::{
    lifecycle_probe_routes, run_signal_bridge, serve_with_lifecycle, LifecycleShutdownTrigger,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::sync::Notify;

#[tokio::test]
async fn immediate_drain() {
    let c = LifecycleController::serving();
    let release = Arc::new(Notify::new());
    let release_hook = release.clone();
    c.register_hook(HookStage::AdmissionStop, "blocked", move |_| {
        let release_hook = release_hook.clone();
        async move {
            release_hook.notified().await;
            Ok(())
        }
    })
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = lifecycle_probe_routes(c.clone(), None, || {
        utoipa::openapi::OpenApi::new(
            utoipa::openapi::Info::new("test", "1"),
            utoipa::openapi::Paths::new(),
        )
    })
    .route("/hello", get(|| async { "hello" }));
    let server = tokio::spawn(serve_with_lifecycle(
        listener,
        app,
        server_http::HttpServerOptions::default(),
        c.clone(),
    ));
    let mut ready_stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    ready_stream
        .write_all(b"GET /readyz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    let mut bytes = Vec::new();
    ready_stream.read_to_end(&mut bytes).await.unwrap();
    assert!(String::from_utf8_lossy(&bytes).starts_with("HTTP/1.1 200"));
    let trigger =
        LifecycleShutdownTrigger::new(c.clone(), Duration::from_secs(5), Duration::ZERO).unwrap();
    let trigger_task = tokio::spawn(run_signal_bridge(trigger, async {}));
    let mut sub = c.subscribe();
    let deadline = tokio::time::timeout(Duration::from_secs(1), sub.wait_shutdown_deadline())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(c.observation().phase, LifecyclePhase::Draining);
    let probe = lifecycle_probe_routes(c.clone(), None, || {
        utoipa::openapi::OpenApi::new(
            utoipa::openapi::Info::new("test", "1"),
            utoipa::openapi::Paths::new(),
        )
    });
    let response = tower::ServiceExt::oneshot(
        probe,
        axum::http::Request::builder()
            .uri("/readyz")
            .body(axum::body::Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(
        response.status(),
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    );
    assert!(matches!(
        tokio::time::timeout(Duration::from_secs(1), tokio::net::TcpStream::connect(addr)).await,
        Ok(Err(_))
    ));
    release.notify_one();
    let report = trigger_task.await.unwrap();
    assert_eq!(report.initiating_generation, 1);
    assert_eq!(report.outcomes.len(), 1);
    assert_eq!(
        report.outcomes[0].status,
        server_lifecycle::HookStatus::Completed
    );
    assert_eq!(c.observation().phase, LifecyclePhase::Stopped);
    assert_eq!(sub.shutdown_deadline(), Some(deadline));
    let server_report = server.await.unwrap();
    assert!(server_report.accepted >= 1 && server_report.streams_admitted >= 1);
}

#[tokio::test]
async fn authorized_trigger() {
    let c = LifecycleController::serving();
    let initial = c.observation();
    let count = Arc::new(AtomicUsize::new(0));
    let observed = count.clone();
    c.register_hook(HookStage::AdmissionStop, "counter", move |_| {
        let observed = observed.clone();
        async move {
            observed.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    })
    .unwrap();
    let trigger =
        LifecycleShutdownTrigger::new(c.clone(), Duration::from_secs(1), Duration::ZERO).unwrap();
    let mut subscription = c.subscribe();
    let (a, b) = tokio::join!(
        trigger.trigger("operator", "one"),
        trigger.trigger("operator", "two")
    );
    assert_eq!(a.initiating_generation, b.initiating_generation);
    assert!(Arc::ptr_eq(&a, &b));
    assert_eq!(count.load(Ordering::SeqCst), 1);
    assert_eq!(
        subscription.shutdown_deadline(),
        Some(subscription.wait_shutdown_deadline().await.unwrap())
    );
    assert_eq!(initial.generation + 1, a.initiating_generation);
    let router = lifecycle_probe_routes(c, None, || {
        utoipa::openapi::OpenApi::new(
            utoipa::openapi::Info::new("test", "1"),
            utoipa::openapi::Paths::new(),
        )
    });
    for path in ["/shutdown", "/drain"] {
        let response = tower::ServiceExt::oneshot(
            router.clone(),
            axum::http::Request::builder()
                .uri(path)
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }
}
