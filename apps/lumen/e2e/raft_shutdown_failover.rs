//! #4069: a Lumen Raft shutdown is one observable, bounded transaction.
//!
//! The runtime already returns a terminal Raft shutdown report.  This target
//! first drives real `RaftHost` members with Lumen's `EngineSm` as a control.
//! It then stops the packaged Lumen process.  The process must publish that
//! report before it closes its authenticated peer listener.
//!
//! `--grace-secs` is the caller-visible shutdown bound.  Test watchdogs only
//! reap a child that would otherwise leak.  They do not assert a product SLA.
//! The process uses a fresh private CA and an mTLS leaf, so no cleartext peer
//! path can satisfy this contract.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::net::TcpListener as StdTcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use lumen::log_entry::RaftLogEntry;
use lumen::raft_sm::EngineSm;
use lumen::storage::Engine;
use lumen::types::{
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest, QueryNode,
    SearchRequest, TermQuery,
};
use lumen::wal::WalRecord;
use raft_runtime::{
    FsyncPolicy, HostConfig, LeadershipHandoff, Membership, NodeId, PhaseStatus, ProposalOutcome,
    RaftHost, RaftStateMachine, RaftStore,
};
use serde_json::{json, Value};
use server_lifecycle::ShutdownDeadline;
use tempfile::{NamedTempFile, TempDir};
use tokio::net::TcpListener;

const COLLECTION: &str = "docs";
const FIELD: &str = "kw";
const ACKNOWLEDGED_ID: &str = "acknowledged-before-shutdown";
const ACKNOWLEDGED_VALUE: &str = "must-survive-raft-failover";
const TEST_WATCHDOG: Duration = Duration::from_secs(15);
const EXIT_SCHEDULING_ALLOWANCE: Duration = Duration::from_secs(2);
const POLL_INTERVAL: Duration = Duration::from_millis(25);

fn schema() -> CreateCollectionRequest {
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

fn create_collection_command() -> Vec<u8> {
    WalRecord::new(RaftLogEntry::CreateCollection {
        collection_id: COLLECTION.to_owned(),
        req: schema(),
    })
    .encode()
    .expect("encode collection command")
}

fn index_command() -> Vec<u8> {
    WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items: vec![IndexItem {
                external_id: ACKNOWLEDGED_ID.to_owned(),
                field: FIELD.to_owned(),
                value: FieldValue::String(ACKNOWLEDGED_VALUE.to_owned()),
                version: None,
            }],
            request_id: None,
        },
    })
    .encode()
    .expect("encode acknowledged index command")
}

