//! #3113 R9 / AC7: the client port's certificate changes under load.
//!
//! The claim being tested is a negative one — a routine renewal costs nothing.
//! No `terraform apply`, no rolling restart, no rebind of the listening socket,
//! and no request that fails because it happened to arrive during the swap.
//! Every one of those is easy to assert *around* and hard to assert *through*,
//! so these tests keep a bounded request stream running across the rotation and
//! count what came back.
//!
//! The listener is the real [`service_http::serve_tls`] the serving binary
//! calls, fed by the real [`lumen::tls::ServingTlsConfig`] seam, over material
//! laid out exactly as `spec.servingTlsSecret` projects it. What is simulated
//! is the certificate authority; what is exercised is everything downstream.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, KeyPair,
};

/// The Service DNS names the leaf answers to — both forms, because the operator
/// requests both and a client verifying either must succeed.
const SERVICE_DNS: [&str; 2] = ["search.acme.svc", "search.acme.svc.cluster.local"];

/// A throwaway certificate authority, standing in for the CAS pool.
struct Authority {
    cert: Certificate,
    key: KeyPair,
}

fn authority(name: &str) -> Authority {
    let mut params = CertificateParams::new(Vec::new()).unwrap();
    params.distinguished_name.push(DnType::CommonName, name);
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    let key = KeyPair::generate().unwrap();
    let cert = params.self_signed(&key).unwrap();
    Authority { cert, key }
}

/// Issue a serving leaf for [`SERVICE_DNS`], signed by `ca`.
fn leaf(ca: &Authority) -> (String, String) {
    let mut params =
        CertificateParams::new(SERVICE_DNS.iter().map(|n| (*n).to_string()).collect::<Vec<_>>())
            .unwrap();
    params
        .distinguished_name
        .push(DnType::CommonName, SERVICE_DNS[0]);
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    let key = KeyPair::generate().unwrap();
    let cert = params.signed_by(&key, &ca.cert, &ca.key).unwrap();
    (cert.pem(), key.serialize_pem())
}

/// Write `tls.crt` / `tls.key` / `ca.crt` into `dir`, the three keys the
/// projected Secret carries.
fn project(dir: &std::path::Path, cert_pem: &str, key_pem: &str, ca_pem: &str) {
    std::fs::write(dir.join("tls.crt"), cert_pem).unwrap();
    std::fs::write(dir.join("tls.key"), key_pem).unwrap();
    std::fs::write(dir.join("ca.crt"), ca_pem).unwrap();
}

fn serving_config(dir: &std::path::Path) -> lumen::tls::ServingTlsConfig {
    lumen::tls::ServingTlsConfig {
        cert: dir.join("tls.crt"),
        key: dir.join("tls.key"),
        ca: dir.join("ca.crt"),
        dns_names: SERVICE_DNS.iter().map(|n| (*n).to_string()).collect(),
    }
}

/// A client that dials the loopback socket while addressing — and verifying —
/// the Kubernetes Service name. This is the same shape `lumen connect` needs:
/// the URL carries the real identity so SNI and hostname verification target
/// it, and only the address resolution is redirected.
fn client(ca_pem: &str, port: u16) -> reqwest::Client {
    reqwest::Client::builder()
        .tls_built_in_root_certs(false)
        .add_root_certificate(reqwest::Certificate::from_pem(ca_pem.as_bytes()).unwrap())
        .resolve(
            SERVICE_DNS[0],
            format!("127.0.0.1:{port}").parse().unwrap(),
        )
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap()
}

