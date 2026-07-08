# service-auth

## Brief

`service-auth` provides shared request-auth middleware for HTTP services:
extract, verify, reject, and inject plumbing plus a `Verifier` trait.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Shared HTTP Request Auth Middleware | - | implemented | verified | smoke | ready | verifier trait and middleware plumbing |

### Shared HTTP Request Auth Middleware

ID: shared-http-request-auth-middleware
Type: DeveloperTool
Root WI: -
Status: verified
Surfaces: Rust API: `service_auth`.
EC Dimensions: behavior: `cargo test -p service-auth` - auth middleware and verifier behavior coverage
Required Verification: smoke
Promise:
HTTP services can share authentication middleware while keeping token crypto and
resource authorization in their appropriate layers.
Gate Inventory: `cargo test -p service-auth`; libs/service-auth/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-http-request-auth-middleware-contract | epic | - | implemented | verified | smoke | `cargo test -p service-auth`; libs/service-auth/src/lib.rs |
