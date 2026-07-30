// HANDWRITE-BEGIN gap="missing-generator:logic:peer-tls-material-validation" tracker="#3112" reason="Deciding whether a candidate leaf would survive a handshake is control flow over rustls' verifier and error taxonomy; no generator primitive expresses it."
//! What a candidate TLS update must prove before anything starts using it.
//!
//! #3112 R3. The property that matters is not "these bytes parse" — it is "a
//! handshake against these bytes would succeed". So validation runs through
//! rustls' own [`WebPkiServerVerifier`] / [`WebPkiClientVerifier`] rather than a
//! hand-rolled chain walk: whatever the handshake would reject, this rejects,
//! for the same reason and at the same instant. A validator that only parsed PEM
//! would happily activate a leaf signed by a CA nobody trusts, and the failure
//! would surface one hop later as a client-side error nobody could attribute
//! back to the rotation that caused it.
//!
//! Two checks rustls' verifiers do not cover are done here directly against the
//! leaf: the SPIFFE URI SAN (webpki verifies DNS names, not URI names) and the
//! validity window as a *number*, which the reload state needs in order to
//! report seconds-to-expiry and to refuse to keep serving past it (R6, R7).

use std::fmt;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rustls::client::danger::ServerCertVerifier;
use rustls::client::WebPkiServerVerifier;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};
use rustls::server::{ParsedCertificate, WebPkiClientVerifier};
use rustls::{CertificateError, RootCertStore};

use crate::install_default_crypto_provider;

/// Why a candidate was refused.
///
/// One variant per failure an operator can actually cause, because the reason is
/// what reaches the status surface (R6) and "reload failed" is not an
/// actionable thing to read at three in the morning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionReason {
    /// A source file could not be read at all.
    Unreadable,
    /// PEM that does not decode, or a bundle with no certificate in it.
    MalformedPem,
    /// A trust bundle with no anchors: activating it would trust nothing and
    /// reject every peer, which is an outage rather than a rotation.
    EmptyTrustBundle,
    /// The private key does not belong to the leaf.
    KeyMismatch,
    /// `notBefore` is in the future.
    NotYetValid,
    /// `notAfter` is in the past.
    Expired,
    /// The leaf lacks the extended key usage the role requires.
    MissingUsage,
    /// The leaf does not chain to any anchor in the trust bundle.
    Untrusted,
    /// The leaf chains and is in date, but does not carry the DNS or SPIFFE
    /// identity this runtime is configured to present.
    WrongIdentity,
}

impl RejectionReason {
    /// The stable machine-readable spelling, for metrics labels and status.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unreadable => "unreadable",
            Self::MalformedPem => "malformed_pem",
            Self::EmptyTrustBundle => "empty_trust_bundle",
            Self::KeyMismatch => "key_mismatch",
            Self::NotYetValid => "not_yet_valid",
            Self::Expired => "expired",
            Self::MissingUsage => "missing_usage",
            Self::Untrusted => "untrusted",
            Self::WrongIdentity => "wrong_identity",
        }
    }
}

impl fmt::Display for RejectionReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A refusal, with a detail string safe to log.
///
/// `detail` is built only from rustls' own error text, expected identity names,
/// and counts. It never carries PEM bodies or filesystem paths, because this
/// string ends up in request-adjacent logs and status conditions (#3112 R6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rejection {
    pub reason: RejectionReason,
    pub detail: String,
}

impl Rejection {
    pub fn new(reason: RejectionReason, detail: impl Into<String>) -> Self {
        Self {
            reason,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for Rejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.reason, self.detail)
    }
}

impl std::error::Error for Rejection {}

/// One projection's worth of PEM, as bytes.
///
/// Bytes rather than paths so that everything below this point is testable
/// without a filesystem, and so a caller that gets its material from a
/// Kubernetes watch instead of a mounted Secret needs no second code path.
#[derive(Clone, PartialEq, Eq)]
pub struct MaterialPem {
    pub cert_chain: Vec<u8>,
    pub key: Vec<u8>,
    pub trust_bundle: Vec<u8>,
}

impl MaterialPem {
    pub fn new(
        cert_chain: impl Into<Vec<u8>>,
        key: impl Into<Vec<u8>>,
        trust_bundle: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            cert_chain: cert_chain.into(),
            key: key.into(),
            trust_bundle: trust_bundle.into(),
        }
    }
}

/// Sizes only. A derived `Debug` here would print a private key into any log
/// line that formatted the enclosing struct (#3112 R6).
impl fmt::Debug for MaterialPem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MaterialPem")
            .field("cert_chain_bytes", &self.cert_chain.len())
            .field("key_bytes", &self.key.len())
            .field("trust_bundle_bytes", &self.trust_bundle.len())
            .finish()
    }
}

