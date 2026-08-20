<!-- HANDWRITE-BEGIN gap="missing-generator:logic:34db7bee" tracker="pending-tracker" reason="Define the HTTP runtime capability boundary." -->
# server-http

## Brief

`server-http` is the shared listener-level HTTP runtime. It composes
`server-tcp` admission, supervision, bounded drain, and connection metrics
with `transport-h2c`'s per-connection HTTP/1.1+h2c protocol machinery.
Service routes and operational endpoint policy remain in `service-http`.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Shared HTTP Runtime | #1776 | sole listener owner over server-tcp and per-connection h2c |

### Shared HTTP Runtime

HTTP services and development tools can serve HTTP/1.1 and h2c through one
listener runtime that uses the same admission, supervision, readiness, and
shutdown contract as raw TCP services. `transport-h2c` never owns a listener;
`service-http` never owns lifecycle state.

- Root WI: #1776
- Surfaces: Rust API: `server_http`.
- Gate — behavior: `cargo test -p server-http -p server-tcp -p transport-h2c` -
  listener ownership, admission metrics, h2c, and drain coverage
- Gate: `cargo test -p server-http -p server-tcp -p transport-h2c`
- Source: `libs/server-http/src/lib.rs`

<!-- HANDWRITE-END -->
- Evidence: `cargo test -p server-http -p server-tcp -p transport-h2c`
