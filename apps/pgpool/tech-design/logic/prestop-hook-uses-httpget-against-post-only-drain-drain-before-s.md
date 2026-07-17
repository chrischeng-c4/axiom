---
id: '1883'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: prestop-get-drain
entry: prestop
nodes:
  prestop: { kind: start, label: "Kubernetes preStop HTTP GET /drain" }
  handler: { kind: process, label: "shared idempotent drain handler" }
  ready: { kind: process, label: "DrainController flips readiness" }
  done: { kind: terminal, label: "SIGTERM arrives after drain starts" }
edges:
  - { from: prestop, to: handler }
  - { from: handler, to: ready }
  - { from: ready, to: done }
---
flowchart TD
  prestop[preStop GET /drain] --> handler[shared drain handler]
  handler --> ready[DrainController flips readiness]
  ready --> done[SIGTERM after drain starts]
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