/// The identity a leaf must actually carry, and the roles it must be usable for.
///
/// Configured by the service, not read off the certificate: the point is to
/// catch the projection that swapped in a valid certificate for *something
/// else*. A leaf that chains correctly but names another workload is exactly
/// the case an "is it valid?" check waves through.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IdentityExpectation {
    /// Every name here must appear in the leaf's DNS SANs.
    pub dns_names: Vec<String>,
    /// Every URI here must appear in the leaf's URI SANs.
    pub spiffe_uris: Vec<String>,
    /// The leaf must be usable to authenticate a server.
    pub require_server_auth: bool,
    /// The leaf must be usable to authenticate a client.
    pub require_client_auth: bool,
}

impl IdentityExpectation {
    /// A serving leaf: presents these names to clients, never dials as a client.
    pub fn serving(dns_names: impl IntoIterator<Item = String>) -> Self {
        Self {
            dns_names: dns_names.into_iter().collect(),
            spiffe_uris: Vec::new(),
            require_server_auth: true,
            require_client_auth: false,
        }
    }

    /// A peer leaf: the same material both accepts and dials, so it must satisfy
    /// both roles or one direction of the mesh silently fails.
    pub fn peer(
        dns_names: impl IntoIterator<Item = String>,
        spiffe_uris: impl IntoIterator<Item = String>,
    ) -> Self {
        Self {
            dns_names: dns_names.into_iter().collect(),
            spiffe_uris: spiffe_uris.into_iter().collect(),
            require_server_auth: true,
            require_client_auth: true,
        }
    }
}

/// Material that has proved everything in [`IdentityExpectation`] and would
/// survive a handshake at the instant it was validated.
pub struct ValidatedMaterial {
    chain: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
    trust: Vec<CertificateDer<'static>>,
    fingerprint: String,
    not_before: SystemTime,
    not_after: SystemTime,
}

impl fmt::Debug for ValidatedMaterial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidatedMaterial")
            .field("fingerprint", &self.fingerprint)
            .field("chain_len", &self.chain.len())
            .field("trust_anchors", &self.trust.len())
            .finish_non_exhaustive()
    }
}

impl ValidatedMaterial {
    /// Lowercase hex sha256 of the leaf DER — the same spelling the certificate
    /// controller writes into status, so "did the runtime pick up the leaf I
    /// issued?" is a string comparison and not a guess (#3110, #3112 R5).
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    pub fn not_before(&self) -> SystemTime {
        self.not_before
    }

    pub fn not_after(&self) -> SystemTime {
        self.not_after
    }

    /// The leaf and any intermediates, end-entity first.
    pub fn chain(&self) -> &[CertificateDer<'static>] {
        &self.chain
    }

    /// The anchors this material was validated against.
    pub fn trust_anchors(&self) -> &[CertificateDer<'static>] {
        &self.trust
    }

    pub fn key(&self) -> PrivateKeyDer<'static> {
        self.key.clone_key()
    }

    /// Whether this material is still in date at `now`.
    pub fn is_valid_at(&self, now: SystemTime) -> bool {
        now >= self.not_before && now < self.not_after
    }

    /// Whole seconds until expiry, saturating at zero once expired.
    pub fn seconds_to_expiry(&self, now: SystemTime) -> u64 {
        self.not_after
            .duration_since(now)
            .map(|left| left.as_secs())
            .unwrap_or(0)
    }
}

