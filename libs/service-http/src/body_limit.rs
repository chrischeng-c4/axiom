// HANDWRITE-BEGIN gap="missing-generator:logic:237df8cf" tracker="#2484" reason="Hand-rolled tower Layer/Service enforcing a streaming request-body byte cap with a structured 413 envelope; pending codegen support for custom Layer/Service state machines."
//! Shared request-body byte cap for a service's data plane, with the
//! crate's `{error, message}` envelope on rejection (#2484).
//!
//! `HttpConfig::body_limit_bytes` (see [`crate::config`]) has always been a
//! documented, service-supplied knob, but nothing in this crate enforced
//! it: every known adopter either hand-rolled its own
//! `axum::extract::DefaultBodyLimit` literal that never actually read the
//! config field (tape, lumen's admin route, keep's own separate CLI flag),
//! or shipped its data plane with no cap at all. [`body_limit_layer`] is
//! the one place that enforcement now lives, so a service wires the config
//! field it already has instead of inventing another local constant.
//!
//! ## What it enforces
//!
//! - A `Content-Length` header over `max_bytes` is rejected immediately —
//!   before the request body is read at all.
//! - A request without (or under) `Content-Length` — chunked/streamed
//!   bodies — is still bounded: the body is wrapped in
//!   [`http_body_util::Limited`], so a streamed body that grows past
//!   `max_bytes` is caught mid-read rather than buffered without bound.
//!   Handlers that extract the body through axum's `Bytes`/`String`/
//!   `Json`/`Form` extractors get the resulting `413` for free — axum-core
//!   recognizes the wrapped body's length-limit error during extraction. A
//!   handler that reads the body through some other path is responsible for
//!   surfacing that error itself; this layer only guarantees the byte
//!   count is bounded, not that every possible body-consumption path
//!   renders a response.
//! - Every `413` this layer's response carries — whether short-circuited on
//!   `Content-Length` or produced downstream once the wrapped body errors
//!   mid-read — is rendered as the crate's [`crate::ErrorEnvelope`] JSON
//!   shape (`{"error": "payload_too_large", "message": ...}`), mirroring
//!   [`crate::admission::admission_middleware`]'s `429` envelope convention
//!   instead of axum's own plain-text rejection body.
//!
//! ## How a service wires it
//!
//! One layer at router composition, over the data plane only — probes stay
//! unbounded, matching this crate's documented probe behavior
//! ([`crate::probes::standard_probe_routes`]):
//!
//! ```ignore
//! let data_plane = my_routes()
//!     .layer(service_http::body_limit_layer(cfg.body_limit_bytes));
//! let app = service_http::standard_probe_routes(readiness, None, openapi)
//!     .merge(data_plane);
//! ```
//!
//! A service that also composes admission control or auth on the data
//! plane can stack this layer with those the same way `.layer(...)` stacks
//! any other tower layer; there is no required ordering relative to them
//! (the byte cap and the request's class/identity are independent checks).
//!
//! ## Recommended default
//!
//! 8 MiB (`8 * 1024 * 1024`) — the value [`crate::config`]'s own tests use
//! for `HttpConfig::body_limit_bytes`, and the literal every known adopter
//! independently converged on before this shared layer existed. A service
//! with materially larger legitimate payloads should size `max_bytes`
//! explicitly rather than inherit this default.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::extract::Request;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use http_body_util::Limited;
use tower::{Layer, Service};

use crate::ApiErr;

/// `tower::Layer` that enforces a request-body byte cap. Build with
/// [`body_limit_layer`]; `.layer(...)` it directly onto a service's
/// data-plane router (see the module docs for placement).
#[derive(Debug, Clone, Copy)]
pub struct BodyLimitLayer {
    max_bytes: usize,
}

impl<S> Layer<S> for BodyLimitLayer {
    type Service = BodyLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        BodyLimitService {
            inner,
            max_bytes: self.max_bytes,
        }
    }
}

