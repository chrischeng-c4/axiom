# claimtoken

## Brief

`claimtoken` provides scoped claim-check access tokens using HMAC-SHA256. Schema
layers sign tokens and resource services verify them so workers can access only
the key scope encoded in the token.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Scoped Claim Tokens | - | implemented | verified | smoke | ready | signs and verifies scoped HMAC claim tokens |

### Scoped Claim Tokens

ID: scoped-claim-tokens
Type: DeveloperTool
Root WI: -
Status: verified
Surfaces: Rust API: `claimtoken` signing and verification primitives.
EC Dimensions: behavior: `cargo test -p claimtoken` - token signing, verification, and scope rejection coverage
Required Verification: smoke
Promise:
Services can issue and verify scoped HMAC claim tokens without sharing
resource-wide credentials with workers.
Gate Inventory: `cargo test -p claimtoken`; libs/claimtoken/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| scoped-claim-tokens-contract | epic | - | implemented | verified | smoke | `cargo test -p claimtoken`; libs/claimtoken/src/lib.rs |
