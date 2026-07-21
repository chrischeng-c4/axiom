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
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: defer-http2-api-contract-verification
requirements:
  exact_live_route_contract:
    id: R2
    text: "Served OpenAPI equals the canonical Defer IR and contains exactly nine method/path operations; real requests exercise every queue, task, dispatch, and backup operation, with backup bytes recovering committed terminal state into a fresh Raft store."
    kind: functional
    risk: high
    verify: cargo test -p defer --test http_api -- --nocapture
  generated_ec_inventory:
    id: R5
    text: "The accepted live one-port and offline codegen EC cases remain generated as separate fail-closed wrappers bound to claim h2c-openapi-route-list."
    kind: regression
    risk: medium
    verify: aw ec check --project defer
  offline_spec_route_twin:
    id: R3
    text: "defer spec emits parseable OpenAPI semantically equal to the canonical IR, an exact nine-operation route twin including batch and backup, and exactly one terminal next marker."
    kind: regression
    risk: high
    verify: cargo test -p defer --test cli_contract offline_spec_and_typed_client_generation_use_one_contract -- --nocapture
  one_port_dual_protocol:
    id: R1
    text: "One bound Defer service URL serves health, readiness, docs, OpenAPI, and metrics with 200 over independent HTTP/1.1 and prior-knowledge h2c clients, and queue state written over h2c is readable over HTTP/1.1."
    kind: functional
    risk: high
    verify: cargo test -p defer --test http_api -- --nocapture
  three_language_codegen:
    id: R4
    text: "TypeScript, Python, and Rust code generation each produces the exact language-specific file inventory and all nine typed operations; Python includes both synchronous and asynchronous h2c clients."
    kind: regression
    risk: high
    verify: cargo test -p defer --test cli_contract offline_spec_and_typed_client_generation_use_one_contract -- --nocapture
---
flowchart TD
    r1[R1 one port dual protocol] --> cargo_test_p_defer_test_http_api_nocapture[cargo test -p defer --test http_api -- --nocapture]
    r2[R2 exact live route contract] --> cargo_test_p_defer_test_http_api_nocapture
    r3[R3 offline spec route twin] --> cargo_test_p_defer_test_cli_contract_offline_spec_and_typed_client_generation_use_one_contract_nocapture[cargo test -p defer --test cli_contract offline_spec_and_typed_client_generation_use_one_contract -- --nocapture]
    r4[R4 three language codegen] --> cargo_test_p_defer_test_cli_contract_offline_spec_and_typed_client_generation_use_one_contract_nocapture
    r5[R5 generated ec inventory] --> aw_ec_check_project_defer[aw ec check --project defer]
```
