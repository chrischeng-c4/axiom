//! Integration tests for the frame-level [`h2c::H2cManager`] against a real,
//! controllable local hyper server speaking HTTP/2 cleartext (h2c).

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use h2c::{H2cError, H2cManager, ManagerConfig};
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::{JoinHandle, JoinSet};

// ---------------------------------------------------------------------------
// A controllable h2c test server: serves a few routes, counts requests, and
// can be torn down completely (every in-flight connection aborted) so the
// reconnect path can be exercised deterministically.
// ---------------------------------------------------------------------------

async fn handle(
    req: Request<Incoming>,
    served: Arc<AtomicU64>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    served.fetch_add(1, Ordering::Relaxed);
    let body = match req.uri().path() {
        // Sleep so a burst of concurrent requests piles up in-flight streams.
        "/slow" => {
            tokio::time::sleep(Duration::from_millis(60)).await;
            Full::new(Bytes::from_static(b"ok"))
        }
        // Echo the request body back verbatim.
        "/echo" => {
            let bytes = req.into_body().collect().await.unwrap().to_bytes();
            Full::new(bytes)
        }
        _ => Full::new(Bytes::from_static(b"ok")),
    };
    Ok(Response::new(body))
}

struct TestServer {
    addr: SocketAddr,
    served: Arc<AtomicU64>,
    stop: Option<oneshot::Sender<()>>,
    handle: JoinHandle<()>,
}

impl TestServer {
    async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        Self::serve(listener, addr, Arc::new(AtomicU64::new(0))).await
    }

    /// Rebind a fresh server on the SAME address (for the reconnect test).
    async fn restart(addr: SocketAddr, served: Arc<AtomicU64>) -> Self {
        // A listening socket frees its port on close, but retry briefly in case
        // the OS hasn't released it yet.
        let listener = loop {
            match TcpListener::bind(addr).await {
                Ok(l) => break l,
                Err(_) => tokio::time::sleep(Duration::from_millis(20)).await,
            }
        };
        Self::serve(listener, addr, served).await
    }

    async fn serve(listener: TcpListener, addr: SocketAddr, served: Arc<AtomicU64>) -> Self {
        let (stop_tx, mut stop_rx) = oneshot::channel();
        let served_loop = served.clone();
        let handle = tokio::spawn(async move {
            let mut conns = JoinSet::new();
            loop {
                tokio::select! {
                    _ = &mut stop_rx => break,
                    accept = listener.accept() => {
                        let Ok((stream, _)) = accept else { break };
                        let served = served_loop.clone();
                        conns.spawn(async move {
                            let io = TokioIo::new(stream);
                            let svc = service_fn(move |req| handle(req, served.clone()));
                            let _ = auto::Builder::new(TokioExecutor::new())
                                .serve_connection(io, svc)
                                .await;
                        });
                    }
                    Some(_) = conns.join_next() => {}
                }
            }
            // Dropping the JoinSet aborts every still-open connection task, so a
            // client sees its connection close.
            conns.abort_all();
        });
        TestServer {
            addr,
            served,
            stop: Some(stop_tx),
            handle,
        }
    }

    fn authority(&self) -> String {
        self.addr.to_string()
    }

    async fn stop(mut self) {
        if let Some(tx) = self.stop.take() {
            let _ = tx.send(());
        }
        let _ = self.handle.await;
    }
}

/// A manager config with short, test-friendly timeouts.
fn cfg(min: usize, max: usize, grow_threshold: usize) -> ManagerConfig {
    ManagerConfig {
        min_connections: min,
        max_connections: max,
        grow_threshold,
        connect_timeout: Duration::from_secs(1),
        request_timeout: Some(Duration::from_secs(2)),
        ping_interval: Duration::from_millis(150),
        idle_timeout: Duration::from_millis(200),
        ..ManagerConfig::default()
    }
}

// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_and_put_roundtrip() {
    let server = TestServer::start().await;
    let mgr = H2cManager::with_config(&server.authority(), cfg(1, 4, 8))
        .await
        .unwrap();

    let resp = mgr.get("/healthz").await.unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.body().as_ref(), b"ok");

    // PUT body is echoed back verbatim (exercises flow-controlled body send).
    let blob = Bytes::from(vec![7u8; 50_000]);
    let resp = mgr.put("/echo", blob.clone()).await.unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.body(), &blob);

    mgr.shutdown().await;
    server.stop().await;
}

#[tokio::test]
async fn concurrency_grows_the_pool_least_loaded() {
    let server = TestServer::start().await;
    // min 1, grow at 8 in-flight, cap 4.
    let mgr = H2cManager::with_config(&server.authority(), cfg(1, 4, 8))
        .await
        .unwrap();

    // 64 concurrent slow requests force the single warm connection past the
    // grow threshold, so the manager opens more (up to the cap).
    let mut tasks = JoinSet::new();
    for _ in 0..64 {
        let mgr = mgr.clone();
        tasks.spawn(async move { mgr.get("/slow").await.map(|r| r.status().as_u16()) });
    }
    let mut ok = 0;
    while let Some(res) = tasks.join_next().await {
        if res.unwrap().unwrap() == 200 {
            ok += 1;
        }
    }
    assert_eq!(ok, 64, "all concurrent requests succeed");

    let stats = mgr.stats().await;
    assert!(
        stats.connections >= 2 && stats.connections <= 4,
        "pool grew under load within the cap, got {}",
        stats.connections
    );
    assert!(stats.total_requests >= 64);
    assert_eq!(stats.total_errors, 0);

    mgr.shutdown().await;
    server.stop().await;
}

#[tokio::test]
async fn reconnects_after_connections_die() {
    let server = TestServer::start().await;
    let addr = server.addr;
    let served = server.served.clone();
    let mgr = H2cManager::with_config(&server.authority(), cfg(1, 2, 8))
        .await
        .unwrap();

    assert_eq!(mgr.get("/healthz").await.unwrap().status(), 200);

    // Tear the server down completely — every open connection is aborted, so
    // the manager's connection goes dead. Then bring a fresh server up on the
    // same port BEFORE the next request (no connectionless window).
    server.stop().await;
    let server2 = TestServer::restart(addr, served).await;

    // The manager detects the dead connection (send error → evict) and retries
    // on a freshly-grown connection to the new server.
    let resp = mgr.get("/healthz").await.unwrap();
    assert_eq!(resp.status(), 200);
    assert!(mgr.stats().await.healthy >= 1);

    mgr.shutdown().await;
    server2.stop().await;
}

#[tokio::test]
async fn shutdown_refuses_new_requests() {
    let server = TestServer::start().await;
    let mgr = H2cManager::with_config(&server.authority(), cfg(1, 2, 8))
        .await
        .unwrap();
    assert_eq!(mgr.get("/healthz").await.unwrap().status(), 200);

    mgr.shutdown().await;
    match mgr.get("/healthz").await {
        Err(H2cError::Shutdown) => {}
        other => panic!("expected Shutdown, got {other:?}"),
    }
    server.stop().await;
}

#[tokio::test]
async fn connect_failure_is_reported() {
    // Nothing is listening on this port → connect fails fast.
    let err = H2cManager::with_config("127.0.0.1:1", cfg(1, 2, 8))
        .await
        .unwrap_err();
    assert!(err.is_connection_lost(), "got {err:?}");
}
