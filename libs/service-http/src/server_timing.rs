// HANDWRITE-BEGIN gap="missing-generator:logic:e9769c9e" tracker="#2490" reason="Stateful axum middleware (per-request phase-append extension + response-driven disclosure gating) is hand-written pending codegen support for middleware with request/response extension state."
//! W3C `Server-Timing` on every response — per-request latency attribution
//! for integrators who have no access to Prometheus metrics or logs.
//!
//! [`server_timing_middleware`] adds one `Server-Timing` response header to
//! every request it wraps:
//!
//! - **Baseline, always present:** `app;dur=<ms>` — the wall-clock time from
//!   this middleware's `next.run` entry to the response leaving it. That is
//!   the same request/response boundary [`crate::transport::trace_layer`]
//!   spans; compose both onto the **same, outermost** layer position (see
//!   the crate root's "What a service wires" example) so `app;dur=` and the
//!   trace span's recorded latency describe the same measurement window
//!   instead of two independently-drifting notions of "how long did this
//!   take". tower-http's `OnResponse::on_response` hook only gets an
//!   immutable `&Response` alongside the latency it measured — there is no
//!   hook to write a header from a value it already computed, so this
//!   middleware cannot literally reuse `TraceLayer`'s internal timer without
//!   forking tower-http; composing at the same outer boundary is the closest
//!   a header-writing layer can get to "don't re-time" without doing that.
//! - **Phase entries, opt-in per response:** handlers push named durations
//!   onto the [`ServerTimingExt`] extension this middleware inserts into
//!   every request (`ext.push("search", elapsed)`), and they render after
//!   `app;dur=` — but only on responses that carry
//!   [`ServerTimingDisclosure::Full`] (see below).
//!
//! ## Disclosure posture — decided once, here
//!
//! The issue that motivated this module asks for total-only breakdown on
//! unauthenticated requests and a full phase breakdown once a request
//! carries a successful auth context. `service-http` cannot make that
//! distinction today:
//!
//! - This crate does not depend on `service-auth`.
//! - `service-auth`'s `auth_middleware<V>` inserts the concrete,
//!   per-service `V::Principal` (e.g. lumen's `AuthContext`) into
//!   **request** extensions on success. There is no crate-neutral "this
//!   request authenticated" marker type — every adopter's principal type is
//!   different, and this crate would have to name one of them to look for
//!   it.
//! - Nothing publishes a **response**-side authentication signal either
//!   (the only place a middleware positioned outside the whole stack, the
//!   way `server_timing_middleware` is meant to be, can observe anything
//!   after the handler has run).
//!
//! Given that, the posture is decided conservatively, once: **every**
//! response defaults to [`ServerTimingDisclosure::TotalOnly`] — `app;dur=`
//! only, no phase entries — regardless of the request's auth state. This is
//! the safe default the originating issue calls out explicitly for the case
//! where "auth state isn't visible at this layer".
//!
//! The hook for later: any layer or handler nested inside
//! `server_timing_middleware` (so, anything that runs during `next.run` —
//! including a service's own auth middleware or the handler itself) may
//! flip one response to full disclosure by inserting
//! `ServerTimingDisclosure::Full` into that **response's** extensions
//! before it returns:
//!
//! ```ignore
//! use axum::response::IntoResponse;
//! use service_http::ServerTimingDisclosure;
//!
//! async fn handler() -> axum::response::Response {
//!     let mut response = "ok".into_response();
//!     // e.g. once request-side auth state is confirmed successful:
//!     response.extensions_mut().insert(ServerTimingDisclosure::Full);
//!     response
//! }
//! ```
//!
//! `server_timing_middleware` never inspects a principal type or
//! credentials itself — it only ever looks for this one marker on the
//! response it gets back. Wiring that marker from a real auth success (in
//! `service-auth` or a service's own auth layer) is deliberately left as
//! follow-up work, not part of this change.

use std::fmt::Write as _;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;

/// Per-request phase-append collector.
///
/// [`server_timing_middleware`] inserts one of these into every request's
/// extensions; handlers pull it out with `Extension<ServerTimingExt>` and
/// call [`push`](ServerTimingExt::push) once per named sub-operation they
/// want visible in the response header. Entries render in push order, after
/// the `app;dur=` baseline, and only when the response ends up carrying
/// [`ServerTimingDisclosure::Full`].
#[derive(Clone, Default)]
pub struct ServerTimingExt(Arc<Mutex<Vec<(String, Duration)>>>);

impl ServerTimingExt {
    /// Append one named phase entry.
    ///
    /// `name` should be a short token (letters, digits, `_`, `-`, `.`); any
    /// other byte is replaced with `_` when the header renders (see
    /// [`sanitize_token`]) rather than dropping the entry or failing the
    /// request — a bad phase name degrades the header, it does not lose
    /// observability.
    pub fn push(&self, name: impl Into<String>, duration: Duration) {
        let mut phases = self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        phases.push((name.into(), duration));
    }

    fn drain(&self) -> Vec<(String, Duration)> {
        let mut phases = self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::mem::take(&mut *phases)
    }
}

