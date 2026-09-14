//! Black-box contract for the Lumen 0.6.1 Raft segment-snapshot serve wiring.
//!
//! A real `lumen serve --wal raft --persistence segment` process must publish
//! a durable `LSEGRAFT` v1 Raft artifact after an acknowledged write. A fresh
//! process using the same durable roots must restore that artifact and return
//! the exact indexed document through the public query API.
//!
//! # Facets
//!
//! - Behavior: the test asserts live Raft status, exact live and restarted queries, a published `LSEGRAFT` magic/version/index, and the recovered index. It runs under `cargo test -p lumen --features raft-wal --test raft_shutdown_failover --test raft_segment_snapshot_archive --test raft_segment_snapshot_archive_wiring` after the README gate is extended.
//! - Security: `apps/lumen/src/bin/lumen.rs` only selects the existing segment-aware archive reader and drives its existing periodic caller. The self-written archive boundary remains covered by `raft_segment_snapshot_archive`; this fixture uses a private temporary mTLS CA.
//! - Performance: the approved #4246 30-minute matrix in `apps/lumen/e2e/perf_gate.rs` is the performance oracle. This local restart contract adds no timing assertion; `WATCHDOG` only bounds fixture cleanup.

use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tempfile::{NamedTempFile, TempDir};

const ARCHIVE_MAGIC: &[u8] = b"LSEGRAFT";
const ARCHIVE_VERSION: u8 = 1;
const COLLECTION: &str = "raft-segment-serve";
const FIELD: &str = "keyword";
const DOCUMENT_ID: &str = "periodic-segment-snapshot";
const DOCUMENT_VALUE: &str = "lsegraft-live-write";
const WATCHDOG: Duration = Duration::from_secs(15);
const POLL_INTERVAL: Duration = Duration::from_millis(25);

struct RaftFixture {
    _root: TempDir,
    raft_data: PathBuf,
    segment_data: PathBuf,
    peer_cert: PathBuf,
    peer_key: PathBuf,
    peer_ca: PathBuf,
}

impl RaftFixture {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("create temporary Raft fixture root");
        let raft_data = root.path().join("raft");
        let segment_data = root.path().join("segments");
        let (peer_cert, peer_key, peer_ca) = write_peer_mtls(root.path());
        Self {
            _root: root,
            raft_data,
            segment_data,
            peer_cert,
            peer_key,
            peer_ca,
        }
    }

    fn spawn(&self) -> LumenProcess {
        let public_port = reserve_port();
        let raft_port = reserve_port();
        assert_ne!(
            public_port, raft_port,
            "reserve separate public and peer ports"
        );
        let stdout = NamedTempFile::new().expect("create Lumen stdout capture");
        let stderr = NamedTempFile::new().expect("create Lumen stderr capture");
        let public_port_arg = public_port.to_string();
        let raft_port_arg = raft_port.to_string();

        let mut command = Command::new(env!("CARGO_BIN_EXE_lumen"));
        command
            .args([
                "serve",
                "--host",
                "127.0.0.1",
                "--port",
                &public_port_arg,
                "--wal",
                "raft",
                "--persistence",
                "segment",
                "--data-dir",
            ])
            .arg(&self.segment_data)
            .arg("--raft-data-dir")
            .arg(&self.raft_data)
            .args([
                "--raft-port",
                &raft_port_arg,
                "--snapshot-secs",
                "1",
                "--log-format",
                "json",
                "--log-level",
                "info",
            ])
            .env("LUMEN_AUTH", "off")
            .env("POD_NAME", "lumen-0")
            .env("SHARD_COUNT", "1")
            .env("REPLICAS_PER_SHARD", "1")
            .env("VOTER_COUNT", "1")
            .env("LUMEN_HEADLESS_SERVICE", "lumen-headless")
            .env("LUMEN_PEER_TLS_CERT", &self.peer_cert)
            .env("LUMEN_PEER_TLS_KEY", &self.peer_key)
            .env("LUMEN_PEER_TLS_CA", &self.peer_ca)
            .env("LUMEN_PEER_MTLS", "on")
            .env_remove("LUMEN_PEERS")
            .env_remove("LUMEN_TLS")
            .env_remove("LUMEN_TLS_CERT")
            .env_remove("LUMEN_TLS_KEY")
            .env_remove("LUMEN_TLS_CA")
            .env_remove("LUMEN_TLS_SERVER_NAMES")
            .env_remove("LUMEN_DATA_DIR")
            .env_remove("LUMEN_RAFT_DATA_DIR")
            .env_remove("LUMEN_PERSISTENCE")
            .env_remove("LUMEN_SNAPSHOT_SECS")
            .env_remove("RUST_LOG")
            .env_remove("LUMEN_LOG_FORMAT")
            .stdout(Stdio::from(stdout.reopen().expect("open stdout capture")))
            .stderr(Stdio::from(stderr.reopen().expect("open stderr capture")));

        let child = command.spawn().expect("spawn packaged Lumen binary");
        LumenProcess {
            child: Some(child),
            public_port,
            stdout,
            stderr,
        }
    }
}

