---
id: libs-service-auth-src-verifier-rs
summary: Lossless rust-source-unit coverage for `libs/service-auth/src/verifier.rs`.
capability_refs:
  - id: shared-http-request-auth-middleware
    role: primary
    claim: shared-http-request-auth-middleware-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Service Auth library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/service-auth/src/verifier.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/service-auth/src/verifier.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Verifier` | libs/service-auth/src/verifier.rs | trait | pub | 21 | pub trait Verifier: Send + Sync + 'static { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! The [`Verifier`] trait — the one thing each service implements.
//!
//! A service's verifier turns request headers into the service's own concrete
//! principal type (or an [`AuthError`]). No `Any`/downcast: the principal type
//! is an associated type, so handlers receive it concretely via
//! `Extension<V::Principal>`. The generic middleware in [`crate::middleware`]
//! plumbs the result; the verifier owns the policy.

use crate::error::AuthError;

/// Authenticate a request from its headers into the service's principal.
///
/// Each HTTP service implements this once. The token crypto a verifier uses
/// (HMAC via `libs/claimtoken`, a role-map, k8s ServiceAccount JWT, OIDC, …) is
/// the implementor's choice and is **not** part of this lib.
///
/// Open / anonymous mode is expressed by the verifier itself: set
/// [`required`](Verifier::required) to `false` and return the service's "open"
/// principal value from [`authenticate`](Verifier::authenticate) when no
/// credential is presented. The middleware does not special-case it.
pub trait Verifier: Send + Sync + 'static {
    /// The service's own principal type, injected into request extensions on
    /// success and read by handlers via `axum::extract::Extension`.
    type Principal: Clone + Send + Sync + 'static;

    /// Authenticate from request headers. Return the principal (which MAY be the
    /// service's "open/anonymous" value when no token is needed), or an
    /// [`AuthError`].
    fn authenticate(&self, headers: &axum::http::HeaderMap) -> Result<Self::Principal, AuthError>;

    /// Whether a credential is required (controls open-mode). Default `true`.
    fn required(&self) -> bool {
        true
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/service-auth/src/verifier.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/service-auth/src/verifier.rs` captured during libs codegen standardization.
```
