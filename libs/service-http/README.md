# service-http

## Brief

`service-http` provides shared HTTP-service policy: standard probe routes,
optional OTLP tracing policy, lifecycle readiness/signal adapters, the metrics
provider seam, and the JSON error envelope. Listener admission and drain state
belong to `server-http` and `server-lifecycle`.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Shared HTTP Service Scaffold | #1640 | implemented | passing | conformance | ready | probes, tracing policy, lifecycle adapters, metrics seam, and error envelope |

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
Tracing defaults to structured pretty/JSON logging. Enable the optional `otlp`
feature and provide an `HttpConfig.otlp_endpoint` plus `ServiceIdentity` to
export traces with stable `service.name` and `service.version` resources. A
missing feature, malformed endpoint, or exporter-construction failure keeps the
service runnable with logging; collector ownership and app-specific business
spans remain outside this crate. The standard trace layer extracts valid W3C
`traceparent`/`tracestate` headers into the request span.

Gate Inventory: `cargo test -p service-http`; `cargo test -p service-http --features otlp --test otlp_tracing`; libs/service-http/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-http-service-scaffold-contract | epic | #1640 | implemented | passing | conformance | `cargo test -p service-http`; `cargo test -p service-http --features otlp --test otlp_tracing`; libs/service-http/src/lib.rs |
