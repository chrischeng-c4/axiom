//! Bounded reverse-proxy runtime with a service-owned upstream policy.

use std::time::Duration;

use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    http::{header, HeaderMap, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use futures_util::StreamExt;
use url::Url;

use crate::ApiErr;

pub trait ReverseProxyPolicy: Clone + Send + Sync + 'static {
    fn select_upstream(
        &self,
        method: &Method,
        uri: &Uri,
    ) -> Result<Url, ReverseProxySelectionError>;

    fn max_body_bytes(&self) -> usize;

    fn upstream_timeout(&self) -> Duration {
        Duration::from_secs(60)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct ReverseProxySelectionError {
    status: StatusCode,
    kind: &'static str,
    message: String,
    retryable: bool,
}

impl ReverseProxySelectionError {
    pub fn new(
        status: StatusCode,
        kind: &'static str,
        message: impl Into<String>,
        retryable: bool,
    ) -> Self {
        Self {
            status,
            kind,
            message: message.into(),
            retryable,
        }
    }

    fn into_response(self) -> Response {
        ApiErr::new(self.status, self.kind, self.message)
            .with_retryable(self.retryable)
            .into_response()
    }
}

#[derive(Clone)]
struct ReverseProxyState<P> {
    policy: P,
    http: reqwest::Client,
}

pub fn reverse_proxy_router<P>(policy: P) -> anyhow::Result<Router>
where
    P: ReverseProxyPolicy,
{
    anyhow::ensure!(
        policy.max_body_bytes() > 0,
        "reverse proxy body limit must be greater than zero"
    );
    anyhow::ensure!(
        !policy.upstream_timeout().is_zero(),
        "reverse proxy timeout must be greater than zero"
    );
    let http = reqwest::Client::builder()
        .timeout(policy.upstream_timeout())
        .build()?;
    Ok(Router::new()
        .fallback(any(forward::<P>))
        .with_state(ReverseProxyState { policy, http }))
}

async fn forward<P>(State(state): State<ReverseProxyState<P>>, request: Request) -> Response
where
    P: ReverseProxyPolicy,
{
    let (parts, body) = request.into_parts();
    let mut url = match state.policy.select_upstream(&parts.method, &parts.uri) {
        Ok(url) => url,
        Err(error) => return error.into_response(),
    };
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return proxy_error(
            StatusCode::BAD_GATEWAY,
            "invalid_upstream",
            "the selected upstream must use http or https and include a host",
            false,
        );
    }
    url.set_path(parts.uri.path());
    url.set_query(parts.uri.query());
    url.set_fragment(None);

    let body = match to_bytes(body, state.policy.max_body_bytes()).await {
        Ok(body) => body,
        Err(error) => {
            return proxy_error(
                StatusCode::PAYLOAD_TOO_LARGE,
                "proxy_body_too_large",
                format!("request body exceeds the configured proxy limit: {error}"),
                false,
            );
        }
    };
    let mut headers = HeaderMap::new();
    for (name, value) in &parts.headers {
        if !is_hop_header(name) && name != header::HOST && name != header::CONTENT_LENGTH {
            headers.append(name, value.clone());
        }
    }

    let upstream = match state
        .http
        .request(parts.method, url.clone())
        .headers(headers)
        .body(body)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            tracing::warn!(%url, %error, "internal role request failed");
            return proxy_error(
                StatusCode::BAD_GATEWAY,
                "upstream_unavailable",
                "the internal service role is unavailable",
                true,
            );
        }
    };
    let status = upstream.status();
    let upstream_headers = upstream.headers().clone();
    let mut stream = upstream.bytes_stream();
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(error) => {
                tracing::warn!(%url, %error, "internal role response ended early");
                return proxy_error(
                    StatusCode::BAD_GATEWAY,
                    "upstream_response_failed",
                    "the internal service role returned an incomplete response",
                    true,
                );
            }
        };
        if bytes.len().saturating_add(chunk.len()) > state.policy.max_body_bytes() {
            return proxy_error(
                StatusCode::BAD_GATEWAY,
                "upstream_response_too_large",
                "the internal service role response exceeds the proxy limit",
                true,
            );
        }
        bytes.extend_from_slice(&chunk);
    }

    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = status;
    for (name, value) in &upstream_headers {
        if !is_hop_header(name) && name != header::CONTENT_LENGTH {
            response.headers_mut().append(name, value.clone());
        }
    }
    response
}

fn proxy_error(
    status: StatusCode,
    kind: &'static str,
    message: impl Into<String>,
    retryable: bool,
) -> Response {
    ApiErr::new(status, kind, message)
        .with_retryable(retryable)
        .into_response()
}

fn is_hop_header(name: &axum::http::HeaderName) -> bool {
    matches!(
        name.as_str(),
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
    )
}
