---
id: relay-core-durable-log
summary: In-process broker core that serves both broadcast (replay from seq) and work-queue (lease / ack / redeliver) delivery over one durable ordered log per subject/shard, reusing the cclab-queue message / routing / retry / revocation models.
fill_sections: [logic, schema, config, unit-test]
---

# relay core — durable log + single/multi/broadcast delivery model

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-core-durable-log-delivery-flow
entry: publish
nodes:
  publish:
    kind: start
    label: "Producer publishes a message to a subject with a delivery intent (singlecast / multicast / broadcast / work-queue)"
  assign_id:
    kind: process
    label: "Derive a deterministic message id (producer key + content) so retries are idempotent"
  dedupe:
    kind: decision
    label: "Has this id already been appended on this subject/shard?"
  dedupe_drop:
    kind: terminal
    label: "Drop the duplicate and return the existing seq (at-least-once, idempotent)"
  append_log:
    kind: process
    label: "Append to the durable ordered log for the subject/shard (RAM ring + disk segment) and assign a monotonic seq"
  classify:
    kind: decision
    label: "Resolve delivery model for the subject via the reused routing model"
  fanout:
    kind: process
    label: "Broadcast/fan-out: every subscriber cursor advances; each subscriber gets every message in order"
  replay:
    kind: terminal
    label: "A (re)connecting subscriber replays from its requested from_seq over the same durable log"
  lease:
    kind: process
    label: "Work-queue/competing: offer the message to exactly one available consumer under a lease"
  lease_ok:
    kind: decision
    label: "Did the leased consumer ack before the lease expired?"
  commit_ack:
    kind: terminal
    label: "Ack: mark the message delivered and advance the committed offset"
  redeliver:
    kind: process
    label: "Lease expiry or nack: requeue for redelivery to another consumer (reuse retry / revocation model)"
edges:
  - from: publish
    to: assign_id
    label: "accept publish"
  - from: assign_id
    to: dedupe
    label: "id derived"
  - from: dedupe
    to: dedupe_drop
    label: "duplicate id"
  - from: dedupe
    to: append_log
    label: "new id"
  - from: append_log
    to: classify
    label: "seq assigned, durably persisted"
  - from: classify
    to: fanout
    label: "broadcast / multicast"
  - from: classify
    to: lease
    label: "work-queue / singlecast"
  - from: fanout
    to: replay
    label: "subscriber subscribes from_seq"
  - from: lease
    to: lease_ok
    label: "awaiting ack"
  - from: lease_ok
    to: commit_ack
    label: "acked in time"
  - from: lease_ok
    to: redeliver
    label: "lease expired or nacked"
  - from: redeliver
    to: lease
    label: "re-offer to another consumer"
---
flowchart TD
    publish([Producer publishes to subject]) --> assign_id[Derive deterministic message id]
    assign_id --> dedupe{Already appended?}
    dedupe -->|duplicate id| dedupe_drop([Drop, return existing seq])
    dedupe -->|new id| append_log[Append to durable ordered log, assign seq]
    append_log --> classify{Delivery model?}
    classify -->|broadcast / multicast| fanout[Fan-out to every subscriber cursor]
    classify -->|work-queue / singlecast| lease[Lease to exactly one consumer]
    fanout --> replay([Subscriber replays from_seq])
    lease --> lease_ok{Acked before lease expiry?}
    lease_ok -->|yes| commit_ack([Ack, advance committed offset])
    lease_ok -->|lease expired / nack| redeliver[Requeue for redelivery]
    redeliver --> lease
```
## Schema
<!-- type: schema lang: yaml -->

```yaml
(fill)
```

## Config
<!-- type: config lang: yaml -->

```yaml
(fill)
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-core-unit-test-plan
entry: start
nodes:
  start:
    kind: start
    label: "Unit test plan pending — to be authored in its own applicability section"
edges: []
---
flowchart TD
    start([unit-test plan pending])
```
