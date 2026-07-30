// HANDWRITE-BEGIN gap="missing-generator:logic:async-verifier" tracker="#2677" reason="Async authentication seam for verifiers that must reach an identity provider, plus the sync-to-async adapter and middleware."
//! The async counterpart to [`Verifier`], for credentials that cannot be
//! resolved from local state alone.
//!
//! [`Verifier::authenticate`] is synchronous, which is correct for every
//! verifier that answers from memory — a role-map lookup, an HMAC check. It
//! cannot express a verifier that must *ask someone else*: Google's token
//! introspection endpoint is a network round trip, and a signing key rotated
//! out from under a cached JWKS has to be refetched before the request can be
//! answered at all.
//!
//! Rather than make [`Verifier::authenticate`] async — which would break every
//! existing implementor for the benefit of one — this module adds a parallel
//! trait and leaves the synchronous one untouched.
//!
//! ## Choosing between them
//!
//! | | [`Verifier`] | [`AsyncVerifier`] |
//! |---|---|---|
//! | Answers from | memory | memory **or** an identity provider |
//! | Middleware | [`auth_middleware`](crate::auth_middleware) | [`async_auth_middleware`] |
//! | Cost of a miss | none | one upstream call |
//!
//! A synchronous verifier reaches the async middleware through [`AsAsync`].
//! There is deliberately no blanket `impl<V: Verifier> AsyncVerifier for V`:
//! it would collide with the direct implementations this trait exists to
//! carry, because a blanket impl over a local trait forecloses every explicit
//! one.

use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::error::AuthError;
use crate::verifier::Verifier;

/// Authenticate a request whose credential may require an upstream call.
///
/// Implement this instead of [`Verifier`] when resolving a credential can
/// involve I/O. Implementors are responsible for keeping the common path off
/// the network — see [`crate::gcp::GoogleVerifier`], which serves cached JWKS
/// keys and cached introspection results without leaving the process.
#[async_trait]
pub trait AsyncVerifier: Send + Sync + 'static {
    /// The service's own principal type, injected into request extensions on
    /// success and read by handlers via `axum::extract::Extension`.
    type Principal: Clone + Send + Sync + 'static;

    /// Authenticate from request headers, awaiting an upstream lookup if the
    /// credential requires one.
    async fn authenticate_async(&self, headers: &HeaderMap) -> Result<Self::Principal, AuthError>;

    /// Whether a credential is required (controls open-mode). Default `true`.
    fn required(&self) -> bool {
        true
    }
}

/// Adapts a synchronous [`Verifier`] to [`AsyncVerifier`] so one router can
/// mix both kinds behind [`async_auth_middleware`].
///
/// The wrapper exists because coherence forbids a blanket impl; it adds no
/// behaviour and no await point.
#[derive(Debug, Clone, Copy, Default)]
pub struct AsAsync<V>(pub V);

#[async_trait]
impl<V: Verifier> AsyncVerifier for AsAsync<V> {
    type Principal = V::Principal;

    async fn authenticate_async(&self, headers: &HeaderMap) -> Result<Self::Principal, AuthError> {
        self.0.authenticate(headers)
    }

    fn required(&self) -> bool {
        Verifier::required(&self.0)
    }
}

/// Generic async auth middleware: extract -> verify -> reject-or-inject.
///
/// The async twin of [`auth_middleware`](crate::auth_middleware), identical in
/// every respect except that it awaits the verifier.
pub async fn async_auth_middleware<V: AsyncVerifier>(
    State(verifier): State<Arc<V>>,
    mut req: Request,
    next: Next,
) -> Response {
    match verifier.authenticate_async(req.headers()).await {
        Ok(principal) => {
            req.extensions_mut().insert(principal);
            next.run(req).await
        }
        Err(e) => e.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::role_map::{Role, RoleMapPrincipal, StaticRoleMapVerifier, TokenClaims};
    use std::collections::HashMap;

    fn claims() -> TokenClaims {
        TokenClaims {
            subject: "tester".into(),
            roles: HashMap::from([("products".to_string(), Role::Read)]),
        }
    }

    #[tokio::test]
    async fn as_async_delegates_to_the_wrapped_sync_verifier() {
        let inner =
            StaticRoleMapVerifier::new(true, HashMap::from([("abc".to_string(), claims())]));
        let verifier = AsAsync(inner);

        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer abc".parse().unwrap(),
        );
        let principal = verifier.authenticate_async(&headers).await.unwrap();
        assert_eq!(principal.subject(), Some("tester"));
        assert!(matches!(principal, RoleMapPrincipal::Token(_)));
    }

    #[tokio::test]
    async fn as_async_propagates_rejection_and_required_flag() {
        let verifier = AsAsync(StaticRoleMapVerifier::new(true, HashMap::new()));
        assert!(AsyncVerifier::required(&verifier));
        let err = verifier
            .authenticate_async(&HeaderMap::new())
            .await
            .unwrap_err();
        assert!(matches!(err, AuthError::Unauthenticated));
    }

    #[tokio::test]
    async fn as_async_preserves_open_mode() {
        let verifier = AsAsync(StaticRoleMapVerifier::open());
        assert!(!AsyncVerifier::required(&verifier));
        let principal = verifier
            .authenticate_async(&HeaderMap::new())
            .await
            .unwrap();
        assert!(matches!(principal, RoleMapPrincipal::Open));
    }
}
// HANDWRITE-END
