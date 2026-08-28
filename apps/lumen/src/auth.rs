// CODEGEN-BEGIN
//! Request auth for the serving API: Kubernetes ServiceAccount identities only.
//!
//! Lumen holds no credentials. A caller presents a short-lived, audience-bound
//! Kubernetes ServiceAccount token; `TokenReview` says who they are and
//! `SubjectAccessReview` says what they may do. Both questions are answered by
//! kube-apiserver, so the only place Lumen's access policy lives is
//! `RoleBinding`s in the cluster — there is no registry file, no token Secret,
//! and nothing for the operator to project (#2869).
//!
//! ## What this deliberately refuses
//!
//! A GKE authenticator will happily verify `alice@example.com` or
//! `svc@project.iam.gserviceaccount.com`, and `TokenReview` will report
//! `authenticated: true` for both. Lumen rejects them. Those principals
//! authenticate to *kube-apiserver*, and Kubernetes RBAC decides whether they
//! may request a token for a named client ServiceAccount; they never
//! authenticate to Lumen directly. Accepting them here would quietly make
//! Lumen a second identity provider for whatever the cluster's authenticator
//! happens to verify. The check is [`service_auth::k8s`]'s: the username must
//! strictly parse as `system:serviceaccount:<namespace>:<name>`.
//!
//! ## The resource mapping
//!
//! Lumen owns exactly this much of the shared mechanism — the translation from
//! a domain operation to the attributes a `SubjectAccessReview` asks about:
//!
//! | Operation | group | resource | name | verb |
//! |---|---|---|---|---|
//! | read a collection | `lumen.axiom.dev` | `lumencollections` | collection id | `get` |
//! | write a collection | `lumen.axiom.dev` | `lumencollections` | collection id | `update` |
//! | administer a collection | `lumen.axiom.dev` | `lumencollections` | collection id | `delete` |
//! | instance admin | `lumen.axiom.dev` | `lumenadmin` | — | per role |
//!
//! Instance-level endpoints (`/admin/*`) are a *separate resource*, not
//! wildcard access to every collection. A grant that lets an operator take a
//! backup should not thereby let them read every document in the fleet, and a
//! grant on one collection should never reach the admin surface.
//!
//! The namespace in every check is the serving instance's own — the one this
//! process runs in — so a caller from another namespace needs a RoleBinding
//! *here*, and holding one where it lives proves nothing.
//!
//! ## Configuration
//!
//! Env (read by [`AuthConfig::from_env`]):
//!
//! - `LUMEN_AUTH=off|disabled|required|in-cluster` — default `off`.
//!   `off`/`disabled` serve without authentication and make **no** Kubernetes
//!   call at all. `required` keeps Managed's private `lumen.axiom.dev`
//!   audience. `in-cluster` accepts the default KSA token Kubernetes mounts.
//! - `LUMEN_AUTH_NAMESPACE` — the namespace every `SubjectAccessReview` is
//!   scoped to. Defaults to `POD_NAMESPACE`, then to the in-cluster
//!   ServiceAccount namespace file. Both required profiles refuse to start
//!   without one: an unscoped check asks a different question than intended.
//!
//! ## Role precedence
//!
//! There is none any more, and that is the point. `admin ⊇ write ⊇ read` was a
//! property of a local role map. Here each role maps to a distinct Kubernetes
//! verb and the cluster's RBAC answers each independently: a RoleBinding
//! granting `delete` does not imply `get` unless it says so.

use std::sync::Arc;

use anyhow::{bail, Result};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use service_auth::k8s::{
    DelegatedAuthConfig, DelegatedAuthError, DelegatedAuthenticator, ResourceAttributes,
    ReviewBackend, ServiceAccountPrincipal,
};
use service_auth::{bearer_token, AsyncVerifier, AuthError as ServiceAuthError};

pub use service_auth::Role;

use crate::types::ApiError;

/// The audience every Managed token must be bound to, and the only audience
/// Managed Lumen requests in `TokenReview`.
///
/// A token minted for the apiserver's own audience — which is what every pod's
/// default ServiceAccount token is — must not open Managed Lumen. Requesting
/// this audience explicitly is what keeps Managed separate. Standalone's
/// explicit `in-cluster` profile uses Kubernetes' default audience instead.
pub const AUDIENCE: &str = "lumen.axiom.dev";

/// The API group every authorization check is asked under.
pub const API_GROUP: &str = "lumen.axiom.dev";

/// The resource a per-collection check names. The collection id is the
/// resource *name*, so a RoleBinding can grant one collection by
/// `resourceNames` instead of a wildcard.
pub const COLLECTIONS_RESOURCE: &str = "lumencollections";

/// The resource instance-wide administration is checked against — backups,
/// restores, resharding, checkpoints. Deliberately not `lumencollections/*`.
pub const ADMIN_RESOURCE: &str = "lumenadmin";

/// The in-cluster file every pod with a mounted ServiceAccount token carries.
/// Lumen needs such a token to call `TokenReview` at all, so this is readable
/// exactly when delegation can work.
const SERVICE_ACCOUNT_NAMESPACE_FILE: &str =
    "/var/run/secrets/kubernetes.io/serviceaccount/namespace";

/// Where a Lumen *control-plane* workload — the operator's reshard driver, the
/// backup runner — finds the token it presents to a serving instance (#2877).
///
/// Deliberately not `/var/run/secrets/kubernetes.io/serviceaccount`: that is
/// the kubelet's default projection, minted for the apiserver's audience, and
/// the whole point is that the two are different credentials. Keeping them in
/// different directories means a wiring mistake produces a missing file rather
/// than a token Lumen will reject for reasons that look like RBAC.
pub const CONTROL_PLANE_TOKEN_MOUNT: &str = "/var/run/secrets/lumen.axiom.dev";

/// Pod volume name for the projection at [`CONTROL_PLANE_TOKEN_MOUNT`].
pub const CONTROL_PLANE_TOKEN_VOLUME: &str = "lumen-admin-token";

/// The file itself.
pub const CONTROL_PLANE_TOKEN_FILE: &str = "/var/run/secrets/lumen.axiom.dev/token";

