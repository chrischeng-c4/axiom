// HANDWRITE-BEGIN gap="missing-generator:logic:k8s-delegated-auth" tracker="#2869" reason="The delegated authenticate/authorize state machine, its fail-closed outage path, and its credential-free metrics; no generator primitive models this policy."
//! Delegating both halves of the question to kube-apiserver.
//!
//! A service that adopts this stops having an opinion about who its callers
//! are. `TokenReview` answers "whose token is this?", `SubjectAccessReview`
//! answers "may they do this?", and everything in between — audience checking,
//! ServiceAccount-only admission, caching, and what to do when the apiserver
//! is unreachable — is the policy in this module.
//!
//! Three properties are worth stating up front, because they are the ones a
//! reader should be able to check:
//!
//! - **Nothing here can produce an allow that the apiserver did not.** The only
//!   sources of a positive answer are a live review and a cache entry that a
//!   live review put there. There is no configuration, no default, and no
//!   error path that yields access.
//! - **A raw token is never stored, logged, or embedded in an error.** The
//!   cache is keyed by a SHA-256 digest of the token, and the only
//!   token-derived value that escapes is a truncated [`fingerprint`], which
//!   correlates audit lines without being a credential.
//! - **The outage path is bounded and one-directional.** When a review fails,
//!   an already-cached answer may be reused for [`CachePolicy::stale_window`]
//!   past its TTL — and then never again. "The apiserver is down" is not a
//!   reason to keep serving.
//!
//! This module knows nothing about any service's resources. It is handed
//! [`ResourceAttributes`] and returns a verdict; naming what a resource *is* is
//! the calling service's job.

use std::sync::Arc;
use std::time::Duration;

use metrics_prometheus::{render, Counter, Sample};
use sha2::{Digest, Sha256};

use super::cache::{CacheOutcome, CachePolicy, Clock, SystemClock, TtlCache};
use super::principal::{PrincipalRejection, ServiceAccountPrincipal};
use super::review::{ResourceAttributes, ReviewBackend, ReviewError};
use crate::AuthError;

/// A SHA-256 digest of a bearer token, used as a cache key.
///
/// The digest is the key precisely so that a memory dump, a debugger, or a
/// `Debug` print of the cache cannot yield a usable credential.
type TokenDigest = [u8; 32];

fn digest(token: &str) -> TokenDigest {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hasher.finalize().into()
}

/// A short, stable, non-reversible handle on a token, for correlating audit
/// lines about the same caller. Six bytes of SHA-256 — enough to correlate,
/// far too little to be a credential.
pub fn fingerprint(token: &str) -> String {
    digest(token)[..6]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Why a credential was not accepted. A classification, never an echo of the
/// value: rejected usernames are routinely email addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthRejection {
    /// No `Authorization: Bearer` credential was presented.
    MissingCredential,
    /// The token is valid, but not for this service. The single most important
    /// rejection here: it is what stops a token minted for kube-apiserver from
    /// being replayed against a delegating service.
    AudienceMismatch,
    /// The reviewed identity is not an acceptable caller.
    Principal(PrincipalRejection),
}

impl AuthRejection {
    /// A stable, credential-free token for logs and metrics.
    pub fn reason(self) -> &'static str {
        match self {
            Self::MissingCredential => "missing_credential",
            Self::AudienceMismatch => "audience_mismatch",
            Self::Principal(inner) => inner.reason(),
        }
    }
}

impl std::fmt::Display for AuthRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.reason())
    }
}

/// The three outcomes a delegated check can have, kept distinct all the way to
/// the HTTP layer.
///
/// Collapsing [`Unavailable`](Self::Unavailable) into a deny would be safe but
/// dishonest — it tells an operator their RBAC is wrong when their apiserver is
/// unreachable. Collapsing it into an allow is the failure this design exists
/// to prevent.
#[derive(Debug, Clone)]
pub enum DelegatedAuthError {
    /// The caller is not who they need to be — 401.
    Unauthenticated(AuthRejection),
    /// The caller is known and not permitted — 403.
    Denied(ResourceAttributes),
    /// No decision could be reached — 503.
    Unavailable(ReviewError),
}

impl DelegatedAuthError {
    /// A stable, credential-free token for logs and metrics.
    pub fn reason(&self) -> &'static str {
        match self {
            Self::Unauthenticated(rejection) => rejection.reason(),
            Self::Denied(_) => "denied",
            Self::Unavailable(error) => error.reason(),
        }
    }
}

impl std::fmt::Display for DelegatedAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unauthenticated(rejection) => write!(f, "unauthenticated: {rejection}"),
            Self::Denied(attributes) => write!(f, "not permitted to {attributes}"),
            Self::Unavailable(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for DelegatedAuthError {}

impl From<DelegatedAuthError> for AuthError {
    fn from(error: DelegatedAuthError) -> Self {
        match error {
            // The rejection reason is a classification, so it is safe to
            // return; it tells an operator reading a client's logs which of
            // the several 401 conditions they hit.
            DelegatedAuthError::Unauthenticated(_) => AuthError::Unauthenticated,
            DelegatedAuthError::Denied(attributes) => {
                AuthError::Forbidden(format!("not permitted to {attributes}"))
            }
            DelegatedAuthError::Unavailable(_) => {
                AuthError::Unavailable("authorization is temporarily unavailable".into())
            }
        }
    }
}

/// A configuration that cannot describe a safe delegated authenticator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissingAudience;

impl std::fmt::Display for MissingAudience {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            "DelegatedAuthConfig::new requires at least one audience; use the explicitly named \
             kubernetes_default constructor only when default Kubernetes ServiceAccount tokens \
             are the intended caller credential",
        )
    }
}

