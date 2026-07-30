// HANDWRITE-BEGIN gap="missing-generator:logic:certificate-reconcile" tracker="#3110" reason="Own the one place that turns a decision into cluster writes, including the authorization check that has to run before a key exists rather than before a Secret is written."
//! Carrying out one decision.
//!
//! Everything difficult already happened elsewhere: [`super::state`] decided
//! *what*, [`super::projection`] decided *where*. This module is the seam, and
//! it has exactly two jobs of its own.
//!
//! **It refuses out-of-scope work first.** Authorization runs before the Secret
//! is read and before a keypair exists — not before the write, which is the
//! natural place to put it and the wrong one. A reconciler that generated a key
//! and asked a CA to sign it, and only then noticed the profile named another
//! instance, has already made the request that mattered. R7 is about what gets
//! *asked for*, not only about what gets stored.
//!
//! **It never widens what it was given.** The reconciler holds one scope, one
//! owner, one issuer. A profile arrives as an argument; a scope does not.

use std::collections::BTreeMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde_json::Value;

use super::issuer::{IssuanceRequest, Issuer, IssuerError, IssuerId};
use super::profile::{CertificateProfile, InstanceScope};
use super::projection::{
    material_secret, read_state, trust_bundle_secret, Owner, TrustBundle, CERT_KEY,
    PRIVATE_KEY_KEY, TRUST_BUNDLE_KEY,
};
use super::state::{next_action, retry_after, Action, Desired, Observed};
use super::status::{redact, CertificateFacts};

/// A Secret as read out of the cluster, already base64-decoded.
#[derive(Clone, Debug, Default)]
pub struct StoredSecret {
    pub data: BTreeMap<String, Vec<u8>>,
    pub annotations: BTreeMap<String, String>,
}

/// Reading and writing the one Secret this lifecycle owns.
///
/// A trait rather than a `kube::Api` so the lifecycle is testable without a
/// cluster, and — the reason that matters more — so the set of operations is
/// finite and inspectable. There is no `delete`.
pub trait SecretStore: Send + Sync {
    fn read<'a>(
        &'a self,
        namespace: &'a str,
        name: &'a str,
    ) -> futures::future::BoxFuture<'a, Result<Option<StoredSecret>, StoreError>>;

    /// Server-side apply `object`. Applying rather than replacing is what lets
    /// [`Action::PublishTrustBundle`] widen `ca.crt` without naming the leaf
    /// keys it must not touch.
    fn apply<'a>(
        &'a self,
        object: Value,
    ) -> futures::future::BoxFuture<'a, Result<(), StoreError>>;
}

/// Why a Secret could not be read or written.
#[derive(Debug)]
pub struct StoreError(pub String);

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for StoreError {}

/// Why a reconcile could not complete.
#[derive(Debug)]
pub enum ReconcileError {
    /// The profile names an instance or namespace this reconciler does not own.
    /// Raised before any key is generated or any Secret is read.
    OutOfScope { requested: String, owned: String },
    Store(StoreError),
    Issuance(IssuerError),
}

impl std::fmt::Display for ReconcileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfScope { requested, owned } => write!(
                f,
                "refusing to act for {requested}: this reconciler owns {owned}"
            ),
            Self::Store(err) => write!(f, "secret store: {}", redact(&err.to_string())),
            Self::Issuance(err) => write!(f, "{}", redact(&err.to_string())),
        }
    }
}

impl std::error::Error for ReconcileError {}

/// What the runtime reports about itself, gathered by the caller.
#[derive(Clone, Debug, Default)]
pub struct RuntimeReport {
    /// Fingerprint of the leaf the process says it is presenting right now.
    pub activated_fingerprint: Option<String>,
    /// Consecutive failures since the last successful reconcile.
    pub consecutive_failures: u32,
}

/// One reconcile's result.
#[derive(Clone, Debug)]
pub struct Outcome {
    /// The step that was taken.
    pub action: Action,
    /// Conditions to merge into the owning resource's status.
    pub facts: CertificateFacts,
    /// When to come back.
    pub requeue_after: Duration,
}

