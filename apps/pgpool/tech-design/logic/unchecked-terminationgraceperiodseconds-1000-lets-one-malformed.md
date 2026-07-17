---
id: '1885'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: drain-timeout-headroom
entry: termination_grace_period_seconds
nodes:
  reserve: { kind: process, label: "reserve fixed SIGKILL headroom" }
  floor: { kind: process, label: "floor usable drain duration at one second" }
  saturate: { kind: process, label: "saturating seconds-to-milliseconds conversion" }
  env: { kind: terminal, label: "render PGPOOL_DRAIN_TIMEOUT_MS" }
edges:
  - { from: termination_grace_period_seconds, to: reserve }
  - { from: reserve, to: floor }
  - { from: floor, to: saturate }
  - { from: saturate, to: env }
---
flowchart TD
  grace["CR grace period seconds"] --> headroom["subtract fixed SIGKILL headroom"]
  headroom --> floor["floor drain at minimum one second"]
  floor --> saturate["saturating * 1000"] --> env["PGPOOL_DRAIN_TIMEOUT_MS"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/k8s/instance.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: render_manifests
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: drain-timeout-headroom-verification
requirements:
  overflow_saturates_and_preserves_headroom:
    id: R1
    text: "u64::MAX grace renders without overflow while normal and tiny grace periods reserve documented drain headroom before SIGKILL."
    kind: regression
    risk: high
    verify: cargo test -p pgpool k8s::instance::tests::drain_timeout_saturates_and_reserves_sigkill_headroom
---
flowchart TD
    r1[R1 overflow saturates and preserves headroom] --> cargo_test_p_pgpool_k8s_instance_tests_drain_timeout_saturates_and_reserves_sigkill_headroom[cargo test -p pgpool k8s::instance::tests::drain_timeout_saturates_and_reserves_sigkill_headroom]
```
