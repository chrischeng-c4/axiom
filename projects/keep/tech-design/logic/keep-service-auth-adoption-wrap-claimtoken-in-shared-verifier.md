---
id: keep-service-auth-adoption
summary: >
  Adopt libs/service-auth's request-auth plumbing in Keep by wrapping Keep's
  existing claimtoken verification in a concrete service_auth::Verifier
  (KeepVerifier -> KeepPrincipal), and route the hand-rolled bearer extraction
  through the shared bearer_token helper, while preserving Keep's optional,
  claim-check-only, bare-id, 403 scope contract per-handler in check_scope.
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: keep-service-auth-adoption-contract
entry: boot
nodes:
  boot: { kind: start, label: "AppState built; with_token_secret(secret) wraps a KeepVerifier when KEEP_TOKEN_SECRET is set" }
  state: { kind: process, label: "AppState.verifier holds Option<Arc<KeepVerifier>>; None means claim-check is open (backward compatible)" }
  worker_op: { kind: process, label: "get_input (read) or put_result (write) calls check_scope on the bare url id before the namespace prefix" }
  optional_gate: { kind: decision, label: "is AppState.verifier Some?" }
  open: { kind: process, label: "no verifier: check_scope is a no-op, claim-check stays open" }
  authenticate: { kind: process, label: "KeepVerifier.authenticate uses service_auth::bearer_token then claimtoken::verify(secret, token, now)" }
  principal: { kind: decision, label: "verify yields KeepPrincipal scope?" }
  authz: { kind: decision, label: "principal.authorizes(id, write): scope.w==id for write else scope.r==id" }
  ok: { kind: process, label: "authorized: handler proceeds, then claim_namespace applies the X-Keep-Namespace prefix" }
  forbidden: { kind: process, label: "missing/invalid/expired/out-of-scope: ApiErr 403 forbidden, exact body preserved" }
  stop: { kind: terminal, label: "claim-check op served or rejected" }
edges:
  - { from: boot, to: state }
  - { from: state, to: worker_op }
  - { from: worker_op, to: optional_gate }
  - { from: optional_gate, to: open, label: "None" }
  - { from: optional_gate, to: authenticate, label: "Some" }
  - { from: open, to: ok }
  - { from: authenticate, to: principal }
  - { from: principal, to: authz, label: "Ok" }
  - { from: principal, to: forbidden, label: "Err" }
  - { from: authz, to: ok, label: "yes" }
  - { from: authz, to: forbidden, label: "no" }
  - { from: ok, to: stop }
  - { from: forbidden, to: stop }
---
flowchart TD
    boot([AppState built; with_token_secret wraps a KeepVerifier when KEEP_TOKEN_SECRET is set]) --> state[AppState.verifier holds Option Arc KeepVerifier; None means claim-check is open]
    state --> worker_op[get_input read or put_result write calls check_scope on the bare url id]
    worker_op --> optional_gate{is AppState.verifier Some}
    optional_gate -- None --> open[no verifier: check_scope is a no-op, claim-check stays open]
    optional_gate -- Some --> authenticate[KeepVerifier.authenticate uses service_auth bearer_token then claimtoken verify]
    open --> ok[authorized: handler proceeds then claim_namespace applies the X-Keep-Namespace prefix]
    authenticate --> principal{verify yields KeepPrincipal scope}
    principal -- Ok --> authz{principal.authorizes id write: scope.w==id for write else scope.r==id}
    principal -- Err --> forbidden[ApiErr 403 forbidden, exact body preserved]
    authz -- yes --> ok
    authz -- no --> forbidden
    ok --> stop([claim-check op served or rejected])
    forbidden --> stop
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: keep-service-auth-adoption-tests
requirements:
  verifier_wraps_claimtoken:
    id: R1
    text: "KeepVerifier implements service_auth::Verifier, wrapping claimtoken::verify to yield a concrete KeepPrincipal scope; a valid in-scope token authenticates and authorizes its bare id."
    kind: behavior
    risk: high
    verify: test
  shared_plumbing_per_handler:
    id: R2
    text: "Keep adopts the shared Verifier trait and service_auth::bearer_token extraction and invokes the verifier per-handler in check_scope rather than as a blanket auth_middleware layer, because the blanket layer gates every route and renders 401, which cannot express Keep's optional, claim-check-only, bare-id model."
    kind: behavior
    risk: high
    verify: test
  preserve_403_contract:
    id: R3
    text: "Keep preserves its wire contract: check_scope is a no-op when no verifier is set, applies only to get_input/put_result on the bare id, and returns 403 with the existing body for missing, invalid, expired, or out-of-scope tokens; an accepted scoped token returns 200."
    kind: behavior
    risk: high
    verify: test
elements:
  keep_auth_unit_tests:
    kind: test
    path: projects/keep/src/http/auth.rs
  keep_http_api:
    kind: test
    path: projects/keep/tests/http_api.rs
relations:
  - { from: keep_auth_unit_tests, verifies: verifier_wraps_claimtoken }
  - { from: keep_auth_unit_tests, verifies: shared_plumbing_per_handler }
  - { from: keep_http_api, verifies: preserve_403_contract }
  - { from: keep_http_api, verifies: shared_plumbing_per_handler }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "KeepVerifier wraps claimtoken"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "shared plumbing per-handler"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "preserve optional 403 contract"
      risk: high
      verifymethod: test
    }
    element keep_auth_unit_tests {
      type: "rs/#[test]"
    }
    element keep_http_api {
      type: "rs/#[tokio::test]"
    }
    keep_auth_unit_tests - verifies -> R1
    keep_auth_unit_tests - verifies -> R2
    keep_http_api - verifies -> R3
    keep_http_api - verifies -> R2
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/keep/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add the libs/service-auth dependency to Keep."
  - path: projects/keep/src/http/auth.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "New module: KeepPrincipal (concrete principal wrapping claimtoken::Scope, with authorizes(id, write)) and KeepVerifier implementing service_auth::Verifier by composing service_auth::bearer_token + claimtoken::verify."
  - path: projects/keep/src/http/mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Declare pub mod auth; replace AppState.token_secret with verifier: Option<Arc<KeepVerifier>>; with_token_secret builds a KeepVerifier so the optional (None=open) semantics and constructor API are preserved."
  - path: projects/keep/src/http/handlers.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Rewrite check_scope to delegate token verification to AppState.verifier (the shared Verifier) and keep the per-handler bare-id scope decision and the exact 403 forbidden body; no-op when no verifier; still called only by get_input and put_result."
  - path: projects/keep/src/http/auth.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Unit tests: KeepVerifier authenticates a valid token to a KeepPrincipal, authorizes the bare id for read/write scope, and returns AuthError for missing/invalid/expired/out-of-scope tokens."
  - path: projects/keep/tests/http_api.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Extend claim-check auth tests: accepted scoped token 200, invalid token 403, out-of-scope token 403, and no-op when no secret, preserving the existing keep_ns_token_checks_bare_key contract."
```
