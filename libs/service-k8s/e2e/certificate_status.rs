//! What the lifecycle publishes, and what it refuses to act on (#3110 R6-R7).
//!
//! Every function here is named `certificate_status_*` on purpose. The gate is
//! `cargo test -p service-k8s certificate_status`, and cargo's filter is a
//! substring match on *test function names*, not on file names — a file full of
//! differently-named tests would let that gate pass having run zero of them.

mod certificate_support;

use std::sync::atomic::{AtomicU64, Ordering};

use certificate_support::*;
use futures::future::BoxFuture;
use serde_json::Value;
use service_k8s::certificate::profile::{
    CertificateIdentity, CertificateProfile, InstanceScope, Purpose,
};
use service_k8s::certificate::reconcile::{
    ReconcileError, Reconciler, RuntimeReport, SecretStore, StoreError, StoredSecret,
};
use service_k8s::certificate::status::redact;
use service_k8s::service::ConditionStatus;

/// A store that records whether it was consulted at all.
///
/// The R7 assertions are about *ordering*, and ordering is invisible if you only
/// look at the end state: a reconciler that read the Secret, generated a key,
/// asked the CA to sign it, and only then noticed the profile named another
/// instance would leave exactly the same empty store behind as one that refused
/// up front.
struct RecordingStore {
    reads: AtomicU64,
    applies: AtomicU64,
}

impl RecordingStore {
    fn new() -> Self {
        Self {
            reads: AtomicU64::new(0),
            applies: AtomicU64::new(0),
        }
    }

    fn reads(&self) -> u64 {
        self.reads.load(Ordering::SeqCst)
    }

    fn applies(&self) -> u64 {
        self.applies.load(Ordering::SeqCst)
    }
}

impl SecretStore for RecordingStore {
    fn read<'a>(
        &'a self,
        _namespace: &'a str,
        _name: &'a str,
    ) -> BoxFuture<'a, Result<Option<StoredSecret>, StoreError>> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        Box::pin(futures::future::ready(Ok(None)))
    }

    fn apply<'a>(&'a self, _object: Value) -> BoxFuture<'a, Result<(), StoreError>> {
        self.applies.fetch_add(1, Ordering::SeqCst);
        Box::pin(futures::future::ready(Ok(())))
    }
}

/// A profile that is internally valid but belongs to a different instance.
fn foreign_profile(namespace: &str, instance: &str) -> CertificateProfile {
    let foreign = InstanceScope::new(namespace, instance, TRUST_DOMAIN);
    CertificateProfile::new(
        &foreign,
        Purpose::Peer,
        &format!("{instance}-0.{instance}-headless.{namespace}.svc.cluster.local"),
        CertificateIdentity {
            dns_names: vec![format!(
                "{instance}-0.{instance}-headless.{namespace}.svc.cluster.local"
            )],
            spiffe_uri: Some(format!(
                "spiffe://{TRUST_DOMAIN}/ns/{namespace}/sa/{instance}"
            )),
        },
        LIFETIME,
        RENEW_BEFORE,
        std::time::Duration::ZERO,
    )
    .expect("the foreign profile is valid on its own terms")
}

#[test]
fn certificate_status_reports_ready_once_material_is_projected() {
    let harness = Harness::new("pool-a", start());
    let profile = peer_profile();
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &harness.store, &harness.issuer);

    let actions = drive(&reconciler, &profile, &RuntimeReport::default(), start(), 6);
    assert!(actions.len() >= 2, "expected a bootstrap sequence: {actions:#?}");

    let settled = step(
        &reconciler,
        &profile,
        &activated(harness.projected_fingerprint(Purpose::Peer)),
        plus_hours(start(), 1),
    );
    let conditions = settled.facts.conditions();
    assert_eq!(conditions[0].type_, "PeerCertificateReady");
    assert_eq!(conditions[0].status, ConditionStatus::True);
    assert_eq!(conditions[1].type_, "PeerCertificateRotating");
    assert_eq!(conditions[1].status, ConditionStatus::False);
}

#[test]
fn certificate_status_says_pending_before_anything_is_issued() {
    let harness = Harness::new("pool-a", start());
    let profile = peer_profile();
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &harness.store, &harness.issuer);

    let first = step(&reconciler, &profile, &RuntimeReport::default(), start());
    let conditions = first.facts.conditions();
    assert_eq!(conditions[0].status, ConditionStatus::False);
    assert_eq!(conditions[0].reason, "Pending");
}

#[test]
fn certificate_status_distinguishes_a_stuck_lifecycle_from_a_new_one() {
    let harness = Harness::new("pool-a", start());
    let profile = peer_profile();
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &harness.store, &harness.issuer);

    let struggling = RuntimeReport {
        activated_fingerprint: None,
        consecutive_failures: 5,
    };
    let outcome = step(&reconciler, &profile, &struggling, start());
    let conditions = outcome.facts.conditions();
    assert_eq!(conditions[0].status, ConditionStatus::False);
    assert_eq!(
        conditions[0].reason, "IssuanceFailing",
        "'pending' and 'has been failing for five attempts' are different pages; an alert can \
         only act on one of them"
    );
}

