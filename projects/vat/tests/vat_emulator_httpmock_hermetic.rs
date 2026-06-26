// HANDWRITE-BEGIN gap="missing-generator:e2e-test:7761ba2f" tracker="pending-tracker" reason="--no-forward proxy: unmatched → 502 hermetic (no upstream), stub still served; default proxy still forwards."
//! Integration test for the http-mock hermetic (`--no-forward`) mode — the
//! full-hermetic sandbox piece. A `--no-forward` proxy blocks an unmatched
//! request (502, no upstream contacted) while a registered stub still serves; a
//! default proxy still forwards. The blocked request targets a REACHABLE local
//! sink, so a non-empty sink would mean a leak — proving the proxy actively
//! refuses to forward, not that the host merely failed to resolve.
//!
//! @command cargo test -p vat --test vat_emulator_httpmock_hermetic -- --nocapture

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use std::sync::mpsc;
use std::time::{Duration, Instant};

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

/// A local HTTP sink that replies `200 routed` and reports each request it got.
fn spawn_sink() -> (u16, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for conn in listener.incoming() {
            let Ok(mut stream) = conn else { continue };
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            let mut buf = [0u8; 8192];
            let n = stream.read(&mut buf).unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            let _ = stream.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\nContent-Type: text/plain\r\n\r\nrouted",
            );
            if tx.send(req).is_err() {
                break;
            }
        }
    });
    (port, rx)
}

/// Spawn the http-mock proxy; `no_forward` toggles hermetic mode.
fn spawn_proxy(no_forward: bool) -> (String, Killed) {
    let port = free_port();
    let addr = format!("127.0.0.1:{port}");
    let tmp = std::env::temp_dir().join(format!("vat-httpmock-hermetic-{port}"));
    std::fs::create_dir_all(&tmp).unwrap();
    let mut args: Vec<String> = vec![
        "emulator".into(),
        "http-mock".into(),
        "--host-port".into(),
        addr.clone(),
        "--ca-path".into(),
        tmp.join("ca.pem").to_string_lossy().into_owned(),
        "--cassette-dir".into(),
        tmp.join("cassettes").to_string_lossy().into_owned(),
    ];
    if no_forward {
        args.push("--no-forward".into());
    }
    let child = Command::new(vat_bin())
        .args(&args)
        .spawn()
        .expect("spawn http-mock proxy");
    wait_for_port(&addr);
    (addr, Killed(child))
}

fn proxy_client(proxy: &str) -> reqwest::Client {
    reqwest::Client::builder()
        .proxy(reqwest::Proxy::all(format!("http://{proxy}")).unwrap())
        .build()
        .unwrap()
}

#[tokio::test]
async fn no_forward_blocks_unmatched_but_serves_stub() {
    let (proxy, _guard) = spawn_proxy(true); // hermetic
    let (sink, rx) = spawn_sink();

    // Register a stub for stub.test — a local match must still be served.
    reqwest::Client::new()
        .post(format!("http://{proxy}/__admin/stubs"))
        .json(&serde_json::json!({
            "match": { "host": "stub.test" },
            "response": { "status": 200, "body": "stubbed" }
        }))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .expect("stub registration failed");

    let client = proxy_client(&proxy);

    // Matched (stub) still serves.
    let ok = client.get("http://stub.test/x").send().await.unwrap();
    assert_eq!(ok.status(), 200);
    assert_eq!(ok.text().await.unwrap(), "stubbed");

    // Unmatched → blocked. The target is a REACHABLE local sink, so if the proxy
    // forwarded, the sink would receive it. It must NOT.
    let blocked = client
        .get(format!("http://127.0.0.1:{sink}/leak"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        blocked.status(),
        502,
        "hermetic proxy should block unmatched"
    );
    let body = blocked.text().await.unwrap();
    assert!(
        body.contains("hermetic") && body.contains("forwarding disabled"),
        "expected hermetic block body, got: {body}"
    );
    assert!(
        rx.recv_timeout(Duration::from_millis(800)).is_err(),
        "sink received a request — the hermetic proxy leaked to the upstream!"
    );
}

#[tokio::test]
async fn default_proxy_forwards_unmatched() {
    let (proxy, _guard) = spawn_proxy(false); // default: forwarding on
    let (sink, rx) = spawn_sink();

    // Unmatched host addressed directly at the local sink: the default proxy
    // forwards it there (offline, deterministic).
    let resp = proxy_client(&proxy)
        .get(format!("http://127.0.0.1:{sink}/fwd"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.text().await.unwrap(), "routed");

    let got = rx
        .recv_timeout(Duration::from_secs(8))
        .expect("default proxy did not forward to the upstream sink");
    assert!(got.contains("GET /fwd"), "wrong request line: {got}");
}
// HANDWRITE-END
