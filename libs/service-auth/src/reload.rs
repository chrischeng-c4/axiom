// HANDWRITE-BEGIN gap="missing-generator:logic:2d86878e" tracker="#1641" reason="Reloadable validated role-map snapshots, audited principals, redacted auth events, and backend-neutral event sinks."
//! Atomic credential-registry reload and redacted authorization audit.
//!
//! The verifier stores one validated role-map snapshot behind a short-lived
//! `RwLock`. Reload parsing and semantic validation happen before the write
//! lock is acquired, so a failed replacement cannot disturb the last-known-
//! good registry. Audit events intentionally have no credential field: an
//! unknown bearer is reported only as an authentication denial.

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use axum::http::HeaderMap;
use serde::Serialize;

use crate::middleware::bearer_token;
use crate::role_map::{Registry, Role, RoleMapDenied, RoleMapPrincipal, TokenClaims};
use crate::{AuthError, Verifier};

/// Production poll cadence for a Secret/CSI-projected token registry. The
/// source file is tiny, and reading it only occurs on this background cadence;
/// requests continue against the last validated in-memory snapshot.
pub const DEFAULT_REGISTRY_FILE_WATCH_INTERVAL: Duration = Duration::from_secs(15);

#[derive(Debug, PartialEq, Eq)]
enum RegistryFileState {
    Bytes(Vec<u8>),
    Unavailable,
}

fn read_registry_file_state(path: &Path) -> RegistryFileState {
    std::fs::read(path)
        .map(RegistryFileState::Bytes)
        .unwrap_or(RegistryFileState::Unavailable)
}

/// Spawn a background watcher for a Secret/CSI-projected registry file using
/// the production cadence. A changed file is fully parsed and validated
/// before [`ReloadableRoleMapVerifier`] adopts it, so invalid rotations retain
/// the previous known-good snapshot. The watcher never logs credential bytes.
pub fn spawn_registry_file_watcher(
    verifier: Arc<ReloadableRoleMapVerifier>,
    path: impl AsRef<Path>,
) -> tokio::task::JoinHandle<()> {
    spawn_registry_file_watcher_with_interval(verifier, path, DEFAULT_REGISTRY_FILE_WATCH_INTERVAL)
}

/// Spawn a registry watcher with an explicit polling cadence. This is public
/// for services with an intentional cadence and for deterministic tests; most
/// services should use [`spawn_registry_file_watcher`].
pub fn spawn_registry_file_watcher_with_interval(
    verifier: Arc<ReloadableRoleMapVerifier>,
    path: impl AsRef<Path>,
    poll_interval: Duration,
) -> tokio::task::JoinHandle<()> {
    spawn_registry_files_watcher_with_interval(verifier, &[path.as_ref().to_owned()], poll_interval)
}

/// Spawn a watcher over every file a service's registry is projected from,
/// using the production cadence.
///
/// The multi-path form exists because the two namespaces can arrive from
/// different Kubernetes objects — an `identities` ConfigMap and a `tokens`
/// Secret (#2764). Reloading only the file that changed would drop the other
/// half of the registry, so a change to *any* watched file re-reads and
/// re-merges *all* of them, and the merged result is adopted as one snapshot.
pub fn spawn_registry_files_watcher(
    verifier: Arc<ReloadableRoleMapVerifier>,
    paths: &[PathBuf],
) -> tokio::task::JoinHandle<()> {
    spawn_registry_files_watcher_with_interval(
        verifier,
        paths,
        DEFAULT_REGISTRY_FILE_WATCH_INTERVAL,
    )
}

/// [`spawn_registry_files_watcher`] with an explicit polling cadence.
pub fn spawn_registry_files_watcher_with_interval(
    verifier: Arc<ReloadableRoleMapVerifier>,
    paths: &[PathBuf],
    poll_interval: Duration,
) -> tokio::task::JoinHandle<()> {
    let paths: Vec<PathBuf> = paths.to_vec();
    let poll_interval = if poll_interval.is_zero() {
        Duration::from_secs(1)
    } else {
        poll_interval
    };
    fn read_all(paths: &[PathBuf]) -> Vec<RegistryFileState> {
        paths.iter().map(|p| read_registry_file_state(p)).collect()
    }
    let initial = read_all(&paths);

    tokio::spawn(async move {
        let mut observed = initial;
        let mut ticker = tokio::time::interval(poll_interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            let current = read_all(&paths);
            if current == observed {
                continue;
            }
            observed = current;
            if verifier.reload_files(&paths).is_err() {
                let named = paths
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                tracing::warn!(
                    target: "service_auth.audit",
                    paths = %named,
                    "credential registry update rejected; retaining last known-good snapshot"
                );
            }
        }
    })
}

