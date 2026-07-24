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

The machine-readable capability contract for `service-http` currently lives
in `libs/service-http/README.md` (`cap_path`); this section names the
one-port HTTP transport contract every adopting service inherits so it stays
a first-class, discoverable listing rather than scattered README prose.
Refs #2491.

### Standard Endpoints (Probes Surface)

`standard_probe_routes()` (`libs/service-http/src/probes.rs`) mounts the five
always-on, auth-exempt, body-limit-exempt routes every k8s-native service
ships on its one serve port: `GET /healthz` (liveness), `GET /readyz`
(readiness — 503 `draining` mid-drain), `GET /metrics` (Prometheus text
format via `MetricsProvider`), `GET /openapi.json` (the service's OpenAPI
document), and `GET /docs` (Swagger UI). `standard_probe_routes_canonical_json`
is the byte-identical-snapshot variant for services that need one canonical
serialization across CLI/offline/live surfaces.

### Trace Context (Accept/Generate + Log Correlation + OTLP Upgrade)

`trace_layer()` (`libs/service-http/src/transport.rs`) is the shared
`tower-http` `TraceLayer` every service composes instead of a hand-rolled
span/correlation layer:

- **Accept**: a valid W3C version-00 `traceparent` header is parsed and its
  `trace_id`/`parent_span_id` preserved; strictly invalid input (wrong
  version, wrong length, non-hex, all-zero ids) is treated as absent rather
  than rejected.
- **Generate**: when no `traceparent` arrives, `request_trace_context`
  mints a fresh local-root `trace_id` and `span_id`.
- **Log correlation**: `CorrelatingMakeSpan` records `trace_id`, `span_id`,
  and `parent_span_id` (when present) on every request span, and those
  fields flow into the structured stdout every service emits
  (`axiom.service.log.v1`) — the schema the `sift` collector ingests — so
  cross-service log correlation works with zero exporter configured.
- **OTLP upgrade**: the `otlp` feature re-exports the same span context to
  full OpenTelemetry export without changing the accept/generate contract.

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

### Not Yet Claimed

Two items live at the same trace-context seam but are **not** implemented
here yet — do not claim them until they ship:

- **Response `Server-Timing`** (planned, #2490) — no `Server-Timing` header
  is emitted today.
- **Outbound `traceparent` injection** — the h2c client side does not yet
  propagate the current span's `traceparent` to downstream service-to-service
  calls; only inbound accept/generate is implemented.
