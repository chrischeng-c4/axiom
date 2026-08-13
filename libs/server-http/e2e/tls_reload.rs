//! Replacing the serving certificate under live HTTP/2 traffic (#3112 R1, R4,
//! R6, R7, AC1).
//!
//! The claim being tested is narrow and load-bearing: a leaf can be swapped
//! without the listener ever being rebound. So the port is captured once, at
//! bind time, and every assertion afterwards is made against that same number —
//! if the implementation ever rebinds, the address changes and the test cannot
//! quietly pass anyway.
//!
//! Requests are driven over real TLS with hyper rather than a helper, because
//! the two facts that matter — which certificate the server presented, and
//! whether the request completed — are only observable from a client that
//! actually handshook.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use axum::{routing::get, Router};
use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper_util::rt::{TokioExecutor, TokioIo};
use peer_tls::material::MaterialPem;
use peer_tls::reload::{MaterialSource, MemoryMaterialSource, ReloadableTls, TlsRuntimeProfile};
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose, Ia5String,
    IsCa, KeyPair, SanType,
};
use server_http::{config_source, serve_tls, TlsListenerMetrics, TlsServerOptions};
use tokio::net::{TcpListener, TcpStream};

const NAME: &str = "lumen.axiom.svc.cluster.local";

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

fn serving_material(ca: &Authority) -> MaterialPem {
    let mut params = CertificateParams::new(Vec::<String>::new()).unwrap();
    params
        .subject_alt_names
        .push(SanType::DnsName(Ia5String::try_from(NAME).unwrap()));
    params.distinguished_name.push(DnType::CommonName, NAME);
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    let key = KeyPair::generate().unwrap();
    let cert = params.signed_by(&key, &ca.cert, &ca.key).unwrap();
    MaterialPem::new(cert.pem(), key.serialize_pem(), ca.cert.pem())
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn client_config(ca: &Authority) -> Arc<rustls::ClientConfig> {
    peer_tls::install_default_crypto_provider();
    let mut roots = rustls::RootCertStore::empty();
    for anchor in rustls_pemfile_certs(&ca.cert.pem()) {
        roots.add(anchor).unwrap();
    }
    let mut config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    config.alpn_protocols = vec![b"h2".to_vec()];
    Arc::new(config)
}

fn rustls_pemfile_certs(pem: &str) -> Vec<rustls::pki_types::CertificateDer<'static>> {
    // The dependency is already here through rustls' own re-exports; parsing one
    // test anchor does not justify another crate in the manifest.
    let mut out = Vec::new();
    for block in pem.split("-----BEGIN CERTIFICATE-----").skip(1) {
        let body = block.split("-----END CERTIFICATE-----").next().unwrap();
        let base64: String = body.chars().filter(|c| !c.is_whitespace()).collect();
        out.push(rustls::pki_types::CertificateDer::from(
            base64_decode(&base64),
        ));
    }
    out
}

fn base64_decode(input: &str) -> Vec<u8> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut acc: u32 = 0;
    let mut bits = 0;
    let mut out = Vec::new();
    for byte in input.bytes() {
        if byte == b'=' {
            break;
        }
        let Some(value) = TABLE.iter().position(|c| *c == byte) else {
            continue;
        };
        acc = (acc << 6) | value as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    out
}

/// One request over a fresh TLS connection, reporting which leaf the server
/// presented.
async fn request(
    addr: std::net::SocketAddr,
    client: Arc<rustls::ClientConfig>,
) -> Result<(String, String), String> {
    let tcp = TcpStream::connect(addr).await.map_err(|e| e.to_string())?;
    let name = rustls::pki_types::ServerName::try_from(NAME).unwrap();
    let tls = tokio_rustls::TlsConnector::from(client)
        .connect(name, tcp)
        .await
        .map_err(|e| e.to_string())?;
    let presented = tls
        .get_ref()
        .1
        .peer_certificates()
        .and_then(|chain| chain.first())
        .map(|leaf| sha256_hex(leaf.as_ref()))
        .ok_or_else(|| "no server certificate".to_string())?;

    let (mut sender, connection) =
        hyper::client::conn::http2::handshake(TokioExecutor::new(), TokioIo::new(tls))
            .await
            .map_err(|e| e.to_string())?;
    let pump = tokio::spawn(async move {
        let _ = connection.await;
    });
    let response = sender
        .send_request(
            http::Request::builder()
                .uri(format!("https://{NAME}/healthz"))
                .body(Empty::<Bytes>::new())
                .unwrap(),
        )
        .await
        .map_err(|e| e.to_string())?;
    let body = response
        .into_body()
        .collect()
        .await
        .map_err(|e| e.to_string())?
        .to_bytes();
    drop(sender);
    let _ = pump.await;
    Ok((presented, String::from_utf8_lossy(&body).to_string()))
}

