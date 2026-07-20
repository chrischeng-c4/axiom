// HANDWRITE-BEGIN gap="missing-generator:e2e-test:relay-peer-mtls" tracker="#1209" reason="Relay Raft integration over the shared authenticated peer listener."
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use raft_runtime::Membership;
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa,
    KeyPair,
};
use relay::{PubCommand, Relay, RelayCoreConfig, RelayRaft};
use tokio::sync::oneshot;

struct Authority {
    cert: Certificate,
    key: KeyPair,
}

fn authority() -> Authority {
    let mut params = CertificateParams::new(Vec::new()).unwrap();
    params
        .distinguished_name
        .push(DnType::CommonName, "Relay test CA");
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    let key = KeyPair::generate().unwrap();
    let cert = params.self_signed(&key).unwrap();
    Authority { cert, key }
}

struct Material {
    _dir: tempfile::TempDir,
    config: relay::peer_tls::PeerTlsConfig,
}

fn material(ca: &Authority) -> Material {
    material_with_trust(ca, ca)
}

fn material_with_trust(identity_ca: &Authority, trust_ca: &Authority) -> Material {
    let mut params = CertificateParams::new(vec!["localhost".into()]).unwrap();
    params
        .distinguished_name
        .push(DnType::CommonName, "localhost");
    params.extended_key_usages = vec![
        ExtendedKeyUsagePurpose::ServerAuth,
        ExtendedKeyUsagePurpose::ClientAuth,
    ];
    let key = KeyPair::generate().unwrap();
    let cert = params
        .signed_by(&key, &identity_ca.cert, &identity_ca.key)
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let cert_path = dir.path().join("tls.crt");
    let key_path = dir.path().join("tls.key");
    let ca_path = dir.path().join("ca.crt");
    std::fs::write(&cert_path, cert.pem()).unwrap();
    std::fs::write(&key_path, key.serialize_pem()).unwrap();
    std::fs::write(&ca_path, trust_ca.cert.pem()).unwrap();
    Material {
        _dir: dir,
        config: relay::peer_tls::PeerTlsConfig {
            cert: cert_path,
            key: key_path,
            ca: ca_path,
            required: true,
        },
    }
}

struct Node {
    raft: Arc<RelayRaft>,
    engine: Arc<Relay>,
    _dir: tempfile::TempDir,
    shutdown: oneshot::Sender<()>,
    serve: tokio::task::JoinHandle<anyhow::Result<()>>,
}

// <HANDWRITE gap="missing-generator:unit-test" tracker="#2175" reason="unit-test section in raft_peer_mtls.rs is hand-written pending codegen support">
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn trusted_relay_peers_replicate_messages_over_mtls() {
    relay::tls::install_default_crypto_provider();
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
        let transport = material.config.peer_transport().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let engine = Arc::new(Relay::new(RelayCoreConfig::in_memory()));
        let raft = Arc::new(
            RelayRaft::spawn_with_peer_transport(
                engine.clone(),
                dir.path(),
                id as u64,
                membership.clone(),
                peers,
                RelayRaft::host_config(8),
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
            engine,
            _dir: dir,
            shutdown,
            serve,
        });
    }

    let deadline = Instant::now() + Duration::from_secs(8);
    let leader = loop {
        if let Some(leader) =
            futures::future::join_all(nodes.iter().map(|node| node.raft.is_leader()))
                .await
                .into_iter()
                .position(|is_leader| is_leader)
        {
            break leader;
        }
        assert!(Instant::now() < deadline, "secure group did not elect");
        tokio::time::sleep(Duration::from_millis(25)).await;
    };
    nodes[leader]
        .raft
        .publish(&PubCommand {
            subject: "secure".into(),
            message_id: "m1".into(),
            payload: serde_json::json!({"secure": true}),
            headers: Default::default(),
            priority: relay::DEFAULT_PRIORITY,
            not_before: None,
            appended_at: chrono::Utc::now(),
        })
        .await
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if nodes
            .iter()
            .all(|node| node.engine.log_len("secure").unwrap_or_default() == 1)
        {
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

#[tokio::test]
async fn untrusted_relay_peer_certificate_is_rejected() {
    relay::tls::install_default_crypto_provider();
    let trusted_ca = authority();
    let attacker_ca = authority();
    let server_material = material(&trusted_ca);
    // The attacker trusts Relay's legitimate server identity, so the only
    // failing direction is its own client certificate, signed by another CA.
    let attacker_material = material_with_trust(&attacker_ca, &trusted_ca);
    let server_transport = server_material.config.peer_transport().unwrap();
    let attacker_transport = attacker_material.config.peer_transport().unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let (server_result, _client_result) = tokio::time::timeout(Duration::from_secs(3), async {
        tokio::join!(
            async {
                let (stream, _) = listener.accept().await.unwrap();
                server_transport.accept(stream).await
            },
            async {
                let stream = tokio::net::TcpStream::connect(address).await.unwrap();
                attacker_transport.connect(stream, "localhost").await
            }
        )
    })
    .await
    .expect("untrusted peer handshake must terminate");

    let error = server_result.expect_err("required mTLS must reject attacker-CA client identity");
    assert!(
        error.to_string().contains("peer TLS server handshake"),
        "unexpected rejection: {error:#}"
    );
}
// </HANDWRITE>
// HANDWRITE-END
