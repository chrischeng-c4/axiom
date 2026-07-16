// HANDWRITE-BEGIN gap="missing-generator:peer-transport-integration-test:11d421bd" tracker="pending-tracker" reason="Exercise real TapeRaft replication over trusted shared mTLS transports and prove an untrusted certificate is rejected before the Raft router handles a request."
//! Tape's peer-transport adapter integration proof (#1805).
//!
//! The shared `raft-runtime` suite proves TLS handshakes in isolation. These
//! tests prove that Tape hands the same shared transport to its Raft hosts,
//! serves their routers on the authenticated listener, and never accepts a
//! peer that the configured CA does not trust.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use raft_runtime::{Membership, PeerTransport};
use rcgen::{BasicConstraints, Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, KeyPair};
use tape::peer_tls::PeerTlsConfig;
use tape::raft::{TapeOutcome, TapeRaft};
use tape::TapeJournal;
use tempfile::TempDir;
use tokio::sync::oneshot;

struct Authority {
    cert: Certificate,
    key: KeyPair,
}

impl Authority {
    fn generate(name: &str) -> Self {
        let mut params = CertificateParams::new(Vec::new()).expect("CA parameters");
        params.distinguished_name.push(DnType::CommonName, name);
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        let key = KeyPair::generate().expect("CA key");
        let cert = params.self_signed(&key).expect("CA certificate");
        Self { cert, key }
    }
}

struct Material {
    _dir: TempDir,
    config: PeerTlsConfig,
}

fn material(signer: &Authority, trust: &Authority, identity: &str) -> Material {
    let mut params = CertificateParams::new(vec![identity.to_owned()]).expect("peer parameters");
    params.distinguished_name.push(DnType::CommonName, identity);
    params.extended_key_usages = vec![
        ExtendedKeyUsagePurpose::ServerAuth,
        ExtendedKeyUsagePurpose::ClientAuth,
    ];
    let key = KeyPair::generate().expect("peer key");
    let cert = params
        .signed_by(&key, &signer.cert, &signer.key)
        .expect("peer certificate");
    let dir = tempfile::tempdir().expect("peer material directory");
    let cert_path = dir.path().join("cert.pem");
    let key_path = dir.path().join("key.pem");
    let ca_path = dir.path().join("ca.pem");
    std::fs::write(&cert_path, cert.pem()).expect("write certificate");
    std::fs::write(&key_path, key.serialize_pem()).expect("write key");
    std::fs::write(&ca_path, trust.cert.pem()).expect("write CA");
    Material {
        _dir: dir,
        config: PeerTlsConfig {
            cert: cert_path,
            key: key_path,
            ca: ca_path,
            required: true,
        },
    }
}

struct Node {
    raft: Arc<TapeRaft>,
    _dir: TempDir,
    shutdown: oneshot::Sender<()>,
    serve: tokio::task::JoinHandle<anyhow::Result<()>>,
}

struct SecureCluster {
    nodes: Vec<Node>,
}

impl SecureCluster {
    async fn start(replicas: u64, material: &Material) -> Self {
        let mut listeners = Vec::new();
        let mut urls = HashMap::new();
        for id in 0..replicas {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind peer listener");
            urls.insert(id, format!("https://localhost:{}", listener.local_addr().unwrap().port()));
            listeners.push(listener);
        }
        let membership = Membership {
            voters: (0..replicas).collect(),
            learners: vec![],
        };
        let mut nodes = Vec::new();
        for (id, listener) in listeners.into_iter().enumerate() {
            let node_id = id as u64;
            let peers = urls
                .iter()
                .filter(|(peer_id, _)| **peer_id != node_id)
                .map(|(peer_id, url)| (*peer_id, url.clone()))
                .collect();
            let journal = Arc::new(Mutex::new(TapeJournal::default()));
            let dir = tempfile::tempdir().expect("raft directory");
            let transport = PeerTransport::from_config(&material.config).expect("shared peer transport");
            let raft = Arc::new(
                TapeRaft::spawn_with_peer_transport(
                    journal,
                    dir.path(),
                    node_id,
                    membership.clone(),
                    peers,
                    TapeRaft::host_config(1024),
                    transport.clone(),
                )
                .expect("spawn tape raft with peer transport"),
            );
            let router = raft.router();
            let (shutdown, shutdown_rx) = oneshot::channel();
            let serve = tokio::spawn(async move {
                transport
                    .serve(listener, router, async move {
                        let _ = shutdown_rx.await;
                    })
                    .await
            });
            nodes.push(Node { raft, _dir: dir, shutdown, serve });
        }
        Self { nodes }
    }

    async fn leader(&self) -> usize {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            for (id, node) in self.nodes.iter().enumerate() {
                if node.raft.is_leader().await {
                    return id;
                }
            }
            assert!(Instant::now() < deadline, "no secure Raft leader elected");
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    async fn wait_converged(&self, want: u64) {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            if self
                .nodes
                .iter()
                .all(|node| node.raft.journal().lock().unwrap().end_offset("orders") >= want)
            {
                return;
            }
            assert!(Instant::now() < deadline, "secure peers did not replicate");
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    async fn shutdown(self) {
        for node in self.nodes {
            let _ = node.shutdown.send(());
            node.serve.await.expect("peer server task").expect("peer server result");
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn trusted_tape_raft_peers_replicate_over_mtls() {
    let ca = Authority::generate("tape trusted peer CA");
    let material = material(&ca, &ca, "localhost");
    let cluster = SecureCluster::start(3, &material).await;
    let leader = cluster.leader().await;
    let (_, outcome) = cluster.nodes[leader]
        .raft
        .propose_append(
            "orders".to_string(),
            None,
            serde_json::json!({"source": "mtls"}),
            100,
        )
        .await
        .expect("replicated append");
    assert!(matches!(outcome, Some(TapeOutcome::Appended(_))));
    cluster.wait_converged(1).await;
    cluster.shutdown().await;
}

#[tokio::test]
async fn untrusted_peer_is_rejected_before_tape_raft_router() {
    let trusted_ca = Authority::generate("trusted tape peer CA");
    let rogue_ca = Authority::generate("rogue tape peer CA");
    let server_material = material(&trusted_ca, &trusted_ca, "localhost");
    let rogue_material = material(&rogue_ca, &trusted_ca, "rogue.local");
    let server_transport = PeerTransport::from_config(&server_material.config).expect("server transport");
    let rogue_transport = PeerTransport::from_config(&rogue_material.config).expect("rogue transport");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind peer listener");
    let address = listener.local_addr().expect("peer listener address");
    let dir = tempfile::tempdir().expect("raft directory");
    let raft = Arc::new(
        TapeRaft::spawn_with_peer_transport(
            Arc::new(Mutex::new(TapeJournal::default())),
            dir.path(),
            0,
            Membership { voters: vec![0], learners: vec![] },
            HashMap::new(),
            TapeRaft::host_config(1024),
            server_transport.clone(),
        )
        .expect("spawn secure tape raft"),
    );
    let (shutdown, shutdown_rx) = oneshot::channel();
    let task = tokio::spawn(async move {
        server_transport
            .serve(listener, raft.router(), async move {
                let _ = shutdown_rx.await;
            })
            .await
    });

    let result = rogue_transport
        .http_client()
        .get(format!("https://localhost:{}/raftz", address.port()))
        .timeout(Duration::from_secs(3))
        .send()
        .await;
    assert!(result.is_err(), "untrusted client must not reach /raftz");
    let _ = shutdown.send(());
    task.await.expect("peer task").expect("peer result");
    drop(raft);
    drop(dir);
}
// HANDWRITE-END
