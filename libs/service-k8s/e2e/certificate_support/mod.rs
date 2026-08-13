//! Shared fixtures for the certificate lifecycle tests.
//!
//! Every test in this group drives the same loop — a scope, an owner, an
//! in-memory store, and a signer whose clock the test moves — so the fixtures
//! live here rather than being re-typed four times with small differences that
//! nobody intended.

#![allow(dead_code)]

use std::time::Duration;

use chrono::{DateTime, Utc};
use service_k8s::certificate::ephemeral::{instant, EphemeralIssuer};
use service_k8s::certificate::profile::{
    CertificateIdentity, CertificateProfile, InstanceScope, Purpose,
};
use service_k8s::certificate::projection::Owner;
use service_k8s::certificate::reconcile::{MemoryStore, Outcome, Reconciler, RuntimeReport};
use service_k8s::certificate::state::Action;

pub const NAMESPACE: &str = "lumen";
pub const INSTANCE: &str = "lumen";
pub const TRUST_DOMAIN: &str = "lumen-prod.svc.id.goog";

/// Twelve-hour leaves renewed two hours early, with no jitter.
///
/// Jitter is zero on purpose: it is proven deterministic by its own unit test,
/// and leaving it on here would mean every assertion about a renewal instant had
/// to be a range. A range assertion passes for reasons other than the one it was
/// written for.
pub const LIFETIME: Duration = Duration::from_secs(12 * 3_600);
pub const RENEW_BEFORE: Duration = Duration::from_secs(2 * 3_600);

pub fn scope() -> InstanceScope {
    InstanceScope::new(NAMESPACE, INSTANCE, TRUST_DOMAIN)
}

pub fn owner() -> Owner {
    Owner {
        api_version: "lumen.dev/v1".into(),
        kind: "Lumen".into(),
        name: INSTANCE.into(),
        uid: "0f7d1f4e-0000-4000-8000-000000000000".into(),
    }
}

pub fn peer_profile() -> CertificateProfile {
    CertificateProfile::new(
        &scope(),
        Purpose::Peer,
        "lumen-0.lumen-headless.lumen.svc.cluster.local",
        CertificateIdentity {
            dns_names: vec!["lumen-0.lumen-headless.lumen.svc.cluster.local".into()],
            spiffe_uri: Some("spiffe://lumen-prod.svc.id.goog/ns/lumen/sa/lumen".into()),
        },
        LIFETIME,
        RENEW_BEFORE,
        Duration::ZERO,
    )
    .expect("peer profile")
}

pub fn serving_profile() -> CertificateProfile {
    CertificateProfile::new(
        &scope(),
        Purpose::Serving,
        "lumen.lumen.svc.cluster.local",
        CertificateIdentity {
            dns_names: vec!["lumen.lumen.svc.cluster.local".into()],
            spiffe_uri: None,
        },
        LIFETIME,
        RENEW_BEFORE,
        Duration::ZERO,
    )
    .expect("serving profile")
}

/// The instant every test starts from.
pub fn start() -> DateTime<Utc> {
    instant(2026, 7, 1, 12)
}

pub fn plus_hours(base: DateTime<Utc>, hours: i64) -> DateTime<Utc> {
    base + chrono::Duration::hours(hours)
}

/// One reconcile.
pub fn step(
    reconciler: &Reconciler<'_>,
    profile: &CertificateProfile,
    runtime: &RuntimeReport,
    now: DateTime<Utc>,
) -> Outcome {
    futures::executor::block_on(reconciler.reconcile(profile, runtime, now))
        .expect("reconcile succeeded")
}

/// Reconcile until the loop settles, returning every action in order.
///
/// Bounded rather than looping until stable: an unbounded drive that never
/// settles hangs the suite instead of failing it, and "this converged" is
/// exactly the property most of these tests are asserting.
pub fn drive(
    reconciler: &Reconciler<'_>,
    profile: &CertificateProfile,
    runtime: &RuntimeReport,
    now: DateTime<Utc>,
    limit: usize,
) -> Vec<Action> {
    let mut actions = Vec::new();
    for _ in 0..limit {
        let outcome = step(reconciler, profile, runtime, now);
        let settled = matches!(
            outcome.action,
            Action::Wait { .. } | Action::AwaitActivation { .. }
        );
        actions.push(outcome.action);
        if settled {
            return actions;
        }
    }
    panic!("lifecycle did not settle within {limit} reconciles: {actions:#?}");
}

/// A store, a signer, and the clock they share.
pub struct Harness {
    pub store: MemoryStore,
    pub issuer: EphemeralIssuer,
}

impl Harness {
    pub fn new(issuer_id: &str, now: DateTime<Utc>) -> Self {
        Self {
            store: MemoryStore::new(),
            issuer: EphemeralIssuer::new(issuer_id, now),
        }
    }

    /// Fingerprint of the leaf currently projected for `purpose`, if any.
    pub fn projected_fingerprint(&self, purpose: Purpose) -> Option<String> {
        let secret = self.store.get(NAMESPACE, &scope().secret_name(purpose))?;
        let pem = secret.data.get("tls.crt")?;
        let pem = std::str::from_utf8(pem).ok()?;
        service_k8s::certificate::projection::parse_leaf(pem)
            .ok()
            .map(|facts| facts.fingerprint)
    }

    pub fn projected_keys(&self, purpose: Purpose) -> Vec<String> {
        self.store
            .get(NAMESPACE, &scope().secret_name(purpose))
            .map(|secret| secret.data.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn trust_bundle_pem(&self, purpose: Purpose) -> String {
        self.store
            .get(NAMESPACE, &scope().secret_name(purpose))
            .and_then(|secret| secret.data.get("ca.crt").cloned())
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or_default()
    }
}

/// A runtime that reports it is presenting whatever is projected — the steady
/// state after a successful reload.
pub fn activated(fingerprint: Option<String>) -> RuntimeReport {
    RuntimeReport {
        activated_fingerprint: fingerprint,
        consecutive_failures: 0,
    }
}