/// The credential a control-plane caller presents, read fresh per call.
///
/// Lumen owns *which* workloads need one and *what audience* it must carry;
/// the reading, the rotation contract, and the redaction rules are
/// [`service_auth::k8s::ProjectedTokenFile`]'s.
pub fn control_plane_token_file() -> service_auth::k8s::ProjectedTokenFile {
    service_auth::k8s::ProjectedTokenFile::new(CONTROL_PLANE_TOKEN_FILE, AUDIENCE)
}

/// What a handler is asking about — one collection, or the instance itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthTarget<'a> {
    Collection(&'a str),
    Admin,
}

impl AuthTarget<'_> {
    /// The `SubjectAccessReview` attributes for this target at `needed`.
    pub fn attributes(&self, namespace: &str, needed: Role) -> ResourceAttributes {
        match self {
            AuthTarget::Collection(id) => ResourceAttributes::new(
                API_GROUP,
                namespace,
                COLLECTIONS_RESOURCE,
                Some((*id).to_string()),
                verb(needed),
            ),
            AuthTarget::Admin => {
                ResourceAttributes::new(API_GROUP, namespace, ADMIN_RESOURCE, None, verb(needed))
            }
        }
    }

    /// How this target is named in a denial message and an audit line.
    fn describe(&self) -> String {
        match self {
            AuthTarget::Collection(id) => (*id).to_string(),
            AuthTarget::Admin => ADMIN_RESOURCE.to_string(),
        }
    }
}

/// The Kubernetes verb a Lumen role maps to.
///
/// These are ordinary RBAC verbs, so a grant is expressible in a plain
/// `Role`/`ClusterRole` with no Lumen-specific vocabulary.
pub fn verb(role: Role) -> &'static str {
    match role {
        Role::Read => "get",
        Role::Write => "update",
        Role::Admin => "delete",
    }
}

/// The credential profile selected by `LUMEN_AUTH`.
///
/// Managed and Standalone are separate on purpose. A default pod token must
/// open only a Standalone instance that explicitly chose `in-cluster`; it
/// must never become an alternate credential for Managed's private audience.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthProfile {
    Off,
    ManagedAudience,
    KubernetesDefault,
}

impl AuthProfile {
    pub fn env_value(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::ManagedAudience => "required",
            Self::KubernetesDefault => "in-cluster",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Whether a caller must present a verifiable identity. `false` makes no
    /// Kubernetes call on any request path.
    pub required: bool,
    /// The namespace every `SubjectAccessReview` is scoped to — the serving
    /// instance's own. Empty is only valid when `required` is `false`.
    pub namespace: String,
    profile: AuthProfile,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self::open()
    }
}

impl AuthConfig {
    pub fn open() -> Self {
        Self {
            required: false,
            namespace: String::new(),
            profile: AuthProfile::Off,
        }
    }

    /// A required config scoped to `namespace`. The verifier still has to be
    /// wired to a review backend before it can authenticate anyone; until it
    /// is, it rejects every request rather than falling back to an open one.
    pub fn required_in(namespace: impl Into<String>) -> Self {
        Self {
            required: true,
            namespace: namespace.into(),
            profile: AuthProfile::ManagedAudience,
        }
    }

    /// Standalone's explicit default-KSA profile.
    pub fn in_cluster(namespace: impl Into<String>) -> Self {
        Self {
            required: true,
            namespace: namespace.into(),
            profile: AuthProfile::KubernetesDefault,
        }
    }

    pub fn profile(&self) -> AuthProfile {
        self.profile
    }

    pub fn from_env() -> Result<Self> {
        let profile = match std::env::var("LUMEN_AUTH") {
            Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
                "required" => AuthProfile::ManagedAudience,
                "in-cluster" => AuthProfile::KubernetesDefault,
                "off" | "disabled" => AuthProfile::Off,
                other => {
                    bail!(
                        "LUMEN_AUTH must be `off`, `disabled`, `required`, or `in-cluster`; got `{other}`"
                    )
                }
            },
            Err(std::env::VarError::NotPresent) => AuthProfile::Off,
            Err(e) => bail!("LUMEN_AUTH must be valid UTF-8: {e}"),
        };
        let required = profile != AuthProfile::Off;

        let namespace = namespace_from_env();

        if required && namespace.is_empty() {
            // Fail closed. A namespace-less SubjectAccessReview asks about a
            // different resource than the one the request touched, and the
            // safest wrong answer is still a wrong answer.
            bail!(
                "LUMEN_AUTH={} needs the serving namespace to scope every \
                 SubjectAccessReview to, and neither LUMEN_AUTH_NAMESPACE nor POD_NAMESPACE is \
                 set and {SERVICE_ACCOUNT_NAMESPACE_FILE} is unreadable. Refusing to start \
                 rather than authorize against an unscoped resource.",
                profile.env_value()
            );
        }

        Ok(Self {
            required,
            namespace,
            profile,
        })
    }

    #[cfg(feature = "delegated-auth")]
    fn delegated_config(&self) -> Result<DelegatedAuthConfig> {
        match self.profile {
            AuthProfile::ManagedAudience => DelegatedAuthConfig::new(vec![AUDIENCE.to_string()])
                .map_err(|e| anyhow::anyhow!("lumen's delegated-auth audience is missing: {e}")),
            AuthProfile::KubernetesDefault => Ok(DelegatedAuthConfig::kubernetes_default()),
            AuthProfile::Off => bail!("auth=off has no delegated token profile"),
        }
    }
}

