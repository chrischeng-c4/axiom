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
  headroom: { kind: process, label: "reserve five seconds for SIGKILL boundary" }
  minimum: { kind: process, label: "floor drain to one second" }
  saturating_ms: { kind: process, label: "saturating conversion to milliseconds" }
  render: { kind: terminal, label: "deployment environment value" }
edges:
  - { from: termination_grace_period_seconds, to: headroom }
  - { from: headroom, to: minimum }
  - { from: minimum, to: saturating_ms }
  - { from: saturating_ms, to: render }
---
flowchart TD
  grace["grace seconds"] --> reserve["minus 5 seconds headroom"] --> floor["minimum 1 second"] --> ms["saturating milliseconds"] --> env["drain timeout env"]
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
