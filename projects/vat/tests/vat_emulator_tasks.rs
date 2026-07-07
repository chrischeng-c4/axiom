// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-tests-vat_emulator_tasks-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Self-contained integration test for the built-in Cloud Tasks emulator.
//! Spawns `vat emulator cloud-tasks`, starts a local sink, and asserts that
//! creating a task targeting the sink results in the emulator POSTing to it.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use base64::Engine;
use serde_json::json;

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
/// @spec projects/vat/tech-design/semantic/source/projects-vat-tests-vat_emulator_tasks-rs.md#source
impl Drop for Killed {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// A one-shot HTTP sink: accept one connection, read the request, reply 200, and
/// send the request bytes back over a channel. Returns (port, receiver).
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

#[tokio::test]
async fn cloud_tasks_emulator_dispatches_task() {
    let port = free_port();
    let addr = format!("127.0.0.1:{port}");
    let child = Command::new(vat_bin())
        .args(["emulator", "cloud-tasks", "--host-port", &addr])
        .spawn()
        .expect("spawn cloud-tasks emulator");
    let _guard = Killed(child);
    wait_for_port(&addr);

    let base = format!("http://{addr}/v2");
    let parent = "projects/demo-vat/locations/us-central1";
    let client = reqwest::Client::new();

    // Create a queue.
    client
        .post(format!("{base}/{parent}/queues"))
        .json(&json!({ "name": format!("{parent}/queues/q") }))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // Create a task targeting the sink (immediate scheduleTime).
    let (sink_port, rx) = spawn_sink();
    let body = base64::engine::general_purpose::STANDARD.encode(b"hello-task");
    client
        .post(format!("{base}/{parent}/queues/q/tasks"))
        .json(&json!({
            "task": {
                "httpRequest": {
                    "url": format!("http://127.0.0.1:{sink_port}/work"),
                    "httpMethod": "POST",
                    "body": body,
                    "oidcToken": { "serviceAccountEmail": "sa@demo-vat.iam.gserviceaccount.com" }
                }
            }
        }))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let got = rx
        .recv_timeout(Duration::from_secs(8))
        .expect("sink did not receive the dispatched task");
    assert!(got.contains("POST /work"), "wrong request line: {got}");
    assert!(got.contains("hello-task"), "missing task body: {got}");
    assert!(
        got.to_lowercase().contains("authorization: bearer "),
        "missing OIDC bearer header: {got}"
    );
}
// CODEGEN-END