/// The serving namespace, from the explicit override, the downward-API env, or
/// the mounted ServiceAccount. Empty when none of the three answers.
fn namespace_from_env() -> String {
    for key in ["LUMEN_AUTH_NAMESPACE", "POD_NAMESPACE"] {
        if let Ok(value) = std::env::var(key) {
            let value = value.trim();
            if !value.is_empty() {
                return value.to_string();
            }
        }
    }
    std::fs::read_to_string(SERVICE_ACCOUNT_NAMESPACE_FILE)
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

/// How this process answers "who is calling?".
enum VerifierMode {
    /// Auth is off. Every request resolves to [`AuthContext::Open`] without
    /// touching the network (#2869 R9).
    Open,
    /// Auth is required but no review backend is wired — a state reachable
    /// only by building [`AuthConfig`] by hand. Every request is rejected;
    /// there is no degradation to `Open`.
    Unwired,
    /// Auth is required and delegated to kube-apiserver.
    Delegated {
        authenticator: Arc<DelegatedAuthenticator>,
        namespace: Arc<str>,
    },
}

/// Lumen's verifier for the shared async auth middleware.
pub struct LumenVerifier(VerifierMode);

impl std::fmt::Debug for LumenVerifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mode = match &self.0 {
            VerifierMode::Open => "open",
            VerifierMode::Unwired => "required-unwired",
            VerifierMode::Delegated { .. } => "delegated",
        };
        f.debug_tuple("LumenVerifier").field(&mode).finish()
    }
}

impl LumenVerifier {
    pub fn new(cfg: Arc<AuthConfig>) -> Self {
        Self(if cfg.required {
            VerifierMode::Unwired
        } else {
            VerifierMode::Open
        })
    }

    /// Delegate to an arbitrary review backend. The serving binary passes a
    /// live kube client; tests pass a scripted one, which is what lets the
    /// whole domain mapping be proven without a cluster.
    pub fn delegated(namespace: &str, backend: Arc<dyn ReviewBackend>) -> Result<Self> {
        let config = DelegatedAuthConfig::new(vec![AUDIENCE.to_string()])
            .map_err(|e| anyhow::anyhow!("lumen's delegated-auth audience is missing: {e}"))?;
        Ok(Self::delegated_with_config(namespace, backend, config))
    }

    /// Standalone's verifier for Kubernetes default ServiceAccount tokens.
    pub fn delegated_in_cluster(namespace: &str, backend: Arc<dyn ReviewBackend>) -> Result<Self> {
        Ok(Self::delegated_with_config(
            namespace,
            backend,
            DelegatedAuthConfig::kubernetes_default(),
        ))
    }

    fn delegated_with_config(
        namespace: &str,
        backend: Arc<dyn ReviewBackend>,
        config: DelegatedAuthConfig,
    ) -> Self {
        Self::with_authenticator(
            namespace,
            Arc::new(DelegatedAuthenticator::new(backend, config)),
        )
    }

    /// Delegate to an already-built authenticator — the seam a deterministic
    /// cache/TTL test uses to install its own clock.
    pub fn with_authenticator(namespace: &str, authenticator: Arc<DelegatedAuthenticator>) -> Self {
        Self(VerifierMode::Delegated {
            authenticator,
            namespace: Arc::from(namespace),
        })
    }

    /// Build the in-cluster verifier, proving both delegation grants before
    /// returning (#2869 R9).
    ///
    /// Every failure here is a startup failure. A process that cannot reach an
    /// apiserver, or whose ServiceAccount lacks `system:auth-delegator`, cannot
    /// authenticate anyone — and discovering that on the first request means
    /// serving 503s while looking healthy.
    #[cfg(feature = "delegated-auth")]
    pub async fn connect(cfg: &AuthConfig) -> Result<Self> {
        use service_auth::k8s::KubeReviewBackend;

        let backend = KubeReviewBackend::in_cluster().await.map_err(|e| {
            anyhow::anyhow!(
                "LUMEN_AUTH={} could not reach kube-apiserver: {e}",
                cfg.profile.env_value()
            )
        })?;
        let delegated = cfg.delegated_config()?;
        // A rejected probe is a successful probe: what is under test is
        // whether the apiserver will answer these two questions at all.
        backend
            .probe_delegation(
                delegated.audiences(),
                &AuthTarget::Admin.attributes(&cfg.namespace, Role::Read),
            )
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "LUMEN_AUTH={} but this instance's ServiceAccount cannot create \
                     TokenReview/SubjectAccessReview for namespace `{}` ({e}). Bind it to \
                     `system:auth-delegator`. Refusing to start.",
                    cfg.profile.env_value(),
                    cfg.namespace
                )
            })?;
        Ok(Self::delegated_with_config(
            &cfg.namespace,
            Arc::new(backend),
            delegated,
        ))
    }

    /// The delegated counters in Prometheus text format — empty when auth is
    /// off, because nothing was measured.
    pub fn render_metrics(&self) -> String {
        match &self.0 {
            VerifierMode::Delegated { authenticator, .. } => authenticator.metrics().render(),
            _ => String::new(),
        }
    }
}

#[async_trait::async_trait]
impl AsyncVerifier for LumenVerifier {
    type Principal = AuthContext;

    async fn authenticate_async(
        &self,
        headers: &HeaderMap,
    ) -> Result<AuthContext, ServiceAuthError> {
        match &self.0 {
            // A server that verifies nothing must never tell a caller its
            // credential was accepted. Absent header: anonymous, served.
            // Present header: rejected, because there is nothing here that
            // could have checked it (#2871).
            VerifierMode::Open => match bearer_token(headers) {
                Some(_) => Err(ServiceAuthError::Unauthenticated),
                None => Ok(AuthContext::Open),
            },
            VerifierMode::Unwired => Err(ServiceAuthError::Unauthenticated),
            VerifierMode::Delegated {
                authenticator,
                namespace,
            } => {
                let token = bearer_token(headers).ok_or(ServiceAuthError::Unauthenticated)?;
                let principal = authenticator.authenticate(token).await?;
                Ok(AuthContext::Delegated {
                    subject: Arc::from(principal.username().as_str()),
                    principal: Arc::new(principal),
                    authenticator: Arc::clone(authenticator),
                    namespace: Arc::clone(namespace),
                })
            }
        }
    }

    fn required(&self) -> bool {
        !matches!(self.0, VerifierMode::Open)
    }
}

/// Resolved auth state attached to every request as an axum extension.
///
/// Authentication happened once, in the middleware. Authorization happens per
/// operation, in the handler, because "may they touch *this* collection?" is a
/// different question per handler and the route alone does not answer it.
#[derive(Clone)]
pub enum AuthContext {
    /// Auth is off. Passes every check, and makes no Kubernetes call.
    Open,
    Delegated {
        principal: Arc<ServiceAccountPrincipal>,
        /// The `system:serviceaccount:<ns>:<name>` rendering, resolved once so
        /// the access log and every denial share one string.
        subject: Arc<str>,
        authenticator: Arc<DelegatedAuthenticator>,
        namespace: Arc<str>,
    },
}

