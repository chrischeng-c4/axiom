# service-http

## Brief

`service-http` provides shared HTTP-service scaffolding: standard probe routes,
tracing init, graceful shutdown, h2c transport, and the JSON error envelope.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Shared HTTP Service Scaffold | - | implemented | verified | smoke | ready | probes, tracing, shutdown, transport, and error envelope |

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
Gate Inventory: `cargo test -p service-http`; libs/service-http/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-http-service-scaffold-contract | epic | - | implemented | verified | smoke | `cargo test -p service-http`; libs/service-http/src/lib.rs |