struct Fixture {
    addr: std::net::SocketAddr,
    tls: ReloadableTls,
    source: Arc<MemoryMaterialSource>,
    metrics: TlsListenerMetrics,
    shutdown: tokio::sync::oneshot::Sender<()>,
    server: tokio::task::JoinHandle<()>,
    served: Arc<AtomicBool>,
}

async fn fixture(ca: &Authority) -> Fixture {
    let source = Arc::new(MemoryMaterialSource::new(serving_material(ca)));
    let tls = ReloadableTls::required(
        TlsRuntimeProfile::serving([NAME.to_string()]),
        source.clone() as Arc<dyn MaterialSource>,
    )
    .expect("startup material");

    let served = Arc::new(AtomicBool::new(false));
    let flag = served.clone();
    let app = Router::new().route(
        "/healthz",
        get(move || {
            let flag = flag.clone();
            async move {
                flag.store(true, Ordering::SeqCst);
                "ok"
            }
        }),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let metrics = TlsListenerMetrics::new();
    let options = TlsServerOptions {
        metrics: metrics.clone(),
        ..Default::default()
    };
    let (shutdown, rx) = tokio::sync::oneshot::channel();
    let for_listener = tls.clone();
    let server = tokio::spawn(serve_tls(
        listener,
        app,
        config_source(move || for_listener.server_config()),
        options,
        async move {
            let _ = rx.await;
        },
    ));
    // Give the accept loop a moment to reach its first await.
    tokio::time::sleep(Duration::from_millis(50)).await;

    Fixture {
        addr,
        tls,
        source,
        metrics,
        shutdown,
        server,
        served,
    }
}

impl Fixture {
    async fn stop(self) {
        let _ = self.shutdown.send(());
        let _ = tokio::time::timeout(Duration::from_secs(5), self.server).await;
    }
}

#[tokio::test]
async fn tls_reload_swaps_the_serving_leaf_without_rebinding_the_listener() {
    let ca = authority("axiom-serving-ca");
    let f = fixture(&ca).await;
    let client = client_config(&ca);

    let (first_leaf, body) = request(f.addr, client.clone()).await.expect("first request");
    assert_eq!(body, "ok");
    assert_eq!(first_leaf, f.tls.fingerprint().unwrap());

    f.source.set(serving_material(&ca));
    f.tls.reload().expect("the renewed leaf activates");
    assert_eq!(f.tls.generation(), 2);

    // Same address, no rebind, no restart — and the next handshake gets the new
    // certificate. That is the entire feature.
    let (second_leaf, body) = request(f.addr, client).await.expect("post-rotation request");
    assert_eq!(body, "ok");
    assert_ne!(second_leaf, first_leaf);
    assert_eq!(second_leaf, f.tls.fingerprint().unwrap());
    assert!(f.served.load(Ordering::SeqCst));

    assert_eq!(f.metrics.snapshot().established, 2);
    assert_eq!(f.metrics.snapshot().refused_without_material, 0);
    f.stop().await;
}

#[tokio::test]
async fn tls_reload_keeps_requests_flowing_while_the_leaf_is_replaced() {
    let ca = authority("axiom-serving-ca");
    let f = fixture(&ca).await;
    let client = client_config(&ca);

    // Rotate underneath a steady stream of requests. Not a single one is allowed
    // to fail: an activation that drops one request in fifty is an activation
    // that shows up as a paging alert during every renewal.
    let renewals: Vec<MaterialPem> = (0..5).map(|_| serving_material(&ca)).collect();
    let rotator = {
        let tls = f.tls.clone();
        let source = f.source.clone();
        tokio::spawn(async move {
            for renewal in renewals {
                tokio::time::sleep(Duration::from_millis(20)).await;
                source.set(renewal);
                tls.reload().expect("each activation succeeds");
            }
        })
    };

    let mut seen = std::collections::HashSet::new();
    for attempt in 0..25 {
        let (leaf, body) = request(f.addr, client.clone())
            .await
            .unwrap_or_else(|e| panic!("request {attempt} failed mid-rotation: {e}"));
        assert_eq!(body, "ok");
        seen.insert(leaf);
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    rotator.await.unwrap();

    assert!(
        seen.len() > 1,
        "the rotation should have been visible on the wire; saw {} distinct leaves",
        seen.len()
    );
    assert_eq!(f.metrics.snapshot().handshake_failures, 0);
    f.stop().await;
}

#[tokio::test]
async fn tls_reload_refuses_connections_rather_than_serving_cleartext_when_material_expires() {
    let ca = authority("axiom-serving-ca");

    // Stand the listener up against a source that has nothing valid to give it,
    // so the accept path sees `None` — the state a process reaches when its leaf
    // expired and no renewal arrived.
    let expired = ReloadableTls::pending(
        TlsRuntimeProfile::serving([NAME.to_string()]),
        Arc::new(MemoryMaterialSource::empty()) as Arc<dyn MaterialSource>,
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let metrics = TlsListenerMetrics::new();
    let (shutdown, rx) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(serve_tls(
        listener,
        Router::new().route("/healthz", get(|| async { "ok" })),
        config_source(move || expired.server_config()),
        TlsServerOptions {
            metrics: metrics.clone(),
            ..Default::default()
        },
        async move {
            let _ = rx.await;
        },
    ));
    tokio::time::sleep(Duration::from_millis(50)).await;

    let outcome = request(addr, client_config(&ca)).await;
    assert!(
        outcome.is_err(),
        "a listener with no valid material must not answer: {outcome:?}"
    );
    assert_eq!(metrics.snapshot().refused_without_material, 1);
    assert_eq!(metrics.snapshot().established, 0);

    let _ = shutdown.send(());
    let _ = tokio::time::timeout(Duration::from_secs(5), server).await;
}

#[tokio::test]
async fn tls_reload_counts_a_handshake_from_an_untrusted_client_as_a_failure_not_a_refusal() {
    let ca = authority("axiom-serving-ca");
    let f = fixture(&ca).await;

    // A client that does not trust our CA. The connection must fail, and it must
    // be attributed to the handshake rather than to missing material — the two
    // have completely different remediations and share one counter only if
    // nobody is paying attention.
    let stranger = authority("someone-elses-ca");
    let outcome = request(f.addr, client_config(&stranger)).await;
    assert!(outcome.is_err(), "an untrusted client must not be served");

    // The client learns the certificate is untrusted before the server learns
    // the client hung up, so the counter is awaited rather than sampled — under
    // TLS 1.3 the two sides find out about a rejection at different times.
    let observed = await_metric(&f.metrics, |snapshot| snapshot.handshake_failures == 1).await;
    assert_eq!(observed.handshake_failures, 1);
    assert_eq!(observed.refused_without_material, 0);
    assert_eq!(observed.established, 0);
    f.stop().await;
}

/// Wait up to five seconds for the counters to satisfy `predicate`, returning
/// the last snapshot either way so the assertion can report what it saw.
async fn await_metric(
    metrics: &TlsListenerMetrics,
    predicate: impl Fn(&server_http::TlsListenerSnapshot) -> bool,
) -> server_http::TlsListenerSnapshot {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        let snapshot = metrics.snapshot();
        if predicate(&snapshot) || std::time::Instant::now() >= deadline {
            return snapshot;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

#[tokio::test]
async fn tls_reload_never_answers_the_tls_port_in_cleartext() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let ca = authority("axiom-serving-ca");
    let f = fixture(&ca).await;

    // The check that would catch a well-meaning "fall back to h2c if the
    // handshake fails" change: the routes are reachable on this port only
    // through TLS.
    let mut plain = TcpStream::connect(f.addr).await.unwrap();
    plain
        .write_all(b"GET /healthz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    let mut response = Vec::new();
    let _ = tokio::time::timeout(
        Duration::from_secs(2),
        plain.read_to_end(&mut response),
    )
    .await;
    assert!(
        !String::from_utf8_lossy(&response).contains("200"),
        "cleartext must not be served on the TLS listener: {}",
        String::from_utf8_lossy(&response)
    );
    f.stop().await;
}
