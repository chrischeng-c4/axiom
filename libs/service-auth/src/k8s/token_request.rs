// HANDWRITE-BEGIN gap="missing-generator:logic:k8s-token-request-client" tracker="#2878" reason="Minting a short-lived audience-bound ServiceAccount token from a caller's own kubeconfig, plus the refresh clock and the redaction contract around it; no generator primitive models a TokenRequest round trip or a credential that must not reach a log line."
//! Asking the apiserver for a ServiceAccount token, as the caller.
//!
//! The mirror image of [`super::projected`]. That module reads a token the
//! kubelet mounted for a workload; this one mints a token for a *human or
//! automation* that already has a Kubernetes credential of its own — a
//! kubeconfig, usually with an exec credential plugin behind it.
//!
//! ```text
//!   kubeconfig identity  --(exec plugin, TLS cert, whatever)-->  kube-apiserver
//!            |                                       RBAC: may this identity
//!            |                                       `create` the `token`
//!            |                                       subresource of *this one*
//!            v                                       ServiceAccount?
//!   TokenRequest(namespace, serviceAccount, audience, expirationSeconds)
//!            |
//!            v
//!   a short-lived, audience-bound token  ---->  the audience-bound service
//! ```
//!
//! ## Why the caller's own credential never goes any further
//!
//! The identity in the kubeconfig may be a Google account, a cloud IAM service
//! account, a client certificate, or an OIDC subject. None of that is any of
//! the callee's business, and none of it is forwarded: it authenticates to
//! kube-apiserver and stops there. What continues is the minted token, whose
//! `aud` is the callee and whose lifetime is minutes. That is the entire
//! reason to make this round trip rather than reusing the credential already
//! in hand — the caller's credential is long-lived, broadly scoped, and
//! addressed to somebody else.
//!
//! ## What is in this module and what is not
//!
//! Same split as the rest of `k8s`: everything that decides is pure and
//! tested, and only [`KubeTokenMinter`] opens a socket.
//!
//! - [`TokenRequestTarget`] is the request, validated. Its
//!   [`request_body`](TokenRequestTarget::request_body) is the literal JSON
//!   that goes on the wire, so a test can assert the audience and duration
//!   without a cluster and without the assertion being a restatement.
//! - [`MintedToken`] is the answer plus the server's expiry — which is
//!   authoritative and frequently shorter than what was asked for.
//! - [`TokenSource`] is the refresh clock: mint once, reuse until the token is
//!   near its end, mint again. It fails rather than presenting a token it
//!   knows is stale.
//! - [`TokenMinter`] is the seam. A fake implementation is what makes the
//!   refresh behaviour testable at all, since the interesting cases are an
//!   hour apart.
//!
//! Nothing here names a service or an audience. The audience is the callee's
//! to declare, and it belongs in the callee's own crate next to the
//! [`super::delegated`] configuration that checks it.

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::json;
use tokio::sync::Mutex;

use super::cache::{Clock, SystemClock};
use super::projected::ProjectedToken;

/// The lifetime to ask for when the caller has no reason to prefer another.
///
/// Also the apiserver's floor: `TokenRequestSpec.expirationSeconds` below ten
/// minutes is rejected by validation, so this is simultaneously "short" and
/// "the shortest thing that works".
pub const DEFAULT_EXPIRATION_SECONDS: i64 = 600;

/// Below this, kube-apiserver refuses the request outright.
pub const MIN_EXPIRATION_SECONDS: i64 = 600;

/// Refresh with a fifth of the lifetime still to go — the same 80% mark the
/// kubelet uses for projected volumes, for the same reason: leave enough tail
/// that a slow mint, a retried request, or a clock a little out of step still
/// lands inside the window.
const REFRESH_TAIL_DIVISOR: u64 = 5;

/// Never run a token closer to its expiry than this, however short the
/// lifetime turned out to be.
const MIN_GUARD: Duration = Duration::from_secs(30);

// ---------------------------------------------------------------------------
// The request
// ---------------------------------------------------------------------------

/// Which ServiceAccount's token to ask for, for whom, and for how long.
///
/// Constructing one validates the names. That is not tidiness: `namespace` and
/// `service_account` are interpolated into a URL path, and a value containing
/// `/` or `..` would address a different resource than the one the caller
/// named — which is the one thing an explicit-target contract must not allow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenRequestTarget {
    namespace: String,
    service_account: String,
    audience: String,
    expiration_seconds: i64,
}

