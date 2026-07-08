// SPEC-MANAGED: apps/tape/tech-design/semantic/source/apps-tape-tests-tape-vs-nats-jetstream-rs.md#unit-test
// <HANDWRITE gap="missing-generator:test:tape-competitor-performance" tracker="#768" reason="Real NATS JetStream competitor benchmark before generated efficiency runners exist.">
use std::fs::File;
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use async_nats::jetstream::consumer::{push, DeliverPolicy, ReplayPolicy};
use futures::StreamExt;
use serde_json::json;
use tape::TapeJournal;
use tempfile::TempDir;

const EVENTS: usize = 20_000;
const PAYLOAD_BYTES: usize = 128;
const REQUIRED_REPLAY_RATIO: f64 = 1.5;

struct NatsServer {
    child: Child,
    _store: TempDir,
}

impl Drop for NatsServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tape_beats_nats_jetstream_on_local_backlog_replay() {
    let (server, url) = spawn_nats_server().await;
    let payload = payload(PAYLOAD_BYTES);
    let tape_replay_us = tape_replay_us(EVENTS, payload.clone());
    let nats_replay_us = nats_jetstream_replay_us(&url, EVENTS, payload.to_string()).await;
    let report = tape::bench::external_replay_win(
        "NATS JetStream stream",
        "local_backlog_full_replay",
        EVENTS,
        PAYLOAD_BYTES,
        tape_replay_us,
        nats_replay_us,
        REQUIRED_REPLAY_RATIO,
        "apps/tape/tests/tape_vs_nats_jetstream.rs starts real nats-server -js and compares JetStream replay to Tape zero-copy replay_refs",
    );

    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    tape::bench::verify_external_replay_win(&report).expect("Tape beats NATS JetStream replay");
    drop(server);
}

fn tape_replay_us(events: usize, payload: serde_json::Value) -> u128 {
    let mut journal = TapeJournal::default();
    for i in 0..events {
        journal.append(
            "orders.created",
            Some(format!("orders.created.{i}")),
            payload.clone(),
            Some(i as u64),
        );
    }
    let started = Instant::now();
    let replayed = journal.replay_refs("orders.created", Some(0), None, Some(events));
    assert_eq!(replayed.len(), events);
    started.elapsed().as_micros().max(1)
}

async fn nats_jetstream_replay_us(url: &str, events: usize, payload: String) -> u128 {
    let client = async_nats::connect(url).await.expect("connect NATS");
    let jetstream = async_nats::jetstream::new(client);
    let stream = jetstream
        .create_stream(async_nats::jetstream::stream::Config {
            name: "TAPE_BENCH".into(),
            subjects: vec!["tape.bench".into()],
            ..Default::default()
        })
        .await
        .expect("create JetStream stream");

    for _ in 0..events {
        jetstream
            .publish("tape.bench", payload.clone().into())
            .await
            .expect("publish to JetStream")
            .await
            .expect("ack JetStream publish");
    }

    let consumer = stream
        .create_consumer(push::OrderedConfig {
            deliver_subject: "tape.bench.deliver".to_string(),
            deliver_policy: DeliverPolicy::All,
            replay_policy: ReplayPolicy::Instant,
            ..Default::default()
        })
        .await
        .expect("create ordered replay consumer");
    let mut messages = consumer.messages().await.expect("open replay messages");
    let started = Instant::now();
    for _ in 0..events {
        let message = messages
            .next()
            .await
            .expect("JetStream replay message")
            .expect("valid JetStream replay message");
        assert_eq!(message.payload.len(), payload.len());
    }
    started.elapsed().as_micros().max(1)
}

async fn spawn_nats_server() -> (NatsServer, String) {
    let mut ports = Vec::new();
    if let Some(port) = free_port() {
        ports.push(port);
    }
    ports.extend(54_222..54_242);

    for port in ports {
        let store = tempfile::tempdir().expect("create NATS store dir");
        let port_string = port.to_string();
        let log_path = store.path().join("nats-server.log");
        let log = File::create(&log_path).expect("create NATS log");
        let child = Command::new("nats-server")
            .args([
                "-js",
                "--addr",
                "127.0.0.1",
                "--port",
                &port_string,
                "--store_dir",
                store.path().to_str().expect("utf-8 temp dir"),
            ])
            .stdout(Stdio::from(log.try_clone().expect("clone NATS log")))
            .stderr(Stdio::from(log))
            .spawn()
            .expect("spawn nats-server; install with `brew install nats-server`");
        let mut server = NatsServer {
            child,
            _store: store,
        };
        let url = format!("nats://127.0.0.1:{port}");
        for _ in 0..100 {
            if async_nats::connect(&url).await.is_ok() {
                return (server, url);
            }
            if matches!(server.child.try_wait(), Ok(Some(_))) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        let log = std::fs::read_to_string(&log_path).unwrap_or_default();
        eprintln!("nats-server candidate port {port} failed:\n{log}");
        let _ = server.child.kill();
        let _ = server.child.wait();
    }
    panic!("nats-server did not become reachable on candidate local ports");
}

fn free_port() -> Option<u16> {
    TcpListener::bind("127.0.0.1:0")
        .ok()
        .and_then(|listener| listener.local_addr().ok())
        .map(|addr| addr.port())
}

fn payload(bytes: usize) -> serde_json::Value {
    json!({
        "id": "bench",
        "body": "x".repeat(bytes),
    })
}
// </HANDWRITE>