/// Parse and check a candidate against `expect` as of `now`.
///
/// The order is deliberate: cheap structural failures first, so an operator who
/// mounted the wrong file gets "malformed_pem" rather than a chain error that
/// reads like a CA problem.
pub fn validate(
    pem: &MaterialPem,
    expect: &IdentityExpectation,
    now: SystemTime,
) -> Result<ValidatedMaterial, Rejection> {
    install_default_crypto_provider();

    let chain = parse_certificates(&pem.cert_chain, "certificate chain")?;
    let trust = parse_certificates(&pem.trust_bundle, "trust bundle")?;
    if trust.is_empty() {
        return Err(Rejection::new(
            RejectionReason::EmptyTrustBundle,
            "trust bundle contains no anchors",
        ));
    }
    let key = parse_private_key(&pem.key)?;

    let leaf = chain
        .first()
        .cloned()
        .ok_or_else(|| Rejection::new(RejectionReason::MalformedPem, "empty certificate chain"))?;
    let intermediates = &chain[1..];

    // The key must belong to the leaf. rustls answers "unknown" for key types it
    // cannot introspect; that is not evidence of a mismatch, so it is not
    // treated as one — `from_der` already encodes exactly that distinction.
    let provider = rustls::crypto::CryptoProvider::get_default()
        .cloned()
        .ok_or_else(|| {
            Rejection::new(
                RejectionReason::MalformedPem,
                "no process-default crypto provider is installed",
            )
        })?;
    rustls::sign::CertifiedKey::from_der(chain.clone(), key.clone_key(), &provider)
        .map_err(|err| Rejection::new(RejectionReason::KeyMismatch, err.to_string()))?;

    let facts = leaf_facts(&leaf)?;
    // Checked before the verifiers so the reported reason is the operator's
    // actual problem: webpki collapses both edges of the window into one error
    // family, and "expired" versus "not yet valid" is the difference between a
    // stalled controller and a clock skew.
    if now < facts.not_before {
        return Err(Rejection::new(
            RejectionReason::NotYetValid,
            "leaf notBefore is in the future",
        ));
    }
    if now >= facts.not_after {
        return Err(Rejection::new(
            RejectionReason::Expired,
            "leaf notAfter is in the past",
        ));
    }

    let mut roots = RootCertStore::empty();
    for anchor in &trust {
        roots.add(anchor.clone()).map_err(|err| {
            Rejection::new(
                RejectionReason::MalformedPem,
                format!("trust anchor rejected: {err}"),
            )
        })?;
    }
    let roots = Arc::new(roots);
    let unix_now = to_unix_time(now);

    if expect.require_client_auth {
        let verifier = WebPkiClientVerifier::builder(Arc::clone(&roots))
            .build()
            .map_err(|err| {
                Rejection::new(
                    RejectionReason::EmptyTrustBundle,
                    format!("client verifier: {err}"),
                )
            })?;
        verifier
            .verify_client_cert(&leaf, intermediates, unix_now)
            .map_err(classify)?;
    }

    if expect.require_server_auth {
        let verifier = WebPkiServerVerifier::builder(Arc::clone(&roots))
            .build()
            .map_err(|err| {
                Rejection::new(
                    RejectionReason::EmptyTrustBundle,
                    format!("server verifier: {err}"),
                )
            })?;
        // Every configured name, not just the first: a leaf reissued for a
        // shrunken topology still verifies against one member's name, and
        // stopping at the first success is how the fleet finds out at dial time.
        let names = if expect.dns_names.is_empty() {
            Vec::new()
        } else {
            expect.dns_names.clone()
        };
        if names.is_empty() {
            verify_chain_only(&verifier, &leaf, intermediates, unix_now)?;
        }
        for name in names {
            let server_name = ServerName::try_from(name.clone()).map_err(|err| {
                Rejection::new(
                    RejectionReason::WrongIdentity,
                    format!("expected name `{name}` is not a valid server name: {err}"),
                )
            })?;
            verifier
                .verify_server_cert(&leaf, intermediates, &server_name, &[], unix_now)
                .map_err(|err| {
                    let mut rejection = classify(err);
                    if rejection.reason == RejectionReason::WrongIdentity {
                        rejection.detail = format!("leaf does not carry the name `{name}`");
                    }
                    rejection
                })?;
        }
    } else {
        // No serverAuth requirement, but the names still have to be on the leaf.
        let parsed = ParsedCertificate::try_from(&leaf)
            .map_err(|err| Rejection::new(RejectionReason::MalformedPem, err.to_string()))?;
        for name in &expect.dns_names {
            let server_name = ServerName::try_from(name.clone()).map_err(|err| {
                Rejection::new(
                    RejectionReason::WrongIdentity,
                    format!("expected name `{name}` is not a valid server name: {err}"),
                )
            })?;
            rustls::client::verify_server_name(&parsed, &server_name).map_err(|_| {
                Rejection::new(
                    RejectionReason::WrongIdentity,
                    format!("leaf does not carry the name `{name}`"),
                )
            })?;
        }
    }

    for uri in &expect.spiffe_uris {
        if !facts.uri_sans.iter().any(|san| san == uri) {
            return Err(Rejection::new(
                RejectionReason::WrongIdentity,
                format!("leaf does not carry the SPIFFE identity `{uri}`"),
            ));
        }
    }

    Ok(ValidatedMaterial {
        chain,
        key,
        trust,
        fingerprint: facts.fingerprint,
        not_before: facts.not_before,
        not_after: facts.not_after,
    })
}

