// HANDWRITE-BEGIN gap="missing-generator:logic:k8s-delegated-auth-root" tracker="#2869" reason="Module composition and the feature boundary between the pure decision logic and the kube transport."
//! Delegating authentication and authorization to kube-apiserver.
//!
//! A service that runs in Kubernetes and is called by workloads in Kubernetes
//! does not need an identity system. Its callers already have one: a
//! short-lived ServiceAccount token. A service explicitly selects either its
//! own audience or Kubernetes' default audiences. This module turns that into
//! a complete request-auth story with no local credential store —
//! `TokenReview` says who the caller is, `SubjectAccessReview` says what they
//! may do, and `RoleBinding`s in the cluster are the only place policy lives.
//!
//! ```text
//!   Authorization: Bearer <projected KSA token>
//!            |
//!            v
//!   DelegatedAuthenticator::authenticate  -- TokenReview -->  apiserver
//!            |                                 (selected profile checked, then
//!            |                                  system:serviceaccount:<ns>:<name>
//!            v                                  strictly parsed)
//!   ServiceAccountPrincipal
//!            |
//!            v
//!   DelegatedAuthenticator::authorize  -- SubjectAccessReview -->  apiserver
//! ```
//!
//! ## The module boundary
//!
//! - [`principal`] decides what a reviewed identity is *allowed to be*.
//! - [`projected`] is the *calling* side of the same story: reading the
//!   audience-bound token this workload was given, per request, so rotation
//!   works and a token minted for someone else never leaves the pod.
//! - [`token_request`] is the calling side for everyone who was not given one:
//!   a client with a kubeconfig asks the apiserver to mint an audience-bound
//!   token for a named ServiceAccount, and forwards that instead of its own
//!   credential.
//! - [`loopback_proxy`] is how such a client lends that token to a program it
//!   wraps without handing the program the token.
//! - [`review`] is the transport seam and its value types — no I/O, just the
//!   trait every backend implements.
//! - [`cache`] holds the TTL and stale-window policy, which is where the
//!   revocation bound is defined.
//! - [`delegated`] composes those into the authenticate/authorize state
//!   machine, and is where the fail-closed guarantees live.
//! - [`kube_backend`] is the only part that opens a socket, and the only part
//!   behind the `k8s` feature. Everything above it is testable without a
//!   cluster, which is why the decision logic has exhaustive tests and this
//!   file's transport has none that pretend to be one.
//!
//! Nothing here names any service's resources. A caller maps its own
//! operations onto [`review::ResourceAttributes`] and keeps that mapping in its
//! own crate, where its own tests can own it.

pub mod cache;
pub mod delegated;
pub mod loopback_proxy;
pub mod principal;
pub mod projected;
pub mod review;
pub mod token_request;

#[cfg(feature = "k8s")]
pub mod kube_backend;

pub use cache::{CacheOutcome, CachePolicy, Clock, ManualClock, SystemClock, TtlCache};
pub use delegated::{
    fingerprint, AuthRejection, DelegatedAuthConfig, DelegatedAuthError, DelegatedAuthMetrics,
    DelegatedAuthenticator, MissingAudience,
};
pub use loopback_proxy::{verifying_client, LoopbackProxy};
pub use principal::{
    PrincipalRejection, ReviewedIdentity, ServiceAccountPrincipal, ServiceAccountRef,
    SERVICE_ACCOUNT_PREFIX,
};
pub use projected::{ProjectedToken, ProjectedTokenError, ProjectedTokenFile};
pub use review::{
    AccessReviewOutcome, ExtraFields, ResourceAttributes, ReviewBackend, ReviewError,
    TokenReviewOutcome,
};
pub use token_request::{
    MintedToken, TokenMinter, TokenRequestError, TokenRequestTarget, TokenSource,
    DEFAULT_EXPIRATION_SECONDS, MIN_EXPIRATION_SECONDS,
};

#[cfg(feature = "k8s")]
pub use kube_backend::KubeReviewBackend;
#[cfg(feature = "k8s")]
pub use token_request::KubeTokenMinter;
// HANDWRITE-END
