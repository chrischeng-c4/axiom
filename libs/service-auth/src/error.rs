// CODEGEN-BEGIN
//! The generic auth rejection type — 401 / 403 / 503 with a small JSON body.
//!
//! Mirrors lumen's `auth.rs` error shape (`{"error": "...", "message": "..."}`)
//! so services that adopt this layer keep the same wire contract. Per-resource
//! authorization (RBAC, scope-vs-key) is a service concern; this lib only owns
//! the transport-level outcomes a [`Verifier`](crate::Verifier) — or an
//! [`AsyncVerifier`](crate::AsyncVerifier) — can produce.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Serialize;

/// JSON error body shared by every rejection: `{"error": "...", "message": "..."}`.
#[derive(Debug, Clone, Serialize)]
struct ErrorBody {
    error: String,
    message: String,
}

/// Why a request was rejected by the auth layer.
///
/// - [`AuthError::Unauthenticated`] → `401 Unauthorized` — no/invalid credential
///   when one was required.
/// - [`AuthError::Forbidden`] → `403 Forbidden` — a valid principal that lacks
///   the needed authorization. The string carries the human-readable reason; a
///   service decides per-resource policy and supplies the message.
/// - [`AuthError::Unavailable`] → `503 Service Unavailable` — the credential
///   could not be judged at all because the identity provider did not answer.
///   Distinct from `401` on purpose: a verifier that reaches an external
///   provider ([`crate::gcp::GoogleVerifier`]) would otherwise report its own
///   outage as "your credential is invalid", which sends every caller to
///   re-issue a credential that was fine.
#[derive(Debug, Clone)]
pub enum AuthError {
    Unauthenticated,
    Forbidden(String),
    Unavailable(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        match self {
            AuthError::Unauthenticated => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody {
                    error: "unauthenticated".into(),
                    message: "valid bearer token required".into(),
                }),
            )
                .into_response(),
            AuthError::Forbidden(message) => (
                StatusCode::FORBIDDEN,
                Json(ErrorBody {
                    error: "forbidden".into(),
                    message,
                }),
            )
                .into_response(),
            AuthError::Unavailable(message) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorBody {
                    error: "auth_unavailable".into(),
                    message,
                }),
            )
                .into_response(),
        }
    }
}
// CODEGEN-END
