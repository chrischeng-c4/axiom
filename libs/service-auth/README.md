# service-auth

## Brief

`service-auth` provides shared request-auth middleware for HTTP services:
extract, verify, reject, and inject plumbing plus a `Verifier` trait.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Shared HTTP Request Auth Middleware | - | implemented | verified | smoke | ready | verifier trait and middleware plumbing |
| Credential Reload and Redacted Authorization Audit | #1641 | implemented | verified | conformance | ready | atomic last-known-good registry replacement and credential-free event hooks |

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

### Credential Reload and Redacted Authorization Audit

ID: credential-reload-redacted-authorization-audit
Type: Security
Root WI: 1641
Status: verified
Surfaces: Rust API: `service_auth::ReloadableRoleMapVerifier`, `service_auth::AuthEventSink`.
EC Dimensions: security: `cargo test -p service-auth` - validated rotation, last-known-good preservation, and credential-free audit events
Required Verification: conformance
Promise:
Services can atomically rotate a validated bearer-token registry without a
restart and can route authorization/reload decisions to logs or metrics without
ever placing raw bearer credentials in the event schema.
Gate Inventory: `cargo test -p service-auth`; libs/service-auth/src/reload.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| credential-reload-redacted-authorization-audit | change | #1641 | implemented | verified | conformance | `cargo test -p service-auth`; libs/service-auth/src/reload.rs |