/// Reconciles one instance's certificate of one purpose.
pub struct Reconciler<'a> {
    scope: &'a InstanceScope,
    owner: &'a Owner,
    store: &'a dyn SecretStore,
    issuer: &'a dyn Issuer,
}

impl<'a> Reconciler<'a> {
    pub fn new(
        scope: &'a InstanceScope,
        owner: &'a Owner,
        store: &'a dyn SecretStore,
        issuer: &'a dyn Issuer,
    ) -> Self {
        Self {
            scope,
            owner,
            store,
            issuer,
        }
    }

    /// Take one step toward `profile`'s desired state.
    pub async fn reconcile(
        &self,
        profile: &CertificateProfile,
        runtime: &RuntimeReport,
        now: DateTime<Utc>,
    ) -> Result<Outcome, ReconcileError> {
        // Before anything else, and specifically before a key exists.
        self.authorize(profile.scope())?;

        let purpose = profile.purpose();
        let secret_name = self.scope.secret_name(purpose);
        let stored = self
            .store
            .read(&self.scope.namespace, &secret_name)
            .await
            .map_err(ReconcileError::Store)?
            .unwrap_or_default();
        let projected = read_state(&stored.data, &stored.annotations);

        let desired = Desired {
            profile,
            issuer: self.issuer.id(),
        };
        let observed = Observed {
            leaf: projected.leaf.clone(),
            trust_bundle: projected.bundle.issuers(),
            activated_fingerprint: runtime.activated_fingerprint.clone(),
            consecutive_failures: runtime.consecutive_failures,
        };
        let action = next_action(&desired, &observed, now);

        let mut bundle = projected.bundle.clone();
        match &action {
            Action::PublishTrustBundle { .. } => {
                let anchor = self
                    .issuer
                    .trust_anchor_pem()
                    .await
                    .map_err(ReconcileError::Issuance)?;
                bundle.insert(self.issuer.id(), anchor);
                self.store
                    .apply(trust_bundle_secret(
                        self.scope,
                        purpose,
                        self.owner,
                        &bundle,
                    ))
                    .await
                    .map_err(ReconcileError::Store)?;
            }
            Action::Issue { .. } => {
                self.issue(profile, &mut bundle).await?;
            }
            Action::RetireIssuers { issuers } => {
                let stale = issuers.clone();
                let keep: Vec<IssuerId> = bundle
                    .issuers()
                    .into_iter()
                    .filter(|id| !stale.contains(id))
                    .collect();
                bundle.retain(&keep);
                self.store
                    .apply(trust_bundle_secret(
                        self.scope,
                        purpose,
                        self.owner,
                        &bundle,
                    ))
                    .await
                    .map_err(ReconcileError::Store)?;
            }
            Action::AwaitActivation { .. } | Action::Wait { .. } => {}
        }

        let facts = CertificateFacts::from_action(
            purpose,
            projected.leaf.as_ref().map(|leaf| leaf.issuer.clone()),
            projected.leaf.as_ref().map(|leaf| leaf.not_after),
            projected.leaf.as_ref().map(|leaf| leaf.fingerprint.as_str()),
            bundle.issuers(),
            runtime.consecutive_failures,
            &action,
        );
        let requeue_after = requeue_for(&action, now, runtime.consecutive_failures);
        Ok(Outcome {
            action,
            facts,
            requeue_after,
        })
    }

    /// Generate, sign, and project a new leaf.
    async fn issue(
        &self,
        profile: &CertificateProfile,
        bundle: &mut TrustBundle,
    ) -> Result<(), ReconcileError> {
        let purpose = profile.purpose();
        let identity_digest = profile.identity_digest();
        // The key is generated here and consumed by the apply below. It is never
        // handed to the issuer -- `build` returns it separately for exactly that
        // reason -- and it is never named in an error.
        let (request, private_key) =
            IssuanceRequest::build(self.scope, profile).map_err(ReconcileError::Issuance)?;
        let private_key_pem = private_key.into_pem();
        let material = self
            .issuer
            .issue(request)
            .await
            .map_err(ReconcileError::Issuance)?;
        if !bundle.contains(&material.issuer) {
            bundle.insert(material.issuer.clone(), material.chain_pem.clone());
        }
        self.store
            .apply(material_secret(
                self.scope,
                purpose,
                self.owner,
                &material,
                &private_key_pem,
                bundle,
                &identity_digest,
            ))
            .await
            .map_err(ReconcileError::Store)?;
        Ok(())
    }

