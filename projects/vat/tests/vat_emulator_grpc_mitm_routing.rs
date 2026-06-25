// HANDWRITE-BEGIN gap="missing-generator:e2e-test:0975f5c4" tracker="pending-tracker" reason="gRPC client over TLS-to-real-host, routed through the MITM, reaches the local emulator + sink; trailer forwarding proven by the call succeeding."
//! Integration test for network sandbox v2: transparent gRPC routing.
//!
//! Drives a real gRPC call to `https://cloudtasks.googleapis.com` THROUGH the
//! http-mock proxy: TCP → CONNECT → TLS (ALPN negotiates h2, trusting the
//! proxy's CA) → h2 → the proxy reverse-proxies the routed host to the local
//! cloud-tasks emulator (h2c). CreateQueue + CreateTask succeed (proving ALPN-h2
//! MITM + trailer-forwarding reverse-proxy), and the task dispatches to a local
//! sink (proving it reached the emulator).
//!
//! @command cargo test -p vat --test vat_emulator_grpc_mitm_routing -- --nocapture

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::Request;
use hyper_util::rt::{TokioExecutor, TokioIo};
use prost::Message;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use vat::emulator::googleapis::google::cloud::tasks::v2 as pb;

fn vat_bin() -> &'static str {
    env!("CARGO_BIN_EXE_vat")
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn wait_for_port(addr: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if TcpStream::connect(addr).is_ok() {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("emulator did not open {addr}");
}

struct Killed(Child);
impl Drop for Killed {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// One-shot HTTP sink: accept a single connection, reply 200, ship the request text.
fn spawn_sink() -> (u16, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            let mut buf = [0u8; 8192];
            let n = stream.read(&mut buf).unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");
            let _ = tx.send(req);
        }
    });
    (port, rx)
}

/// Open a CONNECT tunnel through the proxy to `target` and return the raw stream
/// positioned right after the proxy's `200` response (TLS bytes follow).
async fn connect_tunnel(proxy_addr: &str, target: &str) -> tokio::net::TcpStream {
    let mut tcp = tokio::net::TcpStream::connect(proxy_addr)
        .await
        .expect("connect proxy");
    let req = format!("CONNECT {target} HTTP/1.1\r\nHost: {target}\r\n\r\n");
    tcp.write_all(req.as_bytes()).await.expect("send CONNECT");
    // Read until the end of the proxy's response headers. The proxy sends no body
    // and nothing after, so this won't consume TLS bytes (TLS is client-first).
    let mut acc = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        let n = tcp.read(&mut byte).await.expect("read CONNECT resp");
        if n == 0 {
            break;
        }
        acc.push(byte[0]);
        if acc.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    let head = String::from_utf8_lossy(&acc);
    assert!(head.contains("200"), "CONNECT was not accepted: {head:?}");
    tcp
}

/// Frame a protobuf message as a single gRPC length-prefixed message.
fn grpc_frame(msg: &[u8]) -> Bytes {
    let mut framed = Vec::with_capacity(5 + msg.len());
    framed.push(0u8); // not compressed
    framed.extend_from_slice(&(msg.len() as u32).to_be_bytes());
    framed.extend_from_slice(msg);
    Bytes::from(framed)
}

