use axum::{routing::get, Router};
use bytes::Bytes;
use http_body_util::{BodyExt, Empty, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto,
};
use server_lifecycle::{LifecycleController, LifecyclePhase, ShutdownDeadline};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Notify;
use transport_h2c::{
    serve_connection_with_drain, serve_connection_with_lifecycle, ConnectionOptions,
    ConnectionProtocol, ConnectionTerminal, H2cManager, ManagerConfig,
};

#[derive(Clone)]
struct Gate {
    started: Arc<Notify>,
    release: Arc<Notify>,
}

async fn http1_drain(
    deadline: ShutdownDeadline,
    release: bool,
) -> (Vec<u8>, transport_h2c::ConnectionReport) {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let address = listener.local_addr().unwrap();
    let lifecycle = LifecycleController::serving();
    let gate = Gate {
        started: Arc::new(Notify::new()),
        release: Arc::new(Notify::new()),
    };
    let app_gate = gate.clone();
    let app = Router::new().route(
        "/slow",
        get(move || {
            let gate = app_gate.clone();
            async move {
                gate.started.notify_one();
                gate.release.notified().await;
                "ok"
            }
        }),
    );
    let subscription = lifecycle.subscribe();
    let task = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        serve_connection_with_drain(
            stream,
            app,
            ConnectionOptions::default(),
            subscription,
            deadline,
        )
        .await
    });
    let mut client = tokio::net::TcpStream::connect(address).await.unwrap();
    client
        .write_all(b"GET /slow HTTP/1.1\r\nHost: localhost\r\nConnection: keep-alive\r\n\r\n")
        .await
        .unwrap();
    gate.started.notified().await;
    lifecycle
        .transition(LifecyclePhase::Draining, "test", "drain")
        .unwrap();
    if release {
        gate.release.notify_one();
    }
    let mut response = Vec::new();
    client.read_to_end(&mut response).await.unwrap();
    (response, task.await.unwrap())
}

#[tokio::test]
async fn drain_api() {
    let (response, report) = http1_drain(
        ShutdownDeadline::from_now(Duration::from_secs(2), Duration::ZERO).unwrap(),
        true,
    )
    .await;
    assert!(response.starts_with(b"HTTP/1.1 200"));
    assert!(String::from_utf8_lossy(&response)
        .to_ascii_lowercase()
        .contains("connection: close"));
    assert_eq!(report.protocol, ConnectionProtocol::Http1);
    assert_eq!(report.terminal, ConnectionTerminal::Drained);
    assert_eq!(report.active_at_drain, 1);
    assert_eq!(report.completed, 1);
}

#[tokio::test]
async fn http1_close() {
    let (response, report) = http1_drain(
        ShutdownDeadline::from_now(Duration::from_secs(2), Duration::ZERO).unwrap(),
        true,
    )
    .await;
    assert!(String::from_utf8_lossy(&response)
        .to_ascii_lowercase()
        .contains("connection: close"));
    assert_eq!(report.terminal, ConnectionTerminal::Drained);
}

