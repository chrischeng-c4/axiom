# transport-h2c

## Brief

`transport-h2c` provides shared HTTP/2 cleartext transport: client and pool
helpers, frame-level connection management, logarithmic connection sizing, and
optional HTTP/1.1 plus h2c server support.

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
Surfaces: Rust API: `transport_h2c` connection, manager, and server helpers.
EC Dimensions: behavior: `cargo test -p transport-h2c` - h2c client and pool behavior coverage
Required Verification: smoke
Promise:
Services can reuse one tested h2c transport helper instead of hand-rolling
connection setup, pooling, and concurrency sizing.
Gate Inventory: `cargo test -p transport-h2c`; libs/transport-h2c/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| http2-cleartext-client-helpers-contract | epic | - | implemented | verified | smoke | `cargo test -p transport-h2c`; libs/transport-h2c/src/lib.rs |
