---
id: relay-work-queue-throughput
summary: Work-queue throughput rework — per-(subject,shard) locking for real consumer concurrency, an O(1) next-eligible lease cursor (redeliver min-heap + next_offer + committed watermark) replacing the O(n) scan, and batch lease/ack endpoints to amortize HTTP round-trips. Same exactly-once / epoch-fencing semantics. Standalone.
fill_sections: [logic, schema, rest-api, unit-test, changes]
---

# relay work-queue throughput — per-shard lock + O(1) lease cursor + batch lease/ack

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-work-queue-throughput-flow
entry: req
nodes:
  req:
    kind: start
    label: "Worker calls lease / ack (single or batch) on a subject"
  resolve:
    kind: process
    label: "Resolve (subject, shard) and lock ONLY that shard's mutex (other shards proceed concurrently)"
  which:
    kind: decision
    label: "lease or ack?"
  pick:
    kind: process
    label: "O(1) pick: pop the redeliver min-heap if non-empty (prefer redeliver), else take next_offer and increment it"
  grant:
    kind: process
    label: "Grant Lease(epoch = ++attempt); for lease-batch, repeat up to `max`"
  ack:
    kind: process
    label: "ack(lease_id, epoch): epoch-checked delete; insert into acked; for ack-batch, repeat over the list"
  watermark:
    kind: process
    label: "Advance the committed watermark incrementally (amortized O(1)) while acked contains the next seq"
  reclaim:
    kind: process
    label: "Reconciler: an expired lease pushes its seq back onto the redeliver heap (bumped epoch fences the old worker)"
  done:
    kind: terminal
    label: "Return grants / committed offset; lock released"
edges:
  - { from: req, to: resolve }
  - { from: resolve, to: which }
  - { from: which, to: pick, label: "lease" }
  - { from: pick, to: grant }
  - { from: grant, to: done }
  - { from: which, to: ack, label: "ack" }
  - { from: ack, to: watermark }
  - { from: watermark, to: done }
  - { from: reclaim, to: pick, label: "redelivery-eligible seq re-offered O(1)" }
---
flowchart TD
    req([lease / ack, single or batch]) --> resolve[lock only this shard]
    resolve --> which{lease or ack?}
    which -->|lease| pick[O(1): redeliver-heap else next_offer++]
    pick --> grant[grant epoch lease, up to max]
    grant --> done([return; unlock])
    which -->|ack| ack[epoch-checked delete + acked.insert]
    ack --> watermark[advance committed watermark amortized O(1)]
    watermark --> done
    reclaim[reconciler: expired -> push seq to redeliver heap] --> pick
```
## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: relay-work-queue-throughput#schema
title: Relay Work-Queue Batch Types
description: >
  Batch lease / ack DTOs that amortize HTTP round-trips. A worker leases up to
  `max` entries in one call and acks many in one call. Single lease/ack (from
  #113) are unchanged; the core Lease (with its epoch) is reused.

definitions:
  LeaseBatchRequest:
    type: object
    $id: LeaseBatchRequest
    x-rust-derive: ["Debug", "Clone", "Serialize", "Deserialize"]
    required: [consumer_id, max]
    properties:
      consumer_id: { type: string }
      max: { type: integer, minimum: 1, description: "Maximum entries to lease in this call." }

  LeaseBatchResponse:
    type: object
    $id: LeaseBatchResponse
    x-rust-derive: ["Debug", "Clone", "Serialize", "Deserialize"]
    required: [leases]
    description: "Up to `max` granted leases, in seq order; empty when nothing is ready."
    properties:
      leases:
        type: array
        items: { x-rust-type: "crate::types::Lease" }

  AckOne:
    type: object
    $id: AckOne
    x-rust-derive: ["Debug", "Clone", "Serialize", "Deserialize"]
    required: [lease_id]
    properties:
      lease_id: { type: string }
      epoch:
        oneOf: [{ type: "null" }, { type: integer, minimum: 1 }]

  AckBatchRequest:
    type: object
    $id: AckBatchRequest
    x-rust-derive: ["Debug", "Clone", "Serialize", "Deserialize"]
    required: [acks]
    properties:
      acks:
        type: array
        items: { $ref: "#/definitions/AckOne" }

  AckBatchResponse:
    type: object
    $id: AckBatchResponse
    x-rust-derive: ["Debug", "Clone", "Serialize", "Deserialize"]
    required: [acked]
    description: "How many of the batch were accepted, plus the resulting committed offset."
    properties:
      acked: { type: integer, minimum: 0 }
      committed_seq:
        oneOf: [{ type: "null" }, { type: integer, minimum: 0 }]
```
## Rest Api
<!-- type: rest-api lang: yaml -->