#[tokio::test]
async fn http2_goaway() {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let address = listener.local_addr().unwrap();
    let lifecycle = LifecycleController::serving();
    let gate = Gate {
        started: Arc::new(Notify::new()),
        release: Arc::new(Notify::new()),
    };
    let invoked = Arc::new(AtomicUsize::new(0));
    let app_invoked = invoked.clone();
    let app_gate = gate.clone();
    let fast_invoked = invoked.clone();
    let app = Router::new()
        .route(
            "/slow",
            get(move || {
                let gate = app_gate.clone();
                async move {
                    app_invoked.fetch_add(1, Ordering::SeqCst);
                    gate.started.notify_one();
                    gate.release.notified().await;
                    "ok"
                }
            }),
        )
        .route(
            "/fast",
            get(move || {
                let fast_invoked = fast_invoked.clone();
                async move {
                    fast_invoked.fetch_add(1, Ordering::SeqCst);
                    "fast"
                }
            }),
        );
    let subscription = lifecycle.subscribe();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        serve_connection_with_drain(
            stream,
            app,
            ConnectionOptions::default(),
            subscription,
            ShutdownDeadline::from_now(Duration::from_secs(2), Duration::ZERO).unwrap(),
        )
        .await
    });
    let stream = tokio::net::TcpStream::connect(address).await.unwrap();
    let (mut sender, connection) =
        hyper::client::conn::http2::handshake(TokioExecutor::new(), TokioIo::new(stream))
            .await
            .unwrap();
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let request = Request::builder()
        .version(http::Version::HTTP_2)
        .uri("http://localhost/slow")
        .body(Empty::<Bytes>::new())
        .unwrap();
    let mut first_sender = sender.clone();
    let response = tokio::spawn(async move { first_sender.send_request(request).await });
    gate.started.notified().await;
    lifecycle
        .transition(LifecyclePhase::Draining, "test", "drain")
        .unwrap();
    let mut second_sender = sender.clone();
    let second_request = Request::builder()
        .version(http::Version::HTTP_2)
        .uri("http://localhost/fast")
        .body(Empty::<Bytes>::new())
        .unwrap();
    let second = tokio::time::timeout(
        Duration::from_secs(1),
        second_sender.send_request(second_request),
    )
    .await
    .expect("post-drain request bounded")
    .map(|response| response.status());
    if let Ok(status) = second {
        assert_eq!(status, http::StatusCode::SERVICE_UNAVAILABLE);
    }
    assert_eq!(invoked.load(Ordering::SeqCst), 1);
    gate.release.notify_one();
    assert_eq!(
        response.await.unwrap().unwrap().status(),
        http::StatusCode::OK
    );
    let report = server.await.unwrap();
    assert_eq!(report.protocol, ConnectionProtocol::Http2);
    assert_eq!(report.terminal, ConnectionTerminal::Drained);
    assert_eq!(report.admitted, 1);
    assert_eq!(report.completed, 1);
    assert_eq!(invoked.load(Ordering::SeqCst), 1);
    let mut goaway = false;
    let wait_started = Instant::now();
    while wait_started.elapsed() < Duration::from_secs(1) {
        match sender.ready().await {
            Err(_) => {
                goaway = true;
                break;
            }
            Ok(_) => tokio::time::sleep(Duration::from_millis(10)).await,
        }
    }
    if !goaway {
        let mut third_sender = sender.clone();
        let third_request = Request::builder()
            .version(http::Version::HTTP_2)
            .uri("http://localhost/fast")
            .body(Empty::<Bytes>::new())
            .unwrap();
        let third = tokio::time::timeout(
            Duration::from_secs(1),
            third_sender.send_request(third_request),
        )
        .await
        .expect("post-terminal request bounded");
        match third {
            Err(_) => goaway = true,
            Ok(response) => assert_eq!(response.status(), http::StatusCode::SERVICE_UNAVAILABLE),
        }
        assert_eq!(invoked.load(Ordering::SeqCst), 1);
    }
    assert!(goaway, "h2 client remained usable after terminal drain");
}

#[tokio::test]
async fn deadline_report() {
    let started = Instant::now();
    let (_, report) = http1_drain(
        ShutdownDeadline::from_now(Duration::from_millis(150), Duration::from_millis(25)).unwrap(),
        false,
    )
    .await;
    assert!(started.elapsed() < Duration::from_secs(1));
    assert_eq!(report.terminal, ConnectionTerminal::DeadlineExceeded);
    assert_eq!(report.timed_out, 1);
}

#[tokio::test]
async fn mutation_ambiguity() {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let authority = listener.local_addr().unwrap();
    let started = Arc::new(Notify::new());
    let invoked = Arc::new(AtomicUsize::new(0));
    let task_started = started.clone();
    let task_invoked = invoked.clone();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);
        let svc = service_fn(move |_req: Request<Incoming>| {
            let started = task_started.clone();
            let invoked = task_invoked.clone();
            async move {
                invoked.fetch_add(1, Ordering::SeqCst);
                started.notify_one();
                std::future::pending::<Result<Response<Full<Bytes>>, std::convert::Infallible>>()
                    .await
            }
        });
        auto::Builder::new(TokioExecutor::new())
            .serve_connection_with_upgrades(io, svc)
            .await
    });
    let mut config = ManagerConfig::default();
    config.request_timeout = Some(Duration::from_secs(2));
    let manager = H2cManager::with_config(&authority.to_string(), config)
        .await
        .unwrap();
    let request = tokio::spawn({
        let manager = manager.clone();
        async move { manager.post("/mutate", Bytes::from_static(b"x")).await }
    });
    started.notified().await;
    server.abort();
    let error = request.await.unwrap().unwrap_err();
    assert!(error.is_ambiguous());
    assert_eq!(invoked.load(Ordering::SeqCst), 1);

    let refused_listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let refused_authority = refused_listener.local_addr().unwrap();
    let refused_server = tokio::spawn(async move {
        let (stream, _) = refused_listener.accept().await.unwrap();
        let mut connection = h2::server::handshake(stream).await.unwrap();
        if let Some(Ok((_request, mut respond))) = connection.accept().await {
            respond.send_reset(h2::Reason::REFUSED_STREAM);
            let _ = tokio::time::timeout(Duration::from_secs(1), connection.accept()).await;
        }
    });
    let refused_manager =
        H2cManager::with_config(&refused_authority.to_string(), ManagerConfig::default())
            .await
            .unwrap();
    let refused = refused_manager
        .post("/refused", Bytes::new())
        .await
        .unwrap_err();
    assert!(
        refused.is_refused(),
        "expected REFUSED_STREAM, got {refused:?}"
    );
    refused_server.await.unwrap();
}