/// Stable authorization outcomes exposed to audit/metrics adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationDecision {
    Allow,
    Deny,
}

/// Machine-stable decision reasons. None can carry credential bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationReason {
    Authorized,
    OpenMode,
    MissingBearer,
    UnknownBearer,
    InsufficientRole,
}

/// Machine-stable reload failure classes. Detailed parser/I/O errors are
/// returned to the caller, but the event surface does not echo input bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReloadFailure {
    Read,
    Parse,
    Invalid,
}

/// Redacted lifecycle/audit event delivered to a caller-selected backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum AuthEvent {
    RegistryReload {
        applied: bool,
        revision: u64,
        entries: usize,
        failure: Option<ReloadFailure>,
    },
    AuthorizationDecision {
        decision: AuthorizationDecision,
        reason: AuthorizationReason,
        subject: Option<String>,
        resource: Option<String>,
        needed: Option<Role>,
    },
}

/// Backend-neutral hook for logs, metrics, or SIEM adapters.
pub trait AuthEventSink: Send + Sync {
    fn record(&self, event: &AuthEvent);
}

/// Sink used by tests/dev callers that do not want lifecycle telemetry.
#[derive(Debug, Default)]
pub struct NoopAuthEventSink;

impl AuthEventSink for NoopAuthEventSink {
    fn record(&self, _event: &AuthEvent) {}
}

/// Shared structured tracing adapter. Its fields are copied exclusively from
/// [`AuthEvent`], whose schema cannot represent a bearer credential.
#[derive(Debug, Default)]
pub struct TracingAuthEventSink;

impl AuthEventSink for TracingAuthEventSink {
    fn record(&self, event: &AuthEvent) {
        match event {
            AuthEvent::RegistryReload {
                applied,
                revision,
                entries,
                failure,
            } => {
                if *applied {
                    tracing::info!(
                        target: "service_auth.audit",
                        event = "credential_registry_reload",
                        applied,
                        revision,
                        entries,
                    );
                } else {
                    tracing::warn!(
                        target: "service_auth.audit",
                        event = "credential_registry_reload",
                        applied,
                        revision,
                        entries,
                        failure = ?failure,
                    );
                }
            }
            AuthEvent::AuthorizationDecision {
                decision,
                reason,
                subject,
                resource,
                needed,
            } => {
                let subject = subject.as_deref().unwrap_or("anonymous");
                let resource = resource.as_deref().unwrap_or("-");
                match decision {
                    AuthorizationDecision::Allow => tracing::debug!(
                        target: "service_auth.audit",
                        event = "authorization_decision",
                        decision = ?decision,
                        reason = ?reason,
                        subject,
                        resource,
                        needed = ?needed,
                    ),
                    AuthorizationDecision::Deny => tracing::warn!(
                        target: "service_auth.audit",
                        event = "authorization_decision",
                        decision = ?decision,
                        reason = ?reason,
                        subject,
                        resource,
                        needed = ?needed,
                    ),
                }
            }
        }
    }
}

#[derive(Clone)]
struct RegistrySnapshot {
    revision: u64,
    registry: Registry,
}

/// A principal carrying the shared audit sink used by its verifier.
#[derive(Clone)]
pub struct AuditedRoleMapPrincipal {
    inner: RoleMapPrincipal,
    sink: Arc<dyn AuthEventSink>,
}

impl fmt::Debug for AuditedRoleMapPrincipal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuditedRoleMapPrincipal")
            .field("inner", &self.inner)
            .finish_non_exhaustive()
    }
}