impl std::fmt::Debug for AuthContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthContext::Open => f.write_str("AuthContext::Open"),
            AuthContext::Delegated {
                subject, namespace, ..
            } => f
                .debug_struct("AuthContext::Delegated")
                .field("subject", subject)
                .field("namespace", namespace)
                .finish(),
        }
    }
}

impl AuthContext {
    /// Authorize `needed` on one collection.
    pub async fn ensure(&self, collection_id: &str, needed: Role) -> Result<(), AuthErr> {
        self.authorize(AuthTarget::Collection(collection_id), needed)
            .await
    }

    /// Authorize `needed` on the instance-wide admin surface. A grant on every
    /// collection in the namespace still does not reach here.
    pub async fn ensure_admin(&self, needed: Role) -> Result<(), AuthErr> {
        self.authorize(AuthTarget::Admin, needed).await
    }

    async fn authorize(&self, target: AuthTarget<'_>, needed: Role) -> Result<(), AuthErr> {
        let AuthContext::Delegated {
            principal,
            subject,
            authenticator,
            namespace,
        } = self
        else {
            return Ok(());
        };
        let attributes = target.attributes(namespace, needed);
        authenticator
            .authorize(principal, &attributes)
            .await
            .map_err(|e| AuthErr::new(subject.to_string(), needed, target.describe(), e))
    }

    pub fn subject(&self) -> Option<&str> {
        match self {
            AuthContext::Open => None,
            AuthContext::Delegated { subject, .. } => Some(subject),
        }
    }
}

pub async fn auth_middleware(
    State(verifier): State<Arc<LumenVerifier>>,
    req: Request,
    next: Next,
) -> Response {
    service_auth::async_auth_middleware::<LumenVerifier>(State(verifier), req, next).await
}

/// A failed authorization.
///
/// The two variants stay apart all the way to the status line because they are
/// different facts about the cluster. `Forbidden` means the apiserver answered
/// and the answer was no — retrying will not help, and the fix is a
/// RoleBinding. `Unavailable` means nobody answered — the request may well be
/// permitted, and calling it a denial sends an operator to fix a policy that
/// was never the problem.
#[derive(Debug)]
pub enum AuthErr {
    Forbidden {
        subject: String,
        needed: Role,
        resource: String,
    },
    Unavailable {
        subject: String,
        needed: Role,
        resource: String,
        /// A stable classification (`transport`, `malformed_response`,
        /// `not_delegated`) — never the credential that was presented.
        reason: &'static str,
    },
}

impl AuthErr {
    fn new(subject: String, needed: Role, resource: String, e: DelegatedAuthError) -> Self {
        match e {
            // `authorize` cannot report an authentication failure — the
            // middleware already resolved a principal — but the shared error
            // type carries the variant, and refusing the request is the only
            // safe rendering of a principal that stopped being one.
            DelegatedAuthError::Unauthenticated(_) | DelegatedAuthError::Denied(_) => {
                AuthErr::Forbidden {
                    subject,
                    needed,
                    resource,
                }
            }
            DelegatedAuthError::Unavailable(ref err) => AuthErr::Unavailable {
                subject,
                needed,
                resource,
                reason: err.reason(),
            },
        }
    }

    /// The wire code and message, shared by the HTTP response and the per-item
    /// batch envelope so the two can never drift.
    pub fn wire(&self) -> (StatusCode, &'static str, String) {
        match self {
            AuthErr::Forbidden {
                subject,
                needed,
                resource,
            } => (
                StatusCode::FORBIDDEN,
                "forbidden",
                format!("subject `{subject}` lacks {needed:?} on `{resource}`"),
            ),
            AuthErr::Unavailable {
                needed,
                resource,
                reason,
                ..
            } => (
                StatusCode::SERVICE_UNAVAILABLE,
                "authorization_unavailable",
                format!(
                    "could not authorize {needed:?} on `{resource}`: kube-apiserver did not \
                     answer ({reason})"
                ),
            ),
        }
    }
}

