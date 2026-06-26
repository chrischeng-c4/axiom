//! The [`Verifier`] trait — the one thing each service implements.
//!
//! A service's verifier turns request headers into the service's own concrete
//! principal type (or an [`AuthError`]). No `Any`/downcast: the principal type
//! is an associated type, so handlers receive it concretely via
//! `Extension<V::Principal>`. The generic middleware in [`crate::middleware`]
//! plumbs the result; the verifier owns the policy.

use crate::error::AuthError;

/// Authenticate a request from its headers into the service's principal.
///
/// Each HTTP service implements this once. The token crypto a verifier uses
/// (HMAC via `libs/claimtoken`, a role-map, k8s ServiceAccount JWT, OIDC, …) is
/// the implementor's choice and is **not** part of this lib.
///
/// Open / anonymous mode is expressed by the verifier itself: set
/// [`required`](Verifier::required) to `false` and return the service's "open"
/// principal value from [`authenticate`](Verifier::authenticate) when no
/// credential is presented. The middleware does not special-case it.
pub trait Verifier: Send + Sync + 'static {
    /// The service's own principal type, injected into request extensions on
    /// success and read by handlers via `axum::extract::Extension`.
    type Principal: Clone + Send + Sync + 'static;

    /// Authenticate from request headers. Return the principal (which MAY be the
    /// service's "open/anonymous" value when no token is needed), or an
    /// [`AuthError`].
    fn authenticate(&self, headers: &axum::http::HeaderMap) -> Result<Self::Principal, AuthError>;

    /// Whether a credential is required (controls open-mode). Default `true`.
    fn required(&self) -> bool {
        true
    }
}