impl AuditedRoleMapPrincipal {
    pub fn ensure(&self, resource: &str, needed: Role) -> std::result::Result<(), RoleMapDenied> {
        match self.inner.ensure(resource, needed) {
            Ok(()) => {
                self.sink.record(&AuthEvent::AuthorizationDecision {
                    decision: AuthorizationDecision::Allow,
                    reason: if self.subject().is_some() {
                        AuthorizationReason::Authorized
                    } else {
                        AuthorizationReason::OpenMode
                    },
                    subject: self.subject().map(str::to_owned),
                    resource: Some(resource.to_owned()),
                    needed: Some(needed),
                });
                Ok(())
            }
            Err(denied) => {
                self.sink.record(&AuthEvent::AuthorizationDecision {
                    decision: AuthorizationDecision::Deny,
                    reason: AuthorizationReason::InsufficientRole,
                    subject: Some(denied.subject.clone()),
                    resource: Some(denied.resource.clone()),
                    needed: Some(denied.needed),
                });
                Err(denied)
            }
        }
    }

    pub fn subject(&self) -> Option<&str> {
        self.inner.subject()
    }

    pub fn into_inner(self) -> RoleMapPrincipal {
        self.inner
    }
}

/// Role-map verifier whose registry can be replaced atomically after startup.
#[derive(Clone)]
pub struct ReloadableRoleMapVerifier {
    required: bool,
    snapshot: Arc<RwLock<RegistrySnapshot>>,
    sink: Arc<dyn AuthEventSink>,
    /// Subjects this service presents on its own behalf, which a tenant
    /// registry may therefore not claim (#2679, R4).
    reserved_subjects: Arc<Vec<String>>,
}

impl fmt::Debug for ReloadableRoleMapVerifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReloadableRoleMapVerifier")
            .field("required", &self.required)
            .field("revision", &self.revision())
            .field("entries", &self.entry_count())
            .finish_non_exhaustive()
    }
}

impl ReloadableRoleMapVerifier {
    pub fn new(required: bool, tokens: HashMap<String, TokenClaims>) -> Self {
        Self::with_sink(required, tokens, Arc::new(NoopAuthEventSink))
    }

    pub fn with_sink(
        required: bool,
        tokens: HashMap<String, TokenClaims>,
        sink: Arc<dyn AuthEventSink>,
    ) -> Self {
        Self::with_registry_and_sink(required, Registry::from_tokens(tokens), sink)
    }

    /// Seed from a two-namespace [`Registry`] (#2678), for a service that also
    /// resolves provider-verified identities.
    pub fn with_registry(required: bool, registry: Registry) -> Self {
        Self::with_registry_and_sink(required, registry, Arc::new(NoopAuthEventSink))
    }

    pub fn with_registry_and_sink(
        required: bool,
        registry: Registry,
        sink: Arc<dyn AuthEventSink>,
    ) -> Self {
        Self {
            required,
            snapshot: Arc::new(RwLock::new(RegistrySnapshot {
                revision: 0,
                registry,
            })),
            sink,
            reserved_subjects: Arc::new(Vec::new()),
        }
    }

    /// Reserve subjects the service presents on its own behalf, so no adopted
    /// registry may claim them (#2679, R4).
    ///
    /// lumen's control plane names itself in every admin call it makes. If a
    /// tenant registry could grant that same subject, the operator's calls and
    /// a tenant's calls would be indistinguishable in audit output — the one
    /// thing an attributable control-plane identity exists to prevent. This is
    /// a builder rather than a constructor argument so the reservation is
    /// visible at the call site that makes it.
    #[must_use]
    pub fn reserving_subjects(mut self, subjects: impl IntoIterator<Item = String>) -> Self {
        self.reserved_subjects = Arc::new(subjects.into_iter().collect());
        self
    }

    /// The subjects reserved by [`Self::reserving_subjects`].
    pub fn reserved_subjects(&self) -> &[String] {
        &self.reserved_subjects
    }

    pub fn open() -> Self {
        Self::new(false, HashMap::new())
    }

    /// Wrap an already-resolved principal in this verifier's audit sink.
    ///
    /// [`Verifier::authenticate`] does this internally, but a verifier that
    /// resolves credentials some other way — [`crate::gcp::GoogleVerifier`]
    /// asks an identity provider — needs the same wrapper so its principals
    /// emit the same authorization events. Without it the Google paths would
    /// authorize silently while the bearer path stayed audited.
    pub fn audited(&self, principal: RoleMapPrincipal) -> AuditedRoleMapPrincipal {
        AuditedRoleMapPrincipal {
            inner: principal,
            sink: Arc::clone(&self.sink),
        }
    }

