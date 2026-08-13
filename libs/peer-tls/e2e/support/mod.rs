//! Certificate fixtures for the hot-reload tests (#3112).
//!
//! Every test here needs the same three things — a throwaway CA, a leaf it
//! signed, and a trust bundle — differing only in which CA signed what, which
//! names the leaf claims, and when it is valid. Those three axes are exactly
//! what the reload path branches on, so they are the parameters.

#![allow(dead_code)]

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose, Ia5String,
    IsCa, KeyPair, SanType,
};

/// A throwaway certificate authority. Two of these is what "a different trust
/// domain" means; two *generations* of them is what an issuer rotation means.
pub struct Authority {
    pub cert: Certificate,
    pub key: KeyPair,
}

pub fn authority(name: &str) -> Authority {
    let mut params = CertificateParams::new(Vec::new()).unwrap();
    params.distinguished_name.push(DnType::CommonName, name);
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    let key = KeyPair::generate().unwrap();
    let cert = params.self_signed(&key).unwrap();
    Authority { cert, key }
}

impl Authority {
    pub fn pem(&self) -> String {
        self.cert.pem()
    }
}

/// One issued leaf, in the shape a projected Secret holds it.
pub struct Leaf {
    pub cert_pem: String,
    pub key_pem: String,
}

/// The three axes a reload decision turns on: identity, usage, validity.
pub struct LeafBuilder {
    dns: Vec<String>,
    uris: Vec<String>,
    server_auth: bool,
    client_auth: bool,
    not_before: SystemTime,
    not_after: SystemTime,
}

impl LeafBuilder {
    /// A leaf that is unremarkable in every way: both usages, valid from an
    /// hour ago to a day from now. Tests then break exactly one thing.
    pub fn new() -> Self {
        let now = SystemTime::now();
        Self {
            dns: Vec::new(),
            uris: Vec::new(),
            server_auth: true,
            client_auth: true,
            not_before: now - Duration::from_secs(3600),
            not_after: now + Duration::from_secs(86_400),
        }
    }

    pub fn dns(mut self, names: &[&str]) -> Self {
        self.dns = names.iter().map(|n| (*n).to_string()).collect();
        self
    }

    pub fn spiffe(mut self, uris: &[&str]) -> Self {
        self.uris = uris.iter().map(|u| (*u).to_string()).collect();
        self
    }

    pub fn usages(mut self, server_auth: bool, client_auth: bool) -> Self {
        self.server_auth = server_auth;
        self.client_auth = client_auth;
        self
    }

    pub fn window(mut self, not_before: SystemTime, not_after: SystemTime) -> Self {
        self.not_before = not_before;
        self.not_after = not_after;
        self
    }

    pub fn issue(self, ca: &Authority) -> Leaf {
        let mut params = CertificateParams::new(Vec::<String>::new()).unwrap();
        for name in &self.dns {
            params
                .subject_alt_names
                .push(SanType::DnsName(Ia5String::try_from(name.clone()).unwrap()));
        }
        for uri in &self.uris {
            params
                .subject_alt_names
                .push(SanType::URI(Ia5String::try_from(uri.clone()).unwrap()));
        }
        let common_name = self.dns.first().cloned().unwrap_or_else(|| "leaf".into());
        params
            .distinguished_name
            .push(DnType::CommonName, common_name);
        let mut usages = Vec::new();
        if self.server_auth {
            usages.push(ExtendedKeyUsagePurpose::ServerAuth);
        }
        if self.client_auth {
            usages.push(ExtendedKeyUsagePurpose::ClientAuth);
        }
        params.extended_key_usages = usages;
        params.not_before = offset(self.not_before);
        params.not_after = offset(self.not_after);
        let key = KeyPair::generate().unwrap();
        let cert = params.signed_by(&key, &ca.cert, &ca.key).unwrap();
        Leaf {
            cert_pem: cert.pem(),
            key_pem: key.serialize_pem(),
        }
    }
}

impl Default for LeafBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A trust bundle is just concatenated PEM, which is how a current+next overlap
/// is expressed on disk: two anchors in one file.
pub fn bundle(authorities: &[&Authority]) -> String {
    authorities
        .iter()
        .map(|ca| ca.pem())
        .collect::<Vec<_>>()
        .join("\n")
}

/// What both ends of an attempted handshake concluded.
///
/// Both, separately, because TLS 1.3 lets the client finish before the server
/// has looked at its certificate: a peer-mTLS rejection shows up only on the
/// server side, and asserting on the client would pass while an untrusted peer
/// walked in.
pub struct HandshakeOutcome {
    pub server: Result<Option<Vec<u8>>, String>,
    pub client: Result<(), String>,
    /// Fingerprint of the leaf the client actually saw, in the same lowercase
    /// hex sha256 the controller and the status surface use. This is how "did
    /// the new material reach the wire" is asked without trusting the reloader
    /// to answer questions about itself.
    pub server_leaf_sha256: Option<String>,
}

impl HandshakeOutcome {
    pub fn accepted(&self) -> bool {
        self.server.is_ok() && self.client.is_ok()
    }

