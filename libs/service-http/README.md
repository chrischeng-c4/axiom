# service-http

## Brief

`service-http` provides shared HTTP-service policy: standard probe routes,
request-context propagation, lifecycle readiness/signal adapters, and the JSON
error envelope. Protocol-neutral logging, optional OTLP export, metric-provider
semantics, and lifecycle counters belong to `service-observability`; listener
admission and drain state belong to `server-http` and `server-lifecycle`.

## Production lifecycle composition

Create one `LifecycleController`, pass it to `lifecycle_probe_routes` (or the
canonical JSON variant), and serve through `serve_with_lifecycle`. Use a
cloneable `LifecycleShutdownTrigger` with `run_signal_bridge` or
`shutdown_on_signal` so probes, admission, and the absolute deadline share one
generation. `standard_probe_routes`, `ReadinessHook`, `serve`, and
`shutdown_with_drain` are source-compatible legacy migration adapters only.

## Capabilities

A promise with no gate under it is not claimed.

Each section names the gate that verifies it. A capability with no gate line is
not claimed.

### Shared HTTP Service Scaffold

HTTP services reuse common operational routes and policy adapters instead of
hand-rolling them per service. This crate delegates listener serving to
`server-http` and re-exports readiness/shutdown from `server-lifecycle`.
Compatibility names for logging and metric providers delegate to
`service-observability`; what stays here is the HTTP adapter that extracts
valid W3C `traceparent`/`tracestate` headers into request spans.

- Root WI: #1640
- Gates: `cargo test -p service-http`;
  `cargo test -p service-http --features otlp --test otlp_tracing`
- Source: `libs/service-http/src/lib.rs`


### Standard Endpoints (Probes Surface)

`standard_probe_routes()` (`libs/service-http/src/probes.rs`) mounts the five
always-on, auth-exempt, body-limit-exempt routes every k8s-native service
ships on its one serve port:

- `GET /healthz` — liveness
- `GET /readyz` — readiness; 503 once a service flips its `ReadinessHook` into
  draining
- `GET /metrics` — Prometheus text format via `MetricsProvider`
- `GET /openapi.json` — the service's OpenAPI document
- `GET /docs` — Swagger UI

`standard_probe_routes_canonical_json` is the byte-identical-snapshot variant
for services that need one canonical serialization shared across a CLI twin, an
offline fixture, and the live route.


### Trace Context (Accept-or-Generate + Log Correlation + OTLP Upgrade)

`trace_layer()` (`libs/service-http/src/transport.rs`) is the shared
`tower-http` `TraceLayer` an adopting service composes instead of a
hand-rolled span/correlation layer:

- **Accept** — a valid W3C version-00 `traceparent` header is parsed and its
  `trace_id`/`parent_span_id` preserved; strictly invalid input (wrong version,
  wrong length, non-hex, all-zero ids) is treated as absent rather than
  rejected.
- **Generate** — when no `traceparent` arrives, `request_trace_context` mints a
  fresh local-root `trace_id` and `span_id`.
- **Log correlation** — `CorrelatingMakeSpan` records `trace_id`, `span_id`,
  and `parent_span_id` (when present) on every request span, and those fields
  flow into the structured stdout every service emits, so cross-service log
  correlation works with zero exporter configured.
- **OTLP upgrade** — the `otlp` feature re-exports the same span context to
  full OpenTelemetry export without changing the accept/generate contract.

This piece is inbound-only: it instruments the request a service receives. See
[Outbound Propagation](#outbound-propagation-not-yet-claimed) for the separate,
unimplemented service-to-service leg.


### Server-Timing Response Attribution

`server_timing_middleware` + `ServerTimingExt` + `ServerTimingDisclosure`
(`libs/service-http/src/server_timing.rs`) put a W3C
`Server-Timing: app;dur=<ms>` baseline on every response the middleware wraps,
measured at the same request/response boundary `trace_layer` spans. Handlers
may push named phase entries onto the per-request `ServerTimingExt` extension.

Those entries render only on responses a handler explicitly marks
`ServerTimingDisclosure::Full`. Every response defaults to `TotalOnly`, because
this crate cannot see a request's auth outcome — it does not depend on
`service-auth`, and no crate-neutral "authenticated" signal exists on either the
request or the response. `Full` is the documented hook a service's own auth
layer can use later to gate the phase breakdown on a successful auth context.

**It is a separate opt-in layer from `trace_layer()`**: a service must add
`.layer(axum::middleware::from_fn(server_timing_middleware))` explicitly to
receive it. Per-service adoption status belongs in each consuming service's own
README, not here.

- Root WI: #2490
- Gates: `cargo test -p service-http --test server_timing`
- Source: `libs/service-http/src/server_timing.rs`


### Admission Control

`AdmissionController` + `admission_middleware`
(`libs/service-http/src/admission.rs`) give a service opt-in, per-endpoint-class
token buckets keyed on a caller-owned opaque key. Retained state is a SHA-256
fingerprint only, never the raw key, and decision observers cannot represent a
raw key. Denials render the shared `ErrorEnvelope` with HTTP 429 and
`Retry-After`. An empty policy set is disabled — route classification and policy
values stay the adopting service's decision.

- Root WI: #1642
- Gates: `cargo test -p service-http --lib`
- Source: `libs/service-http/src/admission.rs`


### Error Envelope

`ErrorEnvelope { error, message }` (`libs/service-http/src/error.rs`) is the one
`{"error", "message"}` JSON body every ecosystem service renders for error
responses, paired with a `StatusCode` via `ApiErr`. Services classify their own
domain errors into it (`From<DomainError>`); this crate owns only the generic
envelope, builder, and `utoipa::ToSchema` shape.


### Outbound Propagation (Not Yet Claimed)

Surveyed 2026-07-24: this crate carries no outbound HTTP client at all — no
`reqwest`/`hyper`-client dependency, and no code under `libs/service-http/src/`
that reads or writes a `traceparent` header on a request this crate originates.
Only the inbound accept/generate leg above is implemented.

Do not claim outbound `traceparent` injection for this crate, or for any service
that composes it, until an outbound propagation seam actually ships.