#[tokio::test]
async fn http1_lifecycle_deadline() {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let address = listener.local_addr().unwrap();
    let lifecycle = LifecycleController::serving();
    let app = Router::new().route("/ready", get(|| async { "ready" }));
    let subscription = lifecycle.subscribe();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        serve_connection_with_lifecycle(stream, app, ConnectionOptions::default(), subscription)
            .await
    });
    let mut client = tokio::net::TcpStream::connect(address).await.unwrap();
    client
        .write_all(b"GET /ready HTTP/1.1\r\nHost: localhost\r\nConnection: keep-alive\r\n\r\n")
        .await
        .unwrap();
    let mut response = [0u8; 256];
    let bytes = client.read(&mut response).await.unwrap();
    assert!(String::from_utf8_lossy(&response[..bytes]).contains("200 OK"));
    let deadline = ShutdownDeadline::from_now(Duration::from_secs(2), Duration::ZERO).unwrap();
    let report = lifecycle
        .shutdown(deadline, "test", "lifecycle deadline")
        .await;
    assert_eq!(report.initiating_reason_code, "test");
    let report = tokio::time::timeout(Duration::from_secs(1), server)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(report.protocol, ConnectionProtocol::Http1);
    assert_eq!(report.terminal, ConnectionTerminal::Drained);
    assert_eq!(report.admitted, 1);
}

#[tokio::test]
async fn h2c_lifecycle_deadline() {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let address = listener.local_addr().unwrap();
    let lifecycle = LifecycleController::serving();
    let app = Router::new().route("/ready", get(|| async { "ready" }));
    let subscription = lifecycle.subscribe();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        serve_connection_with_lifecycle(stream, app, ConnectionOptions::default(), subscription)
            .await
    });
    let stream = tokio::net::TcpStream::connect(address).await.unwrap();
    let (mut sender, connection) =
        hyper::client::conn::http2::handshake(TokioExecutor::new(), TokioIo::new(stream))
            .await
            .unwrap();
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let request = Request::builder()
        .version(http::Version::HTTP_2)
        .uri("http://localhost/ready")
        .body(Empty::<Bytes>::new())
        .unwrap();
    let response = sender.send_request(request).await.unwrap();
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(
        response.into_body().collect().await.unwrap().to_bytes(),
        Bytes::from_static(b"ready")
    );
    let deadline = ShutdownDeadline::from_now(Duration::from_secs(2), Duration::ZERO).unwrap();
    let report = lifecycle
        .shutdown(deadline, "test", "lifecycle deadline")
        .await;
    assert_eq!(report.initiating_reason_code, "test");
    let report = tokio::time::timeout(Duration::from_secs(1), server)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(report.protocol, ConnectionProtocol::Http2);
    assert_eq!(report.terminal, ConnectionTerminal::Drained);
    assert_eq!(report.admitted, 1);
}

#[tokio::test]
async fn lifecycle_deadline_missing_fails_boundedly() {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let address = listener.local_addr().unwrap();
    let lifecycle = LifecycleController::serving();
    let subscription = lifecycle.subscribe();
    lifecycle
        .transition(LifecyclePhase::Draining, "manual", "missing deadline")
        .unwrap();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        serve_connection_with_lifecycle(
            stream,
            Router::new(),
            ConnectionOptions::default(),
            subscription,
        )
        .await
    });
    let _client = tokio::net::TcpStream::connect(address).await.unwrap();
    let report = tokio::time::timeout(Duration::from_secs(1), server)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(report.terminal, ConnectionTerminal::Failed);
    assert_eq!(
        report.error.as_deref(),
        Some("lifecycle entered draining without shutdown deadline")
    );
}
