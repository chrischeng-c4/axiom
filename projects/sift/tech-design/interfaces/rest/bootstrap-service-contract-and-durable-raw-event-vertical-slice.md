---
id: "1576"
summary: (fill)
fill_sections: [logic, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-bootstrap-ingest
entry: receive
nodes:
  receive: { kind: start, label: "POST /v1/events from SDK, OTLP producer, or GKE collector" }
  validate: { kind: decision, label: "versioned six-signal envelope and payload valid?" }
  reject: { kind: terminal, label: "return structured 400 without journal append" }
  dedupe: { kind: decision, label: "event_id already committed?" }
  duplicate: { kind: terminal, label: "return prior durable cursor idempotently" }
  append: { kind: process, label: "append canonical raw event; preserve direct metric points, temporality, and exemplars" }
  fsync: { kind: process, label: "fsync journal bytes before acknowledging" }
  accepted: { kind: terminal, label: "return 201 event_id and durable cursor; query and replay read the same journal" }
edges:
  - { from: receive, to: validate }
  - { from: validate, to: reject, label: "no" }
  - { from: validate, to: dedupe, label: "yes" }
  - { from: dedupe, to: duplicate, label: "yes" }
  - { from: dedupe, to: append, label: "no" }
  - { from: append, to: fsync }
  - { from: fsync, to: accepted }
---
flowchart TD
    receive([POST /v1/events]) --> validate{valid versioned envelope?}
    validate -->|no| reject([400 structured error])
    validate -->|yes| dedupe{event_id committed?}
    dedupe -->|yes| duplicate([return prior durable cursor])
    dedupe -->|no| append[append canonical raw event]
    append --> fsync[fsync journal]
    fsync --> accepted([201 event_id and durable cursor])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/sift/Cargo.toml
    action: create
    section: changes
    impl_mode: hand-written
    handwrite_gap: sift-service-manifest
    handwrite_tracker: "1576"
    description: Define the standalone Sift Rust service package and its runtime dependencies.
  - path: projects/sift/src/lib.rs
    action: create
    section: logic
    impl_mode: hand-written
    handwrite_gap: sift-service-core
    handwrite_tracker: "1576"
    description: Implement the versioned operational-event envelope, durable raw journal, idempotency, query, and replay core.
  - path: projects/sift/src/bin/sift.rs
    action: create
    section: logic
    impl_mode: hand-written
    handwrite_gap: sift-service-cli
    handwrite_tracker: "1576"
    description: Implement serve, event, query, replay, spec, llm, upgrade, and issue CLI surfaces.
  - path: projects/sift/tests/ingest_api.rs
    action: create
    section: logic
    impl_mode: hand-written
    handwrite_gap: sift-ingest-contract-tests
    handwrite_tracker: "1576"
    description: Verify ingest acknowledgement, duplicate idempotency, direct metric preservation, query, and replay.
```
