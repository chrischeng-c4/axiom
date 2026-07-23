// SPEC-MANAGED: libs/service-http/tech-design/semantic/source/libs-service-http-src-transport-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! HTTP transport: the h2c serve loop + the standard request-tracing layer.
//! @spec apps/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#logic
//!
//! [`serve`] composes [`server_http::serve_h2c`] (HTTP/1.1 + HTTP/2 cleartext on one port —
//! the in-cluster default `axum::serve` can't do) rather than re-implementing
//! the accept loop. [`trace_layer`] is the one INFO-level span-per-request layer
//! lumen/keep both attach; a service `.layer(...)`s it onto its router.

use tokio::net::TcpListener;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{DefaultMakeSpan, MakeSpan, TraceLayer};

/// Request span maker that preserves standard request fields and, in an
/// OTLP-enabled build, attaches a valid propagated W3C parent context.
#[derive(Debug, Clone, Copy)]
/// @spec libs/service-http/tech-design/semantic/source/libs-service-http-src-transport-rs.md#source
pub struct PropagatingMakeSpan;

impl<B> MakeSpan<B> for PropagatingMakeSpan {
    fn make_span(&mut self, request: &axum::http::Request<B>) -> tracing::Span {
        let mut default = DefaultMakeSpan::new().level(tracing::Level::INFO);
        let span = default.make_span(request);
        #[cfg(feature = "otlp")]
        {
            use opentelemetry::trace::TraceContextExt as _;
            use tracing_opentelemetry::OpenTelemetrySpanExt as _;
            let parent = service_observability::extract_trace_context(request.headers());
            if parent.span().span_context().is_valid() {
                span.set_parent(parent);
            }
        }
        span
    }
}

/// Serve `app` (HTTP/1.1 + h2c on one port) on `listener`, stopping when
/// `shutdown` resolves (e.g. [`crate::signal::shutdown_with_drain`]).
///
/// Thin delegation to [`server_http::serve_h2c`] — the shared HTTP runtime — so
/// a service does not hand-roll the hyper-util auto-builder accept loop.
/// In-flight connections
/// get a bounded grace period after `shutdown` resolves before the process
/// exits.
/// @spec libs/service-http/tech-design/semantic/source/libs-service-http-src-transport-rs.md#source
pub async fn serve(
    listener: TcpListener,
    app: axum::Router,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) {
    server_http::serve_h2c(listener, app, shutdown).await;
}

// <HANDWRITE gap="missing-generator:logic" tracker="1870" reason="logic section in transport.rs is hand-written pending codegen support">
#[derive(Clone, Debug, PartialEq, Eq)]
/// Canonical correlation fields for one inbound HTTP request.
///
/// The context is always available, including in logging-only builds. A valid
/// W3C version 00 `traceparent` preserves its trace and parent span ids; absent
/// or invalid input creates a new local root context.
pub struct RequestTraceContext {
    trace_id: String,
    span_id: String,
    parent_span_id: Option<String>,
    trace_flags: String,
}

impl RequestTraceContext {
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    pub fn span_id(&self) -> &str {
        &self.span_id
    }

    pub fn parent_span_id(&self) -> Option<&str> {
        self.parent_span_id.as_deref()
    }

    pub fn trace_flags(&self) -> &str {
        &self.trace_flags
    }
}

#[derive(Debug)]
struct ParsedTraceParent {
    trace_id: String,
    parent_span_id: String,
    trace_flags: String,
}

/// Parse the inbound W3C context and return a usable request correlation
/// context. Invalid input is deliberately equivalent to no input: business
/// routing continues on a fresh local root trace.
pub fn request_trace_context(headers: &axum::http::HeaderMap) -> RequestTraceContext {
    let parsed = parse_traceparent(headers);
    RequestTraceContext {
        trace_id: parsed
            .as_ref()
            .map(|parent| parent.trace_id.clone())
            .unwrap_or_else(|| fresh_hex_id(16)),
        span_id: fresh_hex_id(8),
        parent_span_id: parsed.as_ref().map(|parent| parent.parent_span_id.clone()),
        trace_flags: parsed
            .as_ref()
            .map(|parent| parent.trace_flags.clone())
            .unwrap_or_else(|| "00".to_string()),
    }
}