impl std::error::Error for MissingAudience {}

/// What this service will accept, and for how long.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegatedAuthConfig {
    audiences: Vec<String>,
    /// An explicit opt-in to TokenReview's Kubernetes-default audience mode.
    /// This can never be reached through [`Self::new`].
    kubernetes_default: bool,
    /// Cache TTLs and the stale window. See [`CachePolicy`].
    pub cache: CachePolicy,
}

impl DelegatedAuthConfig {
    /// Build a configuration. At least one audience is mandatory: a
    /// `TokenReview` with an empty audience list validates against the
    /// apiserver's own audience, which means every pod's default token in the
    /// cluster would authenticate here.
    pub fn new(audiences: Vec<String>) -> Result<Self, MissingAudience> {
        let audiences: Vec<String> = audiences
            .into_iter()
            .filter(|audience| !audience.is_empty())
            .collect();
        if audiences.is_empty() {
            return Err(MissingAudience);
        }
        Ok(Self {
            audiences,
            kubernetes_default: false,
            cache: CachePolicy::default(),
        })
    }

    /// Accept the default ServiceAccount token mounted by Kubernetes.
    ///
    /// This deliberately asks TokenReview to use the apiserver's configured
    /// audiences. It is for in-cluster services whose public contract says a
    /// caller's default KSA identity is the credential. Services that mint a
    /// private audience must continue to use [`Self::new`].
    pub fn kubernetes_default() -> Self {
        Self {
            audiences: Vec::new(),
            kubernetes_default: true,
            cache: CachePolicy::default(),
        }
    }

    /// The audiences to put in TokenReview. Empty is meaningful only when
    /// [`Self::uses_kubernetes_default`] is true; the kube backend then omits
    /// `spec.audiences` rather than sending an empty array.
    pub fn audiences(&self) -> &[String] {
        &self.audiences
    }

    pub fn uses_kubernetes_default(&self) -> bool {
        self.kubernetes_default
    }

    pub fn with_cache_policy(mut self, cache: CachePolicy) -> Self {
        self.cache = cache;
        self
    }
}

/// Counters for the delegated path. Every one of them is derived from a
/// classification, never from a caller-supplied string, so no scrape can be
/// made to carry a credential or an unbounded label set.
#[derive(Debug, Default)]
pub struct DelegatedAuthMetrics {
    pub token_reviews: Counter,
    pub token_cache_hits: Counter,
    pub token_cache_misses: Counter,
    pub token_cache_stale: Counter,
    pub authenticated: Counter,
    pub unauthenticated: Counter,
    pub access_reviews: Counter,
    pub access_cache_hits: Counter,
    pub access_cache_misses: Counter,
    pub access_cache_stale: Counter,
    pub allowed: Counter,
    pub denied: Counter,
    pub review_failures: Counter,
    pub unavailable: Counter,
}

impl DelegatedAuthMetrics {
    pub fn samples(&self) -> Vec<Sample<'static>> {
        vec![
            Sample::new(
                "delegated_auth_token_reviews_total",
                "counter",
                "TokenReview calls made to the apiserver",
                self.token_reviews.get(),
            ),
            Sample::new(
                "delegated_auth_token_cache_hits_total",
                "counter",
                "Authentications served from an unexpired cache entry",
                self.token_cache_hits.get(),
            ),
            Sample::new(
                "delegated_auth_token_cache_misses_total",
                "counter",
                "Authentications that required a TokenReview call",
                self.token_cache_misses.get(),
            ),
            Sample::new(
                "delegated_auth_token_cache_stale_total",
                "counter",
                "Authentications served from an expired cache entry during an apiserver outage",
                self.token_cache_stale.get(),
            ),
            Sample::new(
                "delegated_auth_authenticated_total",
                "counter",
                "Callers accepted as Kubernetes ServiceAccounts",
                self.authenticated.get(),
            ),
            Sample::new(
                "delegated_auth_unauthenticated_total",
                "counter",
                "Credentials rejected before authorization",
                self.unauthenticated.get(),
            ),
            Sample::new(
                "delegated_auth_access_reviews_total",
                "counter",
                "SubjectAccessReview calls made to the apiserver",
                self.access_reviews.get(),
            ),
            Sample::new(
                "delegated_auth_access_cache_hits_total",
                "counter",
                "Authorizations served from an unexpired cache entry",
                self.access_cache_hits.get(),
            ),
            Sample::new(
                "delegated_auth_access_cache_misses_total",
                "counter",
                "Authorizations that required a SubjectAccessReview call",
                self.access_cache_misses.get(),
            ),
            Sample::new(
                "delegated_auth_access_cache_stale_total",
                "counter",
                "Authorizations served from an expired cache entry during an apiserver outage",
                self.access_cache_stale.get(),
            ),
            Sample::new(
                "delegated_auth_allowed_total",
                "counter",
                "Authorization decisions that allowed the operation",
                self.allowed.get(),
            ),
            Sample::new(
                "delegated_auth_denied_total",
                "counter",
                "Authorization decisions that denied the operation",
                self.denied.get(),
            ),
            Sample::new(
                "delegated_auth_review_failures_total",
                "counter",
                "Review calls that returned no usable answer",
                self.review_failures.get(),
            ),
            Sample::new(
                "delegated_auth_unavailable_total",
                "counter",
                "Requests failed closed because no decision could be reached",
                self.unavailable.get(),
            ),
        ]
    }

    /// Prometheus text for this metric set, for a service that exposes the
    /// delegated-auth counters on its own scrape endpoint.
    pub fn render(&self) -> String {
        render(&self.samples())
    }

    fn record_token_cache(&self, outcome: CacheOutcome) {
        match outcome {
            CacheOutcome::Hit => self.token_cache_hits.incr(),
            CacheOutcome::Miss => self.token_cache_misses.incr(),
            CacheOutcome::Stale => self.token_cache_stale.incr(),
        }
    }

    fn record_access_cache(&self, outcome: CacheOutcome) {
        match outcome {
            CacheOutcome::Hit => self.access_cache_hits.incr(),
            CacheOutcome::Miss => self.access_cache_misses.incr(),
            CacheOutcome::Stale => self.access_cache_stale.incr(),
        }
    }
}

