//! Shared request-auth middleware for the ecosystem's HTTP services.
//!
//! This is the service-kit auth layer: the generic **extract -> verify ->
//! reject -> inject** plumbing plus a [`Verifier`] trait each service
//! implements. It owns the transport-level shape of authentication, not the
//! crypto and not per-resource authorization:
//!
//! - **Token crypto is elsewhere.** keep and loom share scoped claim-check
//!   HMAC tokens via `libs/claimtoken`; a service's [`Verifier`] *composes*
//!   that (its `authenticate` calls `claimtoken::verify`). lumen's verifier
//!   instead uses a static role-map. This lib depends on neither — it only
//!   defines the trait those impls satisfy.
//! - **Authorization stays in handlers.** Per-resource policy (lumen's
//!   per-collection RBAC, keep's scope-vs-key) runs in the service's handlers
//!   on the concrete [`Verifier::Principal`], not here. The middleware only
//!   answers "who is this caller?" and injects that principal; "may they touch
//!   *this* resource?" is the handler's call.
//!
//! It layers onto a router built with `libs/service-http`'s data-plane routes:
//! attach [`auth_middleware`] with
//! [`from_fn_with_state`](axum::middleware::from_fn_with_state), passing the
//! service's `Arc<V>` verifier as state.
//!
//! ## Shape
//!
//! 1. A service defines a concrete principal type and a [`Verifier`] whose
//!    `authenticate` turns headers into that principal (or an [`AuthError`]).
//! 2. [`auth_middleware`] runs the verifier, injects the principal into request
//!    extensions on success, and renders the [`AuthError`] on failure.
//! 3. Handlers read the principal concretely via
//!    `axum::extract::Extension<MyPrincipal>` — no `Any`, no downcast.
//!
//! ```ignore
//! use axum::{extract::Extension, http::HeaderMap, middleware::from_fn_with_state};
//! use service_auth::{auth_middleware, bearer_token, AuthError, Verifier};
//! use std::sync::Arc;
//!
//! #[derive(Clone)]
//! struct Principal { subject: String }
//!
//! struct MyVerifier { /* secret / role-map / ... */ }
//!
//! impl Verifier for MyVerifier {
//!     type Principal = Principal;
//!     fn authenticate(&self, headers: &HeaderMap) -> Result<Principal, AuthError> {
//!         let token = bearer_token(headers).ok_or(AuthError::Unauthenticated)?;
//!         // e.g. claimtoken::verify(secret, token, now) — crypto lives there.
//!         Ok(Principal { subject: token.to_string() })
//!     }
//! }
//!
//! async fn handler(Extension(p): Extension<Principal>) -> String { p.subject }
//!
//! let verifier = Arc::new(MyVerifier { /* ... */ });
//! let app = axum::Router::new()
//!     .route("/things", axum::routing::get(handler))
//!     .layer(from_fn_with_state(verifier, auth_middleware::<MyVerifier>));
//! ```

mod error;
mod middleware;
mod verifier;

pub use error::AuthError;
pub use middleware::{auth_middleware, bearer_token};
pub use verifier::Verifier;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body,
        extract::Extension,
        http::{header, HeaderMap, Request, StatusCode},
        middleware::from_fn_with_state,
        routing::get,
        Router,
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;

    // ---- A tiny test verifier -------------------------------------------

    /// The service's concrete principal. With a real service this would carry
    /// the subject/roles/scope; the unit variant keeps the test minimal.
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum TestPrincipal {
        Authed(String),
        Open,
    }

    /// Fixed-token verifier: only `Bearer good` authenticates. `required`
    /// controls open-mode; when `false`, a request without a token resolves to
    /// `TestPrincipal::Open`.
    struct FixedToken {
        required: bool,
    }

    impl Verifier for FixedToken {
        type Principal = TestPrincipal;

        fn authenticate(&self, headers: &HeaderMap) -> Result<TestPrincipal, AuthError> {
            match bearer_token(headers) {
                Some("good") => Ok(TestPrincipal::Authed("subject".into())),
                Some(_) => Err(AuthError::Unauthenticated),
                None if !self.required => Ok(TestPrincipal::Open),
                None => Err(AuthError::Unauthenticated),
            }
        }

        fn required(&self) -> bool {
            self.required
        }
    }

    /// A verifier that always 403s — exercises the Forbidden arm.
    struct AlwaysForbidden;

    impl Verifier for AlwaysForbidden {
        type Principal = TestPrincipal;

        fn authenticate(&self, _headers: &HeaderMap) -> Result<TestPrincipal, AuthError> {
            Err(AuthError::Forbidden("nope".into()))
        }
    }

    /// Probe handler: reflects the injected principal so tests can assert it
    /// reached the handler concretely (no downcast).
    async fn probe(Extension(p): Extension<TestPrincipal>) -> String {
        match p {
            TestPrincipal::Authed(s) => format!("authed:{s}"),
            TestPrincipal::Open => "open".into(),
        }
    }

    fn app<V: Verifier<Principal = TestPrincipal>>(verifier: V) -> Router {
        Router::new()
            .route("/", get(probe))
            .layer(from_fn_with_state(Arc::new(verifier), auth_middleware::<V>))
    }

    async fn call(app: Router, auth: Option<&str>) -> (StatusCode, String) {
        let mut builder = Request::builder().uri("/");
        if let Some(a) = auth {
            builder = builder.header(header::AUTHORIZATION, a);
        }
        let resp = app
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = resp.status();
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(body.to_vec()).unwrap())
    }

    // ---- bearer_token unit test -----------------------------------------

    #[test]
    fn bearer_token_extracts_and_rejects() {
        let mut h = HeaderMap::new();
        h.insert(header::AUTHORIZATION, "Bearer abc123".parse().unwrap());
        assert_eq!(bearer_token(&h), Some("abc123"));

        // Wrong scheme / no scheme / missing header all yield None.
        let mut basic = HeaderMap::new();
        basic.insert(header::AUTHORIZATION, "Basic abc123".parse().unwrap());
        assert_eq!(bearer_token(&basic), None);

        let mut raw = HeaderMap::new();
        raw.insert(header::AUTHORIZATION, "abc123".parse().unwrap());
        assert_eq!(bearer_token(&raw), None);

        assert_eq!(bearer_token(&HeaderMap::new()), None);
    }

    // ---- middleware integration tests (oneshot) -------------------------

    #[tokio::test]
    async fn valid_bearer_injects_principal_and_runs_handler() {
        let (status, body) = call(app(FixedToken { required: true }), Some("Bearer good")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "authed:subject");
    }

    #[tokio::test]
    async fn missing_token_when_required_is_401() {
        let (status, _body) = call(app(FixedToken { required: true }), None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn invalid_token_is_401() {
        let (status, _body) = call(app(FixedToken { required: true }), Some("Bearer bad")).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn open_mode_without_token_injects_open_principal() {
        let (status, body) = call(app(FixedToken { required: false }), None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "open");
    }

    #[tokio::test]
    async fn forbidden_is_403() {
        let (status, body) = call(app(AlwaysForbidden), Some("Bearer good")).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        // Error body uses the shared {"error","message"} shape.
        assert!(body.contains("\"error\":\"forbidden\""));
        assert!(body.contains("nope"));
    }
}
