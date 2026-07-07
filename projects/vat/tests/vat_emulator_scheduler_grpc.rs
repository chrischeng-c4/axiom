// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-tests-vat_emulator_scheduler_grpc-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Integration test for the Cloud Scheduler emulator's gRPC front-end. Spawns
//! `vat emulator cloud-scheduler`, drives the GENERATED gRPC client to
//! CreateJob + RunJob with an httpTarget pointing at a local sink, and asserts
//! the emulator POSTs to it.
//!
//! @command cargo test -p vat --test vat_emulator_scheduler_grpc -- --nocapture

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use vat::emulator::googleapis::google::cloud::scheduler::v1 as pb;

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
/// @spec projects/vat/tech-design/semantic/source/projects-vat-tests-vat_emulator_scheduler_grpc-rs.md#source
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
async fn cloud_scheduler_grpc_fires_job_on_run() {
    let port = free_port();
    let addr = format!("127.0.0.1:{port}");
    let child = Command::new(vat_bin())
        .args(["emulator", "cloud-scheduler", "--host-port", &addr])
        .spawn()
        .expect("spawn cloud-scheduler emulator");
    let _guard = Killed(child);
    wait_for_port(&addr);

    let parent = "projects/demo-vat/locations/us-central1".to_string();
    let job_name = format!("{parent}/jobs/j");

    let mut client =
        pb::cloud_scheduler_client::CloudSchedulerClient::connect(format!("http://{addr}"))
            .await
            .expect("connect gRPC client");

    let (sink_port, rx) = spawn_sink();

    client
        .create_job(pb::CreateJobRequest {
            parent: parent.clone(),
            job: Some(pb::Job {
                name: job_name.clone(),
                schedule: "0 0 1 1 *".to_string(),
                target: Some(pb::job::Target::HttpTarget(pb::HttpTarget {
                    uri: format!("http://127.0.0.1:{sink_port}/fire"),
                    http_method: pb::HttpMethod::Post as i32,
                    body: b"hello-grpc-job".to_vec(),
                    ..Default::default()
                })),
                ..Default::default()
            }),
        })
        .await
        .expect("CreateJob over gRPC");

    // Force-fire now, regardless of schedule.
    client
        .run_job(pb::RunJobRequest {
            name: job_name.clone(),
        })
        .await
        .expect("RunJob over gRPC");

    let got = rx
        .recv_timeout(Duration::from_secs(8))
        .expect("sink did not receive the gRPC-fired job");
    assert!(got.contains("POST /fire"), "wrong request line: {got}");
    assert!(got.contains("hello-grpc-job"), "missing job body: {got}");
}
// CODEGEN-END
