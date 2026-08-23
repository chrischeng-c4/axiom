// HANDWRITE-BEGIN gap="missing-generator:logic:certificate-issuer" tracker="#3110" reason="Own the issuer-neutral request/response boundary and the in-memory keypair+CSR construction, so the CA integration is one replaceable implementation rather than a shape the rest of the lifecycle is written around."
//! The boundary between "this lifecycle" and "whatever signs certificates".
//!
//! Two things live here. First, the [`Issuer`] trait: one method, one request,
//! one response. Production is GCP CA Service; unit and kind verification use
//! [`super::EphemeralIssuer`]. Neither is privileged — the state machine has never
//! heard of either, which is R8.
//!
//! Second, the private key. It is generated here, in memory, and it leaves this
//! module exactly once: inside [`IssuedMaterial`], on its way to a namespaced
//! Secret. It is never written to a file, never logged, never returned in an
//! error, and — the part that is easy to get wrong — the CA never sees it. A
//! CSR carries a *public* key and a proof of possession; that asymmetry is the
//! entire reason certificate issuance can be delegated to a service at all, and
//! R2's "never fetch, export, or store a CA private key" is its mirror image on
//! the other side.

use std::fmt;
use std::time::Duration;

use chrono::{DateTime, Utc};
use futures::future::BoxFuture;

use super::profile::{CertificateProfile, ExtendedUsage, InstanceScope, Purpose};

/// Which CA a leaf came from, or is going to.
///
/// A plain string, but a distinct type: rotation is defined entirely in terms
/// of "the issuer the leaf has" versus "the issuer the operator wants", and
/// those two are easy to transpose when both are `String`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IssuerId(pub String);

impl IssuerId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IssuerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A freshly generated private key.
///
/// Deliberately not `Clone`, not `Debug`, and not `Serialize`. Each of those
/// would be one more way for key material to end up somewhere it was never
/// meant to go, and none of them is needed: the key is generated, carried to the
/// projector, and consumed.
///
/// It is a separate value from [`IssuanceRequest`] rather than a field on it,
/// which is the whole point of the split — an [`Issuer`] implementation is
/// handed the request and cannot reach the key, because the key was never in
/// the thing it was handed.
pub struct PrivateKey(String);

impl PrivateKey {
    /// The PEM, consumed. Taking `self` is deliberate: there is exactly one
    /// caller, the projector, and after it runs the key exists only in the
    /// Secret it wrote.
    pub fn into_pem(self) -> String {
        self.0
    }
}

/// Everything an issuer needs, and nothing it does not.
///
/// Note what is absent: no credentials, no cluster handle, no Secret name. An
/// issuer signs; it does not decide who may ask.
pub struct IssuanceRequest {
    /// Who this is for. Checked against the reconciler's own scope before any
    /// key is generated (R7/AC4).
    pub scope: InstanceScope,
    pub purpose: Purpose,
    pub common_name: String,
    pub dns_names: Vec<String>,
    pub spiffe_uri: Option<String>,
    /// Usages the *pool* is expected to stamp on the leaf. Carried alongside
    /// the CSR rather than inside it because that is how CA Service works: the
    /// issuance policy owns key usage and CA-ness, and
    /// `allow_config_based_issuance = false` (#3109) means the requester cannot
    /// smuggle its own.
    pub extended_key_usages: Vec<ExtendedUsage>,
    pub lifetime: Duration,
    /// The certificate signing request. Carries a *public* key and a proof of
    /// possession — never the key itself.
    pub csr_pem: String,
}

impl IssuanceRequest {
    /// Build a request from a validated profile, generating a fresh keypair in
    /// memory.
    ///
    /// Returns the key separately, so handing the request to an issuer cannot
    /// also hand it the key.
    pub fn build(
        scope: &InstanceScope,
        profile: &CertificateProfile,
    ) -> Result<(Self, PrivateKey), IssuerError> {
        let (private_key_pem, csr_pem) = generate_key_and_csr(profile)?;
        Ok((
            Self {
                scope: scope.clone(),
                purpose: profile.purpose(),
                common_name: profile.common_name().to_string(),
                dns_names: profile.identity().dns_names.clone(),
                spiffe_uri: profile.identity().spiffe_uri.clone(),
                extended_key_usages: profile.extended_key_usages().to_vec(),
                lifetime: profile.lifetime(),
                csr_pem,
            },
            PrivateKey(private_key_pem),
        ))
    }
}