struct LumenProcess {
    child: Option<Child>,
    public_port: u16,
    stdout: NamedTempFile,
    stderr: NamedTempFile,
}

impl LumenProcess {
    fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.public_port)
    }

    fn logs(&self) -> String {
        format!(
            "{}\n{}",
            String::from_utf8_lossy(&fs::read(self.stdout.path()).expect("read stdout capture")),
            String::from_utf8_lossy(&fs::read(self.stderr.path()).expect("read stderr capture")),
        )
    }

    fn assert_running(&mut self) {
        let status = self
            .child
            .as_mut()
            .expect("Lumen child is live")
            .try_wait()
            .expect("poll Lumen child");
        if let Some(status) = status {
            self.child.take();
            panic!(
                "Lumen exited before the contract completed ({status}):\n{}",
                self.logs()
            );
        }
    }

    fn stop_and_logs(&mut self) -> String {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.logs()
    }

    async fn wait_until_ready(&mut self, client: &reqwest::Client) {
        let deadline = Instant::now() + WATCHDOG;
        loop {
            self.assert_running();
            if let Ok(response) = client
                .get(format!("{}/readyz", self.base_url()))
                .send()
                .await
            {
                if response.status().is_success() {
                    return;
                }
            }
            if Instant::now() >= deadline {
                let logs = self.stop_and_logs();
                panic!("Lumen did not answer /readyz before test cleanup:\n{logs}");
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
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
    listener.local_addr().expect("read reserved port").port()
}

fn write_peer_mtls(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let mut ca_params = rcgen::CertificateParams::new(Vec::new()).expect("create CA parameters");
    ca_params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "lumen-e2e-raft-private-ca");
    ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    let ca_key = rcgen::KeyPair::generate().expect("create CA key");
    let ca_cert = ca_params.self_signed(&ca_key).expect("self-sign CA");

    let mut leaf_params =
        rcgen::CertificateParams::new(vec!["lumen-0".to_owned()]).expect("create leaf parameters");
    leaf_params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "lumen-0");
    leaf_params.extended_key_usages = vec![
        rcgen::ExtendedKeyUsagePurpose::ServerAuth,
        rcgen::ExtendedKeyUsagePurpose::ClientAuth,
    ];
    let leaf_key = rcgen::KeyPair::generate().expect("create leaf key");
    let leaf_cert = leaf_params
        .signed_by(&leaf_key, &ca_cert, &ca_key)
        .expect("sign peer mTLS leaf");

    let cert = root.join("peer.crt");
    let key = root.join("peer.key");
    let ca = root.join("peer-ca.crt");
    fs::write(&cert, leaf_cert.pem()).expect("write peer certificate");
    fs::write(&key, leaf_key.serialize_pem()).expect("write peer key");
    fs::write(&ca, ca_cert.pem()).expect("write peer CA");
    (cert, key, ca)
}

async fn cluster_status(process: &mut LumenProcess, client: &reqwest::Client) -> Option<Value> {
    let response = client
        .get(format!("{}/debug/cluster", process.base_url()))
        .send()
        .await
        .ok()?;
    response.status().is_success().then_some(())?;
    response.json::<Value>().await.ok()
}

