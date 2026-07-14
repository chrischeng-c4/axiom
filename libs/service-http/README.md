# service-http

## Brief

`service-http` provides shared HTTP-service scaffolding: standard probe routes,
optional OTLP tracing, graceful shutdown, h2c transport, and the JSON error
envelope.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Shared HTTP Service Scaffold | #1640 | implemented | passing | conformance | ready | probes, optional OTLP tracing, shutdown, transport, and error envelope |

### Shared HTTP Service Scaffold

ID: shared-http-service-scaffold
Type: DeveloperTool
Root WI: -
Status: verified
Surfaces: Rust API: `service_http`.
EC Dimensions: behavior: `cargo test -p service-http` - standard HTTP service scaffold coverage
Required Verification: smoke
Promise:
HTTP services can reuse the common operational endpoints and shutdown/transport
plumbing instead of hand-rolling them per service.
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
