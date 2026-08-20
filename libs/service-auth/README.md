# service-auth

## Brief

`service-auth` provides shared request-auth middleware for HTTP services:
extract, verify, reject, and inject plumbing plus a `Verifier` trait.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Shared HTTP Request Auth Middleware | - | verifier trait and middleware plumbing |
| Credential Reload and Redacted Authorization Audit | #1641 | atomic last-known-good registry replacement and credential-free event hooks |

### Shared HTTP Request Auth Middleware

HTTP services can share authentication middleware while keeping token crypto
and resource authorization in their appropriate layers.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `service_auth`.
- Gate — behavior: `cargo test -p service-auth` - auth middleware and verifier
  behavior coverage
- Gate: `cargo test -p service-auth`
- Source: `libs/service-auth/src/lib.rs`
- Evidence: `cargo test -p service-auth`; libs/service-auth/src/lib.rs

### Credential Reload and Redacted Authorization Audit

Services can atomically rotate a validated bearer-token registry without a
restart and can route authorization/reload decisions to logs or metrics without
ever placing raw bearer credentials in the event schema.

- Root WI: #1641
- Surfaces: Rust API: `service_auth::ReloadableRoleMapVerifier`,
  `service_auth::AuthEventSink`.
- Gate — security: `cargo test -p service-auth` - validated rotation,
  last-known-good preservation, and credential-free audit events
- Gate: `cargo test -p service-auth`
- Source: `libs/service-auth/src/reload.rs`
- Evidence: `cargo test -p service-auth`; libs/service-auth/src/reload.rs
