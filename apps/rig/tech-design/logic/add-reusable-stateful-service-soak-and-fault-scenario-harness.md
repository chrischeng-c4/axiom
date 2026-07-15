---
id: '1645'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: rig-stateful-service-harness
entry: warmup
nodes:
  warmup: { kind: start, label: "Run bounded warm-up action" }
  observe: { kind: process, label: "Capture steady-state observations and evidence" }
  fault: { kind: process, label: "Inject one bounded service fault or restart" }
  recover: { kind: process, label: "Recover or restart the service" }
  verify: { kind: process, label: "Assert domain-specific state continuity" }
  failed: { kind: process, label: "Record failed phase, error, duration, and retained evidence" }
  teardown: { kind: process, label: "Always run bounded teardown and capture its outcome" }
  report: { kind: terminal, label: "Emit deterministic typed scenario report" }
edges:
  - { from: warmup, to: observe, label: "pass" }
  - { from: observe, to: fault, label: "pass" }
  - { from: fault, to: recover, label: "pass" }
  - { from: recover, to: verify, label: "pass" }
  - { from: verify, to: teardown, label: "pass" }
  - { from: warmup, to: failed, label: "fail or timeout" }
  - { from: observe, to: failed, label: "fail or timeout" }
  - { from: fault, to: failed, label: "fail or timeout" }
  - { from: recover, to: failed, label: "fail or timeout" }
  - { from: verify, to: failed, label: "fail or timeout" }
  - { from: failed, to: teardown }
  - { from: teardown, to: report }
---
flowchart TD
    warmup([Run bounded warm-up action]) -->|pass| observe[Capture steady-state observations and evidence]
    observe -->|pass| fault[Inject one bounded service fault or restart]
    fault -->|pass| recover[Recover or restart the service]
    recover -->|pass| verify[Assert domain-specific state continuity]
    warmup -->|fail or timeout| failed[Record failed phase error duration and retained evidence]
    observe -->|fail or timeout| failed
    fault -->|fail or timeout| failed
    recover -->|fail or timeout| failed
    verify -->|fail or timeout| failed
    verify -->|pass| teardown[Always run bounded teardown and capture its outcome]
    failed --> teardown
    teardown --> report([Emit deterministic typed scenario report])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/rig/src/engine/stateful.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "Add the reusable typed warmup/observe/fault/recover/verify/teardown runner with bounded actions, retained evidence, and deterministic reports. generator gap: missing-generator:stateful-harness (#1645)."
  - path: apps/rig/src/engine/mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Export the reusable stateful runner. generator gap: missing-generator:engine-module-export (#1645)."
  - path: apps/rig/tech-design/semantic/source/projects-rig-src-engine-mod-rs.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Keep the semantic source mirror aligned with the engine module export. generator gap: missing-generator:semantic-source-sync (#1645)."
  - path: apps/rig/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Declare the reusable stateful service scenario contract and its shared-consumer boundary. generator gap: missing-generator:capability-doc (#1645)."
  - path: apps/rig/tests/stateful_service_harness.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Exercise the runner against a real bounded local HTTP stateful fixture and prove failure evidence plus teardown behavior. generator gap: missing-generator:stateful-harness-test (#1645)."
  - path: apps/lumen/Cargo.toml
    action: modify
    section: changes
    impl_mode: hand-written
    description: "Add Rig as a dev-only shared scenario runner dependency. generator gap: missing-generator:workspace-dev-dependency (#1645)."
  - path: apps/lumen/tests/rig_stateful_adapter.rs
    action: create
    section: e2e-test
    impl_mode: hand-written
    description: "Bind Lumen search continuity assertions to the shared Rig stateful lifecycle. generator gap: missing-generator:lumen-stateful-adapter (#1645)."
  - path: apps/tape/Cargo.toml
    action: modify
    section: changes
    impl_mode: hand-written
    description: "Add Rig as a dev-only shared scenario runner dependency. generator gap: missing-generator:workspace-dev-dependency (#1645)."
  - path: apps/tape/tests/rig_stateful_adapter.rs
    action: create
    section: e2e-test
    impl_mode: hand-written
    description: "Bind Tape replay and checkpoint continuity assertions to the shared Rig stateful lifecycle. generator gap: missing-generator:tape-stateful-adapter (#1645)."
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: rig-stateful-service-harness-verification
requirements:
  bounded_lifecycle:
    id: R1
    text: "The runner executes warmup, observe, fault, recover, verify, and teardown in fixed order with explicit per-phase and primary-lifecycle time budgets."
    kind: functional
    risk: high
    verify: cargo test -p rig --test stateful_service_harness
  failure_evidence:
    id: R2
    text: "A failed or timed-out primary phase records its phase, error, duration, and already-captured evidence before the runner always executes bounded teardown."
    kind: regression
    risk: high
    verify: cargo test -p rig --test stateful_service_harness
  real_stateful_fixture:
    id: R3
    text: "The shared runner drives a real local HTTP stateful fixture through steady state, fault, recovery, and continuity verification without mocks."
    kind: functional
    risk: high
    verify: cargo test -p rig --test stateful_service_harness
  shared_adapters:
    id: R4
    text: "Lumen and Tape use the same Rig runner API while supplying different domain assertions for search continuity versus ordered replay and checkpoint continuity."
    kind: functional
    risk: medium
    verify: cargo test -p lumen --test rig_stateful_adapter && cargo test -p tape --test rig_stateful_adapter
---
flowchart TD
    r1[R1 bounded lifecycle] --> cargo_test_p_rig_test_stateful_service_harness[cargo test -p rig --test stateful_service_harness]
    r2[R2 failure evidence] --> cargo_test_p_rig_test_stateful_service_harness
    r3[R3 real stateful fixture] --> cargo_test_p_rig_test_stateful_service_harness
    r4[R4 shared adapters] --> cargo_test_p_lumen_test_rig_stateful_adapter_cargo_test_p_tape_test_rig_stateful_adapter[cargo test -p lumen --test rig_stateful_adapter && cargo test -p tape --test rig_stateful_adapter]
```
