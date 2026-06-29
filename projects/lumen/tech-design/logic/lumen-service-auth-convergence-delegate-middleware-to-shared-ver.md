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
id: lumen-service-auth-convergence
entry: request
nodes:
  request: { kind: start, label: "data-plane HTTP request" }
  middleware: { kind: process, label: "service_auth::auth_middleware<LumenVerifier>" }
  verifier: { kind: process, label: "LumenVerifier authenticates headers against AuthConfig" }
  extension: { kind: process, label: "AuthContext inserted into request extensions" }
  handler: { kind: process, label: "handler calls AuthContext::ensure(collection, role)" }
  response: { kind: terminal, label: "existing 401/403/open-mode contract preserved" }
edges:
  - { from: request, to: middleware }
  - { from: middleware, to: verifier }
  - { from: verifier, to: extension }
  - { from: extension, to: handler }
  - { from: handler, to: response }
---
flowchart TD
    request([data-plane request]) --> middleware[service_auth::auth_middleware<LumenVerifier>]
    middleware --> verifier[LumenVerifier + AuthConfig role map]
    verifier --> extension[insert concrete AuthContext]
    extension --> handler[handler RBAC via AuthContext::ensure]
    handler --> response([unchanged 401/403/open-mode contract])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lumen-service-auth-convergence-unit-tests
requirements:
  shared_middleware_contract:
    id: R1
    text: "libs/service-auth continues to prove bearer extraction, required-mode rejection, open-mode pass-through, principal injection, and forbidden rendering."
    kind: behavior
    risk: medium
    verify: test
  lumen_verifier_contract:
    id: R2
    text: "LumenVerifier maps AuthConfig into AuthContext while preserving missing-token open mode, invalid-token 401, and concrete principal injection."
    kind: behavior
    risk: medium
    verify: test
  rbac_contract:
    id: R3
    text: "AuthContext::ensure continues to enforce collection-specific and wildcard roles."
    kind: behavior
    risk: medium
    verify: test
elements:
  service_auth_unit_tests:
    kind: test
    path: libs/service-auth/src/lib.rs
  lumen_auth_unit_tests:
    kind: test
    path: projects/lumen/src/auth.rs
relations:
  - { from: service_auth_unit_tests, verifies: shared_middleware_contract }
  - { from: lumen_auth_unit_tests, verifies: lumen_verifier_contract }
  - { from: lumen_auth_unit_tests, verifies: rbac_contract }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "shared auth middleware contract"
      risk: medium
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "LumenVerifier maps AuthConfig to AuthContext"
      risk: medium
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "per-collection RBAC remains in AuthContext"
      risk: medium
      verifymethod: test
    }
    element service_auth_unit_tests {
      type: "rs/#[test]"
    }
    element lumen_auth_unit_tests {
      type: "rs/#[test]"
    }
    service_auth_unit_tests - verifies -> R1
    lumen_auth_unit_tests - verifies -> R2
    lumen_auth_unit_tests - verifies -> R3
```
