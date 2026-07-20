---
id: apps-tape-tests-tape-vs-nats-jetstream-rs
coverage_kind: semantic
capability_refs:
  - id: "competitor-performance"
    role: primary
    claim: "topic-replay-competitor-performance-baseline"
    gap: "topic-replay-competitor-performance-baseline"
    coverage: partial
    rationale: "The test is the real-service EC gate that proves Tape beats NATS JetStream on local backlog full replay."
fill_sections: [overview, unit-test, changes]
---

# Tape Versus NATS JetStream Replay Test

## Overview
<!-- type: overview lang: markdown -->

`apps/tape/tests/tape_vs_nats_jetstream.rs` starts real release Tape h2c and
local `nats-server -js` services, prepares the same 20,000-event,
128-byte-payload durable backlog, and validates five full replay samples across
both network clients. The p50 gate permits a claim only when Tape is at least
1.5x faster; the report also captures p95/p99, throughput, CPU/RSS, durable
bytes/amplification, and errors.

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-td-flow
---
flowchart TD
    test["cargo test --release -p tape --test tape_vs_nats_jetstream -- --nocapture"] --> nats["spawn nats-server -js"]
    test --> tape["spawn release Tape h2c from durable backlog"]
    nats --> replay_nats["measure five JetStream full-backlog samples"]
    tape --> replay_tape["download and validate five Tape replay samples"]
    replay_nats --> ratio["peer p50 / Tape p50"]
    replay_tape --> ratio
    ratio --> gate{"ratio >= 1.5x"}
    gate -->|yes| pass["NATS JetStream replay win claim allowed"]
    gate -->|no| fail["fail competitor-performance EC"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/tests/tape_vs_nats_jetstream.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Real NATS JetStream competitor benchmark for Tape local backlog replay performance."
```
