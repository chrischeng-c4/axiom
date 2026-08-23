use axum::{routing::get, Router};
use server_http::{serve_h2c_with_lifecycle, HttpServerOptions};
use server_lifecycle::{LifecycleController, ShutdownDeadline};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn listener() -> (TcpListener, LifecycleController) {
    (
        TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap(),
        LifecycleController::serving(),
    )
}

#[tokio::test]
async fn explicit_controller() {
    let (listener, control) = listener().await;
    let deadline = ShutdownDeadline::from_now(Duration::from_millis(100), Duration::ZERO).unwrap();
    let _ = control.shutdown(deadline, "test", "prestarted").await;
    let started = Instant::now();
    let report = serve_h2c_with_lifecycle(
        listener,
        Router::new(),
        HttpServerOptions::default(),
        control,
    )
    .await;
    assert!(started.elapsed() < Duration::from_secs(1));
    assert_eq!(report.accepted, 0);
    assert!(!report.deadline_missing);
}

#[tokio::test]
async fn stop_accept() {
    let (listener, control) = listener().await;
    let addr = listener.local_addr().unwrap();
    let task = tokio::spawn(serve_h2c_with_lifecycle(
        listener,
        Router::new().route("/", get(|| async { "ok" })),
        HttpServerOptions::default(),
        control.clone(),
    ));
    let mut client = tokio::net::TcpStream::connect(addr).await.unwrap();
    client
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: keep-alive\r\n\r\n")
        .await
        .unwrap();
    let mut response = [0_u8; 64];
    let n = tokio::time::timeout(Duration::from_secs(1), client.read(&mut response))
        .await
        .unwrap()
        .unwrap();
    assert!(String::from_utf8_lossy(&response[..n]).contains("200"));
    let _ = control
        .shutdown(
            ShutdownDeadline::from_now(Duration::from_secs(1), Duration::ZERO).unwrap(),
            "test",
            "drain",
        )
        .await;
    let report = task.await.unwrap();
    assert!(report.accepted >= 1);
    assert!(tokio::net::TcpStream::connect(addr).await.is_err());
}

#[tokio::test]
async fn connection_context() {
    let (listener, control) = listener().await;
    let addr = listener.local_addr().unwrap();
    let task = tokio::spawn(serve_h2c_with_lifecycle(
        listener,
        Router::new().route("/", get(|| async { "ok" })),
        HttpServerOptions::default(),
        control.clone(),
    ));
    let mut http1 = tokio::net::TcpStream::connect(addr).await.unwrap();
    http1
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: keep-alive\r\n\r\n")
        .await
        .unwrap();
    let mut buf = [0_u8; 128];
    let n = tokio::time::timeout(Duration::from_secs(1), http1.read(&mut buf))
        .await
        .unwrap()
        .unwrap();
    assert!(String::from_utf8_lossy(&buf[..n]).contains("200"));
    let h2 = transport_h2c::h2c_client().unwrap();
    let response = h2.get(format!("http://{addr}/")).send().await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "ok");
    let _ = control
        .shutdown(
            ShutdownDeadline::from_now(Duration::from_secs(1), Duration::ZERO).unwrap(),
            "test",
            "drain",
        )
        .await;
    let report = task.await.unwrap();
    assert!(report.accepted >= 2);
    assert!(report.streams_admitted >= 2);
    assert!(report.streams_completed >= 2);
}

#[tokio::test]
async fn one_deadline() {
    let (listener, control) = listener().await;
    let addr = listener.local_addr().unwrap();
    let gate = std::sync::Arc::new(tokio::sync::Notify::new());
    let admitted = std::sync::Arc::new(tokio::sync::Notify::new());
    let pending = gate.clone();
    let seen = admitted.clone();
    let task = tokio::spawn(serve_h2c_with_lifecycle(
        listener,
        Router::new().route(
            "/",
            get(move || {
                let pending = pending.clone();
                let seen = seen.clone();
                async move {
                    seen.notify_one();
                    pending.notified().await;
                    "ok"
                }
            }),
        ),
        HttpServerOptions::default(),
        control.clone(),
    ));
    let mut client = tokio::net::TcpStream::connect(addr).await.unwrap();
    client
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: keep-alive\r\n\r\n")
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(1), admitted.notified())
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(10)).await;
    let started = Instant::now();
    let _ = control
        .shutdown(
            ShutdownDeadline::from_now(Duration::from_millis(120), Duration::from_millis(30))
                .unwrap(),
            "test",
            "deadline",
        )
        .await;
    let report = task.await.unwrap();
    assert!(started.elapsed() < Duration::from_millis(500));
    assert!(report.accepted > 0);
    assert!(report.timed_out > 0);
    assert!(report.streams_admitted > 0);
    assert!(report.streams_timed_out > 0);
}

#[tokio::test]
async fn terminal_report() {
    let (listener, control) = listener().await;
    let addr = listener.local_addr().unwrap();
    let gate = std::sync::Arc::new(tokio::sync::Notify::new());
    let slow_gate = gate.clone();
    let task = tokio::spawn(serve_h2c_with_lifecycle(
        listener,
        Router::new().route("/fast", get(|| async { "ok" })).route(
            "/slow",
            get(move || {
                let slow_gate = slow_gate.clone();
                async move {
                    slow_gate.notified().await;
                    "ok"
                }
            }),
        ),
        HttpServerOptions::default(),
        control.clone(),
    ));
    let mut client = tokio::net::TcpStream::connect(addr).await.unwrap();
    client
        .write_all(b"GET /fast HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    let mut response = Vec::new();
    client.read_to_end(&mut response).await.unwrap();
    assert!(String::from_utf8_lossy(&response).contains("200"));
    let mut slow = tokio::net::TcpStream::connect(addr).await.unwrap();
    slow.write_all(b"GET /slow HTTP/1.1\r\nHost: localhost\r\nConnection: keep-alive\r\n\r\n")
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(30)).await;
    let _ = control
        .shutdown(
            ShutdownDeadline::from_now(Duration::from_millis(150), Duration::from_millis(30))
                .unwrap(),
            "test",
            "drain",
        )
        .await;
    let report = task.await.unwrap();
    assert!(report.accepted >= 2 && report.completed >= 1);
    assert!(report.timed_out >= 1);
    assert!(report.streams_admitted >= 2);
    assert!(report.streams_completed >= 1);
    assert!(report.streams_timed_out >= 1);
    assert_eq!(
        report.accepted,
        report.completed + report.failed + report.timed_out + report.unfinished
    );
    assert_eq!(
        report.streams_admitted,
        report.streams_completed
            + report.streams_timed_out
            + report.streams_ambiguous
            + report.streams_refused
    );
}