/// The cache key for one authorization decision.
///
/// It carries the *whole* reviewed identity, not just the username: RBAC binds
/// by group and policy may read `extra`, so two callers with the same username
/// and different groups are genuinely different questions. Keying on the
/// username alone would let one caller's allow answer another caller's request.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DecisionKey {
    identity: super::principal::ReviewedIdentity,
    attributes: ResourceAttributes,
}

/// Authenticates bearer tokens through `TokenReview` and authorizes operations
/// through `SubjectAccessReview`.
pub struct DelegatedAuthenticator {
    backend: Arc<dyn ReviewBackend>,
    config: DelegatedAuthConfig,
    /// Cached authentications. The error arm is cached too, so a stream of bad
    /// tokens cannot be used to generate apiserver load.
    tokens: TtlCache<TokenDigest, Result<ServiceAccountPrincipal, AuthRejection>>,
    decisions: TtlCache<DecisionKey, bool>,
    metrics: Arc<DelegatedAuthMetrics>,
}

impl DelegatedAuthenticator {
    pub fn new(backend: Arc<dyn ReviewBackend>, config: DelegatedAuthConfig) -> Self {
        Self::with_clock(backend, config, Arc::new(SystemClock))
    }

    /// The same authenticator on an injectable clock, so a caller can prove its
    /// own revocation bound without waiting for one.
    pub fn with_clock(
        backend: Arc<dyn ReviewBackend>,
        config: DelegatedAuthConfig,
        clock: Arc<dyn Clock>,
    ) -> Self {
        let policy = config.cache;
        Self {
            backend,
            config,
            tokens: TtlCache::new(policy, clock.clone()),
            decisions: TtlCache::new(policy, clock),
            metrics: Arc::new(DelegatedAuthMetrics::default()),
        }
    }

    pub fn metrics(&self) -> &Arc<DelegatedAuthMetrics> {
        &self.metrics
    }

    pub fn config(&self) -> &DelegatedAuthConfig {
        &self.config
    }

    /// Drop every cached answer. For a service that learns out-of-band that
    /// its policy changed; correctness never depends on it being called.
    pub fn invalidate(&self) {
        self.tokens.clear();
        self.decisions.clear();
    }

    /// Resolve a bearer token to the ServiceAccount that presented it.
    pub async fn authenticate(
        &self,
        token: &str,
    ) -> Result<ServiceAccountPrincipal, DelegatedAuthError> {
        if token.is_empty() {
            self.metrics.unauthenticated.incr();
            return Err(DelegatedAuthError::Unauthenticated(
                AuthRejection::MissingCredential,
            ));
        }
        let key = digest(token);

        if let Some(cached) = self.tokens.get(&key) {
            self.metrics.record_token_cache(CacheOutcome::Hit);
            return self.finish_authentication(cached);
        }
        self.metrics.record_token_cache(CacheOutcome::Miss);

        self.metrics.token_reviews.incr();
        let outcome = match self
            .backend
            .review_token(token, self.config.audiences())
            .await
        {
            Ok(outcome) => outcome,
            Err(error) => {
                self.metrics.review_failures.incr();
                // The only fallback is an answer the apiserver already gave,
                // and only inside the stale window.
                if let Some(cached) = self.tokens.get_stale(&key) {
                    self.metrics.record_token_cache(CacheOutcome::Stale);
                    return self.finish_authentication(cached);
                }
                self.metrics.unavailable.incr();
                return Err(DelegatedAuthError::Unavailable(error));
            }
        };

        let resolved = self.judge(outcome);
        let ttl = match &resolved {
            Ok(_) => self.config.cache.allow_ttl,
            Err(_) => self.config.cache.deny_ttl,
        };
        self.tokens.insert(key, resolved.clone(), ttl);
        self.finish_authentication(resolved)
    }

