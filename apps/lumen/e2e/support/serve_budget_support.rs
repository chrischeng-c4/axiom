//! Shared process and fixture helpers for the bounded serving-capacity e2e
//! targets. This file is intentionally not a test target by itself.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use lumen::aof::{AofReader, AofWriter};
use lumen::log_entry::RaftLogEntry;
use lumen::rdb::{LocalFsRdbStore, RdbStore};
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::Engine;
use lumen::types::{
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
};
use lumen::wal::WalRecord;
use serde_json::{json, Value};

pub const COLLECTION: &str = "replay-budget";
pub const FIELD: &str = "kw";
pub const LARGE_VALUE_BYTES: usize = 6 * 1024 * 1024 - 32 * 1024;
pub const LARGE_VALUE_COUNT: usize = 43;
pub const REQUEST_BODY_MAX_BYTES: usize = 8 * 1024 * 1024;
pub const PENDING_HALF_BYTES: u64 = 128 * 1024 * 1024;
pub const PENDING_HARD_BYTES: u64 = 256 * 1024 * 1024;
pub const STARTUP_DEADLINE: Duration = Duration::from_secs(30);
pub const REQUEST_DEADLINE: Duration = Duration::from_secs(30);
pub const RETRY_DEADLINE: Duration = Duration::from_secs(120);
pub const POLL_INTERVAL: Duration = Duration::from_millis(25);

const MAX_PORT_BIND_ATTEMPTS: usize = 3;
static PORT_BIND_HANDOFF: Mutex<()> = Mutex::new(());

#[derive(Clone, Copy)]
pub enum ServeMode {
    Segment,
    Cbor,
    MemoryOnly,
}

pub struct HttpResponse {
    pub status: u16,
    pub retry_after: Option<String>,
    pub body: Value,
}

pub struct LumenProcess {
    child: Option<Child>,
    pub port: u16,
    port_bind_handoff: Option<MutexGuard<'static, ()>>,
    root: Option<PathBuf>,
    mode: ServeMode,
    body_limit_bytes: Option<usize>,
    bind_attempt: usize,
    stdout: tempfile::NamedTempFile,
    stderr: tempfile::NamedTempFile,
}

impl LumenProcess {
    pub fn spawn(root: Option<&Path>, mode: ServeMode, snapshot_secs: u64) -> Self {
        Self::spawn_attempt(root.map(Path::to_path_buf), mode, snapshot_secs, None, 1)
    }

    /// Set a body limit only on this child process. This keeps a large legal
    /// compatibility query from changing the test runner environment.
    pub fn spawn_with_body_limit(
        root: Option<&Path>,
        mode: ServeMode,
        snapshot_secs: u64,
        body_limit_bytes: usize,
    ) -> Self {
        Self::spawn_attempt(
            root.map(Path::to_path_buf),
            mode,
            snapshot_secs,
            Some(body_limit_bytes),
            1,
        )
    }

    fn spawn_attempt(
        root: Option<PathBuf>,
        mode: ServeMode,
        snapshot_secs: u64,
        body_limit_bytes: Option<usize>,
        bind_attempt: usize,
    ) -> Self {
        let port_bind_handoff = PORT_BIND_HANDOFF
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("reserve loopback port");
        let port = listener.local_addr().expect("reserved port address").port();
        let stdout = tempfile::NamedTempFile::new().expect("create lumen stdout capture");
        let stderr = tempfile::NamedTempFile::new().expect("create lumen stderr capture");

        let mut command = Command::new(env!("CARGO_BIN_EXE_lumen"));
        command
            .args([
                "serve",
                "--host",
                "127.0.0.1",
                "--port",
                &port.to_string(),
                "--wal",
                "embedded",
                "--persistence",
                match mode {
                    ServeMode::Segment => "segment",
                    ServeMode::Cbor | ServeMode::MemoryOnly => "cbor",
                },
                "--snapshot-secs",
                &snapshot_secs.to_string(),
                "--log-level",
                "info",
                "--log-format",
                "json",
            ])
            .env("LUMEN_AUTH", "off")
            .env_remove("RUST_LOG")
            .env_remove("LUMEN_LOG_FORMAT")
            .stdout(Stdio::from(
                stdout.reopen().expect("open lumen stdout capture"),
            ))
            .stderr(Stdio::from(
                stderr.reopen().expect("open lumen stderr capture"),
            ));
        if let Some(body_limit_bytes) = body_limit_bytes {
            command.env("LUMEN_BODY_LIMIT_BYTES", body_limit_bytes.to_string());
        }
        if let Some(root) = &root {
            command.arg("--data-dir").arg(root);
        }
        drop(listener);
        let child = command.spawn().expect("spawn lumen serve");
        Self {
            child: Some(child),
            port,
            port_bind_handoff: Some(port_bind_handoff),
            root,
            mode,
            body_limit_bytes,
            bind_attempt,
            stdout,
            stderr,
        }
    }

