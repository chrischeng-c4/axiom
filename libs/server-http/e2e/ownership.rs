use axum::{routing::get, Router};
use server_http::{serve_h2c_with_lifecycle, HttpServerOptions};
use server_lifecycle::{ConnectionMetrics, LifecycleController, ShutdownDeadline};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Default)]
struct Metrics {
    accepted: AtomicUsize,
    closed: AtomicUsize,
}
impl ConnectionMetrics for Metrics {
    fn connection_accepted(&self) {
        self.accepted.fetch_add(1, Ordering::SeqCst);
    }
    fn connection_closed(&self) {
        self.closed.fetch_add(1, Ordering::SeqCst);
    }
}

#[tokio::test]
async fn ownership_stays_in_tcp_and_transport_layers() {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let lifecycle = LifecycleController::serving();
    let control = lifecycle.clone();
    let metrics = Arc::new(Metrics::default());
    let options = HttpServerOptions {
        connection_metrics: metrics.clone(),
        ..Default::default()
    };
    let task = tokio::spawn(serve_h2c_with_lifecycle(
        listener,
        Router::new().route("/", get(|| async { "ok" })),
        options,
        lifecycle,
    ));
    let mut client = tokio::net::TcpStream::connect(addr).await.unwrap();
    client
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    let mut response = Vec::new();
    client.read_to_end(&mut response).await.unwrap();
    assert!(String::from_utf8_lossy(&response).contains("200"));
    let _ = control
        .shutdown(
            ShutdownDeadline::from_now(Duration::from_secs(1), Duration::ZERO).unwrap(),
            "test",
            "drain",
        )
        .await;
    let report = task.await.unwrap();
    assert_eq!(metrics.accepted.load(Ordering::SeqCst), 1);
    assert_eq!(metrics.closed.load(Ordering::SeqCst), 1);
    assert!(report.streams_admitted >= 1);
}
