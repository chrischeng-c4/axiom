// SPEC-MANAGED: libs/service-http/tech-design/semantic/source/libs-service-http-src-error-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! The shared HTTP error envelope every k8s-native service in the
//! ecosystem renders for its error responses. lumen established this shape
//! first (a `StorageError` → status/kind classification over a
//! `{"error", "message"}` body); this module is the one place it lives so
//! `keep`/`relay`/`loom` converge on the same JSON instead of hand-rolling a
//! coincidentally-similar one. `libs/service-auth`'s own rejection
//! rendering predates this module and is a later convergence — untouched
//! here.
//!
//! [`ErrorEnvelope`] is the wire shape; [`ApiErr`] pairs a `StatusCode` with
//! a short machine-stable `kind` and a human `message`, and renders as
//! [`ErrorEnvelope`] JSON via `IntoResponse`. A service builds one per
//! domain-error classification arm ([`ApiErr::new`]) — this crate only owns
//! the generic envelope + builder, never the domain classification, which
//! stays in the service's own `From<DomainError>` impl.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// The `{error, message}` JSON body every ecosystem HTTP error response
/// renders. `error` is a short machine-stable classification
/// (`"not_found"`, `"bad_request"`, ...); `message` is the human-readable
/// detail.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec libs/service-http/tech-design/semantic/source/libs-service-http-src-error-rs.md#source
pub struct ErrorEnvelope {
    pub error: String,
    pub message: String,
}

/// Status-code + `kind` classification wrapper. Build one with
/// [`ApiErr::new`] from a domain-error match arm; `.into_response()` renders
/// it as [`ErrorEnvelope`] JSON paired with the status code.
/// @spec libs/service-http/tech-design/semantic/source/libs-service-http-src-error-rs.md#source
pub struct ApiErr {
    status: StatusCode,
    kind: &'static str,
    message: String,
}

/// @spec libs/service-http/tech-design/semantic/source/libs-service-http-src-error-rs.md#source
impl ApiErr {
    /// HTTP status + short machine-stable `kind` + human-readable `message`.
    pub fn new(status: StatusCode, kind: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            kind,
            message: message.into(),
        }
    }
}

/// @spec libs/service-http/tech-design/semantic/source/libs-service-http-src-error-rs.md#source
impl IntoResponse for ApiErr {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorEnvelope {
                error: self.kind.to_string(),
                message: self.message,
            }),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn renders_status_and_envelope_json() {
        let resp = ApiErr::new(
            StatusCode::NOT_FOUND,
            "not_found",
            "collection not found: u",
        )
        .into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["error"], "not_found");
        assert_eq!(body["message"], "collection not found: u");
    }

    #[test]
    fn envelope_serializes_error_before_message() {
        let env = ErrorEnvelope {
            error: "bad_request".into(),
            message: "oops".into(),
        };
        let json = serde_json::to_string(&env).unwrap();
        assert_eq!(json, r#"{"error":"bad_request","message":"oops"}"#);
    }

    #[test]
    fn envelope_round_trips_through_deserialize() {
        let raw = r#"{"error":"gone","message":"tombstoned"}"#;
        let env: ErrorEnvelope = serde_json::from_str(raw).unwrap();
        assert_eq!(env.error, "gone");
        assert_eq!(env.message, "tombstoned");
    }
}
// CODEGEN-END