/// `tower::Service` [`BodyLimitLayer`] wraps around an inner router/service.
#[derive(Debug, Clone)]
pub struct BodyLimitService<S> {
    inner: S,
    max_bytes: usize,
}

impl<S> Service<Request> for BodyLimitService<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let max_bytes = self.max_bytes;
        if content_length_exceeds(&request, max_bytes) {
            return Box::pin(async move { Ok(oversized_response()) });
        }

        let (parts, body) = request.into_parts();
        let limited = Body::new(Limited::new(body, max_bytes));
        let request = Request::from_parts(parts, limited);

        // `Service::call` takes `&mut self`, so the future can't hold a
        // borrow of `self.inner` past this call — clone it in, the standard
        // pattern for a boxed-future middleware wrapping a `Clone` service
        // (axum's own `Route` is cheaply `Clone`).
        let mut inner = self.inner.clone();
        Box::pin(async move {
            let response = inner.call(request).await?;
            Ok(rewrite_if_oversized(response))
        })
    }
}

/// `true` when the request declares a `Content-Length` over `max_bytes` —
/// checked before the body is touched at all, so a known-oversized request
/// is rejected without reading a single byte of it. A missing or
/// unparseable header is not itself oversized here; the wrapped body in
/// [`BodyLimitService::call`] is the real enforcement for that case
/// (streamed/chunked bodies never carry a trustworthy `Content-Length`).
fn content_length_exceeds(request: &Request, max_bytes: usize) -> bool {
    request
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok())
        .is_some_and(|len| len > max_bytes)
}

/// Rewrite an inner response into the structured envelope only when it is
/// actually the `413` this layer's wrapped body produced — axum's
/// `Bytes`/`Json`/`String`/`Form` extractors render `413 PAYLOAD_TOO_LARGE`
/// with a plain-text body when [`http_body_util::Limited`]'s length-limit
/// error surfaces during extraction. Any other status passes through
/// untouched.
fn rewrite_if_oversized(response: Response) -> Response {
    if response.status() == StatusCode::PAYLOAD_TOO_LARGE {
        oversized_response()
    } else {
        response
    }
}

/// The structured `413` this layer always renders on rejection —
/// `{"error": "payload_too_large", "message": ...}`, mirroring
/// [`crate::admission::admission_middleware`]'s `429` envelope convention.
fn oversized_response() -> Response {
    ApiErr::new(
        StatusCode::PAYLOAD_TOO_LARGE,
        "payload_too_large",
        "request body exceeds the configured size limit",
    )
    .into_response()
}

/// Build the request-body byte-cap layer for `max_bytes`. `.layer(...)` it
/// onto a service's data-plane router only — see the module docs for
/// placement and the recommended default.
pub fn body_limit_layer(max_bytes: usize) -> BodyLimitLayer {
    BodyLimitLayer { max_bytes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_length_exceeds_is_strict_over_not_at_the_cap() {
        let under = Request::builder()
            .header(header::CONTENT_LENGTH, "10")
            .body(Body::empty())
            .unwrap();
        assert!(!content_length_exceeds(&under, 10));

        let over = Request::builder()
            .header(header::CONTENT_LENGTH, "11")
            .body(Body::empty())
            .unwrap();
        assert!(content_length_exceeds(&over, 10));
    }

    #[test]
    fn content_length_exceeds_is_false_when_header_absent_or_unparseable() {
        let absent = Request::builder().body(Body::empty()).unwrap();
        assert!(!content_length_exceeds(&absent, 10));

        let garbage = Request::builder()
            .header(header::CONTENT_LENGTH, "not-a-number")
            .body(Body::empty())
            .unwrap();
        assert!(!content_length_exceeds(&garbage, 10));
    }

    #[tokio::test]
    async fn oversized_response_renders_the_structured_413_envelope() {
        use http_body_util::BodyExt;

        let response = oversized_response();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["error"], "payload_too_large");
        assert!(body["message"].as_str().unwrap().contains("size limit"));
    }
}
// HANDWRITE-END
