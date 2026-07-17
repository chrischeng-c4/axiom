---
id: '1883'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: prestop-get-drain-contract
entry: prestop_request
nodes:
  prestop_request: { kind: start, label: "Kubernetes GET /drain" }
  route: { kind: process, label: "GET or POST route invokes handlers::drain" }
  transition: { kind: process, label: "shared controller starts drain idempotently" }
  readiness: { kind: terminal, label: "readyz returns draining before SIGTERM" }
edges:
  - { from: prestop_request, to: route }
  - { from: route, to: transition }
  - { from: transition, to: readiness }
---
flowchart TD
  prestop_request[GET /drain] --> route[one drain handler]
  route --> transition[shared controller]
  transition --> readiness[readyz draining]
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/admin/router.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: build_router
  - path: apps/pgpool/src/k8s/instance.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: render_manifests
  - path: apps/pgpool/src/spec.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: openapi_json
  - path: apps/pgpool/tests/admin_plane.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: served_contract_matches_offline_spec
  - path: apps/pgpool/tech-design/semantic/pgpool-stateless-deployment-instance.md
    action: modify
    section: logic
    impl_mode: hand-written
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-prestop-get-drain-verification
requirements:
  offline_served_contract:
    id: R2
    text: "Offline route and OpenAPI surfaces list GET and POST drain methods exactly as the served router does."
    kind: conformance
    risk: high
    verify: cargo test -p pgpool served_contract_matches_offline_spec
  prestop_get_drain:
    id: R1
    text: "Rendered preStop httpGet reaches a served GET /drain route that starts the shared DrainController before SIGTERM."
    kind: regression
    risk: high
    verify: cargo test -p pgpool prestop_get_drain
---
flowchart TD
    r1[R1 prestop get drain] --> cargo_test_p_pgpool_prestop_get_drain[cargo test -p pgpool prestop_get_drain]
    r2[R2 offline served contract] --> cargo_test_p_pgpool_served_contract_matches_offline_spec[cargo test -p pgpool served_contract_matches_offline_spec]
```