/// What comes back: a leaf, the chain that verifies it, and the metadata the
/// state machine reasons about.
#[derive(Clone)]
pub struct IssuedMaterial {
    pub issuer: IssuerId,
    pub certificate_pem: String,
    /// Public CA chain. Public by definition — it is what verifiers are meant
    /// to already have.
    pub chain_pem: String,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    /// Lowercase hex sha256 of the leaf's DER. The identity the runtime reports
    /// once it is actually serving this leaf.
    pub fingerprint: String,
}

impl fmt::Debug for IssuedMaterial {
    /// Hand-written so a `{:?}` in a log line prints metadata, not PEM. The
    /// certificate is public, but a derived `Debug` on a struct that travels
    /// next to key material is a habit worth not having.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IssuedMaterial")
            .field("issuer", &self.issuer)
            .field("not_after", &self.not_after)
            .field("fingerprint", &self.fingerprint)
            .finish_non_exhaustive()
    }
}

/// Why issuance failed.
#[derive(Debug)]
pub enum IssuerError {
    /// The keypair or CSR could not be built. Carries a description, never the
    /// material.
    KeyGeneration(String),
    /// The CA refused, or could not be reached.
    Upstream(String),
    /// The CA returned something that is not a usable leaf.
    Malformed(String),
}

impl fmt::Display for IssuerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyGeneration(detail) => write!(f, "generate key and CSR: {detail}"),
            Self::Upstream(detail) => write!(f, "certificate authority: {detail}"),
            Self::Malformed(detail) => write!(f, "issued certificate is unusable: {detail}"),
        }
    }
}

impl std::error::Error for IssuerError {}

/// Anything that can turn a CSR into a leaf.
///
/// `BoxFuture` rather than `async fn` because the reconciler holds issuers
/// behind a trait object: a service configures one at startup, and which one it
/// is must not leak into the type of everything downstream.
pub trait Issuer: Send + Sync {
    /// The CA this issuer issues from. The state machine compares this against
    /// what the current leaf carries to decide whether a rotation is in flight.
    fn id(&self) -> IssuerId;

    /// Sign `request`, returning the leaf and its chain.
    fn issue<'a>(
        &'a self,
        request: IssuanceRequest,
    ) -> BoxFuture<'a, Result<IssuedMaterial, IssuerError>>;

    /// The anchor a verifier needs in order to accept leaves from this issuer.
    ///
    /// Separate from [`Issuer::issue`] because the overlap ordering depends on
    /// it: trust in the incoming CA has to be published across the fleet
    /// *before* the first leaf it signed appears, and at that moment there is no
    /// leaf to read a chain out of. Public material, by definition — it is what
    /// verifiers are meant to already hold.
    fn trust_anchor_pem<'a>(&'a self) -> BoxFuture<'a, Result<String, IssuerError>>;
}