impl IntoResponse for AuthErr {
    fn into_response(self) -> Response {
        match &self {
            AuthErr::Forbidden {
                subject,
                needed,
                resource,
            } => tracing::warn!(
                target: "lumen.audit",
                event = "rbac_denied",
                %subject,
                resource = %resource,
                needed = ?needed,
            ),
            AuthErr::Unavailable {
                subject,
                needed,
                resource,
                reason,
            } => tracing::warn!(
                target: "lumen.audit",
                event = "rbac_unavailable",
                %subject,
                resource = %resource,
                needed = ?needed,
                reason = %reason,
            ),
        }
        let (status, error, message) = self.wire();
        (
            status,
            Json(ApiError {
                error: error.to_string(),
                message,
            }),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Mutex;

    use async_trait::async_trait;
    use service_auth::k8s::{
        AccessReviewOutcome, ReviewError, ReviewedIdentity, TokenReviewOutcome,
    };

    // Process-global env mutex shared across the env-mutating tests.
    static AUTH_ENV_LOCK: Mutex<()> = Mutex::new(());

    fn clear_auth_env() {
        unsafe {
            std::env::remove_var("LUMEN_AUTH");
            std::env::remove_var("LUMEN_AUTH_NAMESPACE");
            std::env::remove_var("POD_NAMESPACE");
        }
    }

    /// Which half of the apiserver is unreachable, so an outage in
    /// authentication and an outage in authorization can be proven separately.
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Reachable {
        Both,
        Neither,
        /// TokenReview answers; SubjectAccessReview does not.
        AuthenticationOnly,
    }

    /// A cluster under the test's control: one authentication answer plus an
    /// explicit list of `(user, resource, name, verb)` grants, so every
    /// assertion below names the exact RoleBinding it is modelling.
    struct Cluster {
        username: String,
        audiences: Vec<String>,
        grants: Vec<(String, String, Option<String>, String)>,
        reachable: Reachable,
        token_calls: Mutex<Vec<Vec<String>>>,
        asked: Mutex<Vec<ResourceAttributes>>,
    }

    impl Cluster {
        fn with_ksa(namespace: &str, name: &str) -> Self {
            Self {
                username: format!("system:serviceaccount:{namespace}:{name}"),
                audiences: vec![AUDIENCE.to_string()],
                grants: Vec::new(),
                reachable: Reachable::Both,
                token_calls: Mutex::new(Vec::new()),
                asked: Mutex::new(Vec::new()),
            }
        }

        fn as_user(mut self, username: &str) -> Self {
            self.username = username.to_string();
            self
        }

        fn with_audiences(mut self, audiences: &[&str]) -> Self {
            self.audiences = audiences.iter().map(|a| a.to_string()).collect();
            self
        }

        fn granting(mut self, resource: &str, name: Option<&str>, verb: &str) -> Self {
            self.grants.push((
                self.username.clone(),
                resource.to_string(),
                name.map(|n| n.to_string()),
                verb.to_string(),
            ));
            self
        }

        fn reachable(mut self, reachable: Reachable) -> Self {
            self.reachable = reachable;
            self
        }

        fn asked(&self) -> Vec<ResourceAttributes> {
            self.asked.lock().unwrap().clone()
        }

        fn token_calls(&self) -> Vec<Vec<String>> {
            self.token_calls.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl ReviewBackend for Cluster {
        async fn review_token(
            &self,
            _token: &str,
            audiences: &[String],
        ) -> Result<TokenReviewOutcome, ReviewError> {
            self.token_calls.lock().unwrap().push(audiences.to_vec());
            if self.reachable == Reachable::Neither {
                return Err(ReviewError::Transport("apiserver unreachable".into()));
            }
            Ok(TokenReviewOutcome {
                authenticated: true,
                identity: ReviewedIdentity {
                    username: self.username.clone(),
                    uid: "uid-1".into(),
                    groups: vec!["system:serviceaccounts".into()],
                    ..Default::default()
                },
                audiences: self.audiences.clone(),
                error: None,
            })
        }

        async fn review_access(
            &self,
            identity: &ReviewedIdentity,
            attributes: &ResourceAttributes,
        ) -> Result<AccessReviewOutcome, ReviewError> {
            if self.reachable != Reachable::Both {
                return Err(ReviewError::Transport("apiserver unreachable".into()));
            }
            self.asked.lock().unwrap().push(attributes.clone());
            let held = self.grants.iter().any(|(user, resource, name, verb)| {
                user == &identity.username
                    && resource == &attributes.resource
                    && name == &attributes.name
                    && verb == &attributes.verb
            });
            Ok(if held {
                AccessReviewOutcome::allow()
            } else {
                AccessReviewOutcome::deny("no RoleBinding grants this")
            })
        }
    }

    fn verifier(cluster: Cluster) -> (Arc<Cluster>, LumenVerifier) {
        let cluster = Arc::new(cluster);
        let verifier = LumenVerifier::delegated("serving", cluster.clone()).unwrap();
        (cluster, verifier)
    }

    fn in_cluster_verifier(cluster: Cluster) -> (Arc<Cluster>, LumenVerifier) {
        let cluster = Arc::new(cluster);
        let verifier = LumenVerifier::delegated_in_cluster("serving", cluster.clone()).unwrap();
        (cluster, verifier)
    }

    fn bearer(token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            format!("Bearer {token}").parse().unwrap(),
        );
        headers
    }

    async fn context(cluster: Cluster) -> (Arc<Cluster>, AuthContext) {
        let (cluster, verifier) = verifier(cluster);
        let ctx = verifier.authenticate_async(&bearer("t")).await.unwrap();
        (cluster, ctx)
    }

    async fn in_cluster_context(cluster: Cluster) -> (Arc<Cluster>, AuthContext) {
        let (cluster, verifier) = in_cluster_verifier(cluster);
        let ctx = verifier
            .authenticate_async(&bearer("default-ksa"))
            .await
            .unwrap();
        (cluster, ctx)
    }

    // ---- the mapping ----------------------------------------------------

    /// R5: every field of the check a collection read produces, named.
    #[test]
    fn a_collection_read_asks_about_that_collection_by_name() {
        let attributes = AuthTarget::Collection("orders").attributes("serving", Role::Read);
        assert_eq!(attributes.group, "lumen.axiom.dev");
        assert_eq!(attributes.namespace, "serving");
        assert_eq!(attributes.resource, "lumencollections");
        assert_eq!(attributes.name.as_deref(), Some("orders"));
        assert_eq!(attributes.verb, "get");
    }

    /// R5: the three roles are three distinct RBAC verbs, so a grant of one
    /// cannot be read as a grant of another.
    #[test]
    fn each_role_is_its_own_kubernetes_verb() {
        assert_eq!(verb(Role::Read), "get");
        assert_eq!(verb(Role::Write), "update");
        assert_eq!(verb(Role::Admin), "delete");
    }

    /// R6: the admin surface is a different resource with no resource name —
    /// not `lumencollections` with a wildcard.
    #[test]
    fn the_admin_surface_is_a_separate_resource_not_a_wildcard_collection() {
        let attributes = AuthTarget::Admin.attributes("serving", Role::Admin);
        assert_eq!(attributes.resource, "lumenadmin");
        assert_eq!(attributes.name, None);
        assert_ne!(attributes.resource, COLLECTIONS_RESOURCE);
    }

    // ---- authentication -------------------------------------------------

    /// AC1: a Lumen-audience KSA token authenticates.
    #[tokio::test]
    async fn a_lumen_audience_ksa_token_authenticates() {
        let (_, ctx) = context(Cluster::with_ksa("clients", "reader")).await;
        assert_eq!(ctx.subject(), Some("system:serviceaccount:clients:reader"));
    }

    /// AC1: the token every pod already has — bound only to the apiserver's
    /// own audience — does not open Lumen.
    #[tokio::test]
    async fn the_default_pod_token_audience_is_not_accepted() {
        let (_, verifier) = verifier(
            Cluster::with_ksa("clients", "reader")
                .with_audiences(&["https://kubernetes.default.svc"]),
        );
        assert!(matches!(
            verifier.authenticate_async(&bearer("t")).await.unwrap_err(),
            ServiceAuthError::Unauthenticated
        ));
    }

    /// AC2: `authenticated=true` is not enough. A Google user email is
    /// rejected at the Lumen boundary, before any SubjectAccessReview.
    #[tokio::test]
    async fn an_authenticated_google_user_is_rejected_before_authorization() {
        let (cluster, verifier) =
            verifier(Cluster::with_ksa("clients", "reader").as_user("alice@example.com"));
        assert!(matches!(
            verifier.authenticate_async(&bearer("t")).await.unwrap_err(),
            ServiceAuthError::Unauthenticated
        ));
        assert!(
            cluster.asked().is_empty(),
            "a rejected principal must never reach SubjectAccessReview"
        );
    }

    /// AC2: the same for a Google service account — the identity a workload
    /// using ADC would present.
    #[tokio::test]
    async fn an_authenticated_gsa_is_rejected_before_authorization() {
        let (cluster, verifier) = verifier(
            Cluster::with_ksa("clients", "reader").as_user("svc@project.iam.gserviceaccount.com"),
        );
        assert!(verifier.authenticate_async(&bearer("t")).await.is_err());
        assert!(cluster.asked().is_empty());
    }

    /// AC2: a value wearing the reserved prefix without the right shape is
    /// malformed — not a ServiceAccount named `reader:extra`.
    #[tokio::test]
    async fn a_malformed_service_account_username_is_rejected() {
        let (_, verifier) = verifier(
            Cluster::with_ksa("clients", "reader")
                .as_user("system:serviceaccount:clients:reader:extra"),
        );
        assert!(verifier.authenticate_async(&bearer("t")).await.is_err());
    }

    /// R10 / AC7: no credential at all is a 401, and costs the apiserver
    /// nothing.
    #[tokio::test]
    async fn a_request_without_a_credential_is_rejected_without_a_review() {
        let (cluster, verifier) = verifier(Cluster::with_ksa("clients", "reader"));
        assert!(matches!(
            verifier
                .authenticate_async(&HeaderMap::new())
                .await
                .unwrap_err(),
            ServiceAuthError::Unauthenticated
        ));
        assert!(cluster.asked().is_empty());
    }

    #[tokio::test]
    async fn managed_and_standalone_request_distinct_tokenreview_audiences() {
        let (managed_cluster, managed) = verifier(Cluster::with_ksa("clients", "reader"));
        managed
            .authenticate_async(&bearer("managed"))
            .await
            .unwrap();
        assert_eq!(
            managed_cluster.token_calls(),
            vec![vec![AUDIENCE.to_string()]],
            "Managed must keep its private audience"
        );

        let (standalone_cluster, standalone) =
            in_cluster_verifier(Cluster::with_ksa("clients", "reader").with_audiences(&[]));
        standalone
            .authenticate_async(&bearer("default-ksa"))
            .await
            .unwrap();
        assert_eq!(
            standalone_cluster.token_calls(),
            vec![Vec::<String>::new()],
            "Standalone must intentionally use TokenReview's Kubernetes-default audiences"
        );
    }

    #[tokio::test]
    async fn standalone_default_ksa_can_use_collections_but_not_lumenadmin() {
        let (cluster, ctx) = in_cluster_context(
            Cluster::with_ksa("apps", "api")
                .with_audiences(&[])
                .granting(COLLECTIONS_RESOURCE, Some("orders"), "get")
                .granting(COLLECTIONS_RESOURCE, Some("orders"), "update")
                .granting(COLLECTIONS_RESOURCE, Some("orders"), "delete"),
        )
        .await;
        for role in [Role::Read, Role::Write, Role::Admin] {
            ctx.ensure("orders", role).await.unwrap();
        }
        let error = ctx.ensure_admin(Role::Admin).await.unwrap_err();
        assert_eq!(error.wire().0, StatusCode::FORBIDDEN);
        assert_eq!(cluster.token_calls(), vec![Vec::<String>::new()]);
    }

    #[tokio::test]
    async fn standalone_default_profile_preserves_401_and_503_failure_classes() {
        let (_, missing) =
            in_cluster_verifier(Cluster::with_ksa("apps", "api").with_audiences(&[]));
        assert!(matches!(
            missing
                .authenticate_async(&HeaderMap::new())
                .await
                .unwrap_err(),
            ServiceAuthError::Unauthenticated
        ));

        let (_, wrong_identity) = in_cluster_verifier(
            Cluster::with_ksa("apps", "api")
                .with_audiences(&[])
                .as_user("alice@example.com"),
        );
        assert!(matches!(
            wrong_identity
                .authenticate_async(&bearer("bad-identity"))
                .await
                .unwrap_err(),
            ServiceAuthError::Unauthenticated
        ));

        let (_, unavailable) = in_cluster_verifier(
            Cluster::with_ksa("apps", "api")
                .with_audiences(&[])
                .reachable(Reachable::Neither),
        );
        assert!(matches!(
            unavailable
                .authenticate_async(&bearer("default-ksa"))
                .await
                .unwrap_err(),
            ServiceAuthError::Unavailable(_)
        ));

        let (_, ctx) = in_cluster_context(
            Cluster::with_ksa("apps", "api")
                .with_audiences(&[])
                .reachable(Reachable::AuthenticationOnly),
        )
        .await;
        let error = ctx.ensure("orders", Role::Read).await.unwrap_err();
        assert_eq!(error.wire().0, StatusCode::SERVICE_UNAVAILABLE);
    }

    // ---- authorization --------------------------------------------------

    /// AC3: `get` on `lumencollections/orders` reads `orders` — and nothing
    /// else. Each negative is a distinct way that grant could be over-read.
    #[tokio::test]
    async fn a_grant_on_one_collection_reaches_exactly_that_collection_and_verb() {
        let (_, ctx) = context(Cluster::with_ksa("clients", "reader").granting(
            COLLECTIONS_RESOURCE,
            Some("orders"),
            "get",
        ))
        .await;

        assert!(ctx.ensure("orders", Role::Read).await.is_ok());
        // ...not a write of the same collection,
        assert!(ctx.ensure("orders", Role::Write).await.is_err());
        // ...not another collection,
        assert!(ctx.ensure("invoices", Role::Read).await.is_err());
        // ...and not the admin surface.
        assert!(ctx.ensure_admin(Role::Admin).await.is_err());
    }

    /// AC4: the write and admin verbs are separately grantable, and holding
    /// one is not holding another.
    #[tokio::test]
    async fn write_and_admin_verbs_are_granted_independently() {
        let (_, ctx) = context(Cluster::with_ksa("clients", "writer").granting(
            COLLECTIONS_RESOURCE,
            Some("orders"),
            "update",
        ))
        .await;
        assert!(ctx.ensure("orders", Role::Write).await.is_ok());
        assert!(ctx.ensure("orders", Role::Admin).await.is_err());
        assert!(ctx.ensure("orders", Role::Read).await.is_err());
    }

    /// AC4 / R6: `lumenadmin` is granted on its own, and holding it says
    /// nothing about any collection.
    #[tokio::test]
    async fn the_admin_resource_is_granted_on_its_own() {
        let (_, ctx) =
            context(Cluster::with_ksa("ops", "backup").granting(ADMIN_RESOURCE, None, "delete"))
                .await;
        assert!(ctx.ensure_admin(Role::Admin).await.is_ok());
        assert!(ctx.ensure("orders", Role::Read).await.is_err());
    }

    /// AC3: every check is scoped to the serving instance's namespace, so a
    /// RoleBinding in the caller's own namespace does not carry over.
    #[tokio::test]
    async fn every_check_is_scoped_to_the_serving_namespace() {
        let (cluster, ctx) = context(Cluster::with_ksa("elsewhere", "reader")).await;
        let _ = ctx.ensure("orders", Role::Read).await;
        let asked = cluster.asked();
        assert_eq!(asked.len(), 1);
        assert_eq!(asked[0].namespace, "serving");
    }

    /// R4: the whole reviewed identity — not just the username — survives
    /// into the authorization question, so RBAC can bind by group.
    #[tokio::test]
    async fn the_reviewed_identity_survives_into_the_authorization_question() {
        let (_, ctx) = context(Cluster::with_ksa("clients", "reader")).await;
        let AuthContext::Delegated { principal, .. } = &ctx else {
            panic!("a delegated verifier yields a delegated context");
        };
        assert_eq!(principal.identity.uid, "uid-1");
        assert!(principal
            .identity
            .groups
            .contains(&"system:serviceaccounts".to_string()));
    }

    /// R10: a denial is a 403 naming the resource, and carries no credential.
    #[tokio::test]
    async fn a_denial_is_a_403_naming_the_resource_and_no_credential() {
        let (_, ctx) = context(Cluster::with_ksa("clients", "reader")).await;
        let err = ctx.ensure("orders", Role::Read).await.unwrap_err();
        let (status, code, message) = err.wire();
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(code, "forbidden");
        assert!(message.contains("orders"), "{message}");
        assert!(
            message.contains("system:serviceaccount:clients:reader"),
            "{message}"
        );
        assert!(!message.contains("Bearer"), "{message}");
    }

    /// AC7 / R10: an apiserver outage during authorization is a 503, not a
    /// 403 and never an allow. Reporting it as a denial sends an operator to
    /// fix a RoleBinding that was never wrong; reporting it as an allow is the
    /// failure this whole design exists to prevent.
    #[tokio::test]
    async fn an_authorization_outage_is_unavailable_never_denied_and_never_allowed() {
        let (_, ctx) = context(
            Cluster::with_ksa("clients", "reader")
                .granting(COLLECTIONS_RESOURCE, Some("orders"), "get")
                .reachable(Reachable::AuthenticationOnly),
        )
        .await;
        // The caller genuinely holds this grant; the apiserver simply cannot
        // say so. The answer is still not "yes".
        let err = ctx.ensure("orders", Role::Read).await.unwrap_err();
        assert!(matches!(err, AuthErr::Unavailable { .. }), "{err:?}");
        let (status, code, _) = err.wire();
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(code, "authorization_unavailable");
    }

    /// AC7: an outage during authentication is a rejection, not an open
    /// principal.
    #[tokio::test]
    async fn an_authentication_outage_rejects_rather_than_admitting() {
        let (_, verifier) =
            verifier(Cluster::with_ksa("clients", "reader").reachable(Reachable::Neither));
        assert!(verifier.authenticate_async(&bearer("t")).await.is_err());
    }

    /// AC7: the outage classification reaches the wire with a stable reason
    /// and no credential.
    #[test]
    fn an_unavailable_authorization_renders_with_a_stable_reason() {
        let err = AuthErr::Unavailable {
            subject: "system:serviceaccount:clients:reader".into(),
            needed: Role::Read,
            resource: "orders".into(),
            reason: "transport",
        };
        let (status, code, message) = err.wire();
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(code, "authorization_unavailable");
        assert!(message.contains("transport"), "{message}");
    }

    // ---- modes ----------------------------------------------------------

    /// R9: auth off asks Kubernetes nothing — not to authenticate, and not to
    /// authorize.
    #[tokio::test]
    async fn auth_off_never_asks_kubernetes_anything() {
        let verifier = LumenVerifier::new(Arc::new(AuthConfig::open()));
        let ctx = verifier
            .authenticate_async(&HeaderMap::new())
            .await
            .unwrap();
        assert!(matches!(ctx, AuthContext::Open));
        assert!(ctx.ensure("any", Role::Admin).await.is_ok());
        assert!(ctx.ensure_admin(Role::Admin).await.is_ok());
        assert_eq!(ctx.subject(), None);
        assert!(!AsyncVerifier::required(&verifier));
        assert!(verifier.render_metrics().is_empty());
    }

    /// Auth off still refuses a presented credential. Serving it as anonymous
    /// would tell a stale client its token was accepted by a process that
    /// owns no way to check one — the exact silent fallback #2871 pinned.
    #[tokio::test]
    async fn auth_off_rejects_a_presented_credential_rather_than_ignoring_it() {
        let verifier = LumenVerifier::new(Arc::new(AuthConfig::open()));
        assert!(verifier
            .authenticate_async(&bearer("looks-like-a-token"))
            .await
            .is_err());
    }

    /// A required config with no review backend rejects every request. It
    /// never degrades to the open principal, with or without a credential.
    #[tokio::test]
    async fn a_required_but_unwired_verifier_rejects_every_request() {
        let verifier = LumenVerifier::new(Arc::new(AuthConfig::required_in("serving")));
        assert!(AsyncVerifier::required(&verifier));
        assert!(verifier
            .authenticate_async(&HeaderMap::new())
            .await
            .is_err());
        assert!(verifier
            .authenticate_async(&bearer("anything"))
            .await
            .is_err());
    }

    /// R8: the delegated counters are rendered, and name no credential.
    #[tokio::test]
    async fn the_delegated_counters_are_exported_without_credentials() {
        let (_, verifier) = verifier(Cluster::with_ksa("clients", "reader"));
        let ctx = verifier
            .authenticate_async(&bearer("super-secret"))
            .await
            .unwrap();
        let _ = ctx.ensure("orders", Role::Read).await;
        let rendered = verifier.render_metrics();
        assert!(
            rendered.contains("delegated_auth_token_reviews_total"),
            "{rendered}"
        );
        assert!(
            rendered.contains("delegated_auth_denied_total"),
            "{rendered}"
        );
        assert!(!rendered.contains("super-secret"), "{rendered}");
    }

    // ---- configuration --------------------------------------------------

    #[test]
    fn auth_config_open_is_not_required() {
        let config = AuthConfig::open();
        assert!(!config.required);
        assert_eq!(config.profile(), AuthProfile::Off);
    }

    #[test]
    fn auth_config_from_env_open_when_unset() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_auth_env();
        let cfg = AuthConfig::from_env().unwrap();
        assert!(!cfg.required);
    }

    #[test]
    fn auth_config_from_env_accepts_both_disabled_spellings() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        for spelling in ["off", "disabled", "DISABLED", " off "] {
            clear_auth_env();
            unsafe {
                std::env::set_var("LUMEN_AUTH", spelling);
            }
            let cfg = AuthConfig::from_env()
                .unwrap_or_else(|e| panic!("`{spelling}` is a disabled spelling: {e:#}"));
            assert!(!cfg.required);
        }
        clear_auth_env();
    }

    /// R9: `required` resolves the serving namespace every check is scoped to.
    #[test]
    fn auth_config_required_takes_the_serving_namespace_from_the_environment() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_auth_env();
        unsafe {
            std::env::set_var("LUMEN_AUTH", "required");
            std::env::set_var("POD_NAMESPACE", "serving");
        }
        let cfg = AuthConfig::from_env().unwrap();
        assert!(cfg.required);
        assert_eq!(cfg.profile(), AuthProfile::ManagedAudience);
        assert_eq!(cfg.namespace, "serving");

        // The explicit override wins over the downward-API value.
        unsafe {
            std::env::set_var("LUMEN_AUTH_NAMESPACE", "override");
        }
        assert_eq!(AuthConfig::from_env().unwrap().namespace, "override");
        clear_auth_env();
    }

