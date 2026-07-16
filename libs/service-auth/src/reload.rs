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
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use axum::http::HeaderMap;
use serde::Serialize;

use crate::middleware::bearer_token;
use crate::role_map::{Role, RoleMapDenied, RoleMapPrincipal, TokenClaims};
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
    let path = path.as_ref().to_owned();
    let poll_interval = if poll_interval.is_zero() {
        Duration::from_secs(1)
    } else {
        poll_interval
    };
    let initial = read_registry_file_state(&path);

    tokio::spawn(async move {
        let mut observed = initial;
        let mut ticker = tokio::time::interval(poll_interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            let current = read_registry_file_state(&path);
            if current == observed {
                continue;
            }
            observed = current;
            if verifier.reload_file(&path).is_err() {
                tracing::warn!(
                    target: "service_auth.audit",
                    path = %path.display(),
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
    tokens: HashMap<String, TokenClaims>,
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
        Self {
            required,
            snapshot: Arc::new(RwLock::new(RegistrySnapshot {
                revision: 0,
                tokens,
            })),
            sink,
        }
    }

    pub fn open() -> Self {
        Self::new(false, HashMap::new())
    }

    pub fn revision(&self) -> u64 {
        self.read_snapshot().revision
    }

    pub fn entry_count(&self) -> usize {
        self.read_snapshot().tokens.len()
    }

    /// Parse, validate, and atomically adopt an inline registry document.
    pub fn reload_json(&self, json: &str) -> Result<u64> {
        let tokens = match serde_json::from_str::<HashMap<String, TokenClaims>>(json) {
            Ok(tokens) => tokens,
            Err(error) => {
                self.record_reload_failure(ReloadFailure::Parse);
                return Err(error).context("replacement token registry must be JSON");
            }
        };
        self.reload_registry(tokens)
    }

    /// Read, parse, validate, and atomically adopt a registry file.
    pub fn reload_file(&self, path: impl AsRef<Path>) -> Result<u64> {
        let path = path.as_ref();
        let json = match std::fs::read_to_string(path) {
            Ok(json) => json,
            Err(error) => {
                self.record_reload_failure(ReloadFailure::Read);
                return Err(error).with_context(|| format!("read registry {}", path.display()));
            }
        };
        self.reload_json(&json)
    }

    /// Validate and atomically adopt an already-parsed replacement.
    pub fn reload_registry(&self, tokens: HashMap<String, TokenClaims>) -> Result<u64> {
        if let Err(error) = validate_registry(self.required, &tokens) {
            self.record_reload_failure(ReloadFailure::Invalid);
            return Err(error);
        }

        let (revision, entries) = {
            let mut current = self.write_snapshot();
            let revision = current.revision.saturating_add(1);
            *current = RegistrySnapshot { revision, tokens };
            (revision, current.tokens.len())
        };
        self.sink.record(&AuthEvent::RegistryReload {
            applied: true,
            revision,
            entries,
            failure: None,
        });
        Ok(revision)
    }

    fn record_reload_failure(&self, failure: ReloadFailure) {
        let current = self.read_snapshot();
        self.sink.record(&AuthEvent::RegistryReload {
            applied: false,
            revision: current.revision,
            entries: current.tokens.len(),
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
            (_, Some(token)) => self
                .read_snapshot()
                .tokens
                .get(token)
                .cloned()
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

fn validate_registry(required: bool, tokens: &HashMap<String, TokenClaims>) -> Result<()> {
    if required && tokens.is_empty() {
        bail!("auth required but replacement registry is empty");
    }
    for (token, claims) in tokens {
        if token.trim().is_empty() {
            bail!("replacement registry contains an empty token key");
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
}
// HANDWRITE-END