```yaml
openapi: 3.1.0
info:
  title: relay work-queue batch API
  version: 0.1.0
  description: >
    Batch lease / ack over HTTP/2 to amortize round-trips. Same epoch fencing
    and exactly-once semantics as the single-message verbs (#113).
paths:
  /v1/{subject}/lease-batch:
    post:
      operationId: leaseBatch
      summary: Lease up to `max` ready entries in one call (prefers redelivery), each with an epoch.
      parameters:
        - { name: subject, in: path, required: true, schema: { type: string } }
      requestBody:
        required: true
        content:
          application/json: { schema: { $ref: "#/components/schemas/LeaseBatchRequest" } }
          application/cbor: { schema: { $ref: "#/components/schemas/LeaseBatchRequest" } }
      responses:
        "200":
          description: Up to max leases in seq order (possibly empty).
          content:
            application/json: { schema: { $ref: "#/components/schemas/LeaseBatchResponse" } }
            application/cbor: { schema: { $ref: "#/components/schemas/LeaseBatchResponse" } }
  /v1/{subject}/ack-batch:
    post:
      operationId: ackBatch
      summary: Acknowledge many leases in one call; advances the committed offset.
      parameters:
        - { name: subject, in: path, required: true, schema: { type: string } }
      requestBody:
        required: true
        content:
          application/json: { schema: { $ref: "#/components/schemas/AckBatchRequest" } }
          application/cbor: { schema: { $ref: "#/components/schemas/AckBatchRequest" } }
      responses:
        "200":
          description: Count accepted + committed offset.
          content:
            application/json: { schema: { $ref: "#/components/schemas/AckBatchResponse" } }
            application/cbor: { schema: { $ref: "#/components/schemas/AckBatchResponse" } }
components:
  schemas:
    LeaseBatchRequest:
      type: object
      required: [consumer_id, max]
      properties:
        consumer_id: { type: string }
        max: { type: integer, minimum: 1 }
    LeaseBatchResponse:
      type: object
      required: [leases]
      properties:
        leases: { type: array, items: { $ref: "#/components/schemas/Lease" } }
    Lease:
      type: object
      required: [lease_id, seq, subject, shard, consumer_id, granted_at, expires_at, attempt, epoch]
      properties:
        lease_id: { type: string }
        seq: { type: integer, minimum: 0 }
        subject: { type: string }
        shard: { type: integer, minimum: 0 }
        consumer_id: { type: string }
        granted_at: { type: string, format: date-time }
        expires_at: { type: string, format: date-time }
        attempt: { type: integer, minimum: 1 }
        epoch: { type: integer, minimum: 1 }
    AckOne:
      type: object
      required: [lease_id]
      properties:
        lease_id: { type: string }
        epoch: { oneOf: [{ type: "null" }, { type: integer, minimum: 1 }] }
    AckBatchRequest:
      type: object
      required: [acks]
      properties:
        acks: { type: array, items: { $ref: "#/components/schemas/AckOne" } }
    AckBatchResponse:
      type: object
      required: [acked]
      properties:
        acked: { type: integer, minimum: 0 }
        committed_seq: { oneOf: [{ type: "null" }, { type: integer, minimum: 0 }] }
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-wqt-test-plan
entry: start
nodes:
  start: { kind: start, label: "pending" }
edges: []
---
flowchart TD
    start([pending])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
(fill)
```
