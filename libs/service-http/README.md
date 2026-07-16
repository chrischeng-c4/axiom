# service-http

## Brief

`service-http` provides shared HTTP-service policy: standard probe routes,
request-context propagation, lifecycle readiness/signal adapters, and the JSON
error envelope. Protocol-neutral logging, optional OTLP export, metric-provider
semantics, and lifecycle counters belong to `service-observability`; listener
admission and drain state belong to `server-http` and `server-lifecycle`.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Shared HTTP Service Scaffold | #1640 | implemented | passing | conformance | ready | probes, HTTP propagation, lifecycle adapters, and error envelope |
| Opaque-Key Admission Control | #1642 | implemented | verified | conformance | ready | bounded token buckets, redacted events, and standard 429 responses |

### Shared HTTP Service Scaffold

ID: shared-http-service-scaffold
Type: DeveloperTool
Root WI: -
Status: verified
Surfaces: Rust API: `service_http`.
EC Dimensions: behavior: `cargo test -p service-http` - standard HTTP service scaffold coverage
Required Verification: smoke
Promise:
HTTP services can reuse common operational routes and policy adapters instead
of hand-rolling them per service. `service-http` delegates listener serving to
`server-http` and re-exports readiness/shutdown from `server-lifecycle`.
Compatibility names for logging and metric providers delegate to
`service-observability`, while this crate keeps only the HTTP adapter that
extracts valid W3C `traceparent`/`tracestate` headers into request spans.

Gate Inventory: `cargo test -p service-http`; `cargo test -p service-http --features otlp --test otlp_tracing`; libs/service-http/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-http-service-scaffold-contract | epic | #1640 | implemented | passing | conformance | `cargo test -p service-http`; `cargo test -p service-http --features otlp --test otlp_tracing`; libs/service-http/src/lib.rs |

### Opaque-Key Admission Control

ID: opaque-key-admission-control
Type: Security
Root WI: 1642
Status: verified
Surfaces: Rust API: `service_http::AdmissionController`, `service_http::admission_middleware`.
EC Dimensions: security: `cargo test -p service-http admission` - deterministic bounded admission and credential-free event coverage
Required Verification: conformance
Promise:
HTTP services can opt into per-endpoint-class token buckets using caller-owned
opaque keys and policy values. Retained state contains only SHA-256
fingerprints, decision observers cannot represent raw keys, and denials use the
shared error envelope with HTTP 429 and `Retry-After`. Applications keep route
classification and policy ownership; an empty policy set is disabled.
Gate Inventory: `cargo test -p service-http`; libs/service-http/src/admission.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| opaque-key-admission-control | change | #1642 | implemented | verified | conformance | `cargo test -p service-http`; libs/service-http/src/admission.rs |
