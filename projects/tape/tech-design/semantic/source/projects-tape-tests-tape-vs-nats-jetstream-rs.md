---
id: projects-tape-tests-tape-vs-nats-jetstream-rs
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

`projects/tape/tests/tape_vs_nats_jetstream.rs` starts a real local
`nats-server -js`, publishes a 20,000-event, 128-byte-payload JetStream backlog,
and compares full replay of that backlog against Tape's zero-copy local journal
replay. The gate permits a performance win claim only when Tape is at least 1.5x
faster for this named workload.

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-td-flow
---
flowchart TD
    test["cargo test -p tape --test tape_vs_nats_jetstream -- --nocapture"] --> nats["spawn nats-server -js"]
    test --> tape["append same local Tape backlog"]
    nats --> replay_nats["measure JetStream full backlog replay"]
    tape --> replay_tape["measure Tape full backlog replay"]
    replay_nats --> ratio["peer_us / tape_us"]
    replay_tape --> ratio
    ratio --> gate{"ratio >= 1.5x"}
    gate -->|yes| pass["NATS JetStream replay win claim allowed"]
    gate -->|no| fail["fail competitor-performance EC"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/tape/tests/tape_vs_nats_jetstream.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Real NATS JetStream competitor benchmark for Tape local backlog replay performance."
```
