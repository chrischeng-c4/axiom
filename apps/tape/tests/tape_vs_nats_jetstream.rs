// SPEC-MANAGED: apps/tape/tech-design/semantic/source/apps-tape-tests-tape-vs-nats-jetstream-rs.md#unit-test
// <HANDWRITE gap="missing-generator:test:tape-competitor-performance" tracker="#768" reason="Real NATS JetStream competitor benchmark before generated efficiency runners exist.">
use std::fs::File;
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use async_nats::jetstream::consumer::{push, DeliverPolicy, ReplayPolicy};
use futures::StreamExt;
use serde_json::json;
use tempfile::TempDir;

mod support;

const EVENTS: usize = 20_000;
const PAYLOAD_BYTES: usize = 128;
const SAMPLES: usize = 5;
const REQUIRED_REPLAY_RATIO: f64 = 1.5;

struct NatsServer {
    child: Child,
    store: TempDir,
}

impl Drop for NatsServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// <HANDWRITE gap="missing-generator:unit-test" tracker="#2159" reason="unit-test section in tape_vs_nats_jetstream.rs is hand-written pending codegen support">
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tape_beats_nats_jetstream_on_local_backlog_replay() {
    if cfg!(debug_assertions) {
        eprintln!("release-only competitor gate: rerun with `cargo test --release`");
        return;
    }
    let (server, url) = spawn_nats_server().await;
    let payload = payload(PAYLOAD_BYTES);
    let (tape, tape_url) = support::spawn_tape_service(EVENTS, payload.clone()).await;
    let tape_usage_before = support::process_usage(tape.pid());
    let tape_samples = support::tape_service_replay_samples(&tape_url, EVENTS, SAMPLES).await;
    let tape_usage_after = support::process_usage(tape.pid());
    let (nats_samples, nats_usage_before, nats_usage_after) =
        nats_jetstream_replay_samples(&url, EVENTS, payload.to_string(), SAMPLES, server.pid())
            .await;
    let tape_p50 = percentile(&tape_samples, 0.50);
    let nats_p50 = percentile(&nats_samples, 0.50);
    let report = tape::bench::external_replay_win(
        "NATS JetStream stream",
        "local_backlog_full_replay",
        EVENTS,
        PAYLOAD_BYTES,
        tape_p50,
        nats_p50,
        REQUIRED_REPLAY_RATIO,
        "apps/tape/tests/tape_vs_nats_jetstream.rs starts real Tape h2c and NATS JetStream services, then downloads and validates five complete samples of the same durable 20k-event replay across both network boundaries",
    );

    let payload_total = (EVENTS * PAYLOAD_BYTES) as f64;
    let resource_report = serde_json::json!({
        "gate": report,
        "samples": SAMPLES,
        "latency_scope": "warm connection, complete 20k-event backlog transfer and validation; broker setup and publish excluded",
        "tape": {
            "throughput_events_s": throughput(&tape_samples),
            "p50_us": tape_p50,
            "p95_us": percentile(&tape_samples, 0.95),
            "p99_us": percentile(&tape_samples, 0.99),
            "cpu_ms": (tape_usage_after.cpu_seconds - tape_usage_before.cpu_seconds) * 1_000.0,
            "rss_bytes": tape_usage_after.rss_bytes,
            "disk_bytes": tape.disk_bytes(),
            "disk_amplification": tape.disk_bytes() as f64 / payload_total,
            "errors": 0,
        },
        "nats_jetstream": {
            "throughput_events_s": throughput(&nats_samples),
            "p50_us": nats_p50,
            "p95_us": percentile(&nats_samples, 0.95),
            "p99_us": percentile(&nats_samples, 0.99),
            "cpu_ms": (nats_usage_after.cpu_seconds - nats_usage_before.cpu_seconds) * 1_000.0,
            "rss_bytes": nats_usage_after.rss_bytes,
            "disk_bytes": server.disk_bytes(),
            "disk_amplification": server.disk_bytes() as f64 / payload_total,
            "errors": 0,
        },
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&resource_report).unwrap()
    );
    tape::bench::verify_external_replay_win(&report).expect("Tape beats NATS JetStream replay");
    drop(tape);
    drop(server);
}
// </HANDWRITE>

async fn nats_jetstream_replay_samples(
    url: &str,
    events: usize,
    payload: String,
    samples: usize,
    pid: u32,
) -> (Vec<u128>, support::ProcessUsage, support::ProcessUsage) {
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

    let mut replays = Vec::with_capacity(samples);
    for sample in 0..samples {
        let consumer = stream
            .create_consumer(push::OrderedConfig {
                deliver_subject: format!("tape.bench.deliver.{sample}"),
                deliver_policy: DeliverPolicy::All,
                replay_policy: ReplayPolicy::Instant,
                ..Default::default()
            })
            .await
            .expect("create ordered replay consumer");
        replays.push(consumer.messages().await.expect("open replay messages"));
    }
    let usage_before = support::process_usage(pid);
    let mut elapsed = Vec::with_capacity(samples);
    for mut messages in replays {
        let started = Instant::now();
        for _ in 0..events {
            let message = messages
                .next()
                .await
                .expect("JetStream replay message")
                .expect("valid JetStream replay message");
            assert_eq!(message.payload.len(), payload.len());
        }
        elapsed.push(started.elapsed().as_micros().max(1));
    }
    let usage_after = support::process_usage(pid);
    (elapsed, usage_before, usage_after)
}

fn percentile(samples: &[u128], fraction: f64) -> u128 {
    let mut values = samples.to_vec();
    values.sort_unstable();
    values[((values.len() - 1) as f64 * fraction).round() as usize]
}

fn throughput(samples: &[u128]) -> f64 {
    (EVENTS * samples.len()) as f64 / (samples.iter().sum::<u128>() as f64 / 1_000_000.0)
}

impl NatsServer {
    fn pid(&self) -> u32 {
        self.child.id()
    }

    fn disk_bytes(&self) -> u64 {
        support::directory_bytes(self.store.path())
    }
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
        let mut server = NatsServer { child, store };
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