    pub fn revision(&self) -> u64 {
        self.read_snapshot().revision
    }

    /// Entries across both namespaces in the currently adopted snapshot.
    pub fn entry_count(&self) -> usize {
        self.read_snapshot().registry.len()
    }

    /// Look a **bearer secret** up in the currently adopted snapshot.
    ///
    /// For verifiers that reach the registry outside [`Verifier::authenticate`]
    /// — [`crate::gcp::GoogleVerifier`] checks a pre-shared secret here before
    /// spending a network round trip on introspection. Routing it through the
    /// snapshot, instead of holding a separate map, is what keeps such a
    /// verifier subject to the same atomic rotation and last-known-good
    /// guarantees as `authenticate`.
    ///
    /// The returned value is cloned so the caller cannot hold the read lock
    /// across its own work.
    pub fn lookup_secret(&self, token: &str) -> Option<TokenClaims> {
        self.read_snapshot().registry.tokens.get(token).cloned()
    }

    /// Look a **provider-verified identity** up in the currently adopted
    /// snapshot: [`crate::gcp::GoogleVerifier`] turns a Google credential into
    /// a verified email and lands here.
    ///
    /// Deliberately a different map from [`Self::lookup_secret`] (#2678, R1):
    /// sharing one would let a bearer secret whose text happens to be a valid
    /// email match an identity entry and silently inherit its grants.
    pub fn lookup_identity(&self, identity: &str) -> Option<TokenClaims> {
        self.read_snapshot()
            .registry
            .identities
            .get(identity)
            .cloned()
    }

    /// Parse, validate, and atomically adopt an inline registry document.
    pub fn reload_json(&self, json: &str) -> Result<u64> {
        let registry = match Registry::parse(json) {
            Ok(registry) => registry,
            Err(error) => {
                self.record_reload_failure(ReloadFailure::Parse);
                return Err(error).context("replacement credential registry rejected");
            }
        };
        self.reload_registry(registry)
    }

    /// Read, parse, validate, and atomically adopt a registry file.
    pub fn reload_file(&self, path: impl AsRef<Path>) -> Result<u64> {
        self.reload_files(std::slice::from_ref(&path.as_ref().to_owned()))
    }

    /// Re-read every file the registry is projected from, union them, and
    /// adopt the result as one snapshot.
    ///
    /// All-or-nothing on purpose: when `identities` and `tokens` arrive from
    /// different Kubernetes objects (#2764), adopting only the file that
    /// changed would drop the other namespace entirely. A read or parse
    /// failure on any single file leaves the previous snapshot serving.
    pub fn reload_files(&self, paths: &[PathBuf]) -> Result<u64> {
        let mut registry = Registry::default();
        for path in paths {
            let json = match std::fs::read_to_string(path) {
                Ok(json) => json,
                Err(error) => {
                    self.record_reload_failure(ReloadFailure::Read);
                    return Err(error)
                        .with_context(|| format!("read registry {}", path.display()));
                }
            };
            let parsed = match Registry::parse(&json) {
                Ok(parsed) => parsed,
                Err(error) => {
                    self.record_reload_failure(ReloadFailure::Parse);
                    return Err(error).with_context(|| {
                        format!("replacement credential registry {} rejected", path.display())
                    });
                }
            };
            if let Err(error) = registry.try_merge(parsed) {
                self.record_reload_failure(ReloadFailure::Invalid);
                return Err(error)
                    .with_context(|| format!("merging registry {}", path.display()));
            }
        }
        self.reload_registry(registry)
    }

    /// Validate and atomically adopt an already-parsed replacement.
    pub fn reload_registry(&self, registry: Registry) -> Result<u64> {
        if let Err(error) = self.validate(&registry) {
            self.record_reload_failure(ReloadFailure::Invalid);
            return Err(error);
        }

        let (revision, entries) = {
            let mut current = self.write_snapshot();
            let revision = current.revision.saturating_add(1);
            *current = RegistrySnapshot { revision, registry };
            (revision, current.registry.len())
        };
        self.sink.record(&AuthEvent::RegistryReload {
            applied: true,
            revision,
            entries,
            failure: None,
        });
        Ok(revision)
    }

