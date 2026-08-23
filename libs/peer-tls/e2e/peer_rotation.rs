//! Replacing peer material under live replication traffic (#3112 R2, R4, AC2).
//!
//! The failure this guards against is not "the new certificate does not work" —
//! that would be caught anywhere. It is the quieter one where activation is
//! technically correct and something in flight pays for it: a Raft append that
//! was mid-connection when the leaf changed, or a window in which the port
//! accepts anyone because the trust store is briefly empty.
//!
//! So every assertion here happens while a connection is open, and the negative
//! cases are re-checked *after* the rotation rather than only before it.

mod support;

use std::sync::Arc;

use peer_tls::material::MaterialPem;
use peer_tls::reload::{MaterialSource, MemoryMaterialSource, ReloadableTls, TlsRuntimeProfile};
use support::{authority, handshake, open, Authority, LeafBuilder};

const MEMBER: &str = "lumen-0.lumen-peer.axiom.svc.cluster.local";
const SPIFFE: &str = "spiffe://axiom/ns/axiom/instance/lumen-0";
const PEER: &str = "lumen-1.lumen-peer.axiom.svc.cluster.local";
const PEER_SPIFFE: &str = "spiffe://axiom/ns/axiom/instance/lumen-1";

fn material(ca: &Authority, dns: &str, spiffe: &str) -> MaterialPem {
    let leaf = LeafBuilder::new().dns(&[dns]).spiffe(&[spiffe]).issue(ca);
    MaterialPem::new(leaf.cert_pem, leaf.key_pem, ca.pem())
}

fn member(ca: &Authority) -> (ReloadableTls, Arc<MemoryMaterialSource>) {
    let source = Arc::new(MemoryMaterialSource::new(material(ca, MEMBER, SPIFFE)));
    let tls = ReloadableTls::required(
        TlsRuntimeProfile::peer([MEMBER.to_string()], [SPIFFE.to_string()]),
        source.clone() as Arc<dyn MaterialSource>,
    )
    .expect("startup material");
    (tls, source)
}

fn dialer(ca: &Authority) -> Arc<rustls::ClientConfig> {
    let source = Arc::new(MemoryMaterialSource::new(material(
        ca,
        PEER,
        PEER_SPIFFE,
    )));
    ReloadableTls::required(
        TlsRuntimeProfile::peer([PEER.to_string()], [PEER_SPIFFE.to_string()]),
        source as Arc<dyn MaterialSource>,
    )
    .expect("the dialing peer's material")
    .client_config()
    .unwrap()
}

#[tokio::test]
async fn peer_rotation_keeps_an_established_connection_alive_across_an_activation() {
    let ca = authority("axiom-peer-ca");
    let (tls, source) = member(&ca);

    let mut session = open(tls.server_config().unwrap(), dialer(&ca), MEMBER)
        .await
        .expect("the first connection");
    assert_eq!(session.roundtrip(b"append-1").await.unwrap(), b"append-1");
    let before = session.server_leaf_sha256.clone();

    source.set(material(&ca, MEMBER, SPIFFE));
    tls.reload().expect("the renewed leaf activates");
    assert_eq!(tls.generation(), 2);
    assert_ne!(tls.fingerprint().unwrap(), before);

    // The connection that was already up carries on. Nothing tore it down, and
    // nothing renegotiated it onto the new leaf either — it finishes on the
    // material it started with, which is what a bounded drain means.
    assert_eq!(session.roundtrip(b"append-2").await.unwrap(), b"append-2");
    session.close().await;
}

