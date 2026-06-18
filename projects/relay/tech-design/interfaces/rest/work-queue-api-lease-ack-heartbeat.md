---
id: relay-work-queue-api
summary: relay's work-queue face over HTTP/2 — lease (prefer redeliver) / ack / heartbeat with epoch fencing for exactly-one competing-consumer delivery. Generic; relay does not know workflows. Standalone.
fill_sections: [logic, schema, rest-api, unit-test, changes]
---

# relay work-queue API — lease / ack / heartbeat (epoch-fenced)

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-work-queue-api-flow
entry: req
nodes:
  req:
    kind: start
    label: "Worker calls lease / ack / heartbeat for a subject (HTTP/2, competing consumers)"
  which:
    kind: decision
    label: "Which verb?"
  lease_pick:
    kind: process
    label: "lease(consumer): pick a ready entry, PREFER a redeliver-eligible (previously attempted) seq over a fresh one"
  lease_grant:
    kind: process
    label: "Grant Lease(epoch = ++attempt for that seq, expires_at = now + ttl); record by lease_id"
  lease_none:
    kind: terminal
    label: "Nothing ready -> return null (worker backs off)"
  ack_check:
    kind: decision
    label: "ack(lease_id, epoch): lease_id known AND its epoch == epoch?"
  ack_ok:
    kind: terminal
    label: "Delete the lease, mark acked, advance the committed offset"
  ack_noop:
    kind: terminal
    label: "Epoch mismatch or unknown lease_id -> no-op (idempotent; fences a fenced/old worker)"
  hb_check:
    kind: decision
    label: "heartbeat(lease_id, epoch): known AND epoch matches?"
  hb_extend:
    kind: terminal
    label: "Extend expires_at = now + ttl (worker still alive)"
  hb_noop:
    kind: terminal
    label: "Mismatch -> no-op (lease already reclaimed / fenced)"
edges:
  - { from: req, to: which }
  - { from: which, to: lease_pick, label: "lease" }
  - { from: which, to: ack_check, label: "ack" }
  - { from: which, to: hb_check, label: "heartbeat" }
  - { from: lease_pick, to: lease_grant, label: "an entry is ready" }
  - { from: lease_pick, to: lease_none, label: "nothing ready" }
  - { from: ack_check, to: ack_ok, label: "match" }
  - { from: ack_check, to: ack_noop, label: "mismatch / unknown" }
  - { from: hb_check, to: hb_extend, label: "match" }
  - { from: hb_check, to: hb_noop, label: "mismatch" }
---
flowchart TD
    req([lease / ack / heartbeat]) --> which{verb?}
    which -->|lease| lease_pick[pick ready, prefer redeliver]
    lease_pick -->|ready| lease_grant[grant Lease epoch=++attempt]
    lease_pick -->|none| lease_none([null])
    which -->|ack| ack_check{lease_id + epoch match?}
    ack_check -->|yes| ack_ok([delete lease, advance committed])
    ack_check -->|no| ack_noop([no-op, idempotent/fenced])
    which -->|heartbeat| hb_check{known + epoch match?}
    hb_check -->|yes| hb_extend([extend expires_at])
    hb_check -->|no| hb_noop([no-op])
```
## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: relay-work-queue-api#schema
title: Relay Work-Queue API Types
description: >
  Epoch-fencing and heartbeat additions to the work-queue face. The lease grant
  reuses the core Lease (now carrying an epoch); ack and heartbeat carry the
  epoch so a fenced/old worker's late call is a no-op.

definitions:
  Epoch:
    type: integer
    $id: Epoch
    minimum: 1
    description: "Monotonic fencing token for a (subject, shard, seq): bumped on each (re)lease. ack/heartbeat with a stale epoch are no-ops."

  HeartbeatRequest:
    type: object
    $id: HeartbeatRequest
    x-rust-derive: ["Debug", "Clone", "Serialize", "Deserialize"]
    required: [lease_id, epoch]
    description: "Extend a held lease; proves the worker is alive."
    properties:
      lease_id: { type: string }
      epoch: { $ref: "#/definitions/Epoch" }

  HeartbeatResponse:
    type: object
    $id: HeartbeatResponse
    x-rust-derive: ["Debug", "Clone", "Serialize", "Deserialize"]
    required: [extended]
    description: "Whether the lease was extended (false when unknown / fenced)."
    properties:
      extended: { type: boolean }
      expires_at:
        oneOf:
          - { type: "null" }
          - { type: string, format: date-time }

  AckEpoch:
    type: object
    $id: AckEpoch
    description: "ack carries an optional epoch; when present it must match the live lease or the ack is a no-op (fenced). Absent epoch falls back to lease_id-only fencing."
    properties:
      lease_id: { type: string }
      epoch:
        oneOf:
          - { type: "null" }
          - { $ref: "#/definitions/Epoch" }
```
## Rest Api
<!-- type: rest-api lang: yaml -->