fn keyword_total(engine: &Engine) -> u64 {
    engine
        .search(
            COLLECTION,
            SearchRequest {
                query: QueryNode::Term(TermQuery {
                    field: FIELD.to_owned(),
                    value: FieldValue::String(ACKNOWLEDGED_VALUE.to_owned()),
                }),
                limit: 10,
                offset: 0,
                cursor: None,
                routing_key: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .expect("search acknowledged Raft document")
        .total
}

fn host_config() -> HostConfig {
    HostConfig {
        tick: Duration::from_millis(10),
        ..HostConfig::default()
    }
}

struct LumenRaftNode {
    id: NodeId,
    host: Option<Arc<RaftHost>>,
    engine: Arc<Engine>,
    sm: Arc<EngineSm>,
    peer_task: tokio::task::JoinHandle<()>,
    data: TempDir,
}

async fn real_lumen_cluster(size: u64) -> Vec<LumenRaftNode> {
    let mut listeners = Vec::new();
    let mut addresses = Vec::new();
    for id in 0..size {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind real Raft peer listener");
        let port = listener.local_addr().expect("peer listener address").port();
        listeners.push(listener);
        addresses.push((id, format!("http://127.0.0.1:{port}")));
    }

    let membership = Membership {
        voters: (0..size).collect(),
        learners: Vec::new(),
    };
    let config = host_config();
    listeners
        .into_iter()
        .enumerate()
        .map(|(position, listener)| {
            let id = position as NodeId;
            let engine = Arc::new(Engine::new());
            let sm = EngineSm::new(engine.clone(), 0);
            let data = TempDir::new().expect("temporary Raft data directory");
            let store = RaftStore::open(
                data.path().to_str().expect("UTF-8 Raft data path"),
                id,
                FsyncPolicy::Always,
            )
            .expect("open real Raft store");
            let peers = addresses
                .iter()
                .filter(|(peer, _)| *peer != id)
                .map(|(peer, address)| (*peer, address.clone()))
                .collect::<HashMap<_, _>>();
            let host = Arc::new(RaftHost::spawn(
                id,
                membership.clone(),
                peers,
                store,
                sm.clone() as Arc<dyn RaftStateMachine>,
                config,
            ));
            let router = host.router();
            let peer_task = tokio::spawn(async move {
                while let Ok((stream, _)) = listener.accept().await {
                    let router = router.clone();
                    tokio::spawn(async move {
                        let _ = transport_h2c::server::serve_connection(stream, router).await;
                    });
                }
            });
            LumenRaftNode {
                id,
                host: Some(host),
                engine,
                sm,
                peer_task,
                data,
            }
        })
        .collect()
}

async fn await_leader(nodes: &[LumenRaftNode], excluded: Option<usize>) -> usize {
    let deadline = Instant::now() + TEST_WATCHDOG;
    loop {
        for (index, node) in nodes.iter().enumerate() {
            if Some(index) == excluded {
                continue;
            }
            if let Some(host) = node.host.as_ref() {
                if host.is_leader().await {
                    return index;
                }
            }
        }
        assert!(
            Instant::now() < deadline,
            "real Lumen Raft members did not elect a leader"
        );
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

async fn wait_for_applied(nodes: &[LumenRaftNode], index: u64) {
    let deadline = Instant::now() + TEST_WATCHDOG;
    loop {
        if nodes.iter().all(|node| node.sm.applied_index() >= index) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "Raft acknowledgement {index} did not reach every Lumen EngineSm"
        );
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

/// This control uses real members and Lumen's state machine.  It proves that
/// the runtime can quiesce, transfer, report a safe peer close, and replay an
/// acknowledged write.  A process failure below is therefore an integration
/// failure in Lumen's own shutdown path.
#[tokio::test]
async fn real_raft_hosts_keep_an_acknowledged_lumen_write_through_failover_and_cold_recovery() {
    let mut nodes = real_lumen_cluster(3).await;
    let leader = await_leader(&nodes, None).await;
    let leader_host = Arc::clone(nodes[leader].host.as_ref().expect("leader host"));

    leader_host
        .propose(create_collection_command())
        .await
        .expect("create collection through Raft");
    let acknowledged_index = leader_host
        .propose(index_command())
        .await
        .expect("acknowledge Lumen write through Raft");
    wait_for_applied(&nodes, acknowledged_index).await;

    let config = host_config();
    let report = leader_host
        .shutdown_within(
            ShutdownDeadline::from_now(config.rpc_timeout + config.rpc_timeout, Duration::ZERO)
                .expect("valid Raft shutdown deadline"),
        )
        .await;
    assert_eq!(
        report.incomplete_phase, None,
        "shutdown must finish all phases"
    );
    assert!(
        report.peer_listener_close_safe,
        "the peer listener may close only after a terminal safe report"
    );
    assert!(
        report
            .phases
            .iter()
            .all(|phase| phase.status == PhaseStatus::Completed),
        "the report must record each completed shutdown phase: {report:?}"
    );
    assert!(
        matches!(report.handoff, LeadershipHandoff::Transferred { .. }),
        "a multi-voter leader must transfer before it exits: {report:?}"
    );
    assert!(
        matches!(
            leader_host.propose_outcome(index_command()).await,
            ProposalOutcome::RejectedBeforeAdmission { .. }
        ),
        "shutdown must quiesce proposals before it transfers leadership"
    );

    let failover_leader = await_leader(&nodes, Some(leader)).await;
    assert_eq!(
        keyword_total(&nodes[failover_leader].engine),
        1,
        "a live failover voter must retain the acknowledged write"
    );

    let stopped_id = nodes[leader].id;
    let stopped_data = nodes[leader].data.path().to_path_buf();
    nodes[leader].peer_task.abort();
    drop(leader_host);
    drop(nodes[leader].host.take().expect("stopped leader host"));

    let cold_engine = Arc::new(Engine::new());
    let cold_sm = EngineSm::new(cold_engine.clone(), 0);
    let cold_host = RaftHost::spawn(
        stopped_id,
        Membership {
            voters: vec![0, 1, 2],
            learners: Vec::new(),
        },
        HashMap::new(),
        RaftStore::open(
            stopped_data.to_str().expect("UTF-8 cold Raft path"),
            stopped_id,
            FsyncPolicy::Always,
        )
        .expect("cold reopen stopped Raft store"),
        cold_sm.clone() as Arc<dyn RaftStateMachine>,
        host_config(),
    );
    assert!(
        cold_sm.applied_index() >= acknowledged_index,
        "cold EngineSm must replay the acknowledged Raft index"
    );
    assert_eq!(
        keyword_total(&cold_engine),
        1,
        "cold recovery must retain the acknowledged write"
    );
    drop(cold_host);
    for node in &nodes {
        node.peer_task.abort();
    }
}

struct LumenProcess {
    child: Option<Child>,
    public_port: u16,
    raft_data: PathBuf,
    _root: TempDir,
    stdout: NamedTempFile,
    stderr: NamedTempFile,
}

impl LumenProcess {
    fn spawn(grace_secs: u64) -> Self {
        let root = tempfile::tempdir().expect("temporary Lumen process directory");
        let raft_data = root.path().join("raft");
        let (cert, key, ca) = write_peer_mtls(root.path());
        let public_port = reserve_port();
        let raft_port = reserve_port();
        assert_ne!(
            public_port, raft_port,
            "reserve separate public and peer ports"
        );
        let stdout = NamedTempFile::new().expect("Lumen stdout capture");
        let stderr = NamedTempFile::new().expect("Lumen stderr capture");
        let public_port_arg = public_port.to_string();
        let raft_port_arg = raft_port.to_string();
        let grace_arg = grace_secs.to_string();

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
                "--raft-port",
                &raft_port_arg,
                "--raft-data-dir",
            ])
            .arg(&raft_data)
            .args([
                "--grace-secs",
                &grace_arg,
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
            .env("LUMEN_PEER_TLS_CERT", cert)
            .env("LUMEN_PEER_TLS_KEY", key)
            .env("LUMEN_PEER_TLS_CA", ca)
            .env("LUMEN_PEER_MTLS", "on")
            .env_remove("LUMEN_PEERS")
            .env_remove("LUMEN_TLS")
            .env_remove("LUMEN_TLS_CERT")
            .env_remove("LUMEN_TLS_KEY")
            .env_remove("LUMEN_TLS_CA")
            .env_remove("LUMEN_TLS_SERVER_NAMES")
            .env_remove("RUST_LOG")
            .env_remove("LUMEN_LOG_FORMAT")
            .stdout(Stdio::from(stdout.reopen().expect("open stdout capture")))
            .stderr(Stdio::from(stderr.reopen().expect("open stderr capture")));
        let child = command.spawn().expect("spawn packaged Lumen binary");

        Self {
            child: Some(child),
            public_port,
            raft_data,
            _root: root,
            stdout,
            stderr,
        }
    }

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

    async fn wait_until_ready(&mut self, client: &reqwest::Client) {
        let deadline = Instant::now() + TEST_WATCHDOG;
        loop {
            if let Some(status) = self
                .child
                .as_mut()
                .expect("Lumen child is live")
                .try_wait()
                .expect("poll Lumen child")
            {
                panic!("Lumen exited before /readyz ({status}):\n{}", self.logs());
            }
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
                let logs = self.kill_and_logs().await;
                panic!("Lumen did not answer /readyz before test cleanup:\n{logs}");
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }

    #[cfg(unix)]
    fn signal_term(&mut self) {
        let pid = self.child.as_ref().expect("Lumen child is live").id() as libc::pid_t;
        assert_eq!(unsafe { libc::kill(pid, libc::SIGTERM) }, 0, "send SIGTERM");
    }

    async fn wait_for_exit(&mut self) -> ExitStatus {
        let deadline = Instant::now() + TEST_WATCHDOG;
        loop {
            if let Some(status) = self
                .child
                .as_mut()
                .expect("Lumen child is live")
                .try_wait()
                .expect("poll Lumen child")
            {
                self.child.take();
                return status;
            }
            if Instant::now() >= deadline {
                let logs = self.kill_and_logs().await;
                panic!("Lumen did not exit before test cleanup:\n{logs}");
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }

    async fn kill_and_logs(&mut self) -> String {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.logs()
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
    let listener = StdTcpListener::bind(("127.0.0.1", 0)).expect("reserve loopback port");
    listener.local_addr().expect("reserved port address").port()
}

fn write_peer_mtls(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let mut ca_params = rcgen::CertificateParams::new(Vec::new()).expect("CA parameters");
    ca_params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "lumen-e2e-raft-private-ca");
    ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    let ca_key = rcgen::KeyPair::generate().expect("CA key");
    let ca_cert = ca_params.self_signed(&ca_key).expect("self-sign CA");

    let mut leaf_params =
        rcgen::CertificateParams::new(vec!["lumen-0".to_owned()]).expect("leaf parameters");
    leaf_params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "lumen-0");
    leaf_params.extended_key_usages = vec![
        rcgen::ExtendedKeyUsagePurpose::ServerAuth,
        rcgen::ExtendedKeyUsagePurpose::ClientAuth,
    ];
    let leaf_key = rcgen::KeyPair::generate().expect("leaf key");
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

async fn acknowledge_write(client: &reqwest::Client, base: &str) {
    let deadline = Instant::now() + TEST_WATCHDOG;
    loop {
        let response = client
            .put(format!("{base}/collections/{COLLECTION}"))
            .json(&json!({"fields": {FIELD: {"type": "keyword"}}}))
            .send()
            .await;
        if response.as_ref().is_ok_and(|response| {
            response.status().is_success() || response.status().as_u16() == 409
        }) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "single-voter Lumen Raft process never accepted collection creation: {response:?}"
        );
        tokio::time::sleep(POLL_INTERVAL).await;
    }

    loop {
        let response = client
            .post(format!("{base}/collections/{COLLECTION}/index"))
            .json(&json!({"items": [{
                "external_id": ACKNOWLEDGED_ID,
                "field": FIELD,
                "value": ACKNOWLEDGED_VALUE,
            }]}))
            .send()
            .await;
        if response
            .as_ref()
            .is_ok_and(|response| response.status().is_success())
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "single-voter Lumen Raft process never acknowledged the write: {response:?}"
        );
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

fn cold_recover_process_write(raft_data: &Path) {
    let engine = Arc::new(Engine::new());
    let sm = EngineSm::new(engine.clone(), 0);
    let host = RaftHost::spawn(
        0,
        Membership {
            voters: vec![0],
            learners: Vec::new(),
        },
        HashMap::new(),
        RaftStore::open(
            raft_data.to_str().expect("UTF-8 process Raft path"),
            0,
            FsyncPolicy::Always,
        )
        .expect("cold open the process Raft store"),
        sm.clone() as Arc<dyn RaftStateMachine>,
        host_config(),
    );
    assert!(
        sm.applied_index() >= 2,
        "cold process store must replay the write"
    );
    assert_eq!(
        keyword_total(&engine),
        1,
        "acknowledged process write must survive cold recovery"
    );
    drop(host);
}

fn terminal_raft_report(logs: &str) -> Value {
    logs.lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| standard_event_name(event) == Some("raft_shutdown"))
        .unwrap_or_else(|| {
            panic!(
                "Lumen shutdown must publish a terminal raft_shutdown report before it closes the peer listener; logs:\n{logs}"
            )
        })
}

/// `service-observability` owns the JSON envelope.  `event` is its standard
/// root event name; application fields are retained only below `attributes`.
fn standard_event_name(event: &Value) -> Option<&str> {
    event.get("event").and_then(Value::as_str)
}

fn standard_attributes(event: &Value) -> &serde_json::Map<String, Value> {
    event
        .get("attributes")
        .and_then(Value::as_object)
        .unwrap_or_else(|| {
            panic!("raft_shutdown must use the standard JSON attributes envelope: {event}")
        })
}

fn optional_shutdown_attribute<'a>(event: &'a Value, key: &str) -> Option<&'a Value> {
    standard_attributes(event).get(key)
}

fn shutdown_attribute<'a>(event: &'a Value, key: &str) -> &'a Value {
    optional_shutdown_attribute(event, key).unwrap_or_else(|| {
        panic!("raft_shutdown must carry {key:?} in the standard JSON attributes envelope: {event}")
    })
}

/// The process report is the safe-close decision.  The peer listener must not
/// disappear before that decision reaches the observable JSON log stream.
fn assert_json_event_order(logs: &str, first: &str, later: &str) {
    let first_line = logs
        .lines()
        .enumerate()
        .find_map(|(line, raw)| {
            serde_json::from_str::<Value>(raw)
                .ok()
                .is_some_and(|event| standard_event_name(&event) == Some(first))
                .then_some(line)
        })
        .unwrap_or_else(|| panic!("missing JSON {first} event:\n{logs}"));
    let later_line = logs
        .lines()
        .enumerate()
        .skip(first_line + 1)
        .find_map(|(line, raw)| {
            serde_json::from_str::<Value>(raw)
                .ok()
                .is_some_and(|event| standard_event_name(&event) == Some(later))
                .then_some(line)
        })
        .unwrap_or_else(|| panic!("JSON {later} must follow {first}:\n{logs}"));
    assert!(
        first_line < later_line,
        "JSON {first} must precede {later}:\n{logs}"
    );
}

/// The packaged server must invoke its owned `RaftHost` during SIGTERM rather
/// than only draining HTTP and closing the peer listener.  The normal run
/// proves cold recovery.  The zero-grace run requires the named timeout phase.
#[cfg(unix)]
#[tokio::test]
async fn lumen_shutdown_reports_raft_quiesce_failover_and_timeout_boundaries() {
    let client = reqwest::Client::new();
    let mut normal = LumenProcess::spawn(1);
    normal.wait_until_ready(&client).await;
    acknowledge_write(&client, &normal.base_url()).await;
    let normal_started = Instant::now();
    normal.signal_term();
    let normal_status = normal.wait_for_exit().await;
    assert!(
        normal_started.elapsed() <= Duration::from_secs(1) + EXIT_SCHEDULING_ALLOWANCE,
        "a one-second single-voter shutdown must not fall through to an unrelated drain timeout"
    );
    assert!(
        normal_status.success(),
        "a single-voter graceful shutdown must exit cleanly: {}",
        normal.logs()
    );
    cold_recover_process_write(&normal.raft_data);
    let normal_report = terminal_raft_report(&normal.logs());
    assert_eq!(
        shutdown_attribute(&normal_report, "shutdown_budget_ms"),
        &json!(1_000)
    );
    assert_eq!(
        shutdown_attribute(&normal_report, "proposal_admission"),
        &json!("quiesced")
    );
    assert_eq!(
        shutdown_attribute(&normal_report, "handoff"),
        &json!("sole_voter")
    );
    assert!(
        optional_shutdown_attribute(&normal_report, "incomplete_phase")
            .map(Value::is_null)
            .unwrap_or(true),
        "a safe raft_shutdown report may omit incomplete_phase, but must not name one: {normal_report}"
    );
    assert_eq!(
        shutdown_attribute(&normal_report, "peer_listener_close_safe"),
        &json!(true)
    );
    assert_json_event_order(&normal.logs(), "raft_shutdown", "raft_peer_listener_closed");

    let mut expired = LumenProcess::spawn(0);
    expired.wait_until_ready(&client).await;
    let expired_started = Instant::now();
    expired.signal_term();
    let expired_status = expired.wait_for_exit().await;
    assert!(
        expired_started.elapsed() <= EXIT_SCHEDULING_ALLOWANCE,
        "a zero-grace single-voter shutdown must take the bounded abort path"
    );
    assert!(
        !expired_status.success(),
        "a timed-out single-voter shutdown must return a non-success result: {}",
        expired.logs()
    );
    let expired_report = terminal_raft_report(&expired.logs());
    assert_eq!(
        shutdown_attribute(&expired_report, "shutdown_budget_ms"),
        &json!(0)
    );
    assert_eq!(
        shutdown_attribute(&expired_report, "incomplete_phase"),
        &json!("quiesce")
    );
    assert_eq!(
        shutdown_attribute(&expired_report, "peer_listener_close_safe"),
        &json!(false)
    );
    assert_json_event_order(
        &expired.logs(),
        "raft_shutdown",
        "raft_peer_listener_aborted",
    );
}