/// Per-response disclosure decision for the phase breakdown.
///
/// Insert [`Full`](Self::Full) into a **response's** extensions (not the
/// request's — see the module docs) before it returns through
/// [`server_timing_middleware`] to opt that one response into the phase
/// breakdown. Any response without this marker — which today is every
/// response, since nothing in this crate sets it — stays
/// [`TotalOnly`](Self::TotalOnly).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ServerTimingDisclosure {
    /// `app;dur=` only. The safe default.
    #[default]
    TotalOnly,
    /// `app;dur=` plus every phase pushed onto this request's
    /// [`ServerTimingExt`], in push order.
    Full,
}

/// Add a `Server-Timing` response header to every request this middleware
/// wraps.
///
/// Always renders `app;dur=<total-ms>` — the time from this call's
/// `next.run` entry to the response coming back. Appends phase entries from
/// [`ServerTimingExt`] only when the response carries
/// [`ServerTimingDisclosure::Full`] (see the module docs for the posture
/// decision and how a service opts a response in later). A response header
/// value that somehow fails to parse as one (it never should — rendered
/// text is ASCII digits/`.`/`,`/`;`/sanitized tokens) is dropped rather than
/// panicking; the response still returns.
pub async fn server_timing_middleware(mut request: Request, next: Next) -> Response {
    let phases = ServerTimingExt::default();
    request.extensions_mut().insert(phases.clone());

    let start = Instant::now();
    let mut response = next.run(request).await;
    let total = start.elapsed();

    let disclosure = response
        .extensions()
        .get::<ServerTimingDisclosure>()
        .copied()
        .unwrap_or_default();

    let header_value = render_header(total, disclosure, &phases);
    if let Ok(value) = HeaderValue::from_str(&header_value) {
        response
            .headers_mut()
            .insert(server_timing_header_name(), value);
    }
    response
}

fn server_timing_header_name() -> HeaderName {
    HeaderName::from_static("server-timing")
}

fn render_header(
    total: Duration,
    disclosure: ServerTimingDisclosure,
    phases: &ServerTimingExt,
) -> String {
    let mut header = format!("app;dur={}", format_ms(total));
    if disclosure == ServerTimingDisclosure::Full {
        for (name, duration) in phases.drain() {
            // Writing to a `String` cannot fail.
            let _ = write!(
                header,
                ", {};dur={}",
                sanitize_token(&name),
                format_ms(duration)
            );
        }
    }
    header
}

/// Render a [`Duration`] as the millisecond `dur=` value W3C `Server-Timing`
/// expects: a bare (unit-less) number, three decimal places so sub-1ms
/// phases (admission checks, probe handlers) still show as nonzero.
fn format_ms(duration: Duration) -> String {
    format!("{:.3}", duration.as_secs_f64() * 1000.0)
}

/// Reduce `name` to a valid `Server-Timing` metric token: ASCII
/// alphanumerics, `_`, `-`, `.`. Every other byte becomes `_`; an
/// all-invalid or empty name becomes `phase` so the entry still renders
/// (with a visibly generic name) instead of silently disappearing.
fn sanitize_token(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out.push_str("phase");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_ms_renders_three_decimal_places() {
        assert_eq!(format_ms(Duration::ZERO), "0.000");
        assert_eq!(format_ms(Duration::from_millis(5)), "5.000");
        assert_eq!(format_ms(Duration::from_micros(1500)), "1.500");
    }

    #[test]
    fn sanitize_token_keeps_conforming_names() {
        assert_eq!(sanitize_token("search"), "search");
        assert_eq!(sanitize_token("raft_commit"), "raft_commit");
        assert_eq!(sanitize_token("db-lookup.v2"), "db-lookup.v2");
    }

    #[test]
    fn sanitize_token_replaces_disallowed_bytes_and_falls_back() {
        assert_eq!(sanitize_token("db query;name"), "db_query_name");
        assert_eq!(sanitize_token("a,b"), "a_b");
        assert_eq!(sanitize_token(""), "phase");
        assert_eq!(sanitize_token(";;;"), "___");
    }

    #[test]
    fn extension_push_and_drain_preserve_order_and_empty_after() {
        let ext = ServerTimingExt::default();
        ext.push("search", Duration::from_millis(3));
        ext.push("rank", Duration::from_millis(1));
        let drained = ext.drain();
        assert_eq!(
            drained,
            vec![
                ("search".to_string(), Duration::from_millis(3)),
                ("rank".to_string(), Duration::from_millis(1)),
            ]
        );
        assert!(ext.drain().is_empty(), "drain empties the collector");
    }

    #[test]
    fn render_header_total_only_ignores_pending_phases() {
        let ext = ServerTimingExt::default();
        ext.push("search", Duration::from_millis(3));
        let header = render_header(
            Duration::from_millis(10),
            ServerTimingDisclosure::TotalOnly,
            &ext,
        );
        assert_eq!(header, "app;dur=10.000");
    }

    #[test]
    fn render_header_full_appends_phases_after_baseline() {
        let ext = ServerTimingExt::default();
        ext.push("search", Duration::from_millis(3));
        ext.push("rank", Duration::from_micros(500));
        let header = render_header(
            Duration::from_millis(10),
            ServerTimingDisclosure::Full,
            &ext,
        );
        assert_eq!(header, "app;dur=10.000, search;dur=3.000, rank;dur=0.500");
    }
}
// HANDWRITE-END