/// AC7: a rotation changes the served fingerprint, and the bounded stream of
/// requests spanning it has no failures.
///
/// The stream is what makes this a proof rather than a before/after snapshot.
/// A listener that rebound its socket, or one that dropped in-flight
/// connections to pick up new material, would reach the same final fingerprint
/// and fail requests getting there.
#[tokio::test]
async fn a_serving_tls_rotation_keeps_every_request_in_a_bounded_stream() {
    lumen::tls::install_default_crypto_provider();
    let ca = authority("lumen-test-ca");
    let dir = tempfile::tempdir().unwrap();
    let (cert_a, key_a) = leaf(&ca);
    project(dir.path(), &cert_a, &key_a, &ca.cert.pem());

    let tls = Arc::new(serving_config(dir.path()).reloadable().unwrap());
    let first = tls.fingerprint().expect("a leaf is active at startup");

    let served = Arc::new(AtomicU64::new(0));
    let counter = served.clone();
    let app = axum::Router::new().route(
        "/healthz",
        axum::routing::get(move || {
            let counter = counter.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                "ok"
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let source = tls.clone();
    let server = tokio::spawn(async move {
        service_http::serve_tls(
            listener,
            app,
            service_http::config_source(move || source.server_config()),
            async move {
                let _ = shutdown_rx.await;
            },
        )
        .await;
    });

    // A fresh connection per request: pooling would hide the rotation behind a
    // connection established before it, and the property under test is about
    // what a *new* connection gets.
    let mut failures = Vec::new();
    let mut rotated_at = None;
    for i in 0..40u32 {
        if i == 20 {
            let (cert_b, key_b) = leaf(&ca);
            project(dir.path(), &cert_b, &key_b, &ca.cert.pem());
            let generation = tls.reload().expect("a leaf from the same CA must activate");
            rotated_at = Some(generation);
        }
        let client = client(&ca.cert.pem(), port);
        match client
            .get(format!("https://{}/healthz", SERVICE_DNS[0]))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {}
            Ok(response) => failures.push(format!("request {i}: status {}", response.status())),
            Err(error) => failures.push(format!("request {i}: {error}")),
        }
    }

    assert!(
        failures.is_empty(),
        "a routine renewal must cost no request: {failures:?}"
    );
    assert_eq!(served.load(Ordering::SeqCst), 40, "every request reached the router");
    assert_eq!(rotated_at, Some(2), "the rotation must have activated");
    let second = tls.fingerprint().expect("a leaf is active after rotation");
    assert_ne!(
        first, second,
        "the served identity must actually have changed"
    );

    // No restart: the same task, on the same socket, served both generations.
    assert!(
        !server.is_finished(),
        "the listener must survive the rotation"
    );
    let _ = shutdown_tx.send(());
    tokio::time::timeout(Duration::from_secs(5), server)
        .await
        .expect("listener shuts down")
        .expect("listener task");
}

/// The failure direction of the same seam: material the runtime cannot accept
/// is rejected without disturbing what is already serving.
///
/// This is what makes "zero failed requests" above mean something. A reload
/// that swallowed a bad leaf and served it would also report no failures — from
/// clients that had stopped verifying.
#[tokio::test]
async fn a_rejected_tls_rotation_leaves_the_previous_leaf_serving() {
    lumen::tls::install_default_crypto_provider();
    let ca = authority("lumen-test-ca");
    let dir = tempfile::tempdir().unwrap();
    let (cert_a, key_a) = leaf(&ca);
    project(dir.path(), &cert_a, &key_a, &ca.cert.pem());

    let tls = serving_config(dir.path()).reloadable().unwrap();
    let before = tls.fingerprint().expect("a leaf is active");

    // A leaf for a different Service, from an untrusted authority: exactly the
    // two ways a renewal goes wrong, arriving together.
    let stranger = authority("someone-elses-ca");
    let mut params = CertificateParams::new(vec!["other.acme.svc".to_string()]).unwrap();
    params
        .distinguished_name
        .push(DnType::CommonName, "other.acme.svc");
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    let key = KeyPair::generate().unwrap();
    let cert = params
        .signed_by(&key, &stranger.cert, &stranger.key)
        .unwrap();
    project(
        dir.path(),
        &cert.pem(),
        &key.serialize_pem(),
        &stranger.cert.pem(),
    );

    let rejection = tls
        .reload()
        .expect_err("a leaf for another Service must not activate");

    assert_eq!(
        tls.fingerprint().as_deref(),
        Some(before.as_str()),
        "the previous leaf must keep serving"
    );
    assert!(
        tls.server_config().is_some(),
        "a rejected reload must not take the listener out of service"
    );
    let message = rejection.to_string();
    assert!(
        !message.contains("PRIVATE KEY"),
        "a rejection must not carry key material: {message}"
    );
}
