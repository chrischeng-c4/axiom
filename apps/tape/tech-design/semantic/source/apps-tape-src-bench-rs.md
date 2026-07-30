---
id: apps-tape-src-bench-rs
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

`apps/tape/src/bench.rs` defines the first competitive-performance evidence
surface. It measures local append, replay, and checkpoint operations and also
models named real-service replay wins such as the NATS JetStream local backlog
gate. Other replay-log peer wins stay unclaimed until their own real-service
peer runs are added.

WI #3052 AC1 adds `run_durable_benchmark`: a second, independent benchmark
entry point that drives the real `WalStore` + `CommitCoordinator` group-commit
durable path over real HTTP (a real `127.0.0.1:0` socket, not an in-memory
journal call) at varying connection counts, proving durable append throughput
rises with connection count instead of staying flat at the pre-#3052 85-89
ops/s line. It never touches or weakens `run_benchmark`'s in-memory numbers.

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

    durable["run_durable_benchmark(events_per_connection, payload_bytes, connection_counts)"] --> sample["per connection count: fresh TempDir -> WalStore::open -> CommitCoordinator::spawn -> AppState::with_wal -> router -> bind 127.0.0.1:0"]
    sample --> drive["N concurrent client tasks, each POSTing events_per_connection sequential /topics/{topic}/append requests"]
    drive --> measure["ops_per_sec = events / elapsed"]
    measure --> ratio2["scaling_ratio = ops_per_sec(max conns) / ops_per_sec(min conns)"]
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
    test --> durable["durable_append_throughput_rises_with_connection_count asserts scaling_ratio >= 4.0x (16 vs 1 connections)"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/src/bench.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "Local performance regression benchmark and external peer calibration ledger."
  - path: apps/tape/src/bench.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Named real-service replay win reporting for NATS JetStream and future calibrated peers."
  - path: apps/tape/src/bench.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Benchmark verification is exercised through the tape_perf_gate integration test."
  - path: apps/tape/src/bench.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "WI #3052 AC1: run_durable_benchmark drives the real WalStore/CommitCoordinator group-commit path over real HTTP at varying connection counts, reporting per-connection ops/s and the scaling ratio."
```
