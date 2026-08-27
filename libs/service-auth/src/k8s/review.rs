// HANDWRITE-BEGIN gap="missing-generator:logic:k8s-review-backend" tracker="#2869" reason="The TokenReview/SubjectAccessReview transport seam and its request/response value types."
//! The seam between "ask the apiserver" and "decide what the answer means".
//!
//! Everything above this trait is pure: parsing, caching, and policy. Every
//! network round trip is behind [`ReviewBackend`]. That split is what lets the
//! decision logic be tested exhaustively — audience mismatch, malformed
//! responses, outages, revocation windows — without a cluster, and it is what
//! keeps this library free of any opinion about *which* Kubernetes client a
//! service links.
//!
//! The value types are deliberately generic. `ResourceAttributes` is the
//! Kubernetes shape, not any service's: a service maps its own operations onto
//! an API group, a resource, and a verb, and this library never learns what
//! those strings mean.

use std::collections::BTreeMap;
use std::fmt;

use async_trait::async_trait;

use super::principal::ReviewedIdentity;

/// What `TokenReview` said, reduced to the fields a delegating service needs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TokenReviewOutcome {
    pub authenticated: bool,
    pub identity: ReviewedIdentity,
    /// The audiences the apiserver confirmed the token is valid for. Empty
    /// means it returned none, which never satisfies a requested audience.
    pub audiences: Vec<String>,
    /// `status.error`, when the apiserver explained a non-authentication.
    /// Never rendered to a caller — it is upstream diagnostic text.
    pub error: Option<String>,
}

/// The Kubernetes `authorization.k8s.io` resource attributes for one check.
///
/// `namespace` is the namespace the *decision* is made in — the serving
/// workload's own namespace for a service that publishes virtual resources,
/// not the caller's namespace. Getting that backwards would let any caller
/// authorize itself by creating a RoleBinding in a namespace it controls.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceAttributes {
    pub group: String,
    pub namespace: String,
    pub resource: String,
    /// The named object, when the check is about one. `None` is a
    /// collection-wide check and matches a rule with no `resourceNames`.
    pub name: Option<String>,
    pub verb: String,
}

impl ResourceAttributes {
    /// Build one check. `name` is `None` for a collection-wide check.
    pub fn new(
        group: impl Into<String>,
        namespace: impl Into<String>,
        resource: impl Into<String>,
        name: Option<String>,
        verb: impl Into<String>,
    ) -> Self {
        Self {
            group: group.into(),
            namespace: namespace.into(),
            resource: resource.into(),
            name,
            verb: verb.into(),
        }
    }
}

impl fmt::Display for ResourceAttributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}/{}", self.verb, self.group, self.resource)?;
        if let Some(name) = &self.name {
            write!(f, "/{name}")?;
        }
        write!(f, " in {}", self.namespace)
    }
}

/// The outcome of one `SubjectAccessReview`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessReviewOutcome {
    pub allowed: bool,
    /// Kubernetes' explicit deny, which outranks `allowed` when both are set.
    pub denied: bool,
    /// The authorizer's explanation, for logs. Not a caller-facing string.
    pub reason: Option<String>,
    /// `status.evaluationError` — the authorizer partially failed. Treated as
    /// a transport-class failure by the caller, never as an allow.
    pub evaluation_error: Option<String>,
}

impl AccessReviewOutcome {
    pub fn allow() -> Self {
        Self {
            allowed: true,
            denied: false,
            reason: None,
            evaluation_error: None,
        }
    }

    pub fn deny(reason: impl Into<String>) -> Self {
        Self {
            allowed: false,
            denied: true,
            reason: Some(reason.into()),
            evaluation_error: None,
        }
    }

    /// A single boolean, resolving the `allowed`/`denied` pair the way the
    /// Kubernetes contract does: an explicit deny wins.
    pub fn is_allowed(&self) -> bool {
        self.allowed && !self.denied
    }
}

