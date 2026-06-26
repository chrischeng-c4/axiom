//! The generic auth middleware + a `Bearer` token extraction helper.
//!
//! Attach with
//! [`from_fn_with_state`](axum::middleware::from_fn_with_state):
//!
//! ```ignore
//! use axum::middleware::from_fn_with_state;
//! use service_auth::auth_middleware;
//! use std::sync::Arc;
//!
//! let verifier = Arc::new(MyVerifier::from_env()?);
//! let app = router.layer(from_fn_with_state(verifier, auth_middleware::<MyVerifier>));
//! ```
//!
//! On success the concrete `V::Principal` is inserted into the request
//! extensions; handlers read it with `Extension<MyVerifier::Principal>`
//! (or, more usefully, `Extension<MyPrincipal>` — the concrete type). On
//! failure the [`AuthError`](crate::AuthError) is rendered to a response.

use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{header, HeaderMap},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::verifier::Verifier;

/// Extract a `Bearer` token from the `Authorization` header.
///
/// Returns the token with the `Bearer ` prefix stripped, or `None` when the
/// header is missing, non-ASCII, or not a bearer credential.
pub fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

/// Generic auth middleware: extract -> verify -> reject-or-inject.
///
/// Calls [`Verifier::authenticate`] with the request headers. On `Ok`, inserts
/// the concrete `V::Principal` into the request extensions and runs the rest of
/// the stack. On `Err`, short-circuits with the rendered [`AuthError`]. The
/// verifier decides open-mode (via [`Verifier::required`] + returning its open
/// principal); this function is pure plumbing.
pub async fn auth_middleware<V: Verifier>(
    State(verifier): State<Arc<V>>,
    mut req: Request,
    next: Next,
) -> Response {
    match verifier.authenticate(req.headers()) {
        Ok(principal) => {
            req.extensions_mut().insert(principal);
            next.run(req).await
        }
        Err(e) => e.into_response(),
    }
}