/// Chain-only verification when no name was configured. `localhost` is a name
/// webpki will always parse; the verifier's name check is the only part of the
/// result deliberately ignored here, and every other failure still propagates.
fn verify_chain_only(
    verifier: &Arc<WebPkiServerVerifier>,
    leaf: &CertificateDer<'static>,
    intermediates: &[CertificateDer<'static>],
    now: UnixTime,
) -> Result<(), Rejection> {
    let placeholder = ServerName::try_from("localhost").expect("`localhost` is a valid DNS name");
    match verifier.verify_server_cert(leaf, intermediates, &placeholder, &[], now) {
        Ok(_) => Ok(()),
        Err(rustls::Error::InvalidCertificate(
            CertificateError::NotValidForName | CertificateError::NotValidForNameContext { .. },
        )) => Ok(()),
        Err(err) => Err(classify(err)),
    }
}

/// Map a rustls verification failure onto the reason an operator can act on.
fn classify(err: rustls::Error) -> Rejection {
    let detail = err.to_string();
    let reason = match &err {
        rustls::Error::InvalidCertificate(certificate) => match certificate {
            CertificateError::Expired | CertificateError::ExpiredContext { .. } => {
                RejectionReason::Expired
            }
            CertificateError::NotValidYet | CertificateError::NotValidYetContext { .. } => {
                RejectionReason::NotYetValid
            }
            CertificateError::NotValidForName
            | CertificateError::NotValidForNameContext { .. } => RejectionReason::WrongIdentity,
            CertificateError::InvalidPurpose | CertificateError::InvalidPurposeContext { .. } => {
                RejectionReason::MissingUsage
            }
            CertificateError::BadEncoding => RejectionReason::MalformedPem,
            _ => RejectionReason::Untrusted,
        },
        _ => RejectionReason::Untrusted,
    };
    Rejection::new(reason, detail)
}

fn parse_certificates(
    pem: &[u8],
    what: &str,
) -> Result<Vec<CertificateDer<'static>>, Rejection> {
    let mut reader = std::io::BufReader::new(pem);
    let certs = rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| {
            Rejection::new(
                RejectionReason::MalformedPem,
                format!("{what}: {err}"),
            )
        })?;
    if certs.is_empty() && what != "trust bundle" {
        return Err(Rejection::new(
            RejectionReason::MalformedPem,
            format!("{what} contains no certificate"),
        ));
    }
    Ok(certs)
}

fn parse_private_key(pem: &[u8]) -> Result<PrivateKeyDer<'static>, Rejection> {
    let mut reader = std::io::BufReader::new(pem);
    rustls_pemfile::private_key(&mut reader)
        .map_err(|err| Rejection::new(RejectionReason::MalformedPem, format!("private key: {err}")))?
        .ok_or_else(|| {
            Rejection::new(
                RejectionReason::MalformedPem,
                "no private key in the projected key material",
            )
        })
}

struct LeafFacts {
    fingerprint: String,
    not_before: SystemTime,
    not_after: SystemTime,
    uri_sans: Vec<String>,
}

fn leaf_facts(leaf: &CertificateDer<'static>) -> Result<LeafFacts, Rejection> {
    let (_, parsed) = x509_parser::parse_x509_certificate(leaf.as_ref()).map_err(|err| {
        Rejection::new(
            RejectionReason::MalformedPem,
            format!("parse leaf certificate: {err}"),
        )
    })?;
    let not_before = from_unix_seconds(parsed.validity().not_before.timestamp())?;
    let not_after = from_unix_seconds(parsed.validity().not_after.timestamp())?;

    let mut uri_sans = Vec::new();
    if let Ok(Some(san)) = parsed.subject_alternative_name() {
        for name in &san.value.general_names {
            if let x509_parser::extensions::GeneralName::URI(uri) = name {
                uri_sans.push((*uri).to_string());
            }
        }
    }

    Ok(LeafFacts {
        fingerprint: hex_sha256(leaf.as_ref()),
        not_before,
        not_after,
        uri_sans,
    })
}

fn from_unix_seconds(seconds: i64) -> Result<SystemTime, Rejection> {
    if seconds < 0 {
        return Err(Rejection::new(
            RejectionReason::MalformedPem,
            "certificate validity predates the unix epoch",
        ));
    }
    Ok(UNIX_EPOCH + Duration::from_secs(seconds as u64))
}

fn to_unix_time(now: SystemTime) -> UnixTime {
    UnixTime::since_unix_epoch(now.duration_since(UNIX_EPOCH).unwrap_or_default())
}

/// Lowercase hex sha256, no separators — the encoding
/// `service_k8s::certificate` writes into status.
fn hex_sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
// HANDWRITE-END
