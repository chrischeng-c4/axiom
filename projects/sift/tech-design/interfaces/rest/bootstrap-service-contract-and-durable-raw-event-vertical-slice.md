---
id: "1576"
summary: (fill)
fill_sections: [logic]
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
