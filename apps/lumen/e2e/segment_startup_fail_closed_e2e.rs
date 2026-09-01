//! Process-level startup oracle for the segment checkpoint root.
//!
//! This test invokes the packaged `lumen` binary. It does not call the segment
//! store directly, because a root that fails after the listener binds would
//! still expose an unsafe fresh service to callers.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use lumen::aof::AofWriter;
use lumen::log_entry::RaftLogEntry;
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::Engine;
use lumen::types::{
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
};
use lumen::wal::WalRecord;
use serde_json::{json, Value};

const STARTUP_DEADLINE: Duration = Duration::from_secs(15);
const POLL_INTERVAL: Duration = Duration::from_millis(25);
const RECOVERED_COLLECTION: &str = "aof-recovered";
const RECOVERED_EXTERNAL_ID: &str = "replayed-external-id";
const RECOVERED_VALUE: &str = "replayed@example.test";
const CHECKPOINT_COLLECTION: &str = "checkpoint-data";
const CHECKPOINT_EXTERNAL_ID: &str = "checkpoint-external-id";
const CHECKPOINT_VALUE: &str = "checkpoint@example.test";

#[derive(Debug, Eq, PartialEq)]
enum RootEntrySnapshot {
    Directory,
    RegularFile(Vec<u8>),
    Symlink(std::path::PathBuf),
    Other,
}

struct LumenProcess {
    child: Option<Child>,
    port: u16,
}

impl LumenProcess {
    fn spawn(root: &Path) -> Self {
        let port = reserve_port();
        let child = Command::new(env!("CARGO_BIN_EXE_lumen"))
            .args([
                "serve",
                "--host",
                "127.0.0.1",
                "--port",
                &port.to_string(),
                "--data-dir",
            ])
            .arg(root)
            .args([
                "--persistence",
                "segment",
                "--wal",
                "embedded",
                "--log-level",
                "info",
                "--log-format",
                "json",
            ])
            .env("LUMEN_AUTH", "off")
            .env_remove("RUST_LOG")
            .env_remove("LUMEN_LOG_FORMAT")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn lumen serve");
        Self {
            child: Some(child),
            port,
        }
    }

