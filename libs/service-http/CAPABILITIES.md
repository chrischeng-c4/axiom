# Service Http Capabilities

<!-- aw:meta:project-capabilities:start -->
## Brief

Machine-readable capability contract for Service Http.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
<!-- aw:meta:project-capabilities:end -->

## Transport Contract

The machine-readable capability index with per-capability gates lives in
`libs/service-http/README.md` (this project's `cap_path`); this section is
the first-class, discoverable landing for the shared HTTP transport and
observability contract every adopting service inherits by composing this
crate, so the shape stays one place instead of scattered README prose.
Refs #2491.

### Standard Endpoints (Probes Surface)

`standard_probe_routes()` (`libs/service-http/src/probes.rs`) mounts the five
always-on, auth-exempt, body-limit-exempt routes every k8s-native service
ships on its one serve port: `GET /healthz` (liveness), `GET /readyz`
(readiness — 503 once a service flips its `ReadinessHook` into draining),
`GET /metrics` (Prometheus text format via `MetricsProvider`), `GET
/openapi.json` (the service's OpenAPI document), and `GET /docs` (Swagger
UI). `standard_probe_routes_canonical_json` is the byte-identical-snapshot
variant for services that need one canonical serialization shared across a
CLI twin, an offline fixture, and the live route.

### Trace Context (Accept-or-Generate + Log Correlation + OTLP Upgrade)

`trace_layer()` (`libs/service-http/src/transport.rs`) is the shared
`tower-http` `TraceLayer` an adopting service composes instead of a
hand-rolled span/correlation layer:

- **Accept**: a valid W3C version-00 `traceparent` header is parsed and its
  `trace_id`/`parent_span_id` preserved; strictly invalid input (wrong
  version, wrong length, non-hex, all-zero ids) is treated as absent rather
  than rejected.
- **Generate**: when no `traceparent` arrives, `request_trace_context` mints
  a fresh local-root `trace_id` and `span_id`.
- **Log correlation**: `CorrelatingMakeSpan` records `trace_id`, `span_id`,
  and `parent_span_id` (when present) on every request span, and those
  fields flow into the structured stdout every service emits — cross-service
  log correlation works with zero exporter configured.
- **OTLP upgrade**: the `otlp` feature re-exports the same span context to
  full OpenTelemetry export without changing the accept/generate contract.

This piece is inbound-only: it instruments the request a service receives.
See "Outbound Propagation" below for the separate, unimplemented
service-to-service leg.

### Server-Timing Response Attribution

`server_timing_middleware` + `ServerTimingExt` + `ServerTimingDisclosure`
(`libs/service-http/src/server_timing.rs`, #2490) put a W3C `Server-Timing:
app;dur=<ms>` baseline on every response the middleware wraps, measured at
the same request/response boundary `trace_layer` spans. Handlers may push
named phase entries onto the per-request `ServerTimingExt` extension; those
entries render only on responses a handler explicitly marks
`ServerTimingDisclosure::Full` — every response defaults to `TotalOnly`
because this crate has no crate-neutral "authenticated" signal to gate the
phase breakdown on.

Implemented and verified in this crate (`cargo test -p service-http --test
server_timing`); **it is a separate opt-in layer from `trace_layer()`** — a
service must explicitly add `.layer(axum::middleware::from_fn(
server_timing_middleware))` to receive it. Per-service adoption status is
recorded in each consuming service's own capability doc, not here.

### Admission Control

`AdmissionController` + `admission_middleware` (`libs/service-http/src/admission.rs`,
#1642) give a service opt-in, per-endpoint-class token buckets keyed on a
caller-owned opaque key. Retained state is a SHA-256 fingerprint only (never
the raw key); denials render the shared `ErrorEnvelope` with HTTP 429 and
`Retry-After`. An empty policy set is disabled — route classification and
policy values stay the adopting service's decision.

### Error Envelope

`ErrorEnvelope { error, message }` (`libs/service-http/src/error.rs`) is the
one `{"error", "message"}` JSON body every ecosystem service renders for
error responses, paired with a `StatusCode` via `ApiErr`. Services classify
their own domain errors into it (`From<DomainError>`); this crate owns only
the generic envelope, builder, and `utoipa::ToSchema` shape.

### Outbound Propagation (Not Yet Claimed)

Surveyed 2026-07-24 (#2491): this crate carries no outbound HTTP client at
all (no `reqwest`/`hyper`-client dependency, no code under
`libs/service-http/src/` that reads or writes a `traceparent` header on a
request this crate originates). Only the inbound accept/generate leg above
is implemented. Do not claim outbound `traceparent` injection for this crate
or for any service that composes it until an outbound propagation seam
actually ships.