    /// The check that has to happen before a request exists.
    fn authorize(&self, requested: &InstanceScope) -> Result<(), ReconcileError> {
        if self.scope.covers(requested) {
            return Ok(());
        }
        Err(ReconcileError::OutOfScope {
            requested: format!("{}/{}", requested.namespace, requested.instance),
            owned: format!("{}/{}", self.scope.namespace, self.scope.instance),
        })
    }
}

/// How long until the next reconcile.
fn requeue_for(action: &Action, now: DateTime<Utc>, failures: u32) -> Duration {
    match action {
        Action::AwaitActivation { recheck_after, .. } => *recheck_after,
        Action::Wait { until } => (*until - now)
            .to_std()
            .unwrap_or_else(|_| Duration::from_secs(0)),
        // A step that changed something is followed immediately, so the next
        // state in the sequence is entered without waiting out a poll interval.
        // Rotation is a several-step dance; pacing it at the requeue interval
        // would make a rotation take minutes for no reason.
        _ if failures == 0 => Duration::from_secs(0),
        _ => retry_after(failures),
    }
}

/// Keys a caller should expect to find in a fully projected Secret.
pub const PROJECTED_KEYS: [&str; 3] = [CERT_KEY, PRIVATE_KEY_KEY, TRUST_BUNDLE_KEY];

/// An in-memory [`SecretStore`], so the lifecycle can be driven without a
/// cluster. Shipped rather than test-gated for the same reason as
/// [`super::ephemeral::EphemeralIssuer`]: R8's no-cloud build has to be able to
/// exercise something.
pub struct MemoryStore {
    secrets: std::sync::Mutex<BTreeMap<String, StoredSecret>>,
    applies: std::sync::atomic::AtomicU64,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            secrets: std::sync::Mutex::new(BTreeMap::new()),
            applies: std::sync::atomic::AtomicU64::new(0),
        }
    }

    pub fn get(&self, namespace: &str, name: &str) -> Option<StoredSecret> {
        self.secrets
            .lock()
            .expect("memory store")
            .get(&format!("{namespace}/{name}"))
            .cloned()
    }

    pub fn apply_count(&self) -> u64 {
        self.applies.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl SecretStore for MemoryStore {
    fn read<'a>(
        &'a self,
        namespace: &'a str,
        name: &'a str,
    ) -> futures::future::BoxFuture<'a, Result<Option<StoredSecret>, StoreError>> {
        Box::pin(futures::future::ready(Ok(self.get(namespace, name))))
    }

    fn apply<'a>(
        &'a self,
        object: Value,
    ) -> futures::future::BoxFuture<'a, Result<(), StoreError>> {
        let namespace = object["metadata"]["namespace"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        let name = object["metadata"]["name"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        let mut secrets = self.secrets.lock().expect("memory store");
        let entry = secrets
            .entry(format!("{namespace}/{name}"))
            .or_insert_with(StoredSecret::default);
        // Merge semantics, matching a server-side apply of the fields present:
        // a trust-bundle-only apply must leave the leaf keys where they are.
        if let Some(data) = object["stringData"].as_object() {
            for (key, value) in data {
                if let Some(text) = value.as_str() {
                    entry.data.insert(key.clone(), text.as_bytes().to_vec());
                }
            }
        }
        if let Some(annotations) = object["metadata"]["annotations"].as_object() {
            for (key, value) in annotations {
                if let Some(text) = value.as_str() {
                    entry.annotations.insert(key.clone(), text.to_string());
                }
            }
        }
        self.applies
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Box::pin(futures::future::ready(Ok(())))
    }
}
