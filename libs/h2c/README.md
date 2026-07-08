# h2c

## Brief

`h2c` provides shared HTTP/2 cleartext client helpers: a single-connection
client, a round-robin connection pool, and the logarithmic concurrency heuristic
used by service clients.

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
Surfaces: Rust API: `h2c` connection, manager, and server helpers.
EC Dimensions: behavior: `cargo test -p h2c` - h2c client and pool behavior coverage
Required Verification: smoke
Promise:
Services can reuse one tested h2c transport helper instead of hand-rolling
connection setup, pooling, and concurrency sizing.
Gate Inventory: `cargo test -p h2c`; libs/h2c/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| http2-cleartext-client-helpers-contract | epic | - | implemented | verified | smoke | `cargo test -p h2c`; libs/h2c/src/lib.rs |
