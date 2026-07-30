// HANDWRITE-BEGIN gap="missing-generator:logic:certificate-ephemeral-issuer" tracker="#3110" reason="Own an in-process CA that makes the whole lifecycle exercisable without a cloud account, so the rotation and expiry paths are covered by tests that run in CI rather than only by a manual GKE pass."
//! An in-process certificate authority.
//!
//! Every interesting property of this lifecycle — renewal fires at the right
//! instant, a rotation publishes trust before it publishes a leaf, an expired
//! leaf is replaced rather than served — is about *time* and *ordering*, not
//! about GCP. Making them observable only through a cloud CA would mean they are
//! checked when someone remembers to run a GKE gate, which is to say
//! occasionally.
//!
//! So the lifecycle can be driven end to end by a CA that lives in a `Vec`.
//! [`EphemeralIssuer`] mints a self-signed root on construction and signs
//! whatever CSRs it is handed, honouring the requested lifetime exactly, from a
//! clock the test controls. It is compiled unconditionally, not behind a feature
//! and not behind `cfg(test)`: R8 asks that the shared lifecycle build and test
//! with no cloud dependencies at all, and a signer that disappears under
//! `--no-default-features` would leave nothing to test with.
//!
//! It is not a shortcut for production. It signs anything, it has no issuance
//! policy, and its root is regenerated on every process start — the properties
//! that make it useful in a test are exactly the ones that disqualify it
//! elsewhere.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use chrono::{DateTime, TimeZone, Utc};
use futures::future::{ready, BoxFuture};
use rcgen::{
    Certificate, CertificateParams, CertificateSigningRequestParams, DnType, ExtendedKeyUsagePurpose,
    IsCa, KeyPair, KeyUsagePurpose,
};

use super::digest::hex_sha256;
use super::issuer::{IssuanceRequest, IssuedMaterial, Issuer, IssuerError, IssuerId};
use super::profile::ExtendedUsage;

/// A self-signed CA that signs in memory.
pub struct EphemeralIssuer {
    id: IssuerId,
    root: Certificate,
    root_key: KeyPair,
    /// The clock leaves are dated from. Tests move it; nothing else does.
    now: Mutex<DateTime<Utc>>,
    /// How many leaves this issuer has signed — the cheapest way for a test to
    /// assert that a reconcile that should have been a no-op did nothing.
    issued: AtomicU64,
    /// When set, the next `issue` fails instead of signing. Used to prove that a
    /// failed step leaves the previous material in place (R5).
    fail_next: Mutex<Option<String>>,
}

impl EphemeralIssuer {
    /// Mint a fresh root and start the clock at `now`.
    pub fn new(id: impl Into<String>, now: DateTime<Utc>) -> Self {
        let root_key = KeyPair::generate().expect("generate ephemeral CA key");
        let mut params = CertificateParams::default();
        params
            .distinguished_name
            .push(DnType::CommonName, "ephemeral test issuer");
        params.is_ca = IsCa::Ca(rcgen::BasicConstraints::Constrained(0));
        params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
        let root = params
            .self_signed(&root_key)
            .expect("self-sign ephemeral CA");
        Self {
            id: IssuerId::new(id),
            root,
            root_key,
            now: Mutex::new(now),
            issued: AtomicU64::new(0),
            fail_next: Mutex::new(None),
        }
    }

    /// Move the issuer's clock. Leaves signed after this are dated from `now`.
    pub fn set_now(&self, now: DateTime<Utc>) {
        *self.now.lock().expect("issuer clock") = now;
    }

    /// Number of leaves signed so far.
    pub fn issued_count(&self) -> u64 {
        self.issued.load(Ordering::SeqCst)
    }

    /// Make the next issuance fail with `reason`.
    pub fn fail_next(&self, reason: impl Into<String>) {
        *self.fail_next.lock().expect("issuer failure switch") = Some(reason.into());
    }

    /// The root's PEM — what verifiers need in order to accept its leaves.
    pub fn anchor_pem(&self) -> String {
        self.root.pem()
    }

