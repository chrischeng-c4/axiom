---
id: apps-tape-src-lib-rs
coverage_kind: semantic
capability_refs:
  - id: "topic-replay-journal"
    role: primary
    claim: "append-and-replay-contract"
    gap: "append-and-replay-contract"
    coverage: partial
    rationale: "TapeJournal owns ordered append plus offset/time replay for the first local service slice."
  - id: "consumer-checkpoints"
    role: primary
    claim: "durable-consumer-cursor-contract"
    gap: "durable-consumer-cursor-contract"
    coverage: partial
    rationale: "TapeJournal owns checkpoint create/read/advance and stale-write rejection."
  - id: "subscription-delivery-resources"
    role: primary
    claim: "topic-subscription-resource-contract"
    gap: "topic-subscription-resource-contract"
    coverage: partial
    rationale: "TapeJournal owns local topic subscription metadata and preserves pull checkpoint compatibility."
fill_sections: [overview, logic, unit-test, changes]
---

# Tape Local Journal Core

## Overview
<!-- type: overview lang: markdown -->

`apps/tape/src/lib.rs` defines the first Tape replay journal core:
`TapeJournal`, `TapeEvent`, `ConsumerCheckpoint`, and `Subscription`. It is local
and file-serializable, deliberately below the future h2c/raft/operator layers.
The core exposes owned replay for CLI/API callers and zero-copy `replay_refs`
for local backlog scan performance gates.

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-td-flow
---
flowchart TD
    append["append(topic, key, payload, timestamp_ms)"] --> offset["assign contiguous topic offset"]
    offset --> event["persist TapeEvent in topic vector"]
    replay["replay(topic, from_offset, from_timestamp_ms, limit)"] --> filter["filter by offset and optional timestamp"]
    replay_refs["replay_refs(topic, from_offset, from_timestamp_ms, limit)"] --> filter
    filter --> ordered["return events in append order"]
    ordered --> owned["replay returns cloned owned events"]
    ordered --> borrowed["replay_refs returns borrowed events for zero-copy local scans"]
    checkpoint["put_checkpoint(topic, consumer, offset)"] --> endcheck{"offset <= topic end?"}
    endcheck -->|no| beyond["CheckpointBeyondEnd"]
    endcheck -->|yes| stalecheck{"offset >= existing cursor?"}
    stalecheck -->|no| stale["StaleCheckpoint"]
    stalecheck -->|yes| store["store ConsumerCheckpoint"]
    subscription["create_subscription(topic, name, delivery)"] --> unique{"unique topic/name and valid push endpoint?"}
    unique -->|yes| substore["store Subscription metadata without moving checkpoint"]
    unique -->|no| suberror["SubscriptionError"]
    substore --> pull["pull name remains the existing checkpoint consumer identity"]
    substore --> push["push endpoint is stored only; no worker sends requests"]
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-td-flow
---
flowchart TD
    libtest["cargo test -p tape --lib -- --nocapture"] --> replaytest["append_and_replay_by_offset_and_time"]
    libtest --> checkpointtest["checkpoints_advance_and_reject_stale_offsets"]
    libtest --> subscriptiontest["pull_subscription_preserves_checkpoint_compatibility"]
    replaytest --> replayproof["offset and timestamp replay pass"]
    checkpointtest --> checkpointproof["advance, stale rejection, and beyond-end rejection pass"]
    subscriptiontest --> subscriptionproof["create/delete leaves topic/name checkpoint unchanged"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Initial local Tape journal and checkpoint core."
  - path: apps/tape/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add zero-copy replay_refs for local backlog scan performance gates."
  - path: apps/tape/src/lib.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Inline unit tests for local replay and checkpoint behavior."
  - path: apps/tape/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add local Subscription delivery metadata and CRUD; pull resources preserve existing checkpoints and push endpoints remain declarative (#1254)."
```