fn parse_traceparent(headers: &axum::http::HeaderMap) -> Option<ParsedTraceParent> {
    let mut values = headers.get_all("traceparent").iter();
    let value = values.next()?.to_str().ok()?;
    if values.next().is_some() || !value.is_ascii() || value.len() != 55 {
        return None;
    }
    let bytes = value.as_bytes();
    if bytes[2] != b'-' || bytes[35] != b'-' || bytes[52] != b'-' {
        return None;
    }
    let version = &value[0..2];
    let trace_id = &value[3..35];
    let parent_span_id = &value[36..52];
    let trace_flags = &value[53..55];
    if version != "00"
        || !lower_hex(trace_id)
        || !lower_hex(parent_span_id)
        || !lower_hex(trace_flags)
        || all_zero(trace_id)
        || all_zero(parent_span_id)
    {
        return None;
    }
    Some(ParsedTraceParent {
        trace_id: trace_id.to_string(),
        parent_span_id: parent_span_id.to_string(),
        trace_flags: trace_flags.to_string(),
    })
}

fn lower_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn all_zero(value: &str) -> bool {
    value.bytes().all(|byte| byte == b'0')
}

fn fresh_hex_id(byte_len: usize) -> String {
    use sha2::{Digest as _, Sha256};
    use std::fmt::Write as _;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    loop {
        let sequence = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let mut hasher = Sha256::new();
        hasher.update(timestamp.to_le_bytes());
        hasher.update(std::process::id().to_le_bytes());
        hasher.update(sequence.to_le_bytes());
        let digest = hasher.finalize();
        let mut value = String::with_capacity(byte_len * 2);
        for byte in &digest[..byte_len] {
            write!(&mut value, "{byte:02x}").expect("write to String");
        }
        if !all_zero(&value) {
            return value;
        }
    }
}

/// Request span maker that always records canonical correlation fields and,
/// when an OpenTelemetry layer is installed, attaches the same valid W3C
/// parent context to the exported span.
#[derive(Debug, Clone, Copy)]
pub struct CorrelatingMakeSpan;

impl<B> MakeSpan<B> for CorrelatingMakeSpan {
    fn make_span(&mut self, request: &axum::http::Request<B>) -> tracing::Span {
        let context = request_trace_context(request.headers());
        let span = tracing::span!(
            tracing::Level::INFO,
            "request",
            method = %request.method(),
            uri = %request.uri(),
            version = ?request.version(),
            trace_id = %context.trace_id(),
            span_id = %context.span_id(),
            parent_span_id = tracing::field::Empty,
            trace_flags = %context.trace_flags(),
        );
        if let Some(parent_span_id) = context.parent_span_id() {
            span.record("parent_span_id", tracing::field::display(parent_span_id));
        }
        #[cfg(feature = "otlp")]
        {
            use opentelemetry::trace::TraceContextExt as _;
            use tracing_opentelemetry::OpenTelemetrySpanExt as _;
            let parent = service_observability::extract_trace_context(request.headers());
            if parent.span().span_context().is_valid() {
                span.set_parent(parent);
                let exported = span.context();
                let exported = exported.span();
                let exported = exported.span_context();
                if exported.is_valid() {
                    span.record("trace_id", tracing::field::display(exported.trace_id()));
                    span.record("span_id", tracing::field::display(exported.span_id()));
                    span.record(
                        "trace_flags",
                        tracing::field::display(format_args!(
                            "{:02x}",
                            exported.trace_flags().to_u8()
                        )),
                    );
                }
            }
        }
        span
    }
}

// <HANDWRITE gap="missing-generator:logic" tracker="#2420" reason="logic section in transport.rs is hand-written pending codegen support">
/// Emits the collector-neutral terminal record for every HTTP response.
///
/// `tower_http::trace` calls this while the request span is entered, so the
/// JSON formatter inherits method, URI, and W3C fields from
/// [`CorrelatingMakeSpan`]. The event itself adds only the response facts that
/// are not known when the span is created.
#[derive(Debug, Clone, Copy)]
struct RequestCompletionEvent;

impl<B> tower_http::trace::OnResponse<B> for RequestCompletionEvent {
    fn on_response(
        self,
        response: &axum::http::Response<B>,
        latency: std::time::Duration,
        span: &tracing::Span,
    ) {
        tracing::info!(
            parent: span,
            event = "http_request_complete",
            status = response.status().as_u16(),
            latency_ms = latency.as_secs_f64() * 1_000.0,
            "HTTP request completed"
        );
    }
}

/// The standard request-tracing layer: one INFO-level span and one completion
/// event per HTTP request.
///
/// Attach it to the outer router so probe and data-plane routes have the same
/// W3C correlation and completion record. The layer takes no collector
/// configuration: services write standard JSONL and the collector owns routing.
///
/// @spec libs/service-http/tech-design/logic/emit-w3c-correlated-request-completion-events.md#logic
pub fn trace_layer() -> TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    CorrelatingMakeSpan,
    tower_http::trace::DefaultOnRequest,
    RequestCompletionEvent,
> {
    TraceLayer::new_for_http()
        .make_span_with(CorrelatingMakeSpan)
        .on_response(RequestCompletionEvent)
}
// </HANDWRITE>
// CODEGEN-END