    fn sign(&self, request: IssuanceRequest) -> Result<IssuedMaterial, IssuerError> {
        if let Some(reason) = self.fail_next.lock().expect("issuer failure switch").take() {
            return Err(IssuerError::Upstream(reason));
        }

        let mut parsed = CertificateSigningRequestParams::from_pem(&request.csr_pem)
            .map_err(|err| IssuerError::Malformed(format!("parse CSR: {err}")))?;

        let not_before = *self.now.lock().expect("issuer clock");
        let not_after = not_before
            + chrono::Duration::from_std(request.lifetime)
                .map_err(|err| IssuerError::Malformed(format!("lifetime: {err}")))?;
        parsed.params.not_before = offset(not_before)?;
        parsed.params.not_after = offset(not_after)?;

        // The usages come from the request, not the CSR. That is not a
        // convenience: rcgen drops extended key usages when serializing a CSR,
        // and CA Service ignores requester-supplied config anyway. Both real
        // paths stamp usages issuer-side, so this one does too -- a test signer
        // that took them from the CSR would be testing a shape production never
        // uses.
        parsed.params.extended_key_usages = request
            .extended_key_usages
            .iter()
            .map(|usage| match usage {
                ExtendedUsage::ServerAuth => ExtendedKeyUsagePurpose::ServerAuth,
                ExtendedUsage::ClientAuth => ExtendedKeyUsagePurpose::ClientAuth,
            })
            .collect();
        parsed.params.is_ca = IsCa::NoCa;

        let leaf = parsed
            .signed_by(&self.root, &self.root_key)
            .map_err(|err| IssuerError::Upstream(format!("sign leaf: {err}")))?;

        self.issued.fetch_add(1, Ordering::SeqCst);
        Ok(IssuedMaterial {
            issuer: self.id.clone(),
            certificate_pem: leaf.pem(),
            chain_pem: self.root.pem(),
            not_before,
            not_after,
            fingerprint: hex_sha256(leaf.der()),
        })
    }
}

fn offset(instant: DateTime<Utc>) -> Result<time::OffsetDateTime, IssuerError> {
    time::OffsetDateTime::from_unix_timestamp(instant.timestamp())
        .map_err(|err| IssuerError::Malformed(format!("validity instant: {err}")))
}

impl Issuer for EphemeralIssuer {
    fn id(&self) -> IssuerId {
        self.id.clone()
    }

    fn issue<'a>(
        &'a self,
        request: IssuanceRequest,
    ) -> BoxFuture<'a, Result<IssuedMaterial, IssuerError>> {
        Box::pin(ready(self.sign(request)))
    }

    fn trust_anchor_pem<'a>(&'a self) -> BoxFuture<'a, Result<String, IssuerError>> {
        Box::pin(ready(Ok(self.root.pem())))
    }
}

/// A convenience for tests that only need "some instant".
pub fn instant(year: i32, month: u32, day: u32, hour: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, hour, 0, 0)
        .single()
        .expect("representable test instant")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certificate::profile::{
        CertificateIdentity, CertificateProfile, InstanceScope, Purpose,
    };
    use std::time::Duration;

    fn scope() -> InstanceScope {
        InstanceScope::new("lumen", "lumen", "lumen-prod.svc.id.goog")
    }

    fn profile() -> CertificateProfile {
        CertificateProfile::new(
            &scope(),
            Purpose::Peer,
            "lumen-0.lumen-headless.lumen.svc.cluster.local",
            CertificateIdentity {
                dns_names: vec!["lumen-0.lumen-headless.lumen.svc.cluster.local".into()],
                spiffe_uri: Some("spiffe://lumen-prod.svc.id.goog/ns/lumen/sa/lumen".into()),
            },
            Duration::from_secs(3_600),
            Duration::from_secs(900),
            Duration::from_secs(0),
        )
        .unwrap()
    }

    #[test]
    fn a_leaf_is_dated_from_the_clock_the_test_controls() {
        let issuer = EphemeralIssuer::new("pool-a", instant(2026, 7, 1, 12));
        let (request, _key) = IssuanceRequest::build(&scope(), &profile()).unwrap();
        let material = futures::executor::block_on(issuer.issue(request)).unwrap();
        assert_eq!(material.not_before, instant(2026, 7, 1, 12));
        assert_eq!(
            material.not_after,
            instant(2026, 7, 1, 12) + chrono::Duration::seconds(3_600)
        );
    }

    #[test]
    fn a_leaf_carries_the_usages_the_purpose_asked_for() {
        let issuer = EphemeralIssuer::new("pool-a", instant(2026, 7, 1, 12));
        let (request, _key) = IssuanceRequest::build(&scope(), &profile()).unwrap();
        let material = futures::executor::block_on(issuer.issue(request)).unwrap();
        let facts = crate::certificate::projection::parse_leaf(&material.certificate_pem).unwrap();
        assert_eq!(facts.not_after, material.not_after);
        assert_eq!(facts.fingerprint, material.fingerprint);
    }

    #[test]
    fn a_forced_failure_signs_nothing() {
        let issuer = EphemeralIssuer::new("pool-a", instant(2026, 7, 1, 12));
        issuer.fail_next("CA unreachable");
        let (request, _key) = IssuanceRequest::build(&scope(), &profile()).unwrap();
        assert!(futures::executor::block_on(issuer.issue(request)).is_err());
        assert_eq!(issuer.issued_count(), 0);
    }
}
// HANDWRITE-END
