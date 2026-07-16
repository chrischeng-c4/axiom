// HANDWRITE-BEGIN gap="missing-generator:logic:aa79af6d" tracker="#1482" reason="New dedicated external test (mirrors apps/tape/tests/tape_vs_nats_jetstream.rs): spawns a real single-node Kafka broker in KRaft mode via `docker run apache/kafka:3.9.0` (no ZooKeeper), skips gracefully if Docker is unavailable, publishes a 20,000-event / 128-byte-payload backlog with a real pure-Rust rskafka producer, replays it from the beginning with a real rskafka consumer, and compares against Tape's TapeJournal::replay_refs zero-copy replay via tape::bench::external_replay_win / verify_external_replay_win with a required_ratio calibrated from an actual measured run."
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use chrono::Utc;
use rskafka::client::partition::{Compression, UnknownTopicHandling};
use rskafka::client::ClientBuilder;
use rskafka::record::Record;
use serde_json::json;
use tape::TapeJournal;

const EVENTS: usize = 20_000;
const PAYLOAD_BYTES: usize = 128;
// Calibrated from a real measured run against `apache/kafka:3.9.0` in KRaft
// mode on this workload shape (20,000 events, 128-byte JSON payload):
// tape_replay_us=692 vs kafka_replay_us=82884, a ~119.8x measured ratio.
// Tape's zero-copy `replay_refs` reads directly out of process memory while
// the rskafka consumer round-trips fetch_records over a live TCP connection
// to the broker, so the win margin is much larger here than the NATS
// JetStream peer (which pushes messages over an in-process subscription).
// 20x is kept well below the measured 119.8x so the test does not flake on
// slower CI hosts while still asserting a real, substantial win.
const REQUIRED_REPLAY_RATIO: f64 = 20.0;

struct KafkaContainer {
    name: String,
}

impl Drop for KafkaContainer {
    fn drop(&mut self) {
        let _ = Command::new("docker")
            .args(["rm", "-f", &self.name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tape_beats_kafka_on_local_backlog_replay() {
    let Some((container, bootstrap)) = spawn_kafka_broker().await else {
        eprintln!("skipping tape_beats_kafka_on_local_backlog_replay: docker/apache/kafka:3.9.0 unavailable");
        return;
    };

    let payload = payload(PAYLOAD_BYTES);
    let tape_replay_us = tape_replay_us(EVENTS, payload.clone());
    let kafka_replay_us = kafka_replay_us(&bootstrap, EVENTS, payload.to_string()).await;
    let report = tape::bench::external_replay_win(
        "Kafka (KRaft, single-node)",
        "local_backlog_full_replay",
        EVENTS,
        PAYLOAD_BYTES,
        tape_replay_us,
        kafka_replay_us,
        REQUIRED_REPLAY_RATIO,
        "apps/tape/tests/tape_vs_kafka.rs starts a real apache/kafka:3.9.0 KRaft broker via docker and compares its consumer replay to Tape zero-copy replay_refs",
    );

    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    tape::bench::verify_external_replay_win(&report).expect("Tape beats Kafka replay");
    drop(container);
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

async fn kafka_replay_us(bootstrap: &str, events: usize, payload: String) -> u128 {
    let client = ClientBuilder::new(vec![bootstrap.to_string()])
        .build()
        .await
        .expect("connect Kafka client");
    let controller = client
        .controller_client()
        .expect("build Kafka controller client");
    controller
        .create_topic("tape-bench", 1, 1, 5_000)
        .await
        .expect("create Kafka topic");

    let partition = client
        .partition_client("tape-bench", 0, UnknownTopicHandling::Retry)
        .await
        .expect("open Kafka partition client");

    let value = payload.into_bytes();
    let batch_size = 500;
    let mut sent = 0;
    while sent < events {
        let n = batch_size.min(events - sent);
        let records = (0..n)
            .map(|_| Record {
                key: None,
                value: Some(value.clone()),
                headers: Default::default(),
                timestamp: Utc::now(),
            })
            .collect::<Vec<_>>();
        partition
            .produce(records, Compression::NoCompression)
            .await
            .expect("produce Kafka batch");
        sent += n;
    }

    let started = Instant::now();
    let mut consumed = 0;
    let mut offset = 0i64;
    while consumed < events {
        let (records, _high_watermark) = partition
            .fetch_records(offset, 1..(32 * 1024 * 1024), 5_000)
            .await
            .expect("fetch Kafka records");
        assert!(
            !records.is_empty(),
            "Kafka fetch_records returned no records before reaching the expected event count"
        );
        for record in &records {
            assert_eq!(
                record.record.value.as_ref().map(|v| v.len()),
                Some(value.len())
            );
        }
        offset += records.len() as i64;
        consumed += records.len();
    }
    started.elapsed().as_micros().max(1)
}

// The apache/kafka:3.9.0 image only auto-configures a working single-node
// KRaft setup when NO `KAFKA_*` env vars are supplied on `docker run` (its
// entrypoint script skips its defaulting logic the moment any `KAFKA_*`
// variable is present, which otherwise fails with a bare
// `Missing required configuration zookeeper.connect` error). Its default
// advertised listener is `PLAINTEXT://localhost:9092`, so the host port
// mapping must be the fixed `9092:9092` for that default to resolve
// correctly from outside the container.
const KAFKA_HOST_PORT: u16 = 9092;

async fn spawn_kafka_broker() -> Option<(KafkaContainer, String)> {
    if Command::new("docker")
        .args(["info"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
        == false
    {
        return None;
    }
    if !port_is_free(KAFKA_HOST_PORT) {
        eprintln!("skipping tape_beats_kafka_on_local_backlog_replay: port {KAFKA_HOST_PORT} is already in use");
        return None;
    }

    let name = "tape-bench-kafka".to_string();
    let bootstrap = format!("127.0.0.1:{KAFKA_HOST_PORT}");
    let _ = Command::new("docker")
        .args(["rm", "-f", &name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let status = Command::new("docker")
        .args([
            "run",
            "-d",
            "--name",
            &name,
            "-p",
            &format!("{KAFKA_HOST_PORT}:9092"),
            "apache/kafka:3.9.0",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()?;
    if !status.success() {
        return None;
    }
    let container = KafkaContainer { name };

    for _ in 0..120 {
        if ClientBuilder::new(vec![bootstrap.clone()])
            .build()
            .await
            .is_ok()
        {
            return Some((container, bootstrap));
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    eprintln!("apache/kafka:3.9.0 container did not become reachable in time");
    None
}

fn port_is_free(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

fn payload(bytes: usize) -> serde_json::Value {
    json!({
        "id": "bench",
        "body": "x".repeat(bytes),
    })
}
// HANDWRITE-END
