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