async fn wait_for_raft_leader(process: &mut LumenProcess, client: &reqwest::Client) -> Value {
    let deadline = Instant::now() + WATCHDOG;
    loop {
        process.assert_running();
        if let Some(status) = cluster_status(process, client).await {
            if status.get("role").and_then(Value::as_str) == Some("leader") {
                assert_eq!(
                    status.get("pod_name").and_then(Value::as_str),
                    Some("lumen-0"),
                    "public Raft status must describe the served node"
                );
                assert_eq!(
                    status
                        .get("peers")
                        .and_then(Value::as_array)
                        .map(|peers| peers.len()),
                    Some(1),
                    "the fixture must expose one real single-voter Raft member"
                );
                return status;
            }
        }
        if Instant::now() >= deadline {
            let logs = process.stop_and_logs();
            panic!("Lumen did not publish a leader through /debug/cluster:\n{logs}");
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

async fn acknowledge_document(process: &mut LumenProcess, client: &reqwest::Client) {
    let deadline = Instant::now() + WATCHDOG;
    let base = process.base_url();
    loop {
        process.assert_running();
        let collection_ready = match client
            .put(format!("{base}/collections/{COLLECTION}"))
            .json(&json!({"fields": {FIELD: {"type": "keyword"}}}))
            .send()
            .await
        {
            Ok(response) => response.status().is_success() || response.status().as_u16() == 409,
            Err(_) => false,
        };
        if collection_ready {
            if let Ok(response) = client
                .post(format!("{base}/collections/{COLLECTION}/index"))
                .json(&json!({"items": [{
                    "external_id": DOCUMENT_ID,
                    "field": FIELD,
                    "value": DOCUMENT_VALUE,
                }]}))
                .send()
                .await
            {
                if response.status().is_success() {
                    return;
                }
            }
        }
        if Instant::now() >= deadline {
            let logs = process.stop_and_logs();
            panic!("single-voter Raft process never acknowledged the test document:\n{logs}");
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

async fn wait_for_applied_index(
    process: &mut LumenProcess,
    client: &reqwest::Client,
    minimum: u64,
) -> u64 {
    let deadline = Instant::now() + WATCHDOG;
    loop {
        process.assert_running();
        if let Some(status) = cluster_status(process, client).await {
            let applied = status.get("applied_index").and_then(Value::as_u64);
            if let Some(applied) = applied.filter(|index| *index >= minimum) {
                assert_eq!(
                    status.get("role").and_then(Value::as_str),
                    Some("leader"),
                    "the public Raft status must stay leader while the write becomes visible"
                );
                return applied;
            }
        }
        if Instant::now() >= deadline {
            let logs = process.stop_and_logs();
            panic!("/debug/cluster never reached Raft index {minimum}:\n{logs}");
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

async fn assert_exact_query(process: &mut LumenProcess, client: &reqwest::Client, phase: &str) {
    process.assert_running();
    let response = client
        .post(format!(
            "{}/collections/{COLLECTION}/search",
            process.base_url()
        ))
        .json(&json!({
            "query": {"term": {"field": FIELD, "value": DOCUMENT_VALUE}},
            "limit": 10,
            "track_total": true,
        }))
        .send()
        .await
        .expect("query the live Lumen process");
    let status = response.status();
    let body = response
        .json::<Value>()
        .await
        .expect("decode Lumen query response");
    assert!(status.is_success(), "{phase}: search failed: {body}");
    assert_eq!(
        body.get("total").and_then(Value::as_u64),
        Some(1),
        "{phase}: the durable query total must contain exactly the acknowledged document: {body}"
    );
    let ids = body
        .get("hits")
        .and_then(Value::as_array)
        .expect("successful Lumen search has hits")
        .iter()
        .map(|hit| {
            hit.get("external_id")
                .and_then(Value::as_str)
                .expect("search hit has external_id")
                .to_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        vec![DOCUMENT_ID.to_owned()],
        "{phase}: the durable query must return only the acknowledged document"
    );
}

/// Read only named published Raft snapshot artifacts. This never opens a
/// second `RaftStore`, and it never touches a mutable `*.tmp` write path.
fn published_snapshot_artifacts(raft_data: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(raft_data) else {
        return Vec::new();
    };
    let mut paths = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            let name = path.file_name()?.to_str()?;
            (entry.file_type().ok()?.is_file()
                && name.starts_with("raft-0-snap-")
                && name.ends_with(".artifact"))
            .then_some(path)
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

/// `LSEGRAFT` v1 writes magic, version, then the little-endian captured index.
fn lsegraft_index(bytes: &[u8]) -> Option<u64> {
    if !bytes.starts_with(ARCHIVE_MAGIC) || bytes.get(8).copied() != Some(ARCHIVE_VERSION) {
        return None;
    }
    let raw: [u8; 8] = bytes.get(9..17)?.try_into().ok()?;
    Some(u64::from_le_bytes(raw))
}

/// The filename is the store's published Raft index. It is separate from the
/// archive's self-described sequence so a legacy-RDB source negative reaches
/// the magic assertion instead of succeeding through a timeout alone.
fn snapshot_index_from_artifact_name(path: &Path) -> Option<u64> {
    let name = path.file_name()?.to_str()?;
    let stem = name
        .strip_prefix("raft-0-snap-")?
        .strip_suffix(".artifact")?;
    let (index, _term) = stem.split_once('-')?;
    index.parse().ok()
}

async fn wait_for_published_snapshot_artifact(
    process: &mut LumenProcess,
    raft_data: &Path,
    minimum_index: u64,
) -> (PathBuf, Vec<u8>, u64) {
    let deadline = Instant::now() + WATCHDOG;
    loop {
        process.assert_running();
        let logs = process.logs();
        let paths = published_snapshot_artifacts(raft_data);
        for path in &paths {
            let Some(artifact_index) = snapshot_index_from_artifact_name(path) else {
                continue;
            };
            if artifact_index < minimum_index
                || !logs.contains("raft snapshot taken + log compacted")
            {
                continue;
            }
            let Ok(bytes) = fs::read(path) else {
                continue;
            };
            return (path.clone(), bytes, artifact_index);
        }
        if Instant::now() >= deadline {
            let logs = process.stop_and_logs();
            panic!(
                "Lumen never published a Raft snapshot artifact at or after Raft index {minimum_index}; \
                 found {paths:?}; logs:\n{logs}"
            );
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

/// The watchdogs bound child cleanup only. They do not assert a snapshot SLA.
#[tokio::test]
async fn raft_segment_serve_publishes_lsegraft_and_recovers_after_restart() {
    let fixture = RaftFixture::new();
    let client = reqwest::Client::builder()
        .timeout(WATCHDOG)
        .build()
        .expect("build fixture HTTP client");

    let mut first = fixture.spawn();
    first.wait_until_ready(&client).await;
    let initial_status = wait_for_raft_leader(&mut first, &client).await;
    let initial_index = initial_status
        .get("applied_index")
        .and_then(Value::as_u64)
        .expect("public Raft status has applied_index");

    acknowledge_document(&mut first, &client).await;
    let write_index = wait_for_applied_index(
        &mut first,
        &client,
        initial_index
            .checked_add(2)
            .expect("Raft index remains in range"),
    )
    .await;
    assert_exact_query(&mut first, &client, "before restart").await;

    let (artifact, bytes, artifact_index) =
        wait_for_published_snapshot_artifact(&mut first, &fixture.raft_data, write_index).await;
    assert!(
        artifact.is_file(),
        "the periodic snapshot must remain as a published immutable Raft artifact"
    );
    assert!(
        bytes.starts_with(ARCHIVE_MAGIC),
        "the published Raft artifact must use the documented LSEGRAFT magic"
    );
    assert_eq!(
        bytes.get(ARCHIVE_MAGIC.len()).copied(),
        Some(ARCHIVE_VERSION),
        "the published Raft artifact must use LSEGRAFT v1"
    );
    let snapshot_index = lsegraft_index(&bytes);
    assert!(
        snapshot_index.is_some(),
        "the LSEGRAFT header must carry its captured Raft index"
    );
    let snapshot_index = snapshot_index.expect("LSEGRAFT index was asserted present");
    assert_eq!(
        snapshot_index, artifact_index,
        "the LSEGRAFT header index must match the published Raft artifact index"
    );
    assert!(
        snapshot_index >= write_index,
        "the durable archive must cover the write visible through the live API"
    );

    let _first_logs = first.stop_and_logs();
    let mut restarted = fixture.spawn();
    restarted.wait_until_ready(&client).await;
    let _restart_status = wait_for_raft_leader(&mut restarted, &client).await;
    let restored_index = wait_for_applied_index(&mut restarted, &client, snapshot_index).await;
    assert!(
        restored_index >= snapshot_index,
        "the restarted binary must restore the published Raft snapshot before serving"
    );
    assert_exact_query(&mut restarted, &client, "after restart").await;
}
