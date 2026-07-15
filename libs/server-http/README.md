<!-- HANDWRITE-BEGIN gap="missing-generator:logic:34db7bee" tracker="pending-tracker" reason="Define the HTTP runtime capability boundary." -->
# server-http

## Brief

`server-http` is the shared listener-level HTTP runtime. It composes
`server-tcp` admission, supervision, bounded drain, and connection metrics
with `transport-h2c`'s per-connection HTTP/1.1+h2c protocol machinery.
Service routes and operational endpoint policy remain in `service-http`.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Shared HTTP Runtime | #1776 | implementing | planned | conformance | partial | sole listener owner over server-tcp and per-connection h2c |

### Shared HTTP Runtime

ID: shared-http-runtime
Type: DeveloperTool
Root WI: 1776
Status: implementing
Surfaces: Rust API: `server_http`.
EC Dimensions: behavior: `cargo test -p server-http -p server-tcp -p transport-h2c` - listener ownership, admission metrics, h2c, and drain coverage
Required Verification: conformance
Promise:
HTTP services and development tools can serve HTTP/1.1 and h2c through one
listener runtime that uses the same admission, supervision, readiness, and
shutdown contract as raw TCP services. `transport-h2c` never owns a listener;
`service-http` never owns lifecycle state.
Gate Inventory: `cargo test -p server-http -p server-tcp -p transport-h2c`; libs/server-http/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-http-runtime-lifecycle-convergence | change | #1776 | implementing | planned | conformance | `cargo test -p server-http -p server-tcp -p transport-h2c` |

<!-- marker: missing-generator:logic:34db7bee path: libs/server-http/README.md reason: Define the HTTP runtime capability boundary. -->
<!-- HANDWRITE-END -->