    pub fn wait_until_ready(&mut self, snapshot_secs: u64) {
        let deadline = Instant::now() + STARTUP_DEADLINE;
        loop {
            if let Some(status) = self.child().try_wait().expect("poll lumen child") {
                let logs = self.finish_exited_child();
                if logs.contains("Address already in use")
                    && self.bind_attempt < MAX_PORT_BIND_ATTEMPTS
                {
                    let root = self.root.clone();
                    let mode = self.mode;
                    let body_limit_bytes = self.body_limit_bytes;
                    let bind_attempt = self.bind_attempt + 1;
                    debug_assert!(self.port_bind_handoff.is_none());
                    *self = Self::spawn_attempt(
                        root,
                        mode,
                        snapshot_secs,
                        body_limit_bytes,
                        bind_attempt,
                    );
                    continue;
                }
                panic!("lumen exited before /readyz ({status}): {logs}");
            }
            if answers_ready(self.port)
                && self
                    .child()
                    .try_wait()
                    .expect("repoll lumen child")
                    .is_none()
            {
                self.port_bind_handoff.take();
                return;
            }
            if Instant::now() >= deadline {
                let logs = self.stop_and_logs();
                panic!("lumen did not answer /readyz before bounded startup:\n{logs}");
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    pub fn post_json(&self, path: &str, body: &Value) -> HttpResponse {
        self.json_request(reqwest::Method::POST, path, body)
    }

    pub fn put_json(&self, path: &str, body: &Value) -> HttpResponse {
        self.json_request(reqwest::Method::PUT, path, body)
    }

    fn json_request(&self, method: reqwest::Method, path: &str, body: &Value) -> HttpResponse {
        let port = self.port;
        let path = path.to_owned();
        let body = body.clone();
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build process E2E runtime")
            .block_on(async move {
                let response = reqwest::Client::builder()
                    .timeout(REQUEST_DEADLINE)
                    .build()
                    .expect("build process E2E HTTP client")
                    .request(method, format!("http://127.0.0.1:{port}{path}"))
                    .json(&body)
                    .send()
                    .await
                    .expect("send lumen HTTP request");
                let status = response.status().as_u16();
                let retry_after = response
                    .headers()
                    .get(reqwest::header::RETRY_AFTER)
                    .and_then(|value| value.to_str().ok())
                    .map(str::to_owned);
                let raw = response.text().await.expect("read HTTP response body");
                let body = serde_json::from_str(&raw).unwrap_or(Value::String(raw));
                HttpResponse {
                    status,
                    retry_after,
                    body,
                }
            })
    }

    pub fn get_text(&self, path: &str) -> String {
        let port = self.port;
        let path = path.to_owned();
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build process E2E runtime")
            .block_on(async move {
                let response = reqwest::Client::builder()
                    .timeout(REQUEST_DEADLINE)
                    .build()
                    .expect("build process E2E HTTP client")
                    .get(format!("http://127.0.0.1:{port}{path}"))
                    .send()
                    .await
                    .expect("send lumen HTTP get");
                assert!(
                    response.status().is_success(),
                    "{path} must return 2xx, got {}",
                    response.status()
                );
                response.text().await.expect("read HTTP text response")
            })
    }

    pub fn logs(&self) -> String {
        format!(
            "{}\n{}",
            String::from_utf8_lossy(
                &std::fs::read(self.stdout.path()).expect("read lumen stdout capture")
            ),
            String::from_utf8_lossy(
                &std::fs::read(self.stderr.path()).expect("read lumen stderr capture")
            )
        )
    }

    pub fn stop_and_logs(&mut self) -> String {
        let mut child = self.child.take().expect("lumen child is available");
        let _ = child.kill();
        child.wait().expect("wait for lumen child");
        self.port_bind_handoff.take();
        self.logs()
    }

    /// Signal only this owned child. The caller retains the bounded wait and
    /// emergency cleanup, as Docker does before its stop timeout.
    #[cfg(unix)]
    pub fn send_sigterm(&mut self) {
        let pid = self.child().id() as libc::pid_t;
        assert_eq!(unsafe { libc::kill(pid, libc::SIGTERM) }, 0, "signal owned Lumen child");
    }

    fn finish_exited_child(&mut self) -> String {
        self.child.take().expect("lumen child is available");
        self.port_bind_handoff.take();
        self.logs()
    }

    fn child(&mut self) -> &mut Child {
        self.child.as_mut().expect("lumen child is available")
    }
}

impl Drop for LumenProcess {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if std::thread::panicking() {
            eprintln!("lumen serving evidence:\n{}", self.logs());
        }
    }
}

fn answers_ready(port: u16) -> bool {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let Ok(mut stream) = TcpStream::connect_timeout(&address, Duration::from_millis(50)) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(Duration::from_millis(50)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(50)));
    if stream
        .write_all(b"GET /readyz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .is_err()
    {
        return false;
    }
    let mut response = [0u8; 256];
    let Ok(read) = stream.read(&mut response) else {
        return false;
    };
    std::str::from_utf8(&response[..read])
        .map(|response| response.starts_with("HTTP/1.1 200"))
        .unwrap_or(false)
}

pub fn large_keyword_value(ordinal: usize) -> String {
    let mut bytes = vec![b'a'; LARGE_VALUE_BYTES];
    let mut state = (ordinal as u64).wrapping_add(1).wrapping_mul(0x9E37_79B9);
    for byte in &mut bytes {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        *byte = b'a' + (state % 26) as u8;
    }
    let marker = format!("capacity-row-{ordinal:02}-");
    bytes[..marker.len()].copy_from_slice(marker.as_bytes());
    String::from_utf8(bytes).expect("ASCII large Keyword value")
}

pub fn capacity_external_id(ordinal: usize) -> String {
    format!("capacity-row-{ordinal:02}")
}

pub fn raw_payload_bytes() -> usize {
    LARGE_VALUE_BYTES * LARGE_VALUE_COUNT
}

pub fn create_keyword_collection(process: &LumenProcess) {
    let response = process.put_json(
        &format!("/collections/{COLLECTION}"),
        &json!({ "fields": { FIELD: { "type": "keyword" } } }),
    );
    assert_eq!(
        response.status, 200,
        "create replay-budget collection: {}",
        response.body
    );
}

pub fn large_index_body(ordinal: usize) -> Value {
    let body = json!({
        "items": [{
            "external_id": capacity_external_id(ordinal),
            "field": FIELD,
            "value": large_keyword_value(ordinal),
        }]
    });
    assert!(
        serde_json::to_vec(&body)
            .expect("serialize legal capacity request")
            .len()
            < REQUEST_BODY_MAX_BYTES,
        "one capacity request must stay below the public 8 MiB HTTP body boundary",
    );
    body
}

pub fn index_large_keyword(process: &LumenProcess, ordinal: usize) -> HttpResponse {
    process.post_json(
        &format!("/collections/{COLLECTION}/index"),
        &large_index_body(ordinal),
    )
}

pub fn stats_documents(process: &LumenProcess) -> u64 {
    let response = process.get_text(&format!("/collections/{COLLECTION}/stats"));
    serde_json::from_str::<Value>(&response).expect("decode collection stats")["documents_indexed"]
        .as_u64()
        .expect("stats documents_indexed")
}

pub fn assert_keyword_hit(process: &LumenProcess, ordinal: usize) {
    let value = large_keyword_value(ordinal);
    let request = json!({
        "query": { "term": { "field": FIELD, "value": value } },
        "limit": 2,
    });
    assert!(
        serde_json::to_vec(&request)
            .expect("serialize legal capacity query")
            .len()
            < REQUEST_BODY_MAX_BYTES,
        "one capacity query must stay below the public 8 MiB HTTP body boundary",
    );
    let response = process.post_json(&format!("/collections/{COLLECTION}/search"), &request);
    assert_eq!(
        response.status, 200,
        "capacity search response: {}",
        response.body
    );
    assert_eq!(
        response.body["total"], 1,
        "capacity search must retain exact Keyword row {ordinal}: {}",
        response.body
    );
    assert_eq!(
        response.body["hits"][0]["external_id"],
        capacity_external_id(ordinal),
        "capacity search must return the requested caller external ID: {}",
        response.body
    );
}

pub fn metric_u64(metrics: &str, name: &str) -> u64 {
    for line in metrics.lines() {
        let mut fields = line.split_whitespace();
        if fields.next() == Some(name) {
            return fields
                .next()
                .unwrap_or_else(|| panic!("metric {name} has no value: {line}"))
                .parse()
                .unwrap_or_else(|error| {
                    panic!("metric {name} has invalid integer value: {error}")
                });
        }
    }
    panic!("missing Prometheus metric {name}:\n{metrics}");
}

pub fn assert_pending_metrics(metrics: &str) {
    let high_water = metric_u64(metrics, "lumen_pending_change_high_water_bytes");
    let total = metric_u64(metrics, "lumen_pending_change_total_bytes");
    assert!(
        high_water >= PENDING_HALF_BYTES,
        "large legal writes must cross the approved 128 MiB early-checkpoint threshold; high_water={high_water}",
    );
    assert!(
        high_water <= PENDING_HARD_BYTES,
        "pending-change high water must not exceed the approved 256 MiB hard budget; high_water={high_water}",
    );
    assert!(
        total <= PENDING_HARD_BYTES,
        "current pending-change accounting must remain within the approved 256 MiB hard budget; total={total}",
    );
}

pub fn write_large_aof_tail(root: &Path) -> Vec<u64> {
    let mut aof = AofWriter::open(root.join("aof.log")).expect("open official AOF writer");
    let create = RaftLogEntry::CreateCollection {
        collection_id: COLLECTION.into(),
        req: keyword_schema(),
    };
    aof.append(1, &WalRecord::new(create))
        .expect("append schema AOF frame");
    for ordinal in 0..LARGE_VALUE_COUNT {
        let sequence = ordinal as u64 + 2;
        aof.append(
            sequence,
            &WalRecord::new(RaftLogEntry::Index {
                collection_id: COLLECTION.into(),
                req: IndexRequest {
                    items: vec![IndexItem {
                        external_id: capacity_external_id(ordinal),
                        field: FIELD.into(),
                        value: FieldValue::String(large_keyword_value(ordinal)),
                        version: None,
                    }],
                    request_id: None,
                },
            }),
        )
        .expect("append large valid AOF frame");
    }
    aof.sync_strict().expect("strict-sync AOF fixture");
    aof_sequences(root)
}

pub fn aof_sequences(root: &Path) -> Vec<u64> {
    let mut sequences = Vec::new();
    AofReader::replay(root.join("aof.log"), 0, |sequence, _record| {
        sequences.push(sequence);
    })
    .expect("read valid AOF fixture/tail");
    sequences
}

pub fn checkpoint_sequence(root: &Path) -> u64 {
    SegmentRdbStore::new(root)
        .expect("open segment checkpoint root")
        .load_current_generation()
        .expect("read segment CURRENT")
        .expect("replay checkpoint must publish CURRENT")
        .sequence
}

pub fn assert_safe_aof_tail(original: &[u64], surviving: &[u64], checkpoint_seq: u64) {
    for sequence in surviving {
        assert!(
            original.contains(sequence),
            "startup must not add an unexpected AOF frame while recovering: {sequence}",
        );
    }
    for sequence in original {
        if *sequence > checkpoint_seq {
            assert!(
                surviving.contains(sequence),
                "AOF frame {sequence} is later than CURRENT checkpoint {checkpoint_seq} and must remain",
            );
        } else if !surviving.contains(sequence) {
            assert!(
                *sequence <= checkpoint_seq,
                "a removed AOF frame must be covered by CURRENT before trim",
            );
        }
    }
}

pub fn replay_event_u64(logs: &str, field: &str) -> Option<u64> {
    for line in logs.lines() {
        let Ok(event) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if event_string(&event, "message") != Some("AOF startup decision") {
            continue;
        }
        if let Some(value) = event_field(&event, field).and_then(Value::as_u64) {
            return Some(value);
        }
    }
    None
}

fn event_string<'a>(event: &'a Value, field: &str) -> Option<&'a str> {
    event_field(event, field).and_then(Value::as_str)
}

