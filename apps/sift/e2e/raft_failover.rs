//! Three durable Sift voters retain acknowledged data across leader loss.

use std::{collections::HashMap, sync::Arc, time::Duration};

use raft_runtime::{
    FsyncPolicy, HostConfig, Membership, PeerTransport, RaftHost, RaftStateMachine, RaftStore,
};
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa,
    KeyPair,
};
use sift::{durability::SiftStateMachine, DurableJournal, EventEnvelope, EventQuery, SignalKind};
use tempfile::TempDir;
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};

struct Authority {
    cert: Certificate,
    key: KeyPair,
}

impl Authority {
    fn generate() -> Self {
        let mut params = CertificateParams::new(Vec::new()).unwrap();
        params
            .distinguished_name
            .push(DnType::CommonName, "Sift test peer CA");
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        let key = KeyPair::generate().unwrap();
        let cert = params.self_signed(&key).unwrap();
        Self { cert, key }
    }
}

struct PeerMaterial {
    _dir: TempDir,
    config: peer_tls::PeerTlsConfig,
}

fn peer_material(authority: &Authority, id: u64) -> PeerMaterial {
    let mut params = CertificateParams::new(vec!["localhost".to_string()]).unwrap();
    params
        .distinguished_name
        .push(DnType::CommonName, format!("sift-peer-{id}"));
    params.extended_key_usages = vec![
        ExtendedKeyUsagePurpose::ServerAuth,
        ExtendedKeyUsagePurpose::ClientAuth,
    ];
    let key = KeyPair::generate().unwrap();
    let cert = params
        .signed_by(&key, &authority.cert, &authority.key)
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let cert_path = dir.path().join("tls.crt");
    let key_path = dir.path().join("tls.key");
    let ca_path = dir.path().join("ca.crt");
    std::fs::write(&cert_path, cert.pem()).unwrap();
    std::fs::write(&key_path, key.serialize_pem()).unwrap();
    std::fs::write(&ca_path, authority.cert.pem()).unwrap();
    PeerMaterial {
        _dir: dir,
        config: peer_tls::PeerTlsConfig {
            cert: cert_path,
            key: key_path,
            ca: ca_path,
            required: true,
        },
    }
}

struct Node {
    host: Option<Arc<RaftHost>>,
    journal: Arc<DurableJournal>,
    _state_machine: Arc<SiftStateMachine>,
    listener: JoinHandle<anyhow::Result<()>>,
    shutdown: Option<oneshot::Sender<()>>,
    _data: TempDir,
    _material: PeerMaterial,
}

fn event(id: &str) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        "project-a",
        "prod",
        id,
        SignalKind::Log,
        serde_json::json!({"message":"replicated"}),
    );
    event
        .resource
        .insert("service.name".to_string(), "sift-failover-test".to_string());
    event
}

fn command(event: EventEnvelope) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "kind":"append_events",
        "events":[event]
    }))
    .unwrap()
}

async fn wait_for_leader(nodes: &[Node]) -> usize {
    for _ in 0..400 {
        for (index, node) in nodes.iter().enumerate() {
            if let Some(host) = &node.host {
                if host.is_leader().await {
                    return index;
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("three Sift voters did not elect a leader");
}

async fn wait_for_event(nodes: &[Node], id: &str, required: usize) {
    for _ in 0..400 {
        let present = nodes
            .iter()
            .filter(|node| node.host.is_some() && has_event(&node.journal, id))
            .count();
        if present >= required {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("event {id} did not reach {required} live durable voters");
}

fn has_event(journal: &DurableJournal, id: &str) -> bool {
    journal
        .query(EventQuery {
            limit: 100,
            ..EventQuery::default()
        })
        .unwrap()
        .iter()
        .any(|stored| stored.event.event_id == id)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn acknowledged_data_survives_leader_loss_over_mutual_tls() {
    let authority = Authority::generate();
    let mut listeners = Vec::new();
    let mut urls = Vec::new();
    for _ in 0..3 {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        urls.push(format!(
            "https://localhost:{}",
            listener.local_addr().unwrap().port()
        ));
        listeners.push(listener);
    }

    let mut nodes = Vec::new();
    for (id, listener) in listeners.into_iter().enumerate() {
        let id = id as u64;
        let data = tempfile::tempdir().unwrap();
        let journal = Arc::new(DurableJournal::open(data.path()).unwrap());
        let state_machine = Arc::new(SiftStateMachine::open(data.path(), journal.clone()).unwrap());
        let material = peer_material(&authority, id);
        let transport = PeerTransport::from_config(&material.config).unwrap();
        let peers = urls
            .iter()
            .enumerate()
            .filter(|(peer, _)| *peer as u64 != id)
            .map(|(peer, url)| (peer as u64, url.clone()))
            .collect::<HashMap<_, _>>();
        let host = Arc::new(RaftHost::spawn_with_peer_transport(
            id,
            Membership {
                voters: vec![0, 1, 2],
                learners: Vec::new(),
            },
            peers,
            RaftStore::open(data.path().to_str().unwrap(), id, FsyncPolicy::Always).unwrap(),
            state_machine.clone() as Arc<dyn RaftStateMachine>,
            HostConfig::default(),
            transport.clone(),
        ));
        let router = host.router();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let listener_task = tokio::spawn(async move {
            transport
                .serve(listener, router, async {
                    let _ = shutdown_rx.await;
                })
                .await
        });
        nodes.push(Node {
            host: Some(host),
            journal,
            _state_machine: state_machine,
            listener: listener_task,
            shutdown: Some(shutdown_tx),
            _data: data,
            _material: material,
        });
    }

    let first_leader = wait_for_leader(&nodes).await;
    nodes[first_leader]
        .host
        .as_ref()
        .unwrap()
        .propose(command(event("before-failure")))
        .await
        .expect("two durable voters acknowledge the first batch");
    wait_for_event(&nodes, "before-failure", 3).await;

    nodes[first_leader].listener.abort();
    nodes[first_leader].shutdown.take();
    nodes[first_leader].host.take();

    let second_leader = wait_for_leader(&nodes).await;
    assert_ne!(second_leader, first_leader);
    nodes[second_leader]
        .host
        .as_ref()
        .unwrap()
        .propose(command(event("after-failure")))
        .await
        .expect("the surviving quorum accepts a second batch");
    wait_for_event(&nodes, "after-failure", 2).await;

    for node in nodes.iter().filter(|node| node.host.is_some()) {
        assert!(has_event(&node.journal, "before-failure"));
        assert!(has_event(&node.journal, "after-failure"));
    }

    for node in &mut nodes {
        if let Some(host) = node.host.take() {
            let _ = host.shutdown().await;
        }
        if let Some(shutdown) = node.shutdown.take() {
            let _ = shutdown.send(());
        }
    }
    for node in nodes {
        let _ = node.listener.await;
    }
}
