# service-tls

## Brief

`service-tls` provides shared peer-mTLS material loading and rustls server/client
config builders for mutually-authenticated service ports.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Shared Peer mTLS Material Loading | - | implemented | verified | smoke | ready | PEM loaders and rustls client/server config builders |

### Shared Peer mTLS Material Loading

ID: shared-peer-mtls-material-loading
Type: DeveloperTool
Root WI: -
Status: verified
Surfaces: Rust API: `service_tls`.
EC Dimensions: behavior: `cargo test -p service-tls` - TLS material loading and config behavior coverage
Required Verification: smoke
Promise:
Services can load peer mTLS material through a shared prefix-driven contract and
reuse the same rustls setup.
Gate Inventory: `cargo test -p service-tls`; libs/service-tls/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-peer-mtls-material-loading-contract | epic | - | implemented | verified | smoke | `cargo test -p service-tls`; libs/service-tls/src/lib.rs |
