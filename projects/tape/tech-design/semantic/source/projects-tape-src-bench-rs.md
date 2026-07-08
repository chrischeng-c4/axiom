---
id: projects-tape-src-bench-rs
coverage_kind: semantic
capability_refs:
  - id: "competitor-performance"
    role: primary
    claim: "topic-replay-competitor-performance-baseline"
    gap: "topic-replay-competitor-performance-baseline"
    coverage: partial
    rationale: "The benchmark module owns Tape's local replay regression budget and external peer calibration ledger."
fill_sections: [overview, logic, unit-test, changes]
---

# Tape Benchmark Core

## Overview
<!-- type: overview lang: markdown -->

`projects/tape/src/bench.rs` defines the first competitive-performance evidence
surface. It measures local append, replay, and checkpoint operations and also
models named real-service replay wins such as the NATS JetStream local backlog
gate. Other replay-log peer wins stay unclaimed until their own real-service
peer runs are added.

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-td-flow
---
flowchart TD
    bench["run_benchmark(events, payload_bytes)"] --> append["measure append p50/p95"]
    bench --> replay["measure full replay elapsed time"]
    bench --> checkpoint["measure checkpoint p50/p95"]
    bench --> peers["emit peer calibration ledger"]
    peers --> no_win["Kafka/Redpanda/Pulsar/JetStream/RabbitMQ Streams win_claim=false"]
    peers --> exchange["RabbitMQ topic exchange marked not_a_replay_baseline"]
    external["external_replay_win"] --> ratio["peer replay us / Tape replay us"]
    ratio --> gate{"ratio >= required_ratio?"}
    gate -->|yes| win["allow named real-service peer win"]
    gate -->|no| fail["fail named peer win"]
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-td-flow
---
flowchart TD
    test["cargo test -p tape --test tape_perf_gate -- --nocapture"] --> budget["local regression budgets pass"]
    test --> ledger["external broker wins remain unclaimed"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/tape/src/bench.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "Local performance regression benchmark and external peer calibration ledger."
  - path: projects/tape/src/bench.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Named real-service replay win reporting for NATS JetStream and future calibrated peers."
  - path: projects/tape/src/bench.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Benchmark verification is exercised through the tape_perf_gate integration test."
```
