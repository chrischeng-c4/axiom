# transport-h2c

## Brief

`transport-h2c` provides shared HTTP/2 cleartext transport: client and pool
helpers, frame-level connection management, logarithmic connection sizing, and
optional per-connection HTTP/1.1 plus h2c protocol handling. Listener admission,
task supervision, and drain belong to `server-http`/`server-tcp`.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| HTTP/2 Cleartext Client Helpers | - | single connection, pooled connection, and sizing heuristic |

### HTTP/2 Cleartext Client Helpers

Services can reuse one tested h2c transport helper instead of hand-rolling
connection setup, pooling, concurrency sizing, or per-connection protocol
negotiation. The crate never binds a listener.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `transport_h2c` client, manager, and per-connection
  server helpers.
- Gate — behavior: `cargo test -p transport-h2c` - h2c client and pool behavior
  coverage
- Gate: `cargo test -p transport-h2c`
- Source: `libs/transport-h2c/src/lib.rs`
- Evidence: `cargo test -p transport-h2c`; libs/transport-h2c/src/lib.rs
