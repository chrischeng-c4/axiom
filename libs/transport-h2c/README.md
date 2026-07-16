# transport-h2c

## Brief

`transport-h2c` provides shared HTTP/2 cleartext transport: client and pool
helpers, frame-level connection management, logarithmic connection sizing, and
optional per-connection HTTP/1.1 plus h2c protocol handling. Listener admission,
task supervision, and drain belong to `server-http`/`server-tcp`.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| HTTP/2 Cleartext Client Helpers | - | implemented | verified | smoke | ready | single connection, pooled connection, and sizing heuristic |

### HTTP/2 Cleartext Client Helpers

ID: http2-cleartext-client-helpers
Type: DeveloperTool
Root WI: -
Status: verified
Surfaces: Rust API: `transport_h2c` client, manager, and per-connection server helpers.
EC Dimensions: behavior: `cargo test -p transport-h2c` - h2c client and pool behavior coverage
Required Verification: smoke
Promise:
Services can reuse one tested h2c transport helper instead of hand-rolling
connection setup, pooling, concurrency sizing, or per-connection protocol
negotiation. The crate never binds a listener.
Gate Inventory: `cargo test -p transport-h2c`; libs/transport-h2c/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| http2-cleartext-client-helpers-contract | epic | - | implemented | verified | smoke | `cargo test -p transport-h2c`; libs/transport-h2c/src/lib.rs |