fn event_field<'a>(event: &'a Value, field: &str) -> Option<&'a Value> {
    event
        .get(field)
        .or_else(|| event.get("fields").and_then(|fields| fields.get(field)))
        .or_else(|| {
            event
                .get("attributes")
                .and_then(|attributes| attributes.get(field))
        })
}

pub fn wait_for_cbor_snapshot(root: &Path, expected_documents: u64) {
    let deadline = Instant::now() + RETRY_DEADLINE;
    loop {
        let store = LocalFsRdbStore::new(root).expect("open real CBOR snapshot root");
        let snapshot = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build CBOR inspection runtime")
            .block_on(store.load_latest())
            .expect("read latest CBOR snapshot");
        if let Some(snapshot) = snapshot {
            let cold = Engine::new();
            snapshot
                .restore_into(&cold)
                .expect("restore actual latest CBOR snapshot");
            if cold
                .stats(COLLECTION)
                .map(|stats| stats.documents_indexed == expected_documents)
                .unwrap_or(false)
            {
                return;
            }
        }
        if Instant::now() >= deadline {
            panic!(
                "CBOR periodic snapshot did not contain all {expected_documents} indexed documents before bounded cleanup"
            );
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn keyword_schema() -> CreateCollectionRequest {
    CreateCollectionRequest {
        fields: BTreeMap::from([(
            FIELD.to_owned(),
            FieldSpec {
                field_type: FieldType::Keyword,
                analyzer: None,
                multi: None,
                dim: None,
                metric: None,
                backend: None,
                quantize: None,
            },
        )]),
    }
}