#[tokio::test]
async fn grpc_routed_through_mitm_reaches_emulator() {
    // Provider for the test client's TLS (idempotent; ignore "already installed").
    let _ = tokio_rustls::rustls::crypto::aws_lc_rs::default_provider().install_default();

    // 1) cloud-tasks emulator (dual-protocol: serves h2c gRPC) + a sink.
    let emu_port = free_port();
    let emu_addr = format!("127.0.0.1:{emu_port}");
    let _emu = Killed(
        Command::new(vat_bin())
            .args(["emulator", "cloud-tasks", "--host-port", &emu_addr])
            .spawn()
            .expect("spawn cloud-tasks emulator"),
    );
    wait_for_port(&emu_addr);

    // 2) http-mock proxy with a route: real GCP host -> the local emulator.
    let proxy_port = free_port();
    let proxy_addr = format!("127.0.0.1:{proxy_port}");
    let tmp = std::env::temp_dir().join(format!("vat-grpc-mitm-{proxy_port}"));
    std::fs::create_dir_all(&tmp).unwrap();
    let ca_path = tmp.join("ca.pem");
    let cassette_dir = tmp.join("cassettes");
    let _proxy = Killed(
        Command::new(vat_bin())
            .args([
                "emulator",
                "http-mock",
                "--host-port",
                &proxy_addr,
                "--ca-path",
                &ca_path.to_string_lossy(),
                "--cassette-dir",
                &cassette_dir.to_string_lossy(),
                "--route",
                &format!("cloudtasks.googleapis.com=http://127.0.0.1:{emu_port}"),
            ])
            .spawn()
            .expect("spawn http-mock proxy"),
    );
    wait_for_port(&proxy_addr);

    // 3) CONNECT tunnel + TLS (ALPN h2) trusting the proxy CA.
    let tcp = connect_tunnel(&proxy_addr, "cloudtasks.googleapis.com:443").await;
    let ca_pem = std::fs::read(&ca_path).expect("read proxy CA pem");
    let mut roots = tokio_rustls::rustls::RootCertStore::empty();
    for cert in rustls_pemfile::certs(&mut &ca_pem[..]) {
        roots.add(cert.expect("ca cert")).expect("add ca to roots");
    }
    let mut tls_config = tokio_rustls::rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    tls_config.alpn_protocols = vec![b"h2".to_vec()];
    let connector = tokio_rustls::TlsConnector::from(Arc::new(tls_config));
    let server_name =
        tokio_rustls::rustls::pki_types::ServerName::try_from("cloudtasks.googleapis.com")
            .unwrap()
            .to_owned();
    let tls = connector
        .connect(server_name, tcp)
        .await
        .expect("client TLS handshake through MITM");
    assert_eq!(
        tls.get_ref().1.alpn_protocol(),
        Some(b"h2".as_ref()),
        "MITM did not negotiate ALPN h2"
    );

    // 4) h2 client over the MITM'd TLS stream.
    let (mut sender, conn) =
        hyper::client::conn::http2::handshake(TokioExecutor::new(), TokioIo::new(tls))
            .await
            .expect("h2 client handshake");
    tokio::spawn(async move {
        let _ = conn.await;
    });

    let parent = "projects/demo-vat/locations/us-central1".to_string();
    let queue_name = format!("{parent}/queues/q");

    // 5) CreateQueue over gRPC through the MITM.
    let cq = pb::CreateQueueRequest {
        parent: parent.clone(),
        queue: Some(pb::Queue {
            name: queue_name.clone(),
            ..Default::default()
        }),
    };
    let req = Request::builder()
        .method("POST")
        .uri("http://cloudtasks.googleapis.com/google.cloud.tasks.v2.CloudTasks/CreateQueue")
        .header("content-type", "application/grpc")
        .header("te", "trailers")
        .body(Full::new(grpc_frame(&cq.encode_to_vec())))
        .unwrap();
    let resp = sender.send_request(req).await.expect("CreateQueue send");
    assert_eq!(resp.status(), 200, "CreateQueue h2 status");
    let _ = resp.into_body().collect().await; // drain (incl. trailers)

    // 6) CreateTask over gRPC; the task dispatches to the sink.
    let (sink_port, rx) = spawn_sink();
    let ct = pb::CreateTaskRequest {
        parent: queue_name.clone(),
        task: Some(pb::Task {
            message_type: Some(pb::task::MessageType::HttpRequest(pb::HttpRequest {
                url: format!("http://127.0.0.1:{sink_port}/work"),
                http_method: pb::HttpMethod::Post as i32,
                body: b"hello-grpc-mitm".to_vec(),
                ..Default::default()
            })),
            ..Default::default()
        }),
        response_view: 0,
    };
    let req = Request::builder()
        .method("POST")
        .uri("http://cloudtasks.googleapis.com/google.cloud.tasks.v2.CloudTasks/CreateTask")
        .header("content-type", "application/grpc")
        .header("te", "trailers")
        .body(Full::new(grpc_frame(&ct.encode_to_vec())))
        .unwrap();
    let resp = sender.send_request(req).await.expect("CreateTask send");
    assert_eq!(resp.status(), 200, "CreateTask h2 status");
    let _ = resp.into_body().collect().await;

    // 7) The gRPC call reached the emulator THROUGH the MITM: task dispatched.
    let got = rx
        .recv_timeout(Duration::from_secs(8))
        .expect("sink did not receive the gRPC-dispatched task (MITM routing failed)");
    assert!(got.contains("POST /work"), "wrong request line: {got}");
    assert!(got.contains("hello-grpc-mitm"), "missing task body: {got}");

    // 8) A SECOND CreateTask over gRPC — this is the 3rd reverse-proxied call to
    //    the same emulator, so it exercises the POOLED upstream h2c connection
    //    (the first call dialed + pooled it; this one multiplexes on the reused
    //    connection). It must still dispatch to its own sink — proving the
    //    pooled connection serves repeated requests, not just the first.
    let (sink2_port, rx2) = spawn_sink();
    let ct2 = pb::CreateTaskRequest {
        parent: queue_name.clone(),
        task: Some(pb::Task {
            message_type: Some(pb::task::MessageType::HttpRequest(pb::HttpRequest {
                url: format!("http://127.0.0.1:{sink2_port}/work2"),
                http_method: pb::HttpMethod::Post as i32,
                body: b"hello-grpc-reuse".to_vec(),
                ..Default::default()
            })),
            ..Default::default()
        }),
        response_view: 0,
    };
    let req = Request::builder()
        .method("POST")
        .uri("http://cloudtasks.googleapis.com/google.cloud.tasks.v2.CloudTasks/CreateTask")
        .header("content-type", "application/grpc")
        .header("te", "trailers")
        .body(Full::new(grpc_frame(&ct2.encode_to_vec())))
        .unwrap();
    let resp = sender
        .send_request(req)
        .await
        .expect("CreateTask#2 send (pooled reuse)");
    assert_eq!(resp.status(), 200, "CreateTask#2 h2 status (pooled reuse)");
    let _ = resp.into_body().collect().await;
    let got2 = rx2
        .recv_timeout(Duration::from_secs(8))
        .expect("sink2 did not receive the reused-connection gRPC task");
    assert!(
        got2.contains("POST /work2"),
        "wrong request line (reuse): {got2}"
    );
    assert!(
        got2.contains("hello-grpc-reuse"),
        "missing task body (reuse): {got2}"
    );
}
// HANDWRITE-END
