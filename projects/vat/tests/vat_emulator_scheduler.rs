// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-tests-vat_emulator_scheduler-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Self-contained integration test for the built-in Cloud Scheduler emulator.
//! Spawns `vat emulator cloud-scheduler`, creates an httpTarget job, calls
//! `:run`, and asserts the emulator POSTs to the local sink. (Cron-tick firing
//! is not awaited — `:run` is deterministic.)

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
/// @spec projects/vat/tech-design/semantic/source/projects-vat-tests-vat_emulator_scheduler-rs.md#source
impl Drop for Killed {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

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
async fn cloud_scheduler_emulator_fires_job_on_run() {
    let port = free_port();
    let addr = format!("127.0.0.1:{port}");
    let child = Command::new(vat_bin())
        .args(["emulator", "cloud-scheduler", "--host-port", &addr])
        .spawn()
        .expect("spawn cloud-scheduler emulator");
    let _guard = Killed(child);
    wait_for_port(&addr);

    let base = format!("http://{addr}/v1");
    let parent = "projects/demo-vat/locations/us-central1";
    let client = reqwest::Client::new();

    let (sink_port, rx) = spawn_sink();
    let body = base64::engine::general_purpose::STANDARD.encode(b"tick-payload");
    client
        .post(format!("{base}/{parent}/jobs"))
        .json(&json!({
            "name": format!("{parent}/jobs/j"),
            "schedule": "0 0 1 1 *",
            "httpTarget": {
                "uri": format!("http://127.0.0.1:{sink_port}/cron"),
                "httpMethod": "POST",
                "body": body,
            }
        }))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // Force-run regardless of schedule.
    client
        .post(format!("{base}/{parent}/jobs/j:run"))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let got = rx
        .recv_timeout(Duration::from_secs(8))
        .expect("sink did not receive the fired job");
    assert!(got.contains("POST /cron"), "wrong request line: {got}");
    assert!(got.contains("tick-payload"), "missing job body: {got}");
}
// CODEGEN-END
