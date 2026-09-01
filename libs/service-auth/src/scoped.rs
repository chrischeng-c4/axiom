//! Trait-driven scoped authorization middleware.

use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, Method, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::AuthError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScopedAuthorizationOutcome<P> {
    Bypass,
    Authorized(P),
}

#[async_trait]
pub trait ScopedAuthorization: Send + Sync + 'static {
    type Principal: Clone + Send + Sync + 'static;

    async fn authorize_scope(
        &self,
        headers: &HeaderMap,
        method: &Method,
        uri: &Uri,
    ) -> Result<ScopedAuthorizationOutcome<Self::Principal>, AuthError>;
}

pub async fn scoped_authorization_middleware<A: ScopedAuthorization>(
    State(authorizer): State<Arc<A>>,
    mut request: Request,
    next: Next,
) -> Response {
    match authorizer
        .authorize_scope(request.headers(), request.method(), request.uri())
        .await
    {
        Ok(ScopedAuthorizationOutcome::Bypass) => next.run(request).await,
        Ok(ScopedAuthorizationOutcome::Authorized(principal)) => {
            request.extensions_mut().insert(principal);
            next.run(request).await
        }
        Err(error) => error.into_response(),
    }
}
