---
id: lumen-service-auth-convergence
summary: >
  Converge Lumen's bearer-token middleware onto libs/service-auth while keeping
  Lumen's static token role map and per-collection RBAC behavior unchanged.
capability_refs:
  - id: "security-hardening"
    role: primary
    claim: "bearer-token-auth-lumen-auth"
    coverage: partial
    rationale: >
      Issue #744 is the Lumen proving-consumer adoption for the shared
      service-auth verifier and middleware layer from parent epic #692 while
      preserving the bearer/RBAC security behavior.
  - id: "http2-api-list"
    role: contributes
    claim: "ops-metadata-probe-and-metrics-route-list"
    coverage: partial
    rationale: >
      The router keeps auth scoped to data-plane routes so ops metadata,
      probe, metrics, OpenAPI, and docs endpoints remain tokenless.
fill_sections: [logic, unit-test, e2e-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-service-auth-convergence-contract
entry: request
nodes:
  data_plane: { kind: start, label: "Router data-plane routes only" }
  state: { kind: process, label: "Arc<LumenVerifier> wraps existing Arc<AuthConfig>" }
  shared: { kind: process, label: "service_auth::auth_middleware<LumenVerifier>" }
  bearer: { kind: process, label: "service_auth::bearer_token extracts Authorization: Bearer" }
  verify: { kind: decision, label: "AuthConfig required/tokens lookup" }
  open: { kind: process, label: "no token + open mode => AuthContext::Open" }
  authed: { kind: process, label: "known token => AuthContext::Token(TokenClaims)" }
  unauth: { kind: terminal, label: "missing required or invalid token => shared 401 JSON" }
  handler: { kind: process, label: "handler reads Extension<AuthContext>" }
  rbac: { kind: process, label: "AuthContext::ensure keeps per-collection RBAC + audit 403" }
  ops: { kind: terminal, label: "probe/metrics/openapi/docs routes bypass auth unchanged" }
edges:
  - { from: data_plane, to: state }
  - { from: state, to: shared }
  - { from: shared, to: bearer }
  - { from: bearer, to: verify }
  - { from: verify, to: open }
  - { from: verify, to: authed }
  - { from: verify, to: unauth }
  - { from: open, to: handler }
  - { from: authed, to: handler }
  - { from: handler, to: rbac }
  - { from: data_plane, to: ops }
---
flowchart TD
    data_plane([data-plane routes]) --> state[Arc<LumenVerifier> around AuthConfig]
    state --> shared[service_auth::auth_middleware<LumenVerifier>]
    shared --> bearer[service_auth::bearer_token]
    bearer --> verify{required / token lookup}
    verify -->|open, no token| open[AuthContext::Open]
    verify -->|known token| authed[AuthContext::Token]
    verify -->|required missing or invalid| unauth([shared 401 JSON])
    open --> handler[Extension<AuthContext>]
    authed --> handler
    handler --> rbac[AuthContext::ensure + existing 403/audit]
    data_plane -. router split .-> ops([probe/metrics/openapi/docs bypass auth])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lumen-service-auth-convergence-contract-tests
requirements:
  verifier_open_mode:
    id: R1
    text: "LumenVerifier returns AuthContext::Open when auth is not required, no tokens are configured, and no bearer is present."
    kind: behavior
    risk: medium
    verify: test
  verifier_token_mode:
    id: R2
    text: "LumenVerifier returns AuthContext::Token for a known bearer and AuthError::Unauthenticated for an unknown bearer."
    kind: behavior
    risk: medium
    verify: test
  rbac_preserved:
    id: R3
    text: "AuthContext::ensure keeps wildcard and per-collection role precedence unchanged."
    kind: behavior
    risk: medium
    verify: test
  shared_middleware:
    id: R4
    text: "libs/service-auth continues to prove generic middleware injection and rejection."
    kind: behavior
    risk: low
    verify: test
elements:
  lumen_auth_unit_tests:
    kind: test
    path: projects/lumen/src/auth.rs
  service_auth_unit_tests:
    kind: test
    path: libs/service-auth/src/lib.rs
relations:
  - { from: lumen_auth_unit_tests, verifies: verifier_open_mode }
  - { from: lumen_auth_unit_tests, verifies: verifier_token_mode }
  - { from: lumen_auth_unit_tests, verifies: rbac_preserved }
  - { from: service_auth_unit_tests, verifies: shared_middleware }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "open-mode verifier"
      risk: medium
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "token-mode verifier"
      risk: medium
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "RBAC preserved"
      risk: medium
      verifymethod: test
    }
    requirement R4 {
      id: R4
      text: "shared middleware tests"
      risk: low
      verifymethod: test
    }
    element lumen_auth_unit_tests {
      type: "rs/#[test]"
    }
    element service_auth_unit_tests {
      type: "rs/#[test]"
    }
    lumen_auth_unit_tests - verifies -> R1
    lumen_auth_unit_tests - verifies -> R2
    lumen_auth_unit_tests - verifies -> R3
    service_auth_unit_tests - verifies -> R4
```
## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-auth-e2e-contract
    name: "lumen auth e2e contract"
    runner: cargo
    path: projects/lumen/tests/auth_e2e.rs
    command: "cargo test -p lumen --test auth_e2e -- --nocapture"
    verifies:
      - "Required mode rejects missing and invalid Bearer tokens with the shared 401 JSON body."
      - "Valid tokens are injected as AuthContext and handlers keep existing RBAC outcomes."
      - "Metrics, health, and readiness remain outside the data-plane auth layer."
  - id: lumen-authz-matrix-contract
    name: "lumen authz matrix contract"
    runner: cargo
    path: projects/lumen/tests/authz_matrix_e2e.rs
    command: "cargo test -p lumen --test authz_matrix_e2e -- --nocapture"
    verifies:
      - "Every protected route still enforces its route-specific role minimum after middleware delegation."
  - id: lumen-package-regression
    name: "lumen package regression"
    runner: cargo
    path: projects/lumen
    command: "cargo test -p lumen"
    verifies:
      - "The package compiles and the full Lumen regression suite remains green."
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add the workspace service-auth dependency to the Lumen crate."
  - path: projects/lumen/src/auth.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Introduce LumenVerifier, implement service_auth::Verifier<Principal = AuthContext>, use service_auth::bearer_token/shared AuthError for authentication failures, retain AuthContext::ensure for per-collection RBAC and audit logging, and expose an auth_middleware wrapper backed by service_auth::auth_middleware."
  - path: projects/lumen/src/api.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Build Arc<LumenVerifier> from AppState auth and layer the shared auth middleware only over data-plane routes."
  - path: projects/lumen/tech-design/semantic/source/projects-lumen-src-auth-rs.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Synchronize the spec-managed source capture for auth.rs so ownership annotations and source block match the implementation."
  - path: projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Synchronize the spec-managed source capture for api.rs import and middleware wiring."
```
