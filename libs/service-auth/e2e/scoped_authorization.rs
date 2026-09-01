use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    body::Body,
    extract::Extension,
    http::{HeaderMap, Method, Request, StatusCode, Uri},
    middleware::from_fn_with_state,
    routing::get,
    Router,
};
use service_auth::{
    bearer_token, scoped_authorization_middleware, AuthError, ScopedAuthorization,
    ScopedAuthorizationOutcome,
};
use tower::ServiceExt;

struct TestAuthorization;

#[async_trait]
impl ScopedAuthorization for TestAuthorization {
    type Principal = String;

    async fn authorize_scope(
        &self,
        headers: &HeaderMap,
        _method: &Method,
        uri: &Uri,
    ) -> Result<ScopedAuthorizationOutcome<Self::Principal>, AuthError> {
        if uri.path() == "/transport" {
            return Ok(ScopedAuthorizationOutcome::Bypass);
        }
        let token = bearer_token(headers).ok_or(AuthError::Unauthenticated)?;
        if token != "good" {
            return Err(AuthError::Forbidden("scope denied".into()));
        }
        Ok(ScopedAuthorizationOutcome::Authorized("project-a".into()))
    }
}

#[tokio::test]
async fn shared_flow_bypasses_transport_or_injects_a_typed_principal() {
    let app = Router::new()
        .route(
            "/project",
            get(|Extension(project): Extension<String>| async move { project }),
        )
        .route("/transport", get(|| async { "transport" }))
        .layer(from_fn_with_state(
            Arc::new(TestAuthorization),
            scoped_authorization_middleware::<TestAuthorization>,
        ));

    let authorized = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/project")
                .header("authorization", "Bearer good")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(authorized.status(), StatusCode::OK);

    let forbidden = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/project")
                .header("authorization", "Bearer bad")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

    let bypass = app
        .oneshot(
            Request::builder()
                .uri("/transport")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(bypass.status(), StatusCode::OK);
}