    /// Ask whether this identity may perform this operation.
    pub async fn authorize(
        &self,
        principal: &ServiceAccountPrincipal,
        attributes: &ResourceAttributes,
    ) -> Result<(), DelegatedAuthError> {
        let key = DecisionKey {
            identity: principal.identity.clone(),
            attributes: attributes.clone(),
        };

        if let Some(allowed) = self.decisions.get(&key) {
            self.metrics.record_access_cache(CacheOutcome::Hit);
            return self.finish_authorization(allowed, attributes);
        }
        self.metrics.record_access_cache(CacheOutcome::Miss);

        self.metrics.access_reviews.incr();
        let reviewed = self
            .backend
            .review_access(&principal.identity, attributes)
            .await
            .and_then(|outcome| match outcome.evaluation_error {
                // A partial authorizer failure is not a deny and is certainly
                // not an allow — it is an unanswered question.
                Some(detail) => Err(ReviewError::Malformed(format!(
                    "authorizer reported an evaluation error: {detail}"
                ))),
                None => Ok(outcome),
            });

        let outcome = match reviewed {
            Ok(outcome) => outcome,
            Err(error) => {
                self.metrics.review_failures.incr();
                if let Some(allowed) = self.decisions.get_stale(&key) {
                    self.metrics.record_access_cache(CacheOutcome::Stale);
                    return self.finish_authorization(allowed, attributes);
                }
                self.metrics.unavailable.incr();
                return Err(DelegatedAuthError::Unavailable(error));
            }
        };

        let allowed = outcome.is_allowed();
        let ttl = if allowed {
            self.config.cache.allow_ttl
        } else {
            self.config.cache.deny_ttl
        };
        self.decisions.insert(key, allowed, ttl);
        self.finish_authorization(allowed, attributes)
    }

    /// The worst-case delay between a revocation in Kubernetes and this
    /// process refusing the caller.
    pub fn revocation_bound(&self) -> Duration {
        self.config.cache.revocation_bound()
    }

    /// Turn one `TokenReview` response into a caller or a rejection.
    ///
    /// The order is deliberate: the authentication flag, then the audience,
    /// then the identity shape. Reading the identity of a token that was not
    /// minted for this service would be treating an unrelated credential as an
    /// attempt to log in here.
    fn judge(
        &self,
        outcome: super::review::TokenReviewOutcome,
    ) -> Result<ServiceAccountPrincipal, AuthRejection> {
        if !outcome.authenticated {
            return Err(AuthRejection::Principal(
                PrincipalRejection::NotAuthenticated,
            ));
        }
        let audience_accepted = self.config.kubernetes_default
            || outcome
                .audiences
                .iter()
                .any(|granted| self.config.audiences.iter().any(|want| want == granted));
        if !audience_accepted {
            return Err(AuthRejection::AudienceMismatch);
        }
        ServiceAccountPrincipal::from_review(true, outcome.identity)
            .map_err(AuthRejection::Principal)
    }

    fn finish_authentication(
        &self,
        resolved: Result<ServiceAccountPrincipal, AuthRejection>,
    ) -> Result<ServiceAccountPrincipal, DelegatedAuthError> {
        match resolved {
            Ok(principal) => {
                self.metrics.authenticated.incr();
                Ok(principal)
            }
            Err(rejection) => {
                self.metrics.unauthenticated.incr();
                Err(DelegatedAuthError::Unauthenticated(rejection))
            }
        }
    }

