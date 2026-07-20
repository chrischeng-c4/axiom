---
id: '2172'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-measured-performance-applicability
entry: rejected
nodes:
  rejected:
    kind: start
    label: "Independent EC review rejects synthetic performance oracle"
  boundary:
    kind: decision
    label: "Does remediation change Relay domain semantics?"
  exclude_domain:
    kind: terminal
    label: "Stop and create a separate domain WI"
  measure:
    kind: process
    label: "Measure existing durable publish then lease and ack lifecycle"
  oracle:
    kind: process
    label: "Parse machine report independently and enforce pinned floors"
  ec:
    kind: process
    label: "Bind behavior efficiency stability cases through vat and meter"
  done:
    kind: terminal
    label: "Performance EC cannot pass with missing or zero observations"
edges:
  - { from: rejected, to: boundary }
  - { from: boundary, to: exclude_domain, label: "yes" }
  - { from: boundary, to: measure, label: "no" }
  - { from: measure, to: oracle }
  - { from: oracle, to: ec }
  - { from: ec, to: done }
---
flowchart TD
    rejected[rejected synthetic EC] --> boundary{domain semantics change?}
    boundary -->|yes| exclude_domain[separate domain WI]
    boundary -->|no| measure[measure durable lifecycle]
    measure --> oracle[independent parsed oracle]
    oracle --> ec[behavior efficiency stability EC]
    ec --> done[fail closed evidence]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/relay/tests/measured_performance.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Produce a release-mode fsync-always durable publish and lease/ack report in a child test process, then parse it in an independent parent oracle that rejects missing or zero samples and enforces pinned workload floors.
  - path: apps/relay/vat.toml
    action: modify
    section: config
    impl_mode: hand-written
    description: Build and execute the measured release-mode integration test through the meter-perf vat runner.
  - path: apps/relay/external-contracts/competitor-performance/efficiency/perf-gate.md
    action: modify
    section: e2e-test
    impl_mode: hand-written
    description: Replace the synthetic efficiency-only case with executable behavior, measured efficiency, and bounded stability cases.
  - path: apps/relay/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: Declare the measured performance envelope and all RuntimeTool-required EC dimensions without promoting advisory competitor wins.
  - path: apps/relay/docs/perf-gate.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: Record the exact local workload, pinned floors, and current release calibration separately from advisory external-broker results.
  - path: apps/relay/aw.toml
    action: modify
    section: e2e-test
    impl_mode: codegen
    description: Regenerate EC bindings for the three revised competitor-performance cases.
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-measured-performance-gate-verification
requirements:
  existing_work_queue_behavior_remains_correct:
    id: R4
    text: "The measured-oracle change does not alter publish, lease, acknowledgement, redelivery, or committed-watermark behavior."
    kind: regression
    risk: medium
    verify: cargo test -p relay --test work_queue_throughput --test perf_gate
  measured_gate_fails_closed:
    id: R1
    text: "A release-mode child producer runs the fsync-always durable publish and lease/ack lifecycle, emits non-zero machine-readable observations, and an independent parent oracle rejects missing or zero samples before enforcing pinned throughput and latency floors."
    kind: regression
    risk: high
    verify: cargo test --release -p relay --test measured_performance measured_durable_lifecycle_gate -- --exact --ignored --nocapture
  runtime_tool_dimensions_are_covered:
    id: R2
    text: "The competitor-performance EC supplies executable behavior, efficiency, and stability cases with specific observable assertions."
    kind: functional
    risk: high
    verify: aw ec check --project relay
  vat_meter_runs_measured_gate:
    id: R3
    text: "The vat-isolated meter-perf runner executes the release-mode measured gate and not only synthetic in-memory correctness tests."
    kind: integration
    risk: high
    verify: cd apps/relay && ../../target/debug/vat run meter-perf
---
flowchart TD
    r1[R1 measured gate fails closed] --> cargo_test_release_p_relay_test_measured_performance_measured_durable_lifecycle_gate_exact_ignored_nocapture[cargo test --release -p relay --test measured_performance measured_durable_lifecycle_gate -- --exact --ignored --nocapture]
    r2[R2 runtime tool dimensions are covered] --> aw_ec_check_project_relay[aw ec check --project relay]
    r3[R3 vat meter runs measured gate] --> cd_apps_relay_target_debug_vat_run_meter_perf[cd apps/relay && ../../target/debug/vat run meter-perf]
    r4[R4 existing work queue behavior remains correct] --> cargo_test_p_relay_test_work_queue_throughput_test_perf_gate[cargo test -p relay --test work_queue_throughput --test perf_gate]
```
