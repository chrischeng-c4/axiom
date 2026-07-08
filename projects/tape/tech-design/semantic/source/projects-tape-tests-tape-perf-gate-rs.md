---
id: projects-tape-tests-tape-perf-gate-rs
coverage_kind: semantic
capability_refs:
  - id: "competitor-performance"
    role: primary
    claim: "topic-replay-competitor-performance-baseline"
    gap: "topic-replay-competitor-performance-baseline"
    coverage: partial
    rationale: "The test is the EC gate that proves local performance budgets and prevents uncalibrated peer win claims."
fill_sections: [overview, unit-test, changes]
---

# Tape Performance Gate Test

## Overview
<!-- type: overview lang: markdown -->

`projects/tape/tests/tape_perf_gate.rs` verifies Tape's local benchmark and
calibration ledger. It proves local append/replay/checkpoint budget compliance
while refusing external broker win claims without real peer evidence.

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-td-flow
---
flowchart TD
    test["cargo test -p tape --test tape_perf_gate -- --nocapture"] --> local["local benchmark budgets pass"]
    test --> peers["Kafka/Redpanda/Pulsar/JetStream/RabbitMQ Streams status not_calibrated"]
    test --> exchange["RabbitMQ topic exchange status not_a_replay_baseline"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/tape/tests/tape_perf_gate.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Performance regression and calibration-status gate for Tape competitor performance."
```