    pub fn alpn(&self) -> Option<&[u8]> {
        self.server.as_ref().ok().and_then(|a| a.as_deref())
    }
}

/// Run a real handshake over an in-memory duplex and exchange one message, so
/// neither side can report success on a connection that never carried data.
pub async fn handshake(
    server: std::sync::Arc<rustls::ServerConfig>,
    client: std::sync::Arc<rustls::ClientConfig>,
    server_name: &str,
) -> HandshakeOutcome {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    peer_tls::install_default_crypto_provider();
    let (server_io, client_io) = tokio::io::duplex(64 * 1024);
    let acceptor = tokio_rustls::TlsAcceptor::from(server);
    let connector = tokio_rustls::TlsConnector::from(client);
    let name = rustls::pki_types::ServerName::try_from(server_name.to_string()).unwrap();

    let server_side = tokio::spawn(async move {
        let mut stream = acceptor.accept(server_io).await.map_err(|e| e.to_string())?;
        let alpn = stream.get_ref().1.alpn_protocol().map(|p| p.to_vec());
        stream.write_all(b"ok").await.map_err(|e| e.to_string())?;
        stream.flush().await.map_err(|e| e.to_string())?;
        Ok::<_, String>(alpn)
    });

    let client_side = async move {
        let mut stream = connector
            .connect(name, client_io)
            .await
            .map_err(|e| e.to_string())?;
        let seen = stream
            .get_ref()
            .1
            .peer_certificates()
            .and_then(|chain| chain.first())
            .map(|leaf| sha256_hex(leaf.as_ref()));
        let mut buf = [0u8; 2];
        stream.read_exact(&mut buf).await.map_err(|e| e.to_string())?;
        Ok::<_, String>(seen)
    };

    let client = client_side.await;
    let server = server_side.await.unwrap_or_else(|e| Err(e.to_string()));
    let server_leaf_sha256 = client.clone().ok().flatten();
    HandshakeOutcome {
        server,
        client: client.map(|_| ()),
        server_leaf_sha256,
    }
}

/// A connection that stays open across a reload, so "existing connections
/// finish" can be asserted rather than assumed.
pub struct LiveSession {
    stream: tokio_rustls::client::TlsStream<tokio::io::DuplexStream>,
    server: tokio::task::JoinHandle<()>,
    pub server_leaf_sha256: String,
    pub alpn: Option<Vec<u8>>,
}

impl LiveSession {
    /// One request and its response over the already-established connection.
    pub async fn roundtrip(&mut self, message: &[u8]) -> Result<Vec<u8>, String> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        self.stream
            .write_all(message)
            .await
            .map_err(|e| e.to_string())?;
        self.stream.flush().await.map_err(|e| e.to_string())?;
        let mut buf = vec![0u8; message.len()];
        self.stream
            .read_exact(&mut buf)
            .await
            .map_err(|e| e.to_string())?;
        Ok(buf)
    }

    pub async fn close(mut self) {
        use tokio::io::AsyncWriteExt;
        let _ = self.stream.shutdown().await;
        let _ = self.server.await;
    }
}

/// Establish a connection whose server end echoes forever.
pub async fn open(
    server: std::sync::Arc<rustls::ServerConfig>,
    client: std::sync::Arc<rustls::ClientConfig>,
    server_name: &str,
) -> Result<LiveSession, String> {
    peer_tls::install_default_crypto_provider();
    let (server_io, client_io) = tokio::io::duplex(64 * 1024);
    let acceptor = tokio_rustls::TlsAcceptor::from(server);
    let connector = tokio_rustls::TlsConnector::from(client);
    let name = rustls::pki_types::ServerName::try_from(server_name.to_string()).unwrap();

    let server_task = tokio::spawn(async move {
        let Ok(stream) = acceptor.accept(server_io).await else {
            return;
        };
        let (mut reader, mut writer) = tokio::io::split(stream);
        let _ = tokio::io::copy(&mut reader, &mut writer).await;
    });

    let stream = connector
        .connect(name, client_io)
        .await
        .map_err(|e| e.to_string())?;
    let (_, connection) = stream.get_ref();
    let alpn = connection.alpn_protocol().map(|p| p.to_vec());
    let server_leaf_sha256 = connection
        .peer_certificates()
        .and_then(|chain| chain.first())
        .map(|leaf| sha256_hex(leaf.as_ref()))
        .ok_or_else(|| "the server presented no certificate".to_string())?;

    Ok(LiveSession {
        stream,
        server: server_task,
        server_leaf_sha256,
        alpn,
    })
}

/// The certificate controller's spelling: lowercase hex, no separators.
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn offset(instant: SystemTime) -> time::OffsetDateTime {
    let seconds = instant
        .duration_since(UNIX_EPOCH)
        .expect("fixture instants are after the epoch")
        .as_secs() as i64;
    time::OffsetDateTime::from_unix_timestamp(seconds).expect("representable instant")
}

pub fn seconds_from_now(seconds: i64) -> SystemTime {
    let now = SystemTime::now();
    if seconds >= 0 {
        now + Duration::from_secs(seconds as u64)
    } else {
        now - Duration::from_secs((-seconds) as u64)
    }
}