#[test]
fn certificate_status_never_carries_key_material() {
    let harness = Harness::new("pool-a", start());
    let profile = peer_profile();
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &harness.store, &harness.issuer);

    let actions = drive(&reconciler, &profile, &RuntimeReport::default(), start(), 6);
    let _ = actions;

    let secret = harness
        .store
        .get("lumen", &scope.secret_name(Purpose::Peer))
        .expect("secret");
    let key_pem = String::from_utf8(secret.data["tls.key"].clone()).unwrap();
    let key_body: String = key_pem
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect();
    assert!(!key_body.is_empty(), "the fixture must actually have a key");

    let outcome = step(
        &reconciler,
        &profile,
        &activated(harness.projected_fingerprint(Purpose::Peer)),
        plus_hours(start(), 1),
    );
    for condition in outcome.facts.conditions() {
        assert!(
            !condition.message.contains(&key_body),
            "status leaked the private key: {}",
            condition.message
        );
        assert!(
            !condition.message.contains("BEGIN"),
            "status carried a PEM block: {}",
            condition.message
        );
    }
}

#[test]
fn certificate_status_publishes_a_fingerprint_prefix_not_a_certificate() {
    let harness = Harness::new("pool-a", start());
    let profile = peer_profile();
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &harness.store, &harness.issuer);
    drive(&reconciler, &profile, &RuntimeReport::default(), start(), 6);

    let full = harness.projected_fingerprint(Purpose::Peer).unwrap();
    let outcome = step(
        &reconciler,
        &profile,
        &activated(Some(full.clone())),
        plus_hours(start(), 1),
    );
    let published = outcome.facts.fingerprint.expect("a fingerprint");
    assert_eq!(published.len(), 16);
    assert!(full.starts_with(&published));
}

#[test]
fn certificate_status_redacts_a_leaked_pem_block() {
    let leaked = "issuance failed: -----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkq\n-----END PRIVATE KEY-----";
    let cleaned = redact(leaked);
    assert!(!cleaned.contains("MIIEvQIBADANBgkq"));
    assert!(!cleaned.contains("BEGIN PRIVATE KEY"));
}

#[test]
fn certificate_status_redacts_a_leaked_projected_token() {
    let leaked = "CA rejected the request carrying eyJhbGciOiJSUzI1NiIsImtpZCI6ImFiYyJ9.eyJhdWQiOlsic3RzLmdvb2dsZWFwaXMuY29tIl19.c2lnbmF0dXJlYnl0ZXM";
    let cleaned = redact(leaked);
    assert!(!cleaned.contains("eyJhbGciOiJSUzI1NiIsImtpZCI6ImFiYyJ9"));
    assert!(cleaned.contains("[redacted token]"));
}

#[test]
fn certificate_status_redacts_a_leaked_bearer_header() {
    let cleaned = redact("upstream returned 401 for Bearer ya29.c.b0Aaekm1Kexample rest");
    assert!(!cleaned.contains("ya29.c.b0Aaekm1Kexample"));
    assert!(cleaned.ends_with("rest"), "redaction ate the surrounding text: {cleaned}");
}

#[test]
fn certificate_status_refuses_a_foreign_namespace_before_reading_anything() {
    let store = RecordingStore::new();
    let issuer = service_k8s::certificate::ephemeral::EphemeralIssuer::new("pool-a", start());
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &store, &issuer);

    let error = futures::executor::block_on(reconciler.reconcile(
        &foreign_profile("other", "lumen"),
        &RuntimeReport::default(),
        start(),
    ))
    .expect_err("acted for another namespace");

    assert!(
        matches!(error, ReconcileError::OutOfScope { .. }),
        "got {error:?}"
    );
    assert_eq!(store.reads(), 0, "it read a Secret it does not own");
    assert_eq!(store.applies(), 0);
    assert_eq!(
        issuer.issued_count(),
        0,
        "a certificate was requested for another namespace; refusing to *store* it afterwards is \
         too late -- the CA already signed it"
    );
}

#[test]
fn certificate_status_refuses_a_foreign_instance_in_its_own_namespace() {
    let store = RecordingStore::new();
    let issuer = service_k8s::certificate::ephemeral::EphemeralIssuer::new("pool-a", start());
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &store, &issuer);

    let error = futures::executor::block_on(reconciler.reconcile(
        &foreign_profile(NAMESPACE, "lumen-sibling"),
        &RuntimeReport::default(),
        start(),
    ))
    .expect_err("acted for a sibling instance");

    assert!(matches!(error, ReconcileError::OutOfScope { .. }));
    assert_eq!(store.reads(), 0);
    assert_eq!(issuer.issued_count(), 0);
}

#[test]
fn certificate_status_names_both_scopes_when_it_refuses() {
    let store = RecordingStore::new();
    let issuer = service_k8s::certificate::ephemeral::EphemeralIssuer::new("pool-a", start());
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &store, &issuer);

    let error = futures::executor::block_on(reconciler.reconcile(
        &foreign_profile("other", "lumen"),
        &RuntimeReport::default(),
        start(),
    ))
    .expect_err("acted for another namespace");

    let message = error.to_string();
    assert!(message.contains("other/lumen"), "{message}");
    assert!(message.contains("lumen/lumen"), "{message}");
}

#[test]
fn certificate_status_accepts_its_own_scope() {
    // The refusals above are only meaningful if the check is not simply always
    // refusing.
    let store = RecordingStore::new();
    let issuer = service_k8s::certificate::ephemeral::EphemeralIssuer::new("pool-a", start());
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &store, &issuer);

    futures::executor::block_on(reconciler.reconcile(
        &peer_profile(),
        &RuntimeReport::default(),
        start(),
    ))
    .expect("its own scope was refused");
    assert_eq!(store.reads(), 1);
}
