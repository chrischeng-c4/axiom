# claim-token

## Brief

`claim-token` provides scoped claim-check access tokens using HMAC-SHA256. Schema
layers sign tokens and resource services verify them so workers can access only
the key scope encoded in the token.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Scoped Claim Tokens | - | signs and verifies scoped HMAC claim tokens |

### Scoped Claim Tokens

Services can issue and verify scoped HMAC claim tokens without sharing
resource-wide credentials with workers.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `claim_token` signing and verification primitives.
- Gate — behavior: `cargo test -p claim-token` - token signing, verification,
  and scope rejection coverage
- Gate: `cargo test -p claim-token`
- Source: `libs/claim-token/src/lib.rs`
- Evidence: `cargo test -p claim-token`; libs/claim-token/src/lib.rs