    #[test]
    fn auth_config_in_cluster_selects_only_the_kubernetes_default_profile() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_auth_env();
        unsafe {
            std::env::set_var("LUMEN_AUTH", " in-cluster ");
            std::env::set_var("POD_NAMESPACE", "lumen");
        }
        let config = AuthConfig::from_env().unwrap();
        assert!(config.required);
        assert_eq!(config.namespace, "lumen");
        assert_eq!(config.profile(), AuthProfile::KubernetesDefault);
        assert_eq!(
            AuthConfig::required_in("lumen").profile(),
            AuthProfile::ManagedAudience
        );
        assert_eq!(
            AuthConfig::in_cluster("lumen").profile(),
            AuthProfile::KubernetesDefault
        );
        clear_auth_env();
    }

    /// R9: an unscoped SubjectAccessReview asks about a different resource
    /// than the request touched, so `required` refuses to start without a
    /// namespace — unless the pod's own ServiceAccount file supplies one,
    /// which is exactly when delegation can work at all.
    #[test]
    fn auth_config_required_fails_closed_without_a_serving_namespace() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_auth_env();
        unsafe {
            std::env::set_var("LUMEN_AUTH", "required");
        }
        let result = AuthConfig::from_env();
        clear_auth_env();
        if std::path::Path::new(SERVICE_ACCOUNT_NAMESPACE_FILE).exists() {
            // Running inside a pod: the namespace is discoverable, so there is
            // nothing to fail closed about.
            assert!(result.unwrap().required);
            return;
        }
        let message = format!("{:#}", result.unwrap_err());
        assert!(message.contains("LUMEN_AUTH=required"), "{message}");
        assert!(message.contains("SubjectAccessReview"), "{message}");
        assert!(message.contains("Refusing to start"), "{message}");
    }

    #[test]
    fn auth_config_in_cluster_fails_closed_without_a_serving_namespace() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_auth_env();
        unsafe {
            std::env::set_var("LUMEN_AUTH", "in-cluster");
        }
        let result = AuthConfig::from_env();
        clear_auth_env();
        if std::path::Path::new(SERVICE_ACCOUNT_NAMESPACE_FILE).exists() {
            assert_eq!(result.unwrap().profile(), AuthProfile::KubernetesDefault);
            return;
        }
        let message = format!("{:#}", result.unwrap_err());
        assert!(message.contains("LUMEN_AUTH=in-cluster"), "{message}");
        assert!(message.contains("SubjectAccessReview"), "{message}");
        assert!(message.contains("Refusing to start"), "{message}");
    }

    #[test]
    fn auth_config_from_env_rejects_unknown_auth_mode() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_auth_env();
        unsafe {
            std::env::set_var("LUMEN_AUTH", "require");
        }
        let err = AuthConfig::from_env().unwrap_err();
        assert!(err.to_string().contains("LUMEN_AUTH"));
        clear_auth_env();
    }
}
// CODEGEN-END
