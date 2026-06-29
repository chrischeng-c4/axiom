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
