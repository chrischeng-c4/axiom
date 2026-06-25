// HANDWRITE-BEGIN gap="missing-generator:e2e-test:1fc7e986" tracker="pending-tracker" reason="gRPC e2e: generated CloudTasks client → emulator → sink; REST coexists."
//! Integration test for the Cloud Tasks emulator's gRPC front-end. Spawns
//! `vat emulator cloud-tasks`, drives the GENERATED gRPC client over an insecure
//! channel to CreateQueue + CreateTask targeting a local sink, and asserts the
//! emulator POSTs to it — then confirms the REST surface still works on the same
//! port (shared store).
//!
//! @command cargo test -p vat --test vat_emulator_tasks_grpc -- --nocapture

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use std::sync::mpsc;
use std::time::{Duration, Instant};

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
async fn cloud_tasks_grpc_dispatches_task_and_rest_coexists() {
    let port = free_port();
    let addr = format!("127.0.0.1:{port}");
    let child = Command::new(vat_bin())
        .args(["emulator", "cloud-tasks", "--host-port", &addr])
        .spawn()
        .expect("spawn cloud-tasks emulator");
    let _guard = Killed(child);
    wait_for_port(&addr);

    let parent = "projects/demo-vat/locations/us-central1".to_string();
    let queue_name = format!("{parent}/queues/q");

    // gRPC client over an insecure (http) channel to the same host:port.
    let mut client = pb::cloud_tasks_client::CloudTasksClient::connect(format!("http://{addr}"))
        .await
        .expect("connect gRPC client");

    client
        .create_queue(pb::CreateQueueRequest {
            parent: parent.clone(),
            queue: Some(pb::Queue {
                name: queue_name.clone(),
                ..Default::default()
            }),
        })
        .await
        .expect("CreateQueue over gRPC");

    let (sink_port, rx) = spawn_sink();
    client
        .create_task(pb::CreateTaskRequest {
            parent: queue_name.clone(),
            task: Some(pb::Task {
                message_type: Some(pb::task::MessageType::HttpRequest(pb::HttpRequest {
                    url: format!("http://127.0.0.1:{sink_port}/work"),
                    http_method: pb::HttpMethod::Post as i32,
                    body: b"hello-grpc-task".to_vec(),
                    authorization_header: Some(pb::http_request::AuthorizationHeader::OidcToken(
                        pb::OidcToken {
                            service_account_email: "sa@demo-vat.iam.gserviceaccount.com"
                                .to_string(),
                            audience: String::new(),
                        },
                    )),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            response_view: 0,
        })
        .await
        .expect("CreateTask over gRPC");

    let got = rx
        .recv_timeout(Duration::from_secs(8))
        .expect("sink did not receive the gRPC-dispatched task");
    assert!(got.contains("POST /work"), "wrong request line: {got}");
    assert!(got.contains("hello-grpc-task"), "missing task body: {got}");
    assert!(
        got.to_lowercase().contains("authorization: bearer "),
        "missing OIDC bearer header: {got}"
    );

    // REST coexists on the same port: the gRPC-created queue is visible to REST
    // (shared store).
    let rest = reqwest::Client::new()
        .get(format!("http://{addr}/v2/{queue_name}"))
        .send()
        .await
        .expect("REST GET on the same port")
        .error_for_status()
        .expect("REST GET status");
    let body: serde_json::Value = rest.json().await.expect("REST json");
    assert_eq!(
        body.get("name").and_then(|v| v.as_str()),
        Some(queue_name.as_str()),
        "REST should see the gRPC-created queue: {body}"
    );
}
// HANDWRITE-END