    /// Everything a candidate snapshot must satisfy before adoption: the
    /// shared structural rules, plus this verifier's own reservations.
    fn validate(&self, registry: &Registry) -> Result<()> {
        validate_registry(self.required, registry)?;
        if let Some((section, key, subject)) =
            registry.reserved_subject_violation(&self.reserved_subjects)
        {
            bail!(
                "replacement registry `{section}` entry `{key}` claims the reserved subject \
                 `{subject}`, which this service presents on its own behalf"
            );
        }
        Ok(())
    }

    fn record_reload_failure(&self, failure: ReloadFailure) {
        let current = self.read_snapshot();
        self.sink.record(&AuthEvent::RegistryReload {
            applied: false,
            revision: current.revision,
            entries: current.registry.len(),
            failure: Some(failure),
        });
    }

    fn read_snapshot(&self) -> std::sync::RwLockReadGuard<'_, RegistrySnapshot> {
        self.snapshot
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn write_snapshot(&self) -> std::sync::RwLockWriteGuard<'_, RegistrySnapshot> {
        self.snapshot
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn denied(&self, reason: AuthorizationReason) -> AuthError {
        self.sink.record(&AuthEvent::AuthorizationDecision {
            decision: AuthorizationDecision::Deny,
            reason,
            subject: None,
            resource: None,
            needed: None,
        });
        AuthError::Unauthenticated
    }
}

impl Verifier for ReloadableRoleMapVerifier {
    type Principal = AuditedRoleMapPrincipal;

    fn authenticate(&self, headers: &HeaderMap) -> std::result::Result<Self::Principal, AuthError> {
        let principal = match (self.required, bearer_token(headers)) {
            (false, None) => RoleMapPrincipal::Open,
            // Bearer secrets resolve against `tokens` only. A secret that
            // happens to be a valid email must not reach an identity entry
            // (#2678, R1) — presenting a string is not proving an identity.
            (_, Some(token)) => self
                .lookup_secret(token)
                .map(RoleMapPrincipal::Token)
                .ok_or_else(|| self.denied(AuthorizationReason::UnknownBearer))?,
            (true, None) => return Err(self.denied(AuthorizationReason::MissingBearer)),
        };
        Ok(AuditedRoleMapPrincipal {
            inner: principal,
            sink: Arc::clone(&self.sink),
        })
    }

    fn required(&self) -> bool {
        self.required
    }
}

fn validate_registry(required: bool, registry: &Registry) -> Result<()> {
    if required && registry.is_empty() {
        bail!("auth required but replacement registry is empty");
    }
    validate_entries(&registry.tokens, "token")?;
    validate_entries(&registry.identities, "identity")?;
    // An identity key is an email an identity provider vouched for. Rejecting
    // anything else keeps a bearer secret pasted into the wrong section from
    // being adopted as a grant nobody can trace back to a person (#2678, R2).
    for identity in registry.identities.keys() {
        if !identity.contains('@') {
            bail!("replacement registry identity key is not an email address");
        }
    }
    Ok(())
}

