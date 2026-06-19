---
id: relay-default-durable-throughput
summary: Default-durable engine throughput — group-commit fsync (append_many issues one sync_all per batch), publish-batch endpoint, and a persisted committed-offset sidecar recovered on open. Durability is power-safe by default; batched produce/ack amortize the fsync so relay beats JetStream / RabbitMQ. Standalone.
fill_sections: [logic, schema, rest-api, unit-test, changes]
---

# relay default-durable engine throughput — group-commit fsync + publish-batch + persisted committed offset

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-default-durable-flow
entry: open
nodes:
  open:
    kind: start
    label: "Open (subject, shard): replay the NDJSON log, then load the committed-offset sidecar"
  recover:
    kind: process
    label: "WorkQueue.recover(committed): next_offer = committed, so entries <= committed are never re-offered; uncommitted entries redeliver (at-least-once)"
  ready:
    kind: decision
    label: "Request?"
  pub_batch:
    kind: process
    label: "publish-batch: append_many writes every entry's NDJSON line, then ONE sync_all (group commit) -> all durable with a single fsync"
  ack_batch:
    kind: process
    label: "ack-batch: epoch-checked acks advance the committed watermark"
  persist:
    kind: process
    label: "Persist the new committed watermark to the sidecar (group-committed: one write + fsync per ack-batch)"
  done:
    kind: terminal
    label: "Return outcomes / committed offset — durably power-safe by default"
edges:
  - { from: open, to: recover }
  - { from: recover, to: ready }
  - { from: ready, to: pub_batch, label: "publish-batch" }
  - { from: pub_batch, to: done }
  - { from: ready, to: ack_batch, label: "ack-batch" }
  - { from: ack_batch, to: persist }
  - { from: persist, to: done }
---
flowchart TD
    open([open: replay log + load committed]) --> recover[recover: next_offer = committed]
    recover --> ready{request?}
    ready -->|publish-batch| pub_batch[append_many: N lines + ONE fsync]
    pub_batch --> done([durable outcomes])
    ready -->|ack-batch| ack_batch[advance committed watermark]
    ack_batch --> persist[persist committed: 1 write + fsync]
    persist --> done
```
## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: relay-default-durable#schema
title: Relay Publish-Batch Types
description: >
  Batch-publish DTOs for the group-commit produce path. One request carries many
  messages; the server appends them all and issues a single fsync (group commit),
  returning one AppendOutcome per message in order.

definitions:
  PublishBatchItem:
    type: object
    $id: PublishBatchItem
    x-rust-derive: ["Debug", "Clone", "Serialize", "Deserialize"]
    required: [message_id, payload]
    properties:
      message_id: { type: string, description: "Idempotency / dedupe key." }
      payload: { description: "Opaque message body (any JSON value)." }
      headers:
        type: object
        additionalProperties: { type: string }

  PublishBatchRequest:
    type: object
    $id: PublishBatchRequest
    x-rust-derive: ["Debug", "Clone", "Serialize", "Deserialize"]
    required: [messages]
    properties:
      messages:
        type: array
        items: { $ref: "#/definitions/PublishBatchItem" }

  PublishBatchResponse:
    type: object
    $id: PublishBatchResponse
    x-rust-derive: ["Debug", "Clone", "Serialize", "Deserialize"]
    required: [outcomes]
    description: "One AppendOutcome per input message, in order."
    properties:
      outcomes:
        type: array
        items: { x-rust-type: "crate::types::AppendOutcome" }
```
## Rest Api
<!-- type: rest-api lang: yaml -->

```yaml
openapi: 3.1.0
info:
  title: relay publish-batch API
  version: 0.1.0
  description: >
    Group-commit batch produce over HTTP/2. One request carries many messages;
    the server appends them and issues a single fsync (durable, power-safe).
paths:
  /v1/{subject}/publish-batch:
    post:
      operationId: publishBatch
      summary: Append many messages in one durable, group-committed call.
      parameters:
        - { name: subject, in: path, required: true, schema: { type: string } }
      requestBody:
        required: true
        content:
          application/json: { schema: { $ref: "#/components/schemas/PublishBatchRequest" } }
          application/cbor: { schema: { $ref: "#/components/schemas/PublishBatchRequest" } }
      responses:
        "200":
          description: One AppendOutcome per message, in order.
          content:
            application/json: { schema: { $ref: "#/components/schemas/PublishBatchResponse" } }
            application/cbor: { schema: { $ref: "#/components/schemas/PublishBatchResponse" } }
components:
  schemas:
    PublishBatchItem:
      type: object
      required: [message_id, payload]
      properties:
        message_id: { type: string }
        payload: {}
        headers: { type: object, additionalProperties: { type: string } }
    PublishBatchRequest:
      type: object
      required: [messages]
      properties:
        messages: { type: array, items: { $ref: "#/components/schemas/PublishBatchItem" } }
    AppendOutcome:
      type: object
      required: [seq, deduped]
      properties:
        seq: { type: integer, minimum: 0 }
        deduped: { type: boolean }
    PublishBatchResponse:
      type: object
      required: [outcomes]
      properties:
        outcomes: { type: array, items: { $ref: "#/components/schemas/AppendOutcome" } }
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-default-durable-test-plan
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