/// A round trip that did not produce an answer.
///
/// This is not "the caller is unauthorized". It is "the question was not
/// answered", which is why it is a distinct type all the way up: collapsing it
/// into a deny loses the ability to serve a bounded stale decision, and
/// collapsing it into an allow is the failure mode this design exists to make
/// impossible.
#[derive(Debug, Clone)]
pub enum ReviewError {
    /// The request never reached a conclusion — connection, TLS, timeout, or a
    /// non-2xx status.
    Transport(String),
    /// A 2xx response whose body does not answer the question: no `status`, a
    /// `TokenReview` with neither authentication nor error, an authorizer
    /// `evaluationError`.
    Malformed(String),
    /// No client is configured, or the serving identity lacks the delegation
    /// grant. Distinct from `Transport` because it is a deployment error that
    /// will not resolve by retrying.
    NotDelegated(String),
}

impl ReviewError {
    /// A stable, credential-free token for logs and metrics.
    pub fn reason(&self) -> &'static str {
        match self {
            Self::Transport(_) => "transport",
            Self::Malformed(_) => "malformed_response",
            Self::NotDelegated(_) => "not_delegated",
        }
    }
}

impl fmt::Display for ReviewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(detail) => write!(f, "review transport failure: {detail}"),
            Self::Malformed(detail) => write!(f, "malformed review response: {detail}"),
            Self::NotDelegated(detail) => write!(f, "delegated review unavailable: {detail}"),
        }
    }
}

impl std::error::Error for ReviewError {}

/// The two apiserver calls a delegating service makes.
///
/// Implementors receive the bearer token by reference and must not retain,
/// log, or embed it in an error. Everything this trait returns is rendered
/// into logs somewhere.
#[async_trait]
pub trait ReviewBackend: Send + Sync + 'static {
    /// `POST /apis/authentication.k8s.io/v1/tokenreviews` with the given
    /// audiences. An empty slice means omit `spec.audiences` and use the
    /// apiserver's configured audiences. Callers may reach that form only
    /// through an explicit product profile such as
    /// [`super::DelegatedAuthConfig::kubernetes_default`]; ordinary
    /// [`super::DelegatedAuthConfig::new`] still rejects an empty audience.
    async fn review_token(
        &self,
        token: &str,
        audiences: &[String],
    ) -> Result<TokenReviewOutcome, ReviewError>;

    /// `POST /apis/authorization.k8s.io/v1/subjectaccessreviews` for the
    /// reviewed identity against one set of resource attributes.
    async fn review_access(
        &self,
        identity: &ReviewedIdentity,
        attributes: &ResourceAttributes,
    ) -> Result<AccessReviewOutcome, ReviewError>;
}

/// The `extra` map shape both reviews use on the wire.
pub type ExtraFields = BTreeMap<String, Vec<String>>;

#[cfg(test)]
mod tests {
    use super::*;

    fn attrs() -> ResourceAttributes {
        ResourceAttributes::new(
            "example.test",
            "serving",
            "widgets",
            Some("blue".into()),
            "get",
        )
    }

    #[test]
    fn attributes_render_as_a_readable_audit_line() {
        assert_eq!(
            attrs().to_string(),
            "get example.test/widgets/blue in serving"
        );
    }

    #[test]
    fn a_collection_wide_check_renders_without_a_name() {
        let mut collection = attrs();
        collection.name = None;
        assert_eq!(
            collection.to_string(),
            "get example.test/widgets in serving"
        );
    }

    /// The Kubernetes contract: an explicit deny outranks an allow, so a
    /// response carrying both is a deny.
    #[test]
    fn an_explicit_deny_outranks_a_simultaneous_allow() {
        let contradictory = AccessReviewOutcome {
            allowed: true,
            denied: true,
            reason: None,
            evaluation_error: None,
        };
        assert!(!contradictory.is_allowed());
        assert!(AccessReviewOutcome::allow().is_allowed());
        assert!(!AccessReviewOutcome::deny("no rule").is_allowed());
    }

    #[test]
    fn every_review_failure_has_a_stable_credential_free_reason() {
        assert_eq!(ReviewError::Transport("x".into()).reason(), "transport");
        assert_eq!(
            ReviewError::Malformed("x".into()).reason(),
            "malformed_response"
        );
        assert_eq!(
            ReviewError::NotDelegated("x".into()).reason(),
            "not_delegated"
        );
    }
}
// HANDWRITE-END
