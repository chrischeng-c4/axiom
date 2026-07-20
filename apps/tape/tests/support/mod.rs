#![allow(dead_code)] // each integration-test crate uses a different subset

use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use tape::TapeJournal;

pub struct TapeService {
    child: Child,
    store: tempfile::TempDir,
}

#[derive(Debug, Clone, Copy)]
pub struct ProcessUsage {
    pub cpu_seconds: f64,
    pub rss_bytes: u64,
}

impl Drop for TapeService {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Start the real Tape h2c service from a durable journal prepared outside
/// the timed replay region. This keeps competitor setup out of the metric
/// while ensuring Tape is measured across the same process/network boundary.
pub async fn spawn_tape_service(
    events: usize,
    payload: serde_json::Value,
) -> (TapeService, String) {
    let store = tempfile::tempdir().expect("create Tape service store");
    let store_path = store.path().join("journal.json");
    let mut journal = TapeJournal::default();
    for i in 0..events {
        journal.append(
            "orders.created",
            Some(format!("orders.created.{i}")),
            payload.clone(),
            Some(i as u64),
        );
    }
    std::fs::write(
        &store_path,
        serde_json::to_vec(&journal).expect("serialize Tape benchmark journal"),
    )
    .expect("write Tape benchmark journal");

    let port = free_port().expect("allocate Tape benchmark port");
    let bind = format!("127.0.0.1:{port}");
    let url = format!("http://{bind}");
    let child = Command::new(env!("CARGO_BIN_EXE_tape"))
        .args([
            "serve",
            "--bind",
            &bind,
            "--store",
            store_path.to_str().expect("utf-8 Tape store path"),
            "--grace-secs",
            "0",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn Tape benchmark service");
    let mut service = TapeService { child, store };
    let client = reqwest::Client::new();
    for _ in 0..200 {
        if client
            .get(format!("{url}/healthz"))
            .send()
            .await
            .is_ok_and(|response| response.status().is_success())
        {
            return (service, url);
        }
        if matches!(service.child.try_wait(), Ok(Some(_))) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("Tape benchmark service did not become healthy at {url}");
}

pub async fn tape_service_replay_us(url: &str, events: usize) -> u128 {
    tape_service_replay_samples(url, events, 1).await[0]
}

pub async fn tape_service_replay_samples(url: &str, events: usize, samples: usize) -> Vec<u128> {
    let client = transport_h2c::h2c_client().expect("build Tape h2c client");
    let warm = client
        .get(format!("{url}/healthz"))
        .send()
        .await
        .expect("warm Tape h2c connection");
    assert!(warm.status().is_success());

    let mut elapsed = Vec::with_capacity(samples);
    for _ in 0..samples {
        let started = Instant::now();
        let response = client
            .get(format!(
                "{url}/topics/orders.created/replay/stream?from_offset=0&limit={events}"
            ))
            .send()
            .await
            .expect("Tape h2c replay request");
        assert!(response.status().is_success(), "Tape replay status");
        assert_eq!(response.version(), reqwest::Version::HTTP_2);
        assert_eq!(
            response
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some(tape::replay_wire::CONTENT_TYPE)
        );
        let body = response.bytes().await.expect("read Tape replay frames");
        let stats = tape::replay_wire::inspect(&body).expect("validate Tape replay frames");
        assert_eq!(
            stats.events, events,
            "Tape service must return the whole backlog"
        );
        elapsed.push(started.elapsed().as_micros().max(1));
    }
    elapsed
}

impl TapeService {
    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    pub fn disk_bytes(&self) -> u64 {
        directory_bytes(self.store.path())
    }
}

pub fn process_usage(pid: u32) -> ProcessUsage {
    let output = Command::new("ps")
        .args(["-o", "rss=", "-o", "time=", "-p", &pid.to_string()])
        .output()
        .expect("sample child process usage with ps");
    assert!(output.status.success(), "ps failed for child {pid}");
    let stdout = String::from_utf8(output.stdout).expect("ps output is utf-8");
    let mut fields = stdout.split_whitespace();
    let rss_kib = fields
        .next()
        .expect("ps rss field")
        .parse::<u64>()
        .expect("ps rss is numeric");
    let cpu = fields.next().expect("ps cpu-time field");
    ProcessUsage {
        cpu_seconds: parse_cpu_time(cpu),
        rss_bytes: rss_kib.saturating_mul(1024),
    }
}

pub fn directory_bytes(path: &std::path::Path) -> u64 {
    std::fs::read_dir(path)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                directory_bytes(&path)
            } else {
                entry.metadata().map(|meta| meta.len()).unwrap_or_default()
            }
        })
        .sum()
}

fn parse_cpu_time(value: &str) -> f64 {
    let (days, clock) = value
        .split_once('-')
        .map(|(days, clock)| (days.parse::<f64>().unwrap(), clock))
        .unwrap_or((0.0, value));
    let fields = clock
        .split(':')
        .map(|field| field.parse::<f64>().unwrap())
        .collect::<Vec<_>>();
    let seconds = match fields.as_slice() {
        [minutes, seconds] => minutes * 60.0 + seconds,
        [hours, minutes, seconds] => hours * 3600.0 + minutes * 60.0 + seconds,
        _ => panic!("unexpected ps cpu-time value {value}"),
    };
    days * 86_400.0 + seconds
}

fn free_port() -> Option<u16> {
    TcpListener::bind("127.0.0.1:0")
        .ok()
        .and_then(|listener| listener.local_addr().ok())
        .map(|address| address.port())
}