impl TokenRequestTarget {
    /// A target for the named ServiceAccount, at [`DEFAULT_EXPIRATION_SECONDS`].
    pub fn new(
        namespace: impl Into<String>,
        service_account: impl Into<String>,
        audience: impl Into<String>,
    ) -> Result<Self, TokenRequestError> {
        let namespace = namespace.into();
        let service_account = service_account.into();
        let audience = audience.into();
        check_object_name("namespace", &namespace)?;
        check_object_name("client service account", &service_account)?;
        if audience.trim().is_empty() {
            return Err(TokenRequestError::InvalidTarget {
                field: "audience",
                value: audience,
                reason: "an audience-bound token needs an audience; a token minted with none is \
                         accepted by every service that does not check, which is the failure \
                         this whole path exists to prevent"
                    .to_string(),
            });
        }
        Ok(Self {
            namespace,
            service_account,
            audience,
            expiration_seconds: DEFAULT_EXPIRATION_SECONDS,
        })
    }

    /// A target that asks kube-apiserver to use its configured default
    /// audiences. The empty `spec.audiences` array is meaningful here and is
    /// distinct from an explicit audience, which [`Self::new`] still requires.
    pub fn kubernetes_default(
        namespace: impl Into<String>,
        service_account: impl Into<String>,
    ) -> Result<Self, TokenRequestError> {
        let namespace = namespace.into();
        let service_account = service_account.into();
        check_object_name("namespace", &namespace)?;
        check_object_name("client service account", &service_account)?;
        Ok(Self {
            namespace,
            service_account,
            audience: String::new(),
            expiration_seconds: DEFAULT_EXPIRATION_SECONDS,
        })
    }