/// Generate an in-memory P-256 keypair and a CSR carrying the profile's names.
///
/// P-256 rather than RSA: leaves here live for hours and are minted by a
/// controller that may be renewing several at once, so key generation cost is a
/// real operational property, not a benchmark curiosity.
fn generate_key_and_csr(profile: &CertificateProfile) -> Result<(String, String), IssuerError> {
    use rcgen::{CertificateParams, DnType, KeyPair, KeyUsagePurpose, SanType};

    let mut params = CertificateParams::default();
    params
        .distinguished_name
        .push(DnType::CommonName, profile.common_name());
    for name in &profile.identity().dns_names {
        let san = name
            .as_str()
            .try_into()
            .map(SanType::DnsName)
            .map_err(|err| IssuerError::KeyGeneration(format!("DNS SAN {name}: {err}")))?;
        params.subject_alt_names.push(san);
    }
    if let Some(uri) = &profile.identity().spiffe_uri {
        let san = uri
            .as_str()
            .try_into()
            .map(SanType::URI)
            .map_err(|err| IssuerError::KeyGeneration(format!("URI SAN {uri}: {err}")))?;
        params.subject_alt_names.push(san);
    }
    // The two basic usages a TLS leaf needs, and only those. `KeyCertSign` and
    // `CrlSign` are what would turn this into a CA in everything but name, and
    // the issuing pool refuses them anyway (#3109) -- stated on both sides
    // because a requester that asks for them should fail locally, not at the
    // CA, where the failure is a rate-limited API error at 3am.
    params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyEncipherment,
    ];

    let key_pair = KeyPair::generate()
        .map_err(|err| IssuerError::KeyGeneration(format!("generate keypair: {err}")))?;
    let csr = params
        .serialize_request(&key_pair)
        .map_err(|err| IssuerError::KeyGeneration(format!("serialize CSR: {err}")))?;
    let csr_pem = csr
        .pem()
        .map_err(|err| IssuerError::KeyGeneration(format!("encode CSR: {err}")))?;
    Ok((key_pair.serialize_pem(), csr_pem))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certificate::profile::CertificateIdentity;

    fn scope() -> InstanceScope {
        InstanceScope::new("lumen", "lumen", "lumen-prod.svc.id.goog")
    }

    fn peer_profile() -> CertificateProfile {
        CertificateProfile::new(
            &scope(),
            Purpose::Peer,
            "lumen-0.lumen-headless.lumen.svc.cluster.local",
            CertificateIdentity {
                dns_names: vec!["lumen-0.lumen-headless.lumen.svc.cluster.local".into()],
                spiffe_uri: Some("spiffe://lumen-prod.svc.id.goog/ns/lumen/sa/lumen".into()),
            },
            Duration::from_secs(86_400),
            Duration::from_secs(21_600),
            Duration::from_secs(1_800),
        )
        .unwrap()
    }

    #[test]
    fn a_csr_carries_the_requested_names() {
        let (request, _key) = IssuanceRequest::build(&scope(), &peer_profile()).unwrap();
        let parsed =
            rcgen::CertificateSigningRequestParams::from_pem(&request.csr_pem).unwrap();
        let names: Vec<String> = parsed
            .params
            .subject_alt_names
            .iter()
            .map(|san| format!("{san:?}"))
            .collect();
        let joined = names.join(" ");
        assert!(
            joined.contains("lumen-0.lumen-headless.lumen.svc.cluster.local"),
            "got {joined}"
        );
        assert!(
            joined.contains("spiffe://lumen-prod.svc.id.goog/ns/lumen/sa/lumen"),
            "got {joined}"
        );
    }

    #[test]
    fn a_csr_never_asks_to_be_a_ca() {
        let (request, _key) = IssuanceRequest::build(&scope(), &peer_profile()).unwrap();
        let parsed =
            rcgen::CertificateSigningRequestParams::from_pem(&request.csr_pem).unwrap();
        assert_eq!(parsed.params.is_ca, rcgen::IsCa::NoCa);
        assert!(!parsed
            .params
            .key_usages
            .contains(&rcgen::KeyUsagePurpose::KeyCertSign));
        assert!(!parsed
            .params
            .key_usages
            .contains(&rcgen::KeyUsagePurpose::CrlSign));
    }

    #[test]
    fn the_private_key_is_not_in_the_csr() {
        let (request, _key) = IssuanceRequest::build(&scope(), &peer_profile()).unwrap();
        // The property that makes delegated issuance safe at all: what goes to
        // the CA proves possession of the key without containing it.
        assert!(!request.csr_pem.contains("PRIVATE KEY"));
        assert!(request.csr_pem.contains("BEGIN CERTIFICATE REQUEST"));
    }

    #[test]
    fn every_request_gets_a_fresh_key() {
        let a = IssuanceRequest::build(&scope(), &peer_profile())
            .unwrap()
            .1
            .into_pem();
        let b = IssuanceRequest::build(&scope(), &peer_profile())
            .unwrap()
            .1
            .into_pem();
        assert_ne!(a, b, "renewal must not reuse the key it is replacing");
    }
}
// HANDWRITE-END