#[tokio::test]
async fn peer_rotation_puts_the_new_leaf_on_the_wire_for_the_next_handshake() {
    let ca = authority("axiom-peer-ca");
    let (tls, source) = member(&ca);

    let first = open(tls.server_config().unwrap(), dialer(&ca), MEMBER)
        .await
        .unwrap();
    assert_eq!(
        first.server_leaf_sha256,
        tls.fingerprint().unwrap(),
        "the fingerprint the status reports must be the one on the wire"
    );
    assert_eq!(first.alpn.as_deref(), Some(b"h2".as_slice()));

    source.set(material(&ca, MEMBER, SPIFFE));
    tls.reload().unwrap();

    // Fetched fresh, exactly as an accept loop would: this is the atomicity of
    // activation — per connection, no listener involved.
    let second = open(tls.server_config().unwrap(), dialer(&ca), MEMBER)
        .await
        .unwrap();
    assert_ne!(second.server_leaf_sha256, first.server_leaf_sha256);
    assert_eq!(second.server_leaf_sha256, tls.fingerprint().unwrap());

    first.close().await;
    second.close().await;
}

#[tokio::test]
async fn peer_rotation_keeps_rejecting_an_untrusted_dialer_afterwards() {
    let ca = authority("axiom-peer-ca");
    let stranger = authority("someone-elses-ca");
    let (tls, source) = member(&ca);

    let before = handshake(tls.server_config().unwrap(), dialer(&stranger), MEMBER).await;
    assert!(before.server.is_err(), "an untrusted peer must not get in");

    source.set(material(&ca, MEMBER, SPIFFE));
    tls.reload().unwrap();

    // Rotation widens the trust store by carrying the previous anchors. The
    // question is whether it widens it to anything it should not, and the
    // answer has to be re-asked after every activation, not assumed from before.
    let after = handshake(tls.server_config().unwrap(), dialer(&stranger), MEMBER).await;
    assert!(
        after.server.is_err(),
        "activation must not open the port to a foreign CA: {:?}",
        after.server
    );
}

#[tokio::test]
async fn peer_rotation_leaves_a_client_without_a_certificate_no_way_in() {
    let ca = authority("axiom-peer-ca");
    let (tls, source) = member(&ca);

    // A client that trusts us but presents nothing. On a peer port that is not
    // an anonymous request, it is an unauthenticated replication peer.
    let anonymous = {
        let mut roots = rustls::RootCertStore::empty();
        let anchors =
            rustls_pemfile::certs(&mut ca.pem().as_bytes()).collect::<Result<Vec<_>, _>>();
        for anchor in anchors.unwrap() {
            roots.add(anchor).unwrap();
        }
        peer_tls::install_default_crypto_provider();
        let mut config = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        config.alpn_protocols = vec![b"h2".to_vec()];
        Arc::new(config)
    };

    let before = handshake(tls.server_config().unwrap(), anonymous.clone(), MEMBER).await;
    assert!(before.server.is_err());

    source.set(material(&ca, MEMBER, SPIFFE));
    tls.reload().unwrap();

    let after = handshake(tls.server_config().unwrap(), anonymous, MEMBER).await;
    assert!(
        after.server.is_err(),
        "mTLS must still be required after a rotation: {:?}",
        after.server
    );
}

#[tokio::test]
async fn peer_rotation_never_leaves_a_plaintext_window() {
    use tokio::io::AsyncWriteExt;

    let ca = authority("axiom-peer-ca");
    let (tls, source) = member(&ca);

    // Speak Raft directly at the port, with no ClientHello in front of it. The
    // rotation is the interesting moment: a reloader that rebuilt its
    // configuration in place could briefly have none.
    for stage in ["before activation", "after activation"] {
        let config = tls
            .server_config()
            .expect("a peer port is never without a configuration");
        let (server_io, mut client_io) = tokio::io::duplex(4096);
        let acceptor = tokio_rustls::TlsAcceptor::from(config);
        let accepted = tokio::spawn(async move { acceptor.accept(server_io).await.is_ok() });
        client_io.write_all(b"POST /raft/append HTTP/1.1\r\n\r\n").await.unwrap();
        client_io.flush().await.unwrap();
        assert!(
            !accepted.await.unwrap(),
            "plaintext was accepted {stage}"
        );

        source.set(material(&ca, MEMBER, SPIFFE));
        tls.reload().unwrap();
    }
}
