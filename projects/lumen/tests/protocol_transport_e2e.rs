// SPEC-MANAGED: projects/lumen/external-contracts/cli-interface/behavior/cli-interface.md#lumen-cli-interface-protocol-transport
// CODEGEN-BEGIN
// @contract service-listener-http1-and-h2c
//! Lumen server transport contract: the service entrypoint accepts HTTP/1.1
//! and h2c prior-knowledge HTTP/2 on the same socket.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;

struct LumenTransportServer {
    base_url: String,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
    handle: tokio::task::JoinHandle<()>,
}

impl LumenTransportServer {
    async fn start() -> Self {
        let engine = Arc::new(lumen::storage::Engine::new());
        let app = lumen::api::router(lumen::api::AppState::open(engine));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test lumen transport server");
        let addr: SocketAddr = listener.local_addr().expect("local addr");
        let (tx, rx) = tokio::sync::oneshot::channel();
        let handle = tokio::spawn(async move {
            service_http::serve(listener, app, async {
                let _ = rx.await;
            })
            .await;
        });
        let server = Self {
            base_url: format!("http://{addr}"),
            shutdown: Some(tx),
            handle,
        };
        server.wait_ready().await;
        server
    }

    async fn wait_ready(&self) {
        let client = reqwest::Client::new();
        let url = format!("{}/collections", self.base_url);
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        loop {
            if let Ok(resp) = client.get(&url).send().await {
                if resp.status().is_success() {
                    return;
                }
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "lumen transport server did not become ready at {url}"
            );
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        let _ = tokio::time::timeout(Duration::from_secs(6), self.handle).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn lumen_server_accepts_http1_and_h2c_clients_on_one_socket() {
    let server = LumenTransportServer::start().await;
    let url = format!("{}/collections", server.base_url);

    let http1 = reqwest::Client::builder()
        .http1_only()
        .build()
        .expect("http1 client");
    let http1_resp = http1
        .get(&url)
        .send()
        .await
        .expect("http1 GET /collections");
    assert_eq!(http1_resp.version(), reqwest::Version::HTTP_11);
    assert!(http1_resp.status().is_success());
    let http1_body: Value = http1_resp.json().await.expect("http1 json");
    assert_eq!(http1_body, serde_json::json!([]));

    let h2c = h2c::h2c_client().expect("h2c prior-knowledge client");
    let h2_resp = h2c.get(&url).send().await.expect("h2c GET /collections");
    assert_eq!(h2_resp.version(), reqwest::Version::HTTP_2);
    assert!(h2_resp.status().is_success());
    let h2_body: Value = h2_resp.json().await.expect("h2c json");
    assert_eq!(h2_body, serde_json::json!([]));

    server.shutdown().await;
}
// CODEGEN-END