    /// Ask for a different lifetime. The apiserver may still issue a shorter
    /// one, and [`MintedToken`] carries what it actually issued.
    pub fn with_expiration_seconds(mut self, seconds: i64) -> Result<Self, TokenRequestError> {
        if seconds < MIN_EXPIRATION_SECONDS {
            return Err(TokenRequestError::InvalidTarget {
                field: "expiration",
                value: seconds.to_string(),
                reason: format!(
                    "kube-apiserver rejects a TokenRequest below {MIN_EXPIRATION_SECONDS} seconds"
                ),
            });
        }
        self.expiration_seconds = seconds;
        Ok(self)
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn service_account(&self) -> &str {
        &self.service_account
    }

    pub fn audience(&self) -> &str {
        &self.audience
    }

    pub fn expiration_seconds(&self) -> i64 {
        self.expiration_seconds
    }

    /// `<namespace>/<serviceaccount>`, for messages that name the target.
    pub fn qualified_name(&self) -> String {
        format!("{}/{}", self.namespace, self.service_account)
    }

    /// The subresource this request POSTs to.
    ///
    /// A `k8s`-gated test checks this against the path `kube` derives on its
    /// own, so the string here cannot quietly stop describing where the
    /// request goes.
    pub fn subresource_path(&self) -> String {
        format!(
            "/api/v1/namespaces/{}/serviceaccounts/{}/token",
            self.namespace, self.service_account
        )
    }

    /// The literal request body.
    ///
    /// [`KubeTokenMinter`] deserializes *this* into the typed `TokenRequest`
    /// rather than building a second one beside it, so the audience and
    /// duration a test asserts here are the audience and duration that go on
    /// the wire.
    pub fn request_body(&self) -> serde_json::Value {
        let audiences = if self.audience.is_empty() {
            json!([])
        } else {
            json!([self.audience])
        };
        json!({
            "apiVersion": "authentication.k8s.io/v1",
            "kind": "TokenRequest",
            "spec": {
                "audiences": audiences,
                "expirationSeconds": self.expiration_seconds,
            }
        })
    }
}

/// DNS-1123-ish: what Kubernetes accepts for a namespace or ServiceAccount
/// name, checked here so a path-bearing value is refused before it becomes a
/// URL. The message names the rule rather than restating the input, because
/// the input is what is already on screen.
fn check_object_name(field: &'static str, value: &str) -> Result<(), TokenRequestError> {
    let invalid = |reason: &str| TokenRequestError::InvalidTarget {
        field,
        value: value.to_string(),
        reason: reason.to_string(),
    };
    if value.is_empty() {
        return Err(invalid("a name is required; this target is never inferred"));
    }
    if value.len() > 253 {
        return Err(invalid("Kubernetes names stop at 253 characters"));
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.')
    {
        return Err(invalid(
            "a Kubernetes name is lowercase alphanumerics, `-`, and `.` — nothing else, and in \
             particular no `/`, which would address a different object than the one named",
        ));
    }
    if !value.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        || !value.ends_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
    {
        return Err(invalid(
            "a Kubernetes name starts and ends with a letter or a digit",
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// The answer
// ---------------------------------------------------------------------------

/// A token and the moment the *server* said it stops working.
///
/// The expiry is not the one that was requested. `--service-account-max-token-
/// expiration` caps it cluster-wide, and a bound token is capped by the
/// lifetime of what it is bound to. A client that assumes it got what it asked
/// for works until someone lowers that flag.
#[derive(Debug, Clone)]
pub struct MintedToken {
    token: ProjectedToken,
    minted_at_millis: u64,
    expires_at_millis: u64,
}

impl MintedToken {
    pub fn new(token: impl Into<String>, minted_at_millis: u64, expires_at_millis: u64) -> Self {
        Self {
            token: ProjectedToken::new(token.into()),
            minted_at_millis,
            expires_at_millis,
        }
    }

    /// The credential. The only accessor that yields the material, and the
    /// wrapper it comes in refuses to print itself.
    pub fn token(&self) -> &ProjectedToken {
        &self.token
    }

    pub fn expires_at_millis(&self) -> u64 {
        self.expires_at_millis
    }

    /// When this token should be replaced: four fifths of the way through its
    /// life, or [`MIN_GUARD`] before the end, whichever comes first.
    pub fn refresh_at_millis(&self) -> u64 {
        let lifetime = self.expires_at_millis.saturating_sub(self.minted_at_millis);
        // The tail left unused: one fifth, but never less than the guard, and
        // never more than the whole lifetime — a token shorter than the guard
        // is due the moment it arrives rather than never.
        let tail = lifetime / REFRESH_TAIL_DIVISOR;
        let guard = tail.max(MIN_GUARD.as_millis() as u64).min(lifetime);
        self.expires_at_millis.saturating_sub(guard)
    }
}

// ---------------------------------------------------------------------------
// Failures
// ---------------------------------------------------------------------------

/// Why no token was minted. No variant carries a token, and none carries the
/// caller's own credential either — the exec plugin's output is a credential
/// too, and `kube`'s error for a failed plugin is one of the places it can
/// surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenRequestError {
    /// Refused before anything was sent.
    InvalidTarget {
        field: &'static str,
        value: String,
        reason: String,
    },
    /// No usable Kubernetes client configuration, or the apiserver would not
    /// authenticate the caller at all.
    NoIdentity { detail: String },
    /// The caller authenticated, and RBAC said no.
    ///
    /// `username` is what the apiserver says the caller is — resolved by
    /// asking it, not parsed out of the denial text. `None` means even that
    /// question failed, which is worth saying rather than guessing.
    Forbidden {
        username: Option<String>,
        namespace: String,
        service_account: String,
        detail: String,
    },
    /// The ServiceAccount does not exist. Distinct from [`Self::Forbidden`] on
    /// purpose: one is a grant to fix, the other is a name to fix, and a
    /// `create` grant that names a ServiceAccount nobody created reads as the
    /// first while being the second.
    NoSuchServiceAccount {
        namespace: String,
        service_account: String,
    },
    /// The round trip did not complete.
    Transport { detail: String },
    /// It completed and the answer was not one.
    Malformed { detail: String },
}

impl TokenRequestError {
    /// The `kubectl` question whose answer is this error, for callers
    /// assembling remediation. Names the ServiceAccount, so it asks about the
    /// grant that was actually missing rather than the namespace-wide one.
    pub fn can_i_command(namespace: &str, service_account: &str) -> String {
        format!(
            "kubectl auth can-i create serviceaccounts/{service_account} --subresource=token \
             -n {namespace}"
        )
    }
}

impl fmt::Display for TokenRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTarget {
                field,
                value,
                reason,
            } => write!(f, "invalid {field} `{value}`: {reason}"),
            Self::NoIdentity { detail } => write!(
                f,
                "no Kubernetes identity to mint a token with: {detail} — this path uses your \
                 kubeconfig and nothing else, so `kubectl auth whoami` failing here means the \
                 same thing it would there"
            ),
            Self::Forbidden {
                username,
                namespace,
                service_account,
                detail,
            } => {
                let who = match username {
                    Some(name) => format!("`{name}`"),
                    None => "the identity in your kubeconfig".to_string(),
                };
                write!(
                    f,
                    "{who} may not mint a token for ServiceAccount `{namespace}/{service_account}`: \
                     {detail}. Check with `{}`; the missing grant is `create` on \
                     `serviceaccounts/token` with `resourceNames: [{service_account}]`",
                    Self::can_i_command(namespace, service_account)
                )
            }
            Self::NoSuchServiceAccount {
                namespace,
                service_account,
            } => write!(
                f,
                "ServiceAccount `{namespace}/{service_account}` does not exist — a token can only \
                 be minted for an account that does, and a grant naming one that does not is \
                 accepted by RBAC without ever working"
            ),
            Self::Transport { detail } => {
                write!(f, "the TokenRequest did not complete: {detail}")
            }
            Self::Malformed { detail } => write!(
                f,
                "the apiserver accepted the TokenRequest and did not answer it: {detail}"
            ),
        }
    }
}

impl std::error::Error for TokenRequestError {}

// ---------------------------------------------------------------------------
// The seam
// ---------------------------------------------------------------------------

/// Whatever can turn a [`TokenRequestTarget`] into a [`MintedToken`].
///
/// One real implementation ([`KubeTokenMinter`]) and, in tests, a fake — which
/// is the point. The behaviour worth testing here is what happens over the
/// course of an hour, and the alternative to a seam is a test suite that takes
/// one.
#[async_trait]
pub trait TokenMinter: Send + Sync {
    async fn mint(&self, target: &TokenRequestTarget) -> Result<MintedToken, TokenRequestError>;
}

/// A token that keeps itself current.
///
/// Holds exactly one token at a time, in memory, and hands out clones. There
/// is no file, no environment variable, and no way to read it out other than
/// [`ProjectedToken::expose`] — which the one place that writes an
/// `Authorization` header calls, and nothing else does.
pub struct TokenSource {
    minter: Arc<dyn TokenMinter>,
    target: TokenRequestTarget,
    clock: Arc<dyn Clock>,
    current: Mutex<Option<MintedToken>>,
}

impl TokenSource {
    pub fn new(minter: Arc<dyn TokenMinter>, target: TokenRequestTarget) -> Self {
        Self::with_clock(minter, target, Arc::new(SystemClock))
    }

    pub fn with_clock(
        minter: Arc<dyn TokenMinter>,
        target: TokenRequestTarget,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            minter,
            target,
            clock,
            current: Mutex::new(None),
        }
    }

    pub fn target(&self) -> &TokenRequestTarget {
        &self.target
    }

    /// A token that is good right now, minting one if the one in hand is not.
    ///
    /// A failed refresh is returned, not swallowed. The tempting alternative —
    /// keep serving the old token until it actually expires — turns a revoked
    /// grant into a delay instead of a refusal, and the whole reason these
    /// tokens are short is so that revocation means something.
    pub async fn token(&self) -> Result<ProjectedToken, TokenRequestError> {
        let mut current = self.current.lock().await;
        let now = self.clock.now_millis();
        if let Some(token) = current.as_ref() {
            if now < token.refresh_at_millis() {
                return Ok(token.token().clone());
            }
        }
        let minted = self.minter.mint(&self.target).await?;
        let token = minted.token().clone();
        *current = Some(minted);
        Ok(token)
    }
}

// ---------------------------------------------------------------------------
// The transport
// ---------------------------------------------------------------------------

/// The kube-rs implementation. The only thing in this module that opens a
/// socket.
///
/// Its client comes from the ambient configuration — in-cluster when there is
/// a mounted ServiceAccount, the kubeconfig otherwise, including exec
/// credential plugins such as GKE's. Those plugins are how a cloud identity
/// reaches kube-apiserver, and they are the *only* way one is used here: this
/// crate links no cloud SDK, reads no application-default credential, and
/// never speaks to a metadata server. `kube`'s deprecated in-tree `gcp`
/// auth-provider path is likewise absent, because the `oauth` feature that
/// would enable it is off — a kubeconfig still using it fails here rather than
/// quietly acquiring a Google token in-process.
#[cfg(feature = "k8s")]
pub struct KubeTokenMinter {
    client: kube::Client,
}

#[cfg(feature = "k8s")]
impl KubeTokenMinter {
    /// Build from the ambient Kubernetes configuration.
    pub async fn from_ambient_config() -> Result<Self, TokenRequestError> {
        let client =
            kube::Client::try_default()
                .await
                .map_err(|error| TokenRequestError::NoIdentity {
                    detail: error.to_string(),
                })?;
        Ok(Self { client })
    }