```yaml
openapi: 3.1.0
info:
  title: relay work-queue API
  version: 0.1.0
  description: >
    Competing-consumer work-queue verbs over HTTP/2 (h2c). Epoch-fenced:
    lease grants an epoch; ack/heartbeat carry it so a fenced worker's late
    call is a no-op. Generic — relay does not know workflows.
paths:
  /v1/{subject}/lease:
    post:
      operationId: lease
      summary: Lease the next ready entry (prefers a redeliver-eligible seq), returning a Lease with an epoch.
      parameters:
        - { name: subject, in: path, required: true, schema: { type: string } }
      requestBody:
        required: true
        content:
          application/json: { schema: { $ref: "#/components/schemas/LeaseRequest" } }
          application/cbor: { schema: { $ref: "#/components/schemas/LeaseRequest" } }
      responses:
        "200":
          description: A lease (with epoch) or null.
          content:
            application/json: { schema: { $ref: "#/components/schemas/LeaseResponse" } }
            application/cbor: { schema: { $ref: "#/components/schemas/LeaseResponse" } }
  /v1/{subject}/ack:
    post:
      operationId: ack
      summary: Acknowledge a lease; with a matching epoch it deletes the lease and advances the committed offset, otherwise it is a no-op (idempotent / fenced).
      parameters:
        - { name: subject, in: path, required: true, schema: { type: string } }
      requestBody:
        required: true
        content:
          application/json: { schema: { $ref: "#/components/schemas/AckRequest" } }
          application/cbor: { schema: { $ref: "#/components/schemas/AckRequest" } }
      responses:
        "200":
          description: Ack result.
          content:
            application/json: { schema: { $ref: "#/components/schemas/AckResponse" } }
            application/cbor: { schema: { $ref: "#/components/schemas/AckResponse" } }
  /v1/{subject}/heartbeat:
    post:
      operationId: heartbeat
      summary: Extend a held lease (epoch must match); no-op if the lease was reclaimed / fenced.
      parameters:
        - { name: subject, in: path, required: true, schema: { type: string } }
      requestBody:
        required: true
        content:
          application/json: { schema: { $ref: "#/components/schemas/HeartbeatRequest" } }
          application/cbor: { schema: { $ref: "#/components/schemas/HeartbeatRequest" } }
      responses:
        "200":
          description: Heartbeat result.
          content:
            application/json: { schema: { $ref: "#/components/schemas/HeartbeatResponse" } }
            application/cbor: { schema: { $ref: "#/components/schemas/HeartbeatResponse" } }
components:
  schemas:
    LeaseRequest:
      type: object
      required: [consumer_id]
      properties: { consumer_id: { type: string } }
    LeaseResponse:
      type: object
      properties:
        lease:
          oneOf:
            - { type: "null" }
            - { $ref: "#/components/schemas/Lease" }
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
    AckRequest:
      type: object
      required: [lease_id]
      properties:
        lease_id: { type: string }
        epoch:
          oneOf:
            - { type: "null" }
            - { type: integer, minimum: 1 }
    AckResponse:
      type: object
      required: [acked]
      properties:
        acked: { type: boolean }
        committed_seq:
          oneOf:
            - { type: "null" }
            - { type: integer, minimum: 0 }
    HeartbeatRequest:
      type: object
      required: [lease_id, epoch]
      properties:
        lease_id: { type: string }
        epoch: { type: integer, minimum: 1 }
    HeartbeatResponse:
      type: object
      required: [extended]
      properties:
        extended: { type: boolean }
        expires_at:
          oneOf:
            - { type: "null" }
            - { type: string, format: date-time }
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-work-queue-api-test-plan
entry: start
nodes:
  start:
    kind: start
    label: "pending"
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