    fn wait_until_ready(&mut self) {
        let deadline = Instant::now() + STARTUP_DEADLINE;
        loop {
            if answers_ready(self.port) {
                return;
            }
            if self.child().try_wait().expect("poll lumen child").is_some() {
                let output = self.wait_with_output();
                panic!("lumen exited before /readyz: {}", output_text(&output));
            }
            if Instant::now() >= deadline {
                let logs = self.stop_and_logs();
                panic!("lumen did not answer /readyz before the bounded deadline:\n{logs}");
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    /// Wait for a refusal while checking the port throughout the whole bound.
    /// A process that briefly binds and then exits still violates fail-closed
    /// startup, so one post-exit connection attempt is not enough.
    fn wait_for_refusal(&mut self) -> (ExitStatus, String, bool) {
        let deadline = Instant::now() + STARTUP_DEADLINE;
        let mut ever_bound = false;
        loop {
            ever_bound |= TcpStream::connect_timeout(
                &SocketAddr::from(([127, 0, 0, 1], self.port)),
                Duration::from_millis(25),
            )
            .is_ok();
            if self.child().try_wait().expect("poll lumen child").is_some() {
                let output = self.wait_with_output();
                return (output.status, output_text(&output), ever_bound);
            }
            if Instant::now() >= deadline {
                let logs = self.stop_and_logs();
                panic!("lumen kept running instead of refusing startup:\n{logs}");
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    fn child(&mut self) -> &mut Child {
        self.child.as_mut().expect("lumen child is available")
    }

    fn stop_and_logs(&mut self) -> String {
        let mut child = self.child.take().expect("lumen child is available");
        let _ = child.kill();
        output_text(&child.wait_with_output().expect("wait for lumen child"))
    }

    fn wait_with_output(&mut self) -> Output {
        self.child
            .take()
            .expect("lumen child is available")
            .wait_with_output()
            .expect("wait for lumen child")
    }
}

impl Drop for LumenProcess {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn reserve_port() -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("reserve loopback port");
    listener.local_addr().expect("reserved port address").port()
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

fn output_text(output: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// Capture each existing path before a refused start. This never follows a
/// symlink, so a bad mount cannot make the test inspect or change its target.
fn snapshot_root(root: &Path) -> Vec<(std::path::PathBuf, RootEntrySnapshot)> {
    let mut pending = vec![root.to_path_buf()];
    let mut snapshot = Vec::new();

    while let Some(directory) = pending.pop() {
        let mut entries = std::fs::read_dir(&directory)
            .expect("read fixture directory")
            .collect::<std::io::Result<Vec<_>>>()
            .expect("collect fixture directory");
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let relative = path
                .strip_prefix(root)
                .expect("fixture child is below root")
                .to_path_buf();
            let metadata = std::fs::symlink_metadata(&path).expect("inspect fixture child");
            let entry_snapshot = if metadata.file_type().is_symlink() {
                RootEntrySnapshot::Symlink(std::fs::read_link(&path).expect("read fixture link"))
            } else if metadata.is_file() {
                RootEntrySnapshot::RegularFile(std::fs::read(&path).expect("read fixture file"))
            } else if metadata.is_dir() {
                pending.push(path);
                RootEntrySnapshot::Directory
            } else {
                RootEntrySnapshot::Other
            };
            snapshot.push((relative, entry_snapshot));
        }
    }
    snapshot.sort_by(|left, right| left.0.cmp(&right.0));
    snapshot
}

/// One failed startup must not bind a listener, create `CURRENT`, or change a
/// single fixture path. The process is always reaped before this returns.
fn assert_refuses_without_mutation(root: &Path, case: &str) -> String {
    assert!(
        !root.join("CURRENT").exists(),
        "{case} fixture must start without CURRENT"
    );
    let before = snapshot_root(root);
    let mut process = LumenProcess::spawn(root);
    let (status, logs, ever_bound) = process.wait_for_refusal();
    assert!(
        !status.success(),
        "{case} root exited successfully:\n{logs}"
    );
    assert!(
        !ever_bound,
        "{case} root bound a listener before refusing startup:\n{logs}"
    );
    assert!(
        !root.join("CURRENT").exists(),
        "{case} root received a CURRENT pointer"
    );
    assert_eq!(
        snapshot_root(root),
        before,
        "{case} root changed during its refused startup"
    );
    logs
}

fn recovered_schema() -> CreateCollectionRequest {
    CreateCollectionRequest {
        fields: BTreeMap::from([(
            "email".to_string(),
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

fn indexed_checkpoint_engine() -> Arc<Engine> {
    let engine = Arc::new(Engine::new());
    engine
        .create_collection(CHECKPOINT_COLLECTION, recovered_schema())
        .expect("create checkpoint fixture collection");
    engine
        .index(
            CHECKPOINT_COLLECTION,
            IndexRequest {
                items: vec![IndexItem {
                    external_id: CHECKPOINT_EXTERNAL_ID.into(),
                    field: "email".into(),
                    value: FieldValue::String(CHECKPOINT_VALUE.into()),
                    version: None,
                }],
                request_id: None,
            },
        )
        .expect("index checkpoint fixture document");
    engine
}

/// This produces the exact 0.4.28 layout through the public segment writer:
/// `gen-<seq>` has no new generation manifest and no `CURRENT` pointer.
fn write_legacy_0428_checkpoint(root: &Path, sequence: u64) {
    indexed_checkpoint_engine()
        .flush_to_segments(&root.join(format!("gen-{sequence}")), sequence)
        .expect("flush exact legacy checkpoint");
}

/// This produces the current revision layout and its pointer through the
/// production store API. It returns the generated pointer bytes for fixtures
/// that must prove that a stale `CURRENT.tmp` is ignored.
fn write_current_checkpoint(root: &Path, sequence: u64) -> Vec<u8> {
    let store = SegmentRdbStore::new(root).expect("open production checkpoint store");
    let engine = indexed_checkpoint_engine();
    store
        .save(&engine, sequence)
        .expect("save production revision checkpoint");
    std::fs::read(root.join("CURRENT")).expect("read generated CURRENT")
}

fn write_aof_tail(root: &Path) {
    let mut aof = AofWriter::open(root.join("aof.log")).expect("open official AOF writer");
    let entries = [
        RaftLogEntry::CreateCollection {
            collection_id: RECOVERED_COLLECTION.into(),
            req: recovered_schema(),
        },
        RaftLogEntry::Index {
            collection_id: RECOVERED_COLLECTION.into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: RECOVERED_EXTERNAL_ID.into(),
                    field: "email".into(),
                    value: FieldValue::String(RECOVERED_VALUE.into()),
                    version: None,
                }],
                request_id: None,
            },
        },
    ];
    for (index, entry) in entries.into_iter().enumerate() {
        aof.append((index + 1) as u64, &WalRecord::new(entry))
            .expect("append official AOF record");
    }
    aof.sync().expect("sync official AOF writer");
}

fn post_json(port: u16, path: &str, body: Value) -> Value {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build process E2E runtime")
        .block_on(async {
            let response = reqwest::Client::builder()
                .timeout(STARTUP_DEADLINE)
                .build()
                .expect("build process E2E HTTP client")
                .post(format!("http://127.0.0.1:{port}{path}"))
                .json(&body)
                .send()
                .await
                .expect("send replayed search request");
            assert!(
                response.status().is_success(),
                "replayed search must return 2xx: {}",
                response.status()
            );
            response.json().await.expect("decode search JSON")
        })
}

fn assert_checkpoint_document_is_searchable(process: &LumenProcess) {
    let body = post_json(
        process.port,
        &format!("/collections/{CHECKPOINT_COLLECTION}/search"),
        json!({
            "query": { "term": { "field": "email", "value": CHECKPOINT_VALUE } },
            "limit": 10
        }),
    );
    assert_eq!(body["total"], 1, "checkpoint search result: {body}");
    assert_eq!(
        body["hits"][0]["external_id"], CHECKPOINT_EXTERNAL_ID,
        "checkpoint search result: {body}"
    );
}

#[test]
fn fresh_root_starts_and_logs_empty_initialization() {
    let root = tempfile::tempdir().expect("fresh segment root");
    let mut process = LumenProcess::spawn(root.path());
    process.wait_until_ready();

    assert_eq!(
        std::fs::read(root.path().join("CURRENT")).expect("read fresh CURRENT"),
        b"empty\n"
    );
    let logs = process.stop_and_logs();
    assert!(
        logs.contains("segment checkpoint startup decision")
            && logs.contains("initialized_empty_root"),
        "fresh root must log its explicit empty-root decision:\n{logs}"
    );
}

#[test]
fn unknown_root_refuses_before_current_or_listener() {
    let root = tempfile::tempdir().expect("unknown segment root");
    let direct_sentinel = b"keep this direct foreign file unchanged\n";
    let direct_foreign = root.path().join("alpha-foreign");
    std::fs::write(&direct_foreign, direct_sentinel).expect("write direct foreign sentinel");
    let foreign = root.path().join("foreign-layout");
    std::fs::create_dir(&foreign).expect("create unknown root entry");
    let sentinel = b"keep this foreign root unchanged\n";
    std::fs::write(foreign.join("sentinel"), sentinel).expect("write foreign sentinel");

    let logs = assert_refuses_without_mutation(root.path(), "unknown directory");

    assert_eq!(
        std::fs::read(foreign.join("sentinel")).expect("read foreign sentinel"),
        sentinel
    );
    assert_eq!(
        std::fs::read(&direct_foreign).expect("read direct foreign sentinel"),
        direct_sentinel
    );
    assert!(
        logs.contains("unrecognized non-empty segment checkpoint root entry")
            && logs.contains("refusing to initialize CURRENT")
            && logs.contains(
                "[alpha-foreign (regular file), foreign-layout (directory)]"
            ),
        "unknown root refusal must name every sorted direct child and the fail-closed reason:\n{logs}"
    );
}

#[test]
fn aof_only_root_starts_and_logs_recovered_empty_baseline() {
    let root = tempfile::tempdir().expect("AOF-only segment root");
    std::fs::write(root.path().join("aof.log"), b"").expect("write empty AOF");

    let mut process = LumenProcess::spawn(root.path());
    process.wait_until_ready();

    assert_eq!(
        std::fs::read(root.path().join("CURRENT")).expect("read AOF-only CURRENT"),
        b"empty\n"
    );
    let logs = process.stop_and_logs();
    assert!(
        logs.contains("segment checkpoint startup decision")
            && logs.contains("recovered_uncommitted_empty"),
        "AOF-only root must log its checkpoint decision:\n{logs}"
    );
    assert!(
        logs.contains("AOF startup decision") && logs.contains("no_tail"),
        "AOF-only root must log that the valid AOF had no replay tail:\n{logs}"
    );
}

#[test]
fn aof_only_root_replays_records_into_the_real_process() {
    let root = tempfile::tempdir().expect("AOF-tail-only segment root");
    write_aof_tail(root.path());

    let mut process = LumenProcess::spawn(root.path());
    process.wait_until_ready();
    let body = post_json(
        process.port,
        &format!("/collections/{RECOVERED_COLLECTION}/search"),
        json!({
            "query": { "term": { "field": "email", "value": RECOVERED_VALUE } },
            "limit": 10
        }),
    );

    assert_eq!(body["total"], 1, "replayed search result: {body}");
    assert_eq!(
        body["hits"][0]["external_id"], RECOVERED_EXTERNAL_ID,
        "replayed search result: {body}"
    );
    assert_eq!(
        std::fs::read(root.path().join("CURRENT")).expect("read AOF-tail CURRENT"),
        b"empty\n"
    );
    let logs = process.stop_and_logs();
    assert!(
        logs.contains("segment checkpoint startup decision")
            && logs.contains("recovered_uncommitted_empty"),
        "AOF-tail root must log its checkpoint decision:\n{logs}"
    );
    assert!(
        logs.contains("AOF startup decision") && logs.contains("tail_replayed"),
        "AOF-tail root must log that it replayed the durable AOF tail:\n{logs}"
    );
}

#[test]
fn compact_aof_temp_without_aof_refuses_before_current_or_listener() {
    let root = tempfile::tempdir().expect("compact-temp-only segment root");
    let compact = root.path().join("aof.log.compact.tmp");
    let bytes = b"incomplete compact AOF bytes\n";
    std::fs::write(&compact, bytes).expect("write compact AOF temp");

    let mut process = LumenProcess::spawn(root.path());
    let (status, logs, ever_bound) = process.wait_for_refusal();

    assert!(
        !status.success(),
        "compact-only root exited successfully:\n{logs}"
    );
    assert!(
        !ever_bound,
        "compact-only root bound a listener before refusing startup:\n{logs}"
    );
    assert!(
        !root.path().join("CURRENT").exists(),
        "compact-only root must not receive an empty CURRENT pointer"
    );
    assert_eq!(
        std::fs::read(compact).expect("read compact AOF temp"),
        bytes
    );
    assert!(
        logs.contains("aof.log.compact.tmp requires regular aof.log beside it"),
        "compact-only root refusal must explain its required regular AOF:\n{logs}"
    );
}

#[test]
fn exact_0428_legacy_generation_is_adopted_and_searchable() {
    let root = tempfile::tempdir().expect("0.4.28 legacy segment root");
    write_legacy_0428_checkpoint(root.path(), 42);
    assert!(
        !root.path().join("CURRENT").exists(),
        "exact legacy fixture must have no current pointer"
    );

    let mut process = LumenProcess::spawn(root.path());
    process.wait_until_ready();
    assert_checkpoint_document_is_searchable(&process);
    assert_eq!(
        std::fs::read(root.path().join("CURRENT")).expect("read adopted CURRENT"),
        b"generation:gen-42\n"
    );
    let logs = process.stop_and_logs();
    assert!(
        logs.contains("segment checkpoint startup decision")
            && logs.contains("adopted_legacy_0428"),
        "exact legacy adoption must log its decision:\n{logs}"
    );
}

#[test]
fn legal_current_generation_restarts_with_searchable_data() {
    let root = tempfile::tempdir().expect("current generation segment root");
    let current = write_current_checkpoint(root.path(), 42);

    let mut process = LumenProcess::spawn(root.path());
    process.wait_until_ready();
    assert_checkpoint_document_is_searchable(&process);
    assert_eq!(
        std::fs::read(root.path().join("CURRENT")).expect("read restored CURRENT"),
        current
    );
    let logs = process.stop_and_logs();
    assert!(
        logs.contains("segment checkpoint startup decision")
            && logs.contains("restored_current_generation"),
        "legal CURRENT restart must log its restore decision:\n{logs}"
    );
}

#[test]
fn regular_stale_current_temp_is_ignored_without_losing_current_data() {
    let root = tempfile::tempdir().expect("stale CURRENT.tmp segment root");
    let current = write_current_checkpoint(root.path(), 42);
    let current_temp = root.path().join("CURRENT.tmp");
    std::fs::copy(root.path().join("CURRENT"), &current_temp)
        .expect("copy generated pointer to stale CURRENT.tmp");

    let mut process = LumenProcess::spawn(root.path());
    process.wait_until_ready();
    assert_checkpoint_document_is_searchable(&process);
    assert_eq!(
        std::fs::read(root.path().join("CURRENT")).expect("read restored CURRENT"),
        current
    );
    assert_eq!(
        std::fs::read(&current_temp).expect("read stale CURRENT.tmp"),
        current,
        "startup must ignore the stale pointer instead of rewriting the active pointer"
    );
    let logs = process.stop_and_logs();
    assert!(
        logs.contains("segment checkpoint startup decision")
            && logs.contains("restored_current_generation"),
        "stale CURRENT.tmp restart must log its restore decision:\n{logs}"
    );
}

#[cfg(unix)]
#[test]
fn root_symlink_refuses_before_listener_or_mutation() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().expect("symlink segment root");
    let target = tempfile::tempdir().expect("symlink target");
    symlink(target.path(), root.path().join("foreign-link")).expect("create root symlink");

    let logs = assert_refuses_without_mutation(root.path(), "root symlink");
    assert!(
        logs.contains("foreign-link (symlink)")
            && logs.contains("unrecognized non-empty segment checkpoint root entry"),
        "symlink refusal must list its root entry and reason:\n{logs}"
    );
}

#[test]
fn unpointed_revision_generation_refuses_before_listener_or_mutation() {
    let root = tempfile::tempdir().expect("unpointed revision segment root");
    write_current_checkpoint(root.path(), 42);
    std::fs::remove_file(root.path().join("CURRENT"))
        .expect("remove current pointer for crash fixture");

    let logs = assert_refuses_without_mutation(root.path(), "unpointed revision generation");
    assert!(
        logs.contains("unpointed revision generation")
            && logs.contains("refusing to select or initialize it"),
        "unpointed revision refusal must name the pointer failure:\n{logs}"
    );
}

#[test]
fn exact_legacy_beside_unknown_content_refuses_before_listener_or_mutation() {
    let root = tempfile::tempdir().expect("mixed legacy segment root");
    write_legacy_0428_checkpoint(root.path(), 42);
    let foreign = root.path().join("foreign-layout");
    std::fs::create_dir(&foreign).expect("create mixed unknown root entry");
    std::fs::write(foreign.join("sentinel"), b"mixed root must survive")
        .expect("write mixed root sentinel");

    let logs = assert_refuses_without_mutation(root.path(), "legacy plus unknown content");
    assert!(
        logs.contains("gen-42 (directory)")
            && logs.contains("foreign-layout (directory)")
            && logs.contains("unrecognized non-empty segment checkpoint root entry"),
        "mixed legacy root must list both direct entries and refuse adoption:\n{logs}"
    );
}