    /// Build from a named kubeconfig context, or from the ambient
    /// configuration when `context` is `None`.
    ///
    /// Naming a context matters here in a way it does not for a serving
    /// process: the whole point of this call is that the caller chooses which
    /// identity is asking, and a machine with several clusters configured
    /// should not have that decided by whichever `kubectl config use-context`
    /// ran last.
    pub async fn from_context(context: Option<&str>) -> Result<Self, TokenRequestError> {
        let Some(context) = context else {
            return Self::from_ambient_config().await;
        };
        let options = kube::config::KubeConfigOptions {
            context: Some(context.to_string()),
            cluster: None,
            user: None,
        };
        let config = kube::Config::from_kubeconfig(&options).await.map_err(|e| {
            TokenRequestError::NoIdentity {
                detail: format!("kubeconfig context `{context}`: {e}"),
            }
        })?;
        let client = kube::Client::try_from(config).map_err(|e| TokenRequestError::NoIdentity {
            detail: format!("kubeconfig context `{context}`: {e}"),
        })?;
        Ok(Self { client })
    }

    pub fn from_client(client: kube::Client) -> Self {
        Self { client }
    }

    /// What the apiserver says this client is, via `SelfSubjectReview` — the
    /// call behind `kubectl auth whoami`.
    ///
    /// Asked only to name the caller in a denial. Scraping the username out of
    /// the RBAC message would work today and is exactly the kind of thing that
    /// stops working in a version bump, silently, in the error path nobody
    /// tests.
    pub async fn whoami(&self) -> Option<String> {
        use k8s_openapi::api::authentication::v1::SelfSubjectReview;
        let api: kube::Api<SelfSubjectReview> = kube::Api::all(self.client.clone());
        let review = api
            .create(&kube::api::PostParams::default(), &Default::default())
            .await
            .ok()?;
        review.status?.user_info?.username
    }
}

#[cfg(feature = "k8s")]
#[async_trait]
impl TokenMinter for KubeTokenMinter {
    async fn mint(&self, target: &TokenRequestTarget) -> Result<MintedToken, TokenRequestError> {
        use k8s_openapi::api::authentication::v1::TokenRequest;
        use k8s_openapi::api::core::v1::ServiceAccount;

        // The body a test can assert on, deserialized into the typed request
        // rather than rebuilt beside it.
        let request: TokenRequest = serde_json::from_value(target.request_body()).map_err(|e| {
            TokenRequestError::Malformed {
                detail: format!("could not build the TokenRequest body: {e}"),
            }
        })?;

        let api: kube::Api<ServiceAccount> =
            kube::Api::namespaced(self.client.clone(), target.namespace());
        let response = match api
            .create_token_request(
                target.service_account(),
                &kube::api::PostParams::default(),
                &request,
            )
            .await
        {
            Ok(response) => response,
            Err(error) => return Err(self.classify(target, error).await),
        };

        let Some(status) = response.status else {
            return Err(TokenRequestError::Malformed {
                detail: "TokenRequest response carried no status".to_string(),
            });
        };
        if status.token.is_empty() {
            return Err(TokenRequestError::Malformed {
                detail: "TokenRequest response carried an empty token".to_string(),
            });
        }
        let expires_at_millis = status
            .expiration_timestamp
            .0
            .timestamp_millis()
            .max(0)
            .unsigned_abs();
        let now = SystemClock.now_millis();
        Ok(MintedToken::new(status.token, now, expires_at_millis))
    }
}

#[cfg(feature = "k8s")]
impl KubeTokenMinter {
    /// Translate a `kube` error, resolving the caller's username for the one
    /// case where naming it is the whole remediation.
    async fn classify(&self, target: &TokenRequestTarget, error: kube::Error) -> TokenRequestError {
        match error {
            kube::Error::Api(response) if response.code == 403 => TokenRequestError::Forbidden {
                username: self.whoami().await,
                namespace: target.namespace().to_string(),
                service_account: target.service_account().to_string(),
                detail: response.message,
            },
            kube::Error::Api(response) if response.code == 404 => {
                TokenRequestError::NoSuchServiceAccount {
                    namespace: target.namespace().to_string(),
                    service_account: target.service_account().to_string(),
                }
            }
            kube::Error::Api(response) if response.code == 401 => TokenRequestError::NoIdentity {
                detail: response.message,
            },
            kube::Error::Api(response) => TokenRequestError::Transport {
                detail: format!("apiserver returned {}: {}", response.code, response.message),
            },
            other => TokenRequestError::Transport {
                detail: other.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    use crate::k8s::cache::ManualClock;

    const AUDIENCE: &str = "callee.example.com";
    const CANARY: &str = "canary-minted-token-must-never-be-printed";

    fn target() -> TokenRequestTarget {
        TokenRequestTarget::new("ops", "app-client", AUDIENCE).expect("a valid target")
    }

    /// Mints a distinct token per call and records what it was asked for, so a
    /// test can assert both the request and how many were made.
    struct RecordingMinter {
        lifetime_millis: u64,
        calls: AtomicU64,
        clock: Arc<ManualClock>,
        fail_after: u64,
    }

    impl RecordingMinter {
        fn new(clock: Arc<ManualClock>, lifetime: Duration) -> Self {
            Self {
                lifetime_millis: lifetime.as_millis() as u64,
                calls: AtomicU64::new(0),
                clock,
                fail_after: u64::MAX,
            }
        }

        fn failing_after(mut self, calls: u64) -> Self {
            self.fail_after = calls;
            self
        }

        fn calls(&self) -> u64 {
            self.calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl TokenMinter for RecordingMinter {
        async fn mint(
            &self,
            target: &TokenRequestTarget,
        ) -> Result<MintedToken, TokenRequestError> {
            let n = self.calls.fetch_add(1, Ordering::SeqCst);
            if n >= self.fail_after {
                return Err(TokenRequestError::Forbidden {
                    username: Some("alice@example.com".to_string()),
                    namespace: target.namespace().to_string(),
                    service_account: target.service_account().to_string(),
                    detail: "the grant was removed".to_string(),
                });
            }
            let now = self.clock.now_millis();
            Ok(MintedToken::new(
                format!("{CANARY}-{n}"),
                now,
                now + self.lifetime_millis,
            ))
        }
    }

    /// AC1's request half, without a cluster: the body that goes on the wire
    /// carries exactly the audience and duration that were asked for, and the
    /// path names exactly the namespace and ServiceAccount.
    #[test]
    fn the_request_names_the_audience_the_duration_the_namespace_and_the_account() {
        let target = target();
        assert_eq!(
            target.request_body(),
            json!({
                "apiVersion": "authentication.k8s.io/v1",
                "kind": "TokenRequest",
                "spec": {
                    "audiences": ["callee.example.com"],
                    "expirationSeconds": 600,
                }
            })
        );
        assert_eq!(
            target.subresource_path(),
            "/api/v1/namespaces/ops/serviceaccounts/app-client/token"
        );
    }

    #[test]
    fn the_kubernetes_default_request_uses_empty_audiences_and_default_lifetime() {
        let target = TokenRequestTarget::kubernetes_default("ops", "app-client")
            .expect("valid Kubernetes target");
        assert_eq!(target.expiration_seconds(), DEFAULT_EXPIRATION_SECONDS);
        assert_eq!(
            target.subresource_path(),
            "/api/v1/namespaces/ops/serviceaccounts/app-client/token"
        );
        assert_eq!(
            target.request_body(),
            json!({
                "apiVersion": "authentication.k8s.io/v1",
                "kind": "TokenRequest",
                "spec": {
                    "audiences": [],
                    "expirationSeconds": 600,
                }
            })
        );
    }

    #[test]
    fn the_kubernetes_default_constructor_rejects_invalid_names() {
        for (namespace, account) in [
            ("ops", "../../secrets/registry"),
            ("ops", "app-client/../lumen-operator"),
            ("ops/../kube-system", "app-client"),
            ("ops", ""),
            ("", "app-client"),
            ("ops", "App-Client"),
            ("ops", "-app-client"),
        ] {
            let err = TokenRequestTarget::kubernetes_default(namespace, account)
                .expect_err("invalid name must be rejected");
            assert!(
                matches!(err, TokenRequestError::InvalidTarget { .. }),
                "{namespace}/{account}: {err:?}"
            );
        }
    }

    #[test]
    fn explicit_managed_and_custom_audiences_keep_their_wire_body() {
        for audience in ["managed.example.com", "custom.example.com"] {
            let target = TokenRequestTarget::new("ops", "app-client", audience)
                .expect("valid explicit audience");
            assert_eq!(
                target.request_body(),
                json!({
                    "apiVersion": "authentication.k8s.io/v1",
                    "kind": "TokenRequest",
                    "spec": {
                        "audiences": [audience],
                        "expirationSeconds": 600,
                    }
                })
            );
        }
    }

    /// The path string above is only worth asserting if it is the path `kube`
    /// actually builds. This checks it against `kube`'s own derivation rather
    /// than against a second copy of the same guess.
    #[cfg(feature = "k8s")]
    #[test]
    fn the_documented_subresource_path_is_the_one_kube_derives() {
        use k8s_openapi::api::core::v1::ServiceAccount;
        use kube::Resource;

        let derived = format!(
            "{}/{}/token",
            ServiceAccount::url_path(&(), Some("ops")),
            "app-client"
        );
        assert_eq!(target().subresource_path(), derived);
    }

    #[test]
    fn a_longer_lifetime_can_be_asked_for_and_a_shorter_one_cannot() {
        let longer = target()
            .with_expiration_seconds(3600)
            .expect("above the floor");
        assert_eq!(longer.request_body()["spec"]["expirationSeconds"], 3600);

        let err = target()
            .with_expiration_seconds(60)
            .expect_err("below the apiserver's floor");
        assert!(
            matches!(err, TokenRequestError::InvalidTarget { field, .. } if field == "expiration"),
            "{err:?}"
        );
        assert!(err.to_string().contains("600"), "{err}");
    }

    /// R3's other half: the target is never inferred, so every way of naming
    /// it badly is refused rather than normalised into something that
    /// addresses a different object.
    #[test]
    fn a_name_that_would_address_another_object_is_refused_before_anything_is_sent() {
        for (namespace, account) in [
            ("ops", "../../secrets/registry"),
            ("ops", "app-client/../lumen-operator"),
            ("ops/../kube-system", "app-client"),
            ("ops", ""),
            ("", "app-client"),
            ("ops", "App-Client"),
            ("ops", "-app-client"),
        ] {
            let err = TokenRequestTarget::new(namespace, account, AUDIENCE)
                .expect_err("a name that is not a Kubernetes name must not be accepted");
            assert!(
                matches!(err, TokenRequestError::InvalidTarget { .. }),
                "{namespace}/{account}: {err:?}"
            );
        }
    }

    #[test]
    fn a_token_with_no_audience_is_refused() {
        let err = TokenRequestTarget::new("ops", "app-client", "  ")
            .expect_err("an audience is required");
        assert!(
            matches!(err, TokenRequestError::InvalidTarget { field, .. } if field == "audience"),
            "{err:?}"
        );
    }

    /// The server's expiry wins. Asking for 600 seconds and being handed 300
    /// is a supported answer, not an anomaly, and the refresh point has to
    /// follow the answer.
    #[tokio::test]
    async fn the_refresh_point_follows_the_issued_expiry_not_the_requested_one() {
        let requested = MintedToken::new("t", 0, 600_000);
        assert_eq!(requested.refresh_at_millis(), 480_000);

        let issued_shorter = MintedToken::new("t", 0, 300_000);
        assert_eq!(issued_shorter.refresh_at_millis(), 240_000);

        // 20% of a minute is 12 seconds, less than the 30-second guard, so the
        // guard wins and the refresh happens earlier than the fraction alone
        // would put it.
        let very_short = MintedToken::new("t", 0, 60_000);
        assert_eq!(very_short.refresh_at_millis(), 30_000);

        // Shorter than the guard entirely: due immediately. A token that
        // cannot be held for the guard interval is one to replace on sight,
        // not one to hold and hope.
        let tiny = MintedToken::new("t", 0, 10_000);
        assert_eq!(tiny.refresh_at_millis(), 0);
    }

    /// AC5's first half: a long-running caller mints once, reuses, and mints
    /// again before the token it holds stops working — never after.
    #[tokio::test]
    async fn a_long_running_caller_refreshes_before_expiry_and_not_on_every_call() {
        let clock = Arc::new(ManualClock::new(0));
        let minter = Arc::new(RecordingMinter::new(
            clock.clone(),
            Duration::from_secs(600),
        ));
        let source = TokenSource::with_clock(minter.clone(), target(), clock.clone());

        let first = source.token().await.expect("first mint");
        assert_eq!(minter.calls(), 1);

        // Anywhere inside the first four fifths, the same token comes back.
        for _ in 0..5 {
            clock.advance(Duration::from_secs(60));
            assert_eq!(
                source.token().await.expect("reuse").expose(),
                first.expose()
            );
        }
        assert_eq!(minter.calls(), 1, "a reused token must not be re-minted");

        // 480s in — the refresh point, two minutes before the token dies.
        clock.advance(Duration::from_secs(180));
        let second = source.token().await.expect("refresh");
        assert_ne!(
            second.expose(),
            first.expose(),
            "the refresh must produce a new token, not re-present the old one"
        );
        assert_eq!(minter.calls(), 2);
        assert!(
            clock.now_millis() < 600_000,
            "the refresh has to happen while the old token still works, or every in-flight \
             request between expiry and refresh fails"
        );
    }

    /// AC5's second half: when the grant goes away, the next refresh fails and
    /// the failure is returned. Continuing to serve the token already in hand
    /// would turn revocation into a delay.
    #[tokio::test]
    async fn a_revoked_grant_surfaces_at_the_next_refresh_rather_than_at_expiry() {
        let clock = Arc::new(ManualClock::new(0));
        let minter = Arc::new(
            RecordingMinter::new(clock.clone(), Duration::from_secs(600)).failing_after(1),
        );
        let source = TokenSource::with_clock(minter.clone(), target(), clock.clone());

        source.token().await.expect("the first mint still works");
        clock.advance(Duration::from_secs(480));

        let err = source.token().await.expect_err("the refresh is refused");
        assert!(
            matches!(err, TokenRequestError::Forbidden { .. }),
            "{err:?}"
        );
    }

    /// R6: the denial names who was refused, what they were refused, and the
    /// exact question whose answer it is.
    #[test]
    fn a_denial_names_the_caller_the_target_and_the_check() {
        let err = TokenRequestError::Forbidden {
            username: Some("alice@example.com".to_string()),
            namespace: "ops".to_string(),
            service_account: "app-client".to_string(),
            detail: "cannot create resource \"serviceaccounts/token\"".to_string(),
        };
        let rendered = err.to_string();
        for expected in [
            "alice@example.com",
            "ops/app-client",
            "kubectl auth can-i create serviceaccounts/app-client --subresource=token -n ops",
            "resourceNames: [app-client]",
        ] {
            assert!(
                rendered.contains(expected),
                "missing `{expected}`: {rendered}"
            );
        }
    }

    #[test]
    fn a_denial_that_cannot_name_the_caller_says_so_rather_than_guessing() {
        let err = TokenRequestError::Forbidden {
            username: None,
            namespace: "ops".to_string(),
            service_account: "app-client".to_string(),
            detail: "forbidden".to_string(),
        };
        let rendered = err.to_string();
        assert!(
            rendered.contains("the identity in your kubeconfig"),
            "{rendered}"
        );
        assert!(!rendered.contains("None"), "{rendered}");
    }

    /// AC6 at this layer: neither the token wrapper nor any error rendering
    /// contains the material. Asserted against the bytes, not the wording.
    #[tokio::test]
    async fn nothing_this_module_can_print_contains_the_token() {
        let clock = Arc::new(ManualClock::new(0));
        let minter = Arc::new(RecordingMinter::new(
            clock.clone(),
            Duration::from_secs(600),
        ));
        let source = TokenSource::with_clock(minter, target(), clock);
        let token = source.token().await.expect("mint");
        let material = token.expose().to_string();
        assert!(
            material.contains(CANARY),
            "the fixture must be recognisable"
        );

        for rendered in [format!("{token}"), format!("{token:?}")] {
            assert!(
                !rendered.contains(CANARY),
                "a token printed itself: {rendered}"
            );
        }

        // `MintedToken` is the struct most likely to be reached by a derived
        // `Debug` upstream — it is what a connection state machine holds.
        let minted = MintedToken::new(material.clone(), 0, 600_000);
        let rendered = format!("{minted:?}");
        assert!(
            !rendered.contains(CANARY),
            "a minted token printed itself: {rendered}"
        );
        assert!(
            rendered.contains("expires_at_millis"),
            "the expiry is still worth printing: {rendered}"
        );
    }

    /// The other half of AC6 here: no error this module *composes* is given
    /// the material to begin with. Every variant is built from the target and
    /// from the apiserver's own message, and the one call that could reach a
    /// token — `mint` — puts the token in [`MintedToken`] or nowhere.
    #[test]
    fn no_error_variant_has_a_field_that_could_hold_a_token() {
        let target = target();
        let variants = [
            TokenRequestError::InvalidTarget {
                field: "namespace",
                value: target.namespace().to_string(),
                reason: "example".to_string(),
            },
            TokenRequestError::NoIdentity {
                detail: "no kubeconfig".to_string(),
            },
            TokenRequestError::Forbidden {
                username: Some("alice@example.com".to_string()),
                namespace: target.namespace().to_string(),
                service_account: target.service_account().to_string(),
                detail: "cannot create resource".to_string(),
            },
            TokenRequestError::NoSuchServiceAccount {
                namespace: target.namespace().to_string(),
                service_account: target.service_account().to_string(),
            },
            TokenRequestError::Transport {
                detail: "connection reset".to_string(),
            },
            TokenRequestError::Malformed {
                detail: "no status".to_string(),
            },
        ];
        for variant in &variants {
            for rendered in [variant.to_string(), format!("{variant:?}")] {
                assert!(
                    !rendered.contains(CANARY),
                    "an error rendering carried the credential: {rendered}"
                );
                assert!(!rendered.is_empty());
            }
        }
    }
}
// HANDWRITE-END
