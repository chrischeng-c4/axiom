// HANDWRITE-BEGIN gap="missing-generator:e2e-test:defer-peer-mtls" tracker="#766" reason="Defer Raft integration over the shared authenticated peer listener."
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::Utc;
use defer::{CreateTask, DeferRaft, DeferScheduler, QueuePolicy, Target, TaskStatus};
use raft_runtime::{Membership, PeerTransport};
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa,
    KeyPair,
};
use tokio::sync::oneshot;

struct Authority {
    cert: Certificate,
    key: KeyPair,
}

fn authority() -> Authority {
    let mut params = CertificateParams::new(Vec::new()).unwrap();
    params
        .distinguished_name
        .push(DnType::CommonName, "Defer test CA");
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    let key = KeyPair::generate().unwrap();
    let cert = params.self_signed(&key).unwrap();
    Authority { cert, key }
}

struct Material {
    _dir: tempfile::TempDir,
    config: peer_tls::PeerTlsConfig,
}

fn material(ca: &Authority) -> Material {
    let mut params = CertificateParams::new(vec!["localhost".into()]).unwrap();
    params
        .distinguished_name
        .push(DnType::CommonName, "localhost");
    params.extended_key_usages = vec![
        ExtendedKeyUsagePurpose::ServerAuth,
        ExtendedKeyUsagePurpose::ClientAuth,
    ];
    let key = KeyPair::generate().unwrap();
    let cert = params.signed_by(&key, &ca.cert, &ca.key).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let cert_path = dir.path().join("tls.crt");
    let key_path = dir.path().join("tls.key");
    let ca_path = dir.path().join("ca.crt");
    std::fs::write(&cert_path, cert.pem()).unwrap();
    std::fs::write(&key_path, key.serialize_pem()).unwrap();
    std::fs::write(&ca_path, ca.cert.pem()).unwrap();
    Material {
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
    raft: Arc<DeferRaft>,
    _dir: tempfile::TempDir,
    shutdown: oneshot::Sender<()>,
    serve: tokio::task::JoinHandle<anyhow::Result<()>>,
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn trusted_defer_peers_replicate_scheduler_state_over_mtls() {
    let material = material(&authority());
    let mut listeners = Vec::new();
    let mut urls = HashMap::new();
    for id in 0..3u64 {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        urls.insert(
            id,
            format!(
                "https://localhost:{}",
                listener.local_addr().unwrap().port()
            ),
        );
        listeners.push(listener);
    }
    let membership = Membership {
        voters: vec![0, 1, 2],
        learners: vec![],
    };
    let mut nodes = Vec::new();
    for (id, listener) in listeners.into_iter().enumerate() {
        let peers = urls
            .iter()
            .filter(|(peer, _)| **peer != id as u64)
            .map(|(peer, url)| (*peer, url.clone()))
            .collect();
        let transport = PeerTransport::from_config(&material.config).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let raft = Arc::new(
            DeferRaft::spawn_with_peer_transport(
                Arc::new(Mutex::new(DeferScheduler::new())),
                dir.path(),
                id as u64,
                membership.clone(),
                peers,
                DeferRaft::host_config(8),
                transport.clone(),
            )
            .unwrap(),
        );
        let router = raft.router();
        let (shutdown, rx) = oneshot::channel();
        let serve = tokio::spawn(async move {
            transport
                .serve(listener, router, async move {
                    let _ = rx.await;
                })
                .await
        });
        nodes.push(Node {
            raft,
            _dir: dir,
            shutdown,
            serve,
        });
    }

    let deadline = Instant::now() + Duration::from_secs(8);
    let leader = loop {
        let mut leader = None;
        for (id, node) in nodes.iter().enumerate() {
            if node.raft.is_leader().await {
                leader = Some(id);
                break;
            }
        }
        if let Some(leader) = leader {
            break leader;
        }
        assert!(Instant::now() < deadline, "secure group did not elect");
        tokio::time::sleep(Duration::from_millis(25)).await;
    };
    nodes[leader]
        .raft
        .configure_queue("jobs".into(), QueuePolicy::default())
        .await
        .unwrap();
    nodes[leader]
        .raft
        .create_task(
            "jobs".into(),
            CreateTask {
                task_id: "secure".into(),
                target: Target {
                    url: "https://target.test".into(),
                    method: "POST".into(),
                    headers: Default::default(),
                },
                payload: serde_json::json!({"secure": true}),
                schedule_at: Utc::now(),
                priority: 10,
                max_attempts: 3,
            },
        )
        .await
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if nodes.iter().all(|node| {
            matches!(
                node.raft
                    .scheduler()
                    .lock()
                    .unwrap()
                    .status("jobs", "secure")
                    .unwrap(),
                Some(TaskStatus::Scheduled)
            )
        }) {
            break;
        }
        assert!(Instant::now() < deadline, "secure state did not converge");
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    for node in nodes {
        let _ = node.shutdown.send(());
        node.serve.await.unwrap().unwrap();
    }
}
// HANDWRITE-END