fn validate_entries(entries: &HashMap<String, TokenClaims>, kind: &str) -> Result<()> {
    for (key, claims) in entries {
        if key.trim().is_empty() {
            bail!("replacement registry contains an empty {kind} key");
        }
        if claims.subject.trim().is_empty() {
            bail!("replacement registry contains an empty subject");
        }
        if claims
            .roles
            .keys()
            .any(|resource| resource.trim().is_empty())
        {
            bail!("replacement registry contains an empty resource key");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::header;
    use std::sync::Mutex;

    #[derive(Default)]
    struct RecordingSink(Mutex<Vec<AuthEvent>>);

    impl AuthEventSink for RecordingSink {
        fn record(&self, event: &AuthEvent) {
            self.0.lock().unwrap().push(event.clone());
        }
    }

    fn claims(subject: &str, role: Role) -> TokenClaims {
        TokenClaims {
            subject: subject.to_owned(),
            roles: HashMap::from([("resource".to_owned(), role)]),
        }
    }

    fn headers(token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            format!("Bearer {token}").parse().unwrap(),
        );
        headers
    }

    #[test]
    fn valid_rotation_is_immediately_visible_and_advances_revision() {
        let verifier = ReloadableRoleMapVerifier::new(
            true,
            HashMap::from([("old".to_owned(), claims("alice", Role::Read))]),
        );
        assert_eq!(
            verifier.authenticate(&headers("old")).unwrap().subject(),
            Some("alice")
        );
        let revision = verifier
            .reload_json(r#"{"new":{"subject":"bob","roles":{"resource":"write"}}}"#)
            .unwrap();
        assert_eq!(revision, 1);
        assert!(verifier.authenticate(&headers("old")).is_err());
        let principal = verifier.authenticate(&headers("new")).unwrap();
        assert_eq!(principal.subject(), Some("bob"));
        assert!(principal.ensure("resource", Role::Write).is_ok());
    }

    #[test]
    fn invalid_replacements_preserve_last_known_good_snapshot() {
        let verifier = ReloadableRoleMapVerifier::new(
            true,
            HashMap::from([("old".to_owned(), claims("alice", Role::Admin))]),
        );
        assert!(verifier.reload_json("not-json").is_err());
        assert!(verifier.reload_json("{}").is_err());
        assert!(verifier
            .reload_json(r#"{"new":{"subject":"","roles":{}}}"#)
            .is_err());
        assert_eq!(verifier.revision(), 0);
        assert_eq!(verifier.entry_count(), 1);
        assert!(verifier.authenticate(&headers("old")).is_ok());
    }

    // -- #2678 AC5: rotation covers both namespaces ------------------------

    /// A rotation that adds an identity is visible to `lookup_identity` and
    /// stays invisible to the bearer path — the reload-time half of the same
    /// disjointness the verifier enforces per request.
    #[test]
    fn rotation_can_add_identities_without_widening_the_bearer_namespace() {
        let verifier = ReloadableRoleMapVerifier::new(
            true,
            HashMap::from([("s3cret".to_owned(), claims("svc", Role::Admin))]),
        );

        verifier
            .reload_json(
                r#"{"tokens":{"s3cret":{"subject":"svc","roles":{"*":"admin"}}},
                    "identities":{"a@b.com":{"subject":"dev","roles":{"*":"read"}}}}"#,
            )
            .unwrap();

        assert_eq!(verifier.entry_count(), 2);
        assert_eq!(verifier.lookup_identity("a@b.com").unwrap().subject, "dev");
        assert!(verifier.lookup_secret("a@b.com").is_none());
        assert!(
            verifier.authenticate(&headers("a@b.com")).is_err(),
            "presenting the email as a bearer secret must not authenticate"
        );
    }

    /// AC5. A malformed identity-keyed rotation is rejected whole — the
    /// previous registry keeps serving rather than half-applying. An identity
    /// key that is not an email address is the specific malformation an
    /// operator hits by pasting a bearer secret into the wrong section.
    #[test]
    fn a_malformed_identity_rotation_leaves_the_previous_registry_serving() {
        let verifier = ReloadableRoleMapVerifier::with_registry(
            true,
            Registry {
                tokens: HashMap::from([("s3cret".to_owned(), claims("svc", Role::Admin))]),
                identities: HashMap::from([("a@b.com".to_owned(), claims("dev", Role::Read))]),
            },
        );

        for bad in [
            // an identity key that is not an email — a pasted bearer secret
            r#"{"identities":{"not-an-email":{"subject":"dev","roles":{"*":"read"}}}}"#,
            // an identity entry with no subject to audit
            r#"{"identities":{"a@b.com":{"subject":"","roles":{"*":"read"}}}}"#,
            // the section is present but is not a map of claims
            r#"{"identities":["a@b.com"]}"#,
        ] {
            assert!(verifier.reload_json(bad).is_err(), "accepted `{bad}`");
        }

        assert_eq!(verifier.revision(), 0);
        assert_eq!(verifier.entry_count(), 2);
        assert_eq!(verifier.lookup_identity("a@b.com").unwrap().subject, "dev");
        assert!(verifier.authenticate(&headers("s3cret")).is_ok());
    }

    #[test]
    fn authorization_events_are_typed_and_credential_free() {
        let sink = Arc::new(RecordingSink::default());
        let verifier = ReloadableRoleMapVerifier::with_sink(
            true,
            HashMap::from([("supersecret".to_owned(), claims("alice", Role::Read))]),
            sink.clone(),
        );
        let principal = verifier.authenticate(&headers("supersecret")).unwrap();
        assert!(principal.ensure("resource", Role::Write).is_err());
        assert!(verifier.authenticate(&headers("another-secret")).is_err());

        let events = sink.0.lock().unwrap();
        let json = serde_json::to_string(&*events).unwrap();
        assert!(!json.contains("supersecret"));
        assert!(!json.contains("another-secret"));
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"credential\""));
        assert!(events.iter().any(|event| matches!(
            event,
            AuthEvent::AuthorizationDecision {
                decision: AuthorizationDecision::Deny,
                reason: AuthorizationReason::InsufficientRole,
                subject: Some(subject),
                ..
            } if subject == "alice"
        )));
    }

    #[test]
    fn failed_file_reload_emits_read_failure_without_losing_registry() {
        let sink = Arc::new(RecordingSink::default());
        let verifier = ReloadableRoleMapVerifier::with_sink(
            true,
            HashMap::from([("old".to_owned(), claims("alice", Role::Read))]),
            sink.clone(),
        );
        assert!(verifier
            .reload_file("/definitely/missing/registry.json")
            .is_err());
        assert!(verifier.authenticate(&headers("old")).is_ok());
        assert!(sink.0.lock().unwrap().iter().any(|event| matches!(
            event,
            AuthEvent::RegistryReload {
                applied: false,
                failure: Some(ReloadFailure::Read),
                ..
            }
        )));
    }

    #[tokio::test]
    async fn file_watcher_adopts_a_valid_replacement_without_restart() {
        let path = std::env::temp_dir().join(format!(
            "service-auth-registry-watch-{}.json",
            std::process::id()
        ));
        std::fs::write(
            &path,
            r#"{"old":{"subject":"alice","roles":{"resource":"read"}}}"#,
        )
        .unwrap();
        let verifier = Arc::new(ReloadableRoleMapVerifier::new(
            true,
            HashMap::from([("old".to_owned(), claims("alice", Role::Read))]),
        ));
        let task = spawn_registry_file_watcher_with_interval(
            Arc::clone(&verifier),
            &path,
            Duration::from_millis(5),
        );

        // Let the task capture the initial file before replacing the mounted
        // content, then wait until the shared verifier publishes the change.
        tokio::task::yield_now().await;
        std::fs::write(
            &path,
            r#"{"new":{"subject":"bob","roles":{"resource":"admin"}}}"#,
        )
        .unwrap();
        for _ in 0..20 {
            if verifier.authenticate(&headers("new")).is_ok() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }

        assert!(verifier.authenticate(&headers("old")).is_err());
        assert_eq!(
            verifier.authenticate(&headers("new")).unwrap().subject(),
            Some("bob")
        );
        task.abort();
        std::fs::remove_file(path).ok();
    }

    // -- #2764: reloading a registry that arrives as two files -------------

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("service-auth-reload-2764-{name}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// The trap this test exists for: reloading only the file that changed
    /// would publish a snapshot containing that file alone, silently dropping
    /// the other namespace. A ConfigMap edit would revoke every bearer secret.
    #[test]
    fn reloading_one_of_two_files_keeps_the_other_file_serving() {
        let dir = scratch("both");
        let identities = dir.join("identities.json");
        let tokens = dir.join("token-registry.json");
        std::fs::write(
            &identities,
            r#"{"identities":{"a@b.com":{"subject":"dev","roles":{"resource":"read"}}}}"#,
        )
        .unwrap();
        std::fs::write(
            &tokens,
            r#"{"tokens":{"s3cret":{"subject":"svc","roles":{"resource":"write"}}}}"#,
        )
        .unwrap();

        let verifier = ReloadableRoleMapVerifier::new(
            true,
            HashMap::from([("s3cret".to_owned(), claims("svc", Role::Write))]),
        );
        let paths = vec![identities.clone(), tokens.clone()];
        verifier.reload_files(&paths).unwrap();
        assert!(verifier.authenticate(&headers("s3cret")).is_ok());
        assert!(verifier.lookup_identity("a@b.com").is_some());

        // Edit only the ConfigMap half.
        std::fs::write(
            &identities,
            r#"{"identities":{"c@d.com":{"subject":"ops","roles":{"resource":"admin"}}}}"#,
        )
        .unwrap();
        verifier.reload_files(&paths).unwrap();

        assert!(verifier.lookup_identity("a@b.com").is_none());
        assert!(verifier.lookup_identity("c@d.com").is_some());
        assert!(
            verifier.authenticate(&headers("s3cret")).is_ok(),
            "editing the identity map must not revoke the bearer secrets"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    /// All-or-nothing across sources. A half-written ConfigMap must not take
    /// the Secret's entries down with it.
    #[test]
    fn one_unreadable_source_leaves_the_previous_snapshot_serving() {
        let dir = scratch("partial");
        let good = dir.join("identities.json");
        std::fs::write(
            &good,
            r#"{"identities":{"a@b.com":{"subject":"dev","roles":{"resource":"read"}}}}"#,
        )
        .unwrap();

        let sink = Arc::new(RecordingSink::default());
        let verifier = ReloadableRoleMapVerifier::with_sink(
            true,
            HashMap::from([("old".to_owned(), claims("alice", Role::Read))]),
            sink.clone(),
        );

        let err = verifier
            .reload_files(&[good.clone(), dir.join("does-not-exist.json")])
            .unwrap_err();
        assert!(format!("{err:#}").contains("does-not-exist"), "{err:#}");

        assert!(verifier.authenticate(&headers("old")).is_ok());
        assert!(
            verifier.lookup_identity("a@b.com").is_none(),
            "the readable half must not be published on its own"
        );
        assert!(sink.0.lock().unwrap().iter().any(|event| matches!(
            event,
            AuthEvent::RegistryReload {
                applied: false,
                failure: Some(ReloadFailure::Read),
                ..
            }
        )));

        std::fs::remove_dir_all(&dir).ok();
    }

    /// #2679. A tenant-editable ConfigMap that grants the control plane's own
    /// subject is rejected at reload, not at request time: accepting it would
    /// let a tenant credential act as the operator and sign every audit line
    /// with the operator's name.
    #[test]
    fn a_replacement_claiming_the_reserved_subject_is_refused() {
        let verifier = ReloadableRoleMapVerifier::new(
            true,
            HashMap::from([("old".to_owned(), claims("alice", Role::Read))]),
        )
        .reserving_subjects(["lumen-control-plane".to_owned()]);
        assert_eq!(verifier.reserved_subjects(), ["lumen-control-plane"]);

        let err = verifier
            .reload_json(
                r#"{"identities":{"tenant@b.com":{"subject":"lumen-control-plane","roles":{"resource":"admin"}}}}"#,
            )
            .unwrap_err();
        let message = format!("{err:#}");
        assert!(message.contains("lumen-control-plane"), "{message}");
        assert!(message.contains("tenant@b.com"), "{message}");

        assert_eq!(verifier.revision(), 0);
        assert!(verifier.authenticate(&headers("old")).is_ok());
    }

    /// The watcher's contract when the registry spans two files: a change to
    /// either one republishes the union.
    #[tokio::test]
    async fn the_multi_file_watcher_republishes_the_union_on_any_change() {
        let dir = scratch(&format!("watch-{}", std::process::id()));
        let identities = dir.join("identities.json");
        let tokens = dir.join("token-registry.json");
        std::fs::write(&identities, r#"{"identities":{}}"#).unwrap();
        std::fs::write(
            &tokens,
            r#"{"tokens":{"s3cret":{"subject":"svc","roles":{"resource":"write"}}}}"#,
        )
        .unwrap();

        let verifier = Arc::new(ReloadableRoleMapVerifier::new(
            true,
            HashMap::from([("s3cret".to_owned(), claims("svc", Role::Write))]),
        ));
        let task = spawn_registry_files_watcher_with_interval(
            Arc::clone(&verifier),
            &[identities.clone(), tokens.clone()],
            Duration::from_millis(5),
        );

        tokio::task::yield_now().await;
        std::fs::write(
            &identities,
            r#"{"identities":{"a@b.com":{"subject":"dev","roles":{"resource":"read"}}}}"#,
        )
        .unwrap();
        for _ in 0..40 {
            if verifier.lookup_identity("a@b.com").is_some() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }

        assert!(verifier.lookup_identity("a@b.com").is_some());
        assert!(
            verifier.authenticate(&headers("s3cret")).is_ok(),
            "the untouched file stays in the published snapshot"
        );

        task.abort();
        std::fs::remove_dir_all(&dir).ok();
    }
}
// HANDWRITE-END
