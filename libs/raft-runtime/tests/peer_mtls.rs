// HANDWRITE-BEGIN gap="missing-generator:unit-test:6c364d29" tracker="#1643" reason="Ephemeral CA/certificate tests for mutual success, hostname mismatch, untrusted client rejection, and explicit reload preservation."
use std::path::PathBuf;
use std::time::Duration;

use axum::routing::get;
use axum::Router;
use raft_runtime::PeerTransport;
use rcgen::{
    date_time_ymd, BasicConstraints, Certificate, CertificateParams, DnType,
    ExtendedKeyUsagePurpose, IsCa, KeyPair,
};
use tempfile::TempDir;
use tokio::sync::oneshot;

struct Authority {
    cert: Certificate,
    key: KeyPair,
}

impl Authority {
    fn generate(name: &str) -> Self {
        let mut params = CertificateParams::new(Vec::new()).unwrap();
        params.distinguished_name.push(DnType::CommonName, name);
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        let key = KeyPair::generate().unwrap();
        let cert = params.self_signed(&key).unwrap();
        Self { cert, key }
    }
}

struct Material {
    _dir: TempDir,
    config: peer_tls::PeerTlsConfig,
}

fn material(signer: &Authority, trust: &Authority, identity: &str, expired: bool) -> Material {
    let mut params = CertificateParams::new(vec![identity.to_owned()]).unwrap();
    params.distinguished_name.push(DnType::CommonName, identity);
    params.extended_key_usages = vec![
        ExtendedKeyUsagePurpose::ServerAuth,
        ExtendedKeyUsagePurpose::ClientAuth,
    ];
    if expired {
        params.not_before = date_time_ymd(2019, 1, 1);
        params.not_after = date_time_ymd(2020, 1, 1);
    }
    let key = KeyPair::generate().unwrap();
    let cert = params.signed_by(&key, &signer.cert, &signer.key).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let cert_path = dir.path().join("cert.pem");
    let key_path = dir.path().join("key.pem");
    let ca_path = dir.path().join("ca.pem");
    std::fs::write(&cert_path, cert.pem()).unwrap();
    std::fs::write(&key_path, key.serialize_pem()).unwrap();
    std::fs::write(&ca_path, trust.cert.pem()).unwrap();
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

async fn request(
    server: PeerTransport,
    client: &PeerTransport,
) -> Result<(reqwest::StatusCode, String), reqwest::Error> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let task = tokio::spawn(async move {
        server
            .serve(
                listener,
                Router::new().route("/raftz", get(|| async { "authenticated" })),
                async move {
                    let _ = shutdown_rx.await;
                },
            )
            .await
    });

    let result = client
        .http_client()
        .get(format!("https://localhost:{}/raftz", addr.port()))
        .timeout(Duration::from_secs(3))
        .send()
        .await;
    let result = match result {
        Ok(response) => {
            let status = response.status();
            response.text().await.map(|body| (status, body))
        }
        Err(error) => Err(error),
    };
    let _ = shutdown_tx.send(());
    task.await.unwrap().unwrap();
    result
}

#[tokio::test]
async fn trusted_mutual_peers_reach_the_http2_router() {
    let ca = Authority::generate("peer test ca");
    let server_material = material(&ca, &ca, "localhost", false);
    let client_material = material(&ca, &ca, "client.local", false);
    let server = PeerTransport::from_config(&server_material.config).unwrap();
    let client = PeerTransport::from_config(&client_material.config).unwrap();

    let (status, body) = request(server, &client).await.unwrap();
    assert_eq!(status, reqwest::StatusCode::OK);
    assert_eq!(body, "authenticated");
}

#[tokio::test]
async fn hostname_mismatch_fails_before_http_dispatch() {
    let ca = Authority::generate("peer test ca");
    let server_material = material(&ca, &ca, "not-localhost.invalid", false);
    let client_material = material(&ca, &ca, "client.local", false);
    let server = PeerTransport::from_config(&server_material.config).unwrap();
    let client = PeerTransport::from_config(&client_material.config).unwrap();

    assert!(request(server, &client).await.is_err());
}

#[tokio::test]
async fn untrusted_client_and_expired_server_fail_closed() {
    let trusted = Authority::generate("trusted ca");
    let untrusted = Authority::generate("untrusted ca");
    let server_material = material(&trusted, &trusted, "localhost", false);
    let bad_client_material = material(&untrusted, &trusted, "client.local", false);
    let server = PeerTransport::from_config(&server_material.config).unwrap();
    let bad_client = PeerTransport::from_config(&bad_client_material.config).unwrap();
    assert!(request(server, &bad_client).await.is_err());

    let expired_server_material = material(&trusted, &trusted, "localhost", true);
    let good_client_material = material(&trusted, &trusted, "client.local", false);
    let expired_server = PeerTransport::from_config(&expired_server_material.config).unwrap();
    let good_client = PeerTransport::from_config(&good_client_material.config).unwrap();
    assert!(request(expired_server, &good_client).await.is_err());
}

#[tokio::test]
async fn reload_is_atomic_and_preserves_last_known_good_on_error() {
    let ca1 = Authority::generate("first ca");
    let first = material(&ca1, &ca1, "localhost", false);
    let transport = PeerTransport::from_config(&first.config).unwrap();
    assert_eq!(transport.generation(), 1);

    let mut invalid = first.config.clone();
    invalid.cert = PathBuf::from("/definitely/missing-peer-cert.pem");
    assert!(transport.reload(&invalid).is_err());
    assert_eq!(transport.generation(), 1);
    let (_, body) = request(transport.clone(), &transport).await.unwrap();
    assert_eq!(body, "authenticated");

    let ca2 = Authority::generate("rotated ca");
    let rotated = material(&ca2, &ca2, "localhost", false);
    assert_eq!(transport.reload(&rotated.config).unwrap(), 2);
    let (_, body) = request(transport.clone(), &transport).await.unwrap();
    assert_eq!(body, "authenticated");
}
// HANDWRITE-END
