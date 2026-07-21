---
id: '2219'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: defer-http2-api-contract-verification
entry: bind_one_port
nodes:
  bind_one_port: { kind: start, label: "bind one Defer service listener" }
  probe_http1: { kind: process, label: "probe health readiness docs OpenAPI and metrics over HTTP/1.1" }
  probe_h2c: { kind: process, label: "probe the same endpoints over prior-knowledge h2c" }
  protocol_ok: { kind: decision, label: "same listener serves both protocols and cross-protocol queue state?" }
  exercise_routes: { kind: process, label: "exercise all nine queue task dispatch and backup operations" }
  live_ok: { kind: decision, label: "served OpenAPI equals canonical IR and every operation executes statefully?" }
  render_offline: { kind: process, label: "render offline OpenAPI routes and typed clients from canonical IR" }
  offline_ok: { kind: decision, label: "exact route twin files symbols and terminal markers?" }
  fail: { kind: terminal, label: "HTTP API claim fails closed" }
  verified: { kind: terminal, label: "one-port live and offline HTTP API contract verified" }
  shared: { kind: terminal, label: "shared transport and codegen remain library-owned" }
edges:
  - { from: bind_one_port, to: probe_http1 }
  - { from: probe_http1, to: probe_h2c }
  - { from: probe_h2c, to: protocol_ok }
  - { from: protocol_ok, to: exercise_routes, label: "yes" }
  - { from: protocol_ok, to: fail, label: "no" }
  - { from: exercise_routes, to: live_ok }
  - { from: live_ok, to: render_offline, label: "yes" }
  - { from: live_ok, to: fail, label: "no" }
  - { from: render_offline, to: offline_ok }
  - { from: offline_ok, to: verified, label: "yes" }
  - { from: offline_ok, to: fail, label: "no" }
  - { from: bind_one_port, to: shared, label: "ownership boundary" }
---
flowchart TD
    bind_one_port([bind one service listener]) --> probe_http1[probe standard endpoints over HTTP/1.1]
    probe_http1 --> probe_h2c[probe same endpoints over h2c]
    probe_h2c --> protocol_ok{same listener and state?}
    protocol_ok -->|yes| exercise_routes[exercise exact nine domain operations]
    protocol_ok -->|no| fail([claim fails closed])
    exercise_routes --> live_ok{canonical equality and stateful journeys exact?}
    live_ok -->|yes| render_offline[render offline spec routes and clients]
    live_ok -->|no| fail
    render_offline --> offline_ok{exact twin files symbols and terminal markers?}
    offline_ok -->|yes| verified([HTTP API claim verified])
    offline_ok -->|no| fail
    bind_one_port -->|ownership boundary| shared([shared libraries unchanged])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/defer/src/bin/defer.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: spec
    reason: "Own the offline OpenAPI/routes projection and exact nine-operation route twin emitted from the Defer CLI."
  - path: apps/defer/tests/http_api.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: h2c_routes_probes_openapi_metrics_dispatch_and_auth_are_live
    reason: "Own the one-listener HTTP/1.1 and h2c probes, canonical served OpenAPI equality, exact nine-operation inventory, stateful route journeys, and backup recovery oracle."
  - path: apps/defer/tests/cli_contract.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: offline_spec_and_typed_client_generation_use_one_contract
    reason: "Own semantic equality of offline and canonical OpenAPI, the exact routes twin, and exact TypeScript, Python, and Rust client file and symbol inventories."
```