    fn finish_authorization(
        &self,
        allowed: bool,
        attributes: &ResourceAttributes,
    ) -> Result<(), DelegatedAuthError> {
        if allowed {
            self.metrics.allowed.incr();
            Ok(())
        } else {
            self.metrics.denied.incr();
            Err(DelegatedAuthError::Denied(attributes.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    use async_trait::async_trait;

    use super::super::cache::ManualClock;
    use super::super::principal::ReviewedIdentity;
    use super::super::review::{AccessReviewOutcome, TokenReviewOutcome};
    use super::*;

    /// The audience this test suite's fictional service asks for. Deliberately
    /// not any real service's: AC8 requires the shared library's tests to know
    /// no product's resource strings.
    const AUDIENCE: &str = "reviews.example.test";

    /// A backend a test drives directly: it records what it was asked and
    /// returns whatever the test queued.
    #[derive(Default)]
    struct ScriptedBackend {
        token: Mutex<Option<Result<TokenReviewOutcome, ReviewError>>>,
        access: Mutex<Option<Result<AccessReviewOutcome, ReviewError>>>,
        token_calls: Mutex<Vec<Vec<String>>>,
        access_calls: Mutex<Vec<(ReviewedIdentity, ResourceAttributes)>>,
        /// Every token the backend was handed, so a test can assert none of
        /// them reached anywhere they should not have.
        seen_tokens: Mutex<Vec<String>>,
    }

    impl ScriptedBackend {
        fn with_token(self, outcome: Result<TokenReviewOutcome, ReviewError>) -> Self {
            *self.token.lock().unwrap() = Some(outcome);
            self
        }

        fn with_access(self, outcome: Result<AccessReviewOutcome, ReviewError>) -> Self {
            *self.access.lock().unwrap() = Some(outcome);
            self
        }

        fn set_token(&self, outcome: Result<TokenReviewOutcome, ReviewError>) {
            *self.token.lock().unwrap() = Some(outcome);
        }

        fn set_access(&self, outcome: Result<AccessReviewOutcome, ReviewError>) {
            *self.access.lock().unwrap() = Some(outcome);
        }

        fn token_call_count(&self) -> usize {
            self.token_calls.lock().unwrap().len()
        }

        fn access_call_count(&self) -> usize {
            self.access_calls.lock().unwrap().len()
        }
    }

    #[async_trait]
    impl ReviewBackend for ScriptedBackend {
        async fn review_token(
            &self,
            token: &str,
            audiences: &[String],
        ) -> Result<TokenReviewOutcome, ReviewError> {
            self.seen_tokens.lock().unwrap().push(token.to_string());
            self.token_calls.lock().unwrap().push(audiences.to_vec());
            self.token
                .lock()
                .unwrap()
                .clone()
                .expect("test queued no TokenReview outcome")
        }

        async fn review_access(
            &self,
            identity: &ReviewedIdentity,
            attributes: &ResourceAttributes,
        ) -> Result<AccessReviewOutcome, ReviewError> {
            self.access_calls
                .lock()
                .unwrap()
                .push((identity.clone(), attributes.clone()));
            self.access
                .lock()
                .unwrap()
                .clone()
                .expect("test queued no SubjectAccessReview outcome")
        }
    }

    fn config() -> DelegatedAuthConfig {
        DelegatedAuthConfig::new(vec![AUDIENCE.to_string()]).unwrap()
    }

    fn reviewed(username: &str, audiences: &[&str]) -> TokenReviewOutcome {
        TokenReviewOutcome {
            authenticated: true,
            identity: ReviewedIdentity {
                username: username.into(),
                uid: "uid-1".into(),
                groups: vec!["system:serviceaccounts".into()],
                extra: BTreeMap::new(),
            },
            audiences: audiences.iter().map(|a| a.to_string()).collect(),
            error: None,
        }
    }

    fn attributes() -> ResourceAttributes {
        ResourceAttributes::new(
            "example.test",
            "serving",
            "widgets",
            Some("blue".into()),
            "get",
        )
    }

    fn authenticator(
        backend: Arc<ScriptedBackend>,
        clock: Arc<ManualClock>,
    ) -> DelegatedAuthenticator {
        DelegatedAuthenticator::with_clock(backend, config(), clock)
    }

    fn fixture() -> (Arc<ScriptedBackend>, Arc<ManualClock>) {
        (
            Arc::new(ScriptedBackend::default()),
            Arc::new(ManualClock::new(1_000_000)),
        )
    }

    #[tokio::test]
    async fn a_token_minted_for_this_service_authenticates() {
        let backend = Arc::new(ScriptedBackend::default().with_token(Ok(reviewed(
            "system:serviceaccount:tenant-a:reader",
            &[AUDIENCE],
        ))));
        let auth = authenticator(backend.clone(), Arc::new(ManualClock::new(0)));

        let principal = auth.authenticate("opaque-token").await.unwrap();
        assert_eq!(principal.namespace(), "tenant-a");
        assert_eq!(principal.name(), "reader");
        assert_eq!(
            backend.token_calls.lock().unwrap()[0],
            vec![AUDIENCE.to_string()],
            "the audience must be requested explicitly, never left to the apiserver default"
        );
    }

    /// The unsafe-looking empty audience request is reachable only through
    /// the constructor that names why it exists. TokenReview, not this
    /// library, then validates the token against the apiserver's audiences.
    #[tokio::test]
    async fn the_explicit_kubernetes_default_profile_accepts_a_default_ksa_token() {
        let backend = Arc::new(
            ScriptedBackend::default()
                .with_token(Ok(reviewed("system:serviceaccount:tenant-a:reader", &[]))),
        );
        let config = DelegatedAuthConfig::kubernetes_default();
        assert!(config.uses_kubernetes_default());
        let auth = DelegatedAuthenticator::with_clock(
            backend.clone(),
            config,
            Arc::new(ManualClock::new(0)),
        );

        let principal = auth.authenticate("default-ksa-token").await.unwrap();
        assert_eq!(principal.namespace(), "tenant-a");
        assert_eq!(principal.name(), "reader");
        assert_eq!(
            backend.token_calls.lock().unwrap()[0],
            Vec::<String>::new(),
            "the explicit Kubernetes-default profile must omit requested audiences"
        );
    }

    #[tokio::test]
    async fn the_kubernetes_default_profile_still_rejects_bad_identity_shapes() {
        for (username, authenticated, reason) in [
            ("alice@example.com", true, "not_a_service_account"),
            (
                "system:serviceaccount:tenant-a:reader",
                false,
                "not_authenticated",
            ),
        ] {
            let mut outcome = reviewed(username, &[]);
            outcome.authenticated = authenticated;
            let backend = Arc::new(ScriptedBackend::default().with_token(Ok(outcome)));
            let auth = DelegatedAuthenticator::with_clock(
                backend,
                DelegatedAuthConfig::kubernetes_default(),
                Arc::new(ManualClock::new(0)),
            );
            let error = auth.authenticate("default-ksa-token").await.unwrap_err();
            assert_eq!(error.reason(), reason);
        }
    }

    /// AC1: a token the apiserver considers valid, but not for us.
    #[tokio::test]
    async fn a_token_for_another_audience_is_rejected_even_though_it_is_valid() {
        for granted in [
            vec!["https://kubernetes.default.svc"],
            vec!["some.other.service"],
            vec![],
        ] {
            let backend = Arc::new(ScriptedBackend::default().with_token(Ok(reviewed(
                "system:serviceaccount:tenant-a:reader",
                &granted,
            ))));
            let auth = authenticator(backend, Arc::new(ManualClock::new(0)));

            let error = auth.authenticate("opaque-token").await.unwrap_err();
            assert_eq!(
                error.reason(),
                "audience_mismatch",
                "audiences {granted:?} must not satisfy {AUDIENCE}"
            );
        }
    }

    #[tokio::test]
    async fn an_unauthenticated_review_is_a_401_not_a_503() {
        let backend = Arc::new(
            ScriptedBackend::default().with_token(Ok(TokenReviewOutcome {
                authenticated: false,
                error: Some("token expired".into()),
                ..Default::default()
            })),
        );
        let auth = authenticator(backend, Arc::new(ManualClock::new(0)));

        let error = auth.authenticate("expired-token").await.unwrap_err();
        assert_eq!(error.reason(), "not_authenticated");
        assert!(matches!(error, DelegatedAuthError::Unauthenticated(_)));
    }

    /// AC2: `authenticated: true` is not enough. This is the rejection that
    /// keeps a delegating service from becoming a second identity provider.
    #[tokio::test]
    async fn a_verified_non_service_account_is_rejected_before_any_authorization() {
        let backend = Arc::new(
            ScriptedBackend::default()
                .with_token(Ok(reviewed("alice@example.com", &[AUDIENCE])))
                .with_access(Ok(AccessReviewOutcome::allow())),
        );
        let auth = authenticator(backend.clone(), Arc::new(ManualClock::new(0)));

        let error = auth.authenticate("google-token").await.unwrap_err();
        assert_eq!(error.reason(), "not_a_service_account");
        assert_eq!(
            backend.access_call_count(),
            0,
            "authorization must never be reached for a rejected identity"
        );
    }

    /// R4: the authorizer is entitled to everything the authenticator learned.
    #[tokio::test]
    async fn the_whole_reviewed_identity_reaches_the_access_review() {
        let mut outcome = reviewed("system:serviceaccount:tenant-a:reader", &[AUDIENCE]);
        outcome.identity.groups = vec![
            "system:serviceaccounts".into(),
            "system:serviceaccounts:tenant-a".into(),
        ];
        outcome.identity.extra = BTreeMap::from([(
            "authentication.kubernetes.io/pod-name".to_string(),
            vec!["client-0".to_string()],
        )]);
        let expected = outcome.identity.clone();
        let backend = Arc::new(
            ScriptedBackend::default()
                .with_token(Ok(outcome))
                .with_access(Ok(AccessReviewOutcome::allow())),
        );
        let auth = authenticator(backend.clone(), Arc::new(ManualClock::new(0)));

        let principal = auth.authenticate("opaque-token").await.unwrap();
        auth.authorize(&principal, &attributes()).await.unwrap();

        let (identity, sent) = backend.access_calls.lock().unwrap()[0].clone();
        assert_eq!(identity, expected);
        assert_eq!(sent, attributes());
    }

    #[tokio::test]
    async fn a_denied_operation_is_a_403_naming_the_operation_not_the_caller() {
        let backend = Arc::new(
            ScriptedBackend::default()
                .with_token(Ok(reviewed(
                    "system:serviceaccount:tenant-a:reader",
                    &[AUDIENCE],
                )))
                .with_access(Ok(AccessReviewOutcome::deny("no matching rule"))),
        );
        let auth = authenticator(backend, Arc::new(ManualClock::new(0)));

        let principal = auth.authenticate("opaque-token").await.unwrap();
        let error = auth.authorize(&principal, &attributes()).await.unwrap_err();
        assert!(matches!(error, DelegatedAuthError::Denied(_)));
        assert_eq!(
            error.to_string(),
            "not permitted to get example.test/widgets/blue in serving"
        );
    }

    /// AC7: a transport failure with nothing cached fails closed as 503 —
    /// distinct from both 401 and 403, so an operator can tell the difference.
    #[tokio::test]
    async fn an_outage_with_no_cached_answer_fails_closed() {
        let backend = Arc::new(
            ScriptedBackend::default()
                .with_token(Err(ReviewError::Transport("connection refused".into()))),
        );
        let auth = authenticator(backend, Arc::new(ManualClock::new(0)));

        let error = auth.authenticate("opaque-token").await.unwrap_err();
        assert!(matches!(error, DelegatedAuthError::Unavailable(_)));
        assert_eq!(error.reason(), "transport");
    }

    #[tokio::test]
    async fn a_missing_delegation_grant_fails_closed_and_says_so() {
        let backend = Arc::new(ScriptedBackend::default().with_token(Err(
            ReviewError::NotDelegated("serviceaccount cannot create tokenreviews".into()),
        )));
        let auth = authenticator(backend, Arc::new(ManualClock::new(0)));

        let error = auth.authenticate("opaque-token").await.unwrap_err();
        assert_eq!(error.reason(), "not_delegated");
    }

    /// AC7: an authorizer that partially failed has not said "no", but it has
    /// certainly not said "yes".
    #[tokio::test]
    async fn a_partial_authorizer_failure_is_not_an_allow() {
        let backend = Arc::new(
            ScriptedBackend::default()
                .with_token(Ok(reviewed(
                    "system:serviceaccount:tenant-a:reader",
                    &[AUDIENCE],
                )))
                .with_access(Ok(AccessReviewOutcome {
                    allowed: true,
                    denied: false,
                    reason: None,
                    evaluation_error: Some("webhook authorizer unreachable".into()),
                })),
        );
        let auth = authenticator(backend, Arc::new(ManualClock::new(0)));

        let principal = auth.authenticate("opaque-token").await.unwrap();
        let error = auth.authorize(&principal, &attributes()).await.unwrap_err();
        assert!(matches!(error, DelegatedAuthError::Unavailable(_)));
        assert_eq!(error.reason(), "malformed_response");
    }

    /// A second request for the same token asks the apiserver nothing.
    #[tokio::test]
    async fn a_repeated_request_is_answered_from_cache() {
        let (backend, clock) = fixture();
        backend.set_token(Ok(reviewed(
            "system:serviceaccount:tenant-a:reader",
            &[AUDIENCE],
        )));
        backend.set_access(Ok(AccessReviewOutcome::allow()));
        let auth = authenticator(backend.clone(), clock);

        for _ in 0..5 {
            let principal = auth.authenticate("opaque-token").await.unwrap();
            auth.authorize(&principal, &attributes()).await.unwrap();
        }
        assert_eq!(backend.token_call_count(), 1);
        assert_eq!(backend.access_call_count(), 1);
        assert_eq!(auth.metrics().token_cache_hits.get(), 4);
        assert_eq!(auth.metrics().access_cache_hits.get(), 4);
    }

    /// AC6: a revoked allow stops working, on a schedule, with the apiserver
    /// answering normally the whole time.
    #[tokio::test]
    async fn a_revoked_allow_expires_within_the_documented_bound() {
        let (backend, clock) = fixture();
        backend.set_token(Ok(reviewed(
            "system:serviceaccount:tenant-a:reader",
            &[AUDIENCE],
        )));
        backend.set_access(Ok(AccessReviewOutcome::allow()));
        let auth = authenticator(backend.clone(), clock.clone());

        let principal = auth.authenticate("opaque-token").await.unwrap();
        auth.authorize(&principal, &attributes()).await.unwrap();

        // The RoleBinding is removed in Kubernetes.
        backend.set_access(Ok(AccessReviewOutcome::deny("no matching rule")));
        auth.authorize(&principal, &attributes())
            .await
            .expect("inside the TTL the stale allow is still served");

        clock.advance(Duration::from_secs(301));
        let error = auth.authorize(&principal, &attributes()).await.unwrap_err();
        assert!(matches!(error, DelegatedAuthError::Denied(_)));
        assert_eq!(auth.revocation_bound(), Duration::from_secs(360));
    }

    /// AC6: the outage path is bounded. Inside the window the last known
    /// answer is served; past it there is no path back to it.
    #[tokio::test]
    async fn an_outage_serves_a_stale_allow_only_inside_the_window() {
        let (backend, clock) = fixture();
        backend.set_token(Ok(reviewed(
            "system:serviceaccount:tenant-a:reader",
            &[AUDIENCE],
        )));
        backend.set_access(Ok(AccessReviewOutcome::allow()));
        let auth = authenticator(backend.clone(), clock.clone());

        let principal = auth.authenticate("opaque-token").await.unwrap();
        auth.authorize(&principal, &attributes()).await.unwrap();

        backend.set_access(Err(ReviewError::Transport("connection refused".into())));
        clock.advance(Duration::from_secs(301));
        auth.authorize(&principal, &attributes())
            .await
            .expect("30s past expiry is inside the 60s stale window");
        assert_eq!(auth.metrics().access_cache_stale.get(), 1);

        clock.advance(Duration::from_secs(60));
        let error = auth.authorize(&principal, &attributes()).await.unwrap_err();
        assert!(
            matches!(error, DelegatedAuthError::Unavailable(_)),
            "past the stale window an outage must fail closed, not keep serving"
        );
    }

    /// The same bound applies to authentication: an outage cannot turn a
    /// short-lived token into an indefinite one.
    #[tokio::test]
    async fn an_outage_cannot_extend_an_authentication_indefinitely() {
        let (backend, clock) = fixture();
        backend.set_token(Ok(reviewed(
            "system:serviceaccount:tenant-a:reader",
            &[AUDIENCE],
        )));
        let auth = authenticator(backend.clone(), clock.clone());
        auth.authenticate("opaque-token").await.unwrap();

        backend.set_token(Err(ReviewError::Transport("connection refused".into())));
        clock.advance(Duration::from_secs(301));
        auth.authenticate("opaque-token")
            .await
            .expect("inside the stale window");

        clock.advance(Duration::from_secs(60));
        let error = auth.authenticate("opaque-token").await.unwrap_err();
        assert!(matches!(error, DelegatedAuthError::Unavailable(_)));
        assert_eq!(auth.metrics().token_cache_stale.get(), 1);
    }

    /// A rejection is cached too, so a flood of bad tokens is not a way to
    /// generate apiserver load — but only for the short deny TTL.
    #[tokio::test]
    async fn a_rejection_is_cached_briefly_and_then_re_reviewed() {
        let (backend, clock) = fixture();
        backend.set_token(Ok(reviewed("alice@example.com", &[AUDIENCE])));
        let auth = authenticator(backend.clone(), clock.clone());

        for _ in 0..3 {
            assert!(auth.authenticate("google-token").await.is_err());
        }
        assert_eq!(backend.token_call_count(), 1);

        clock.advance(Duration::from_secs(31));
        assert!(auth.authenticate("google-token").await.is_err());
        assert_eq!(backend.token_call_count(), 2);
    }

    /// Two callers whose usernames match but whose groups differ are two
    /// different authorization questions.
    #[tokio::test]
    async fn the_decision_cache_key_covers_the_whole_identity_not_just_the_username() {
        let (backend, clock) = fixture();
        backend.set_access(Ok(AccessReviewOutcome::allow()));
        let auth = authenticator(backend.clone(), clock);

        let mut first = reviewed("system:serviceaccount:tenant-a:reader", &[AUDIENCE]);
        first.identity.groups = vec!["group-one".into()];
        let mut second = first.clone();
        second.identity.groups = vec!["group-two".into()];

        let a = ServiceAccountPrincipal::from_review(true, first.identity).unwrap();
        let b = ServiceAccountPrincipal::from_review(true, second.identity).unwrap();
        auth.authorize(&a, &attributes()).await.unwrap();
        auth.authorize(&b, &attributes()).await.unwrap();

        assert_eq!(
            backend.access_call_count(),
            2,
            "a differing group list must not be answered from the other caller's entry"
        );
    }

    /// Different resources are different questions, even for one caller.
    #[tokio::test]
    async fn each_resource_and_verb_is_its_own_cached_decision() {
        let (backend, clock) = fixture();
        backend.set_access(Ok(AccessReviewOutcome::allow()));
        let auth = authenticator(backend.clone(), clock);
        let principal = ServiceAccountPrincipal::from_review(
            true,
            reviewed("system:serviceaccount:tenant-a:reader", &[AUDIENCE]).identity,
        )
        .unwrap();

        let read = attributes();
        let mut write = attributes();
        write.verb = "update".into();
        let mut other = attributes();
        other.name = Some("green".into());

        for check in [&read, &write, &other] {
            auth.authorize(&principal, check).await.unwrap();
        }
        assert_eq!(backend.access_call_count(), 3);
    }

    /// A configuration with no audience is refused at construction, because
    /// there is no safe way to run without one.
    #[test]
    fn a_configuration_without_an_audience_is_refused() {
        assert_eq!(DelegatedAuthConfig::new(vec![]), Err(MissingAudience));
        assert_eq!(
            DelegatedAuthConfig::new(vec![String::new()]),
            Err(MissingAudience)
        );
        assert!(DelegatedAuthConfig::new(vec![AUDIENCE.into()]).is_ok());
        assert!(DelegatedAuthConfig::kubernetes_default().uses_kubernetes_default());
        assert!(DelegatedAuthConfig::kubernetes_default()
            .audiences()
            .is_empty());
    }

    #[tokio::test]
    async fn an_empty_credential_never_reaches_the_apiserver() {
        let (backend, clock) = fixture();
        let auth = authenticator(backend.clone(), clock);
        let error = auth.authenticate("").await.unwrap_err();
        assert_eq!(error.reason(), "missing_credential");
        assert_eq!(backend.token_call_count(), 0);
    }

    /// R10 / AC7: no error rendering, and no metric, may carry the token.
    #[tokio::test]
    async fn no_rendered_error_or_metric_contains_the_token() {
        let secret = "super-secret-bearer-token";
        let (backend, clock) = fixture();
        backend.set_token(Ok(reviewed("alice@example.com", &[AUDIENCE])));
        let auth = authenticator(backend.clone(), clock);

        let error = auth.authenticate(secret).await.unwrap_err();
        assert!(!error.to_string().contains(secret));
        assert!(!format!("{error:?}").contains(secret));
        assert!(!auth.metrics().render().contains(secret));
        assert!(!fingerprint(secret).contains(secret));
        assert_eq!(fingerprint(secret).len(), 12);
        assert_eq!(
            fingerprint(secret),
            fingerprint(secret),
            "a fingerprint must be stable to be useful for correlation"
        );
        assert_ne!(fingerprint(secret), fingerprint("another-token"));
    }

    #[tokio::test]
    async fn the_rendered_metrics_declare_every_counter() {
        let (backend, clock) = fixture();
        backend.set_token(Ok(reviewed(
            "system:serviceaccount:tenant-a:reader",
            &[AUDIENCE],
        )));
        backend.set_access(Ok(AccessReviewOutcome::allow()));
        let auth = authenticator(backend, clock);
        let principal = auth.authenticate("opaque-token").await.unwrap();
        auth.authorize(&principal, &attributes()).await.unwrap();

        let rendered = auth.metrics().render();
        for name in [
            "delegated_auth_token_reviews_total",
            "delegated_auth_token_cache_misses_total",
            "delegated_auth_authenticated_total",
            "delegated_auth_access_reviews_total",
            "delegated_auth_allowed_total",
            "delegated_auth_denied_total",
            "delegated_auth_unavailable_total",
        ] {
            assert!(rendered.contains(name), "{name} is missing from the scrape");
        }
        assert!(rendered.contains("delegated_auth_allowed_total 1"));
    }

    /// The 503 arm must not be reachable by a caller-shaped input; it exists
    /// only for the apiserver being unreachable.
    #[test]
    fn the_http_mapping_keeps_the_three_outcomes_distinct() {
        let unauthenticated: AuthError =
            DelegatedAuthError::Unauthenticated(AuthRejection::AudienceMismatch).into();
        assert!(matches!(unauthenticated, AuthError::Unauthenticated));

        let denied: AuthError = DelegatedAuthError::Denied(attributes()).into();
        match denied {
            AuthError::Forbidden(message) => {
                assert!(message.contains("get example.test/widgets/blue in serving"))
            }
            other => panic!("a deny must be 403, got {other:?}"),
        }

        let unavailable: AuthError =
            DelegatedAuthError::Unavailable(ReviewError::Transport("down".into())).into();
        assert!(matches!(unavailable, AuthError::Unavailable(_)));
    }

    #[tokio::test]
    async fn invalidating_the_cache_forces_a_fresh_review() {
        let (backend, clock) = fixture();
        backend.set_token(Ok(reviewed(
            "system:serviceaccount:tenant-a:reader",
            &[AUDIENCE],
        )));
        let auth = authenticator(backend.clone(), clock);

        auth.authenticate("opaque-token").await.unwrap();
        auth.invalidate();
        auth.authenticate("opaque-token").await.unwrap();
        assert_eq!(backend.token_call_count(), 2);
    }
}
// HANDWRITE-END
