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
entry: suite
nodes:
  suite: { kind: start, label: "default-durable throughput tests" }
  t_default: { kind: process, label: "inspect RelayCoreConfig::default()" }
  a_default: { kind: terminal, label: "assert data_dir is non-empty and fsync is power-safe (Always) -> durable by default" }
  t_batch: { kind: process, label: "publish_batch of N messages on a disk-backed relay" }
  a_batch: { kind: terminal, label: "assert N outcomes with seqs 0..N-1; a repeated id in the batch is deduped" }
  t_recover_log: { kind: process, label: "publish_batch N, drop the relay, reopen from the same data_dir" }
  a_recover_log: { kind: terminal, label: "assert all N entries recovered (durable log)" }
  t_recover_commit: { kind: process, label: "publish N, lease+ack the first K via ack-batch (persisted), drop, reopen" }
  a_recover_commit: { kind: terminal, label: "assert committed_seq recovered = K-1 and the next lease resumes at K (committed entries not redelivered)" }
edges:
  - { from: suite, to: t_default }
  - { from: t_default, to: a_default }
  - { from: suite, to: t_batch }
  - { from: t_batch, to: a_batch }
  - { from: suite, to: t_recover_log }
  - { from: t_recover_log, to: a_recover_log }
  - { from: suite, to: t_recover_commit }
  - { from: t_recover_commit, to: a_recover_commit }
---
flowchart TD
    suite([durable suite]) --> t_default[default config]
    t_default --> a_default([data_dir set + power-safe fsync])
    suite --> t_batch[publish_batch N]
    t_batch --> a_batch([N outcomes, dedupe in batch])
    suite --> t_recover_log[reopen after batch]
    t_recover_log --> a_recover_log([log recovered])
    suite --> t_recover_commit[ack K, reopen]
    t_recover_commit --> a_recover_commit([committed=K-1, resume at K])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/relay/src/log.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "append_many: write a batch of entries then issue ONE sync_all (group commit). Persist/load the committed-offset sidecar (<subject>__shardN.commit), group-committed."
  - path: projects/relay/src/workqueue.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "recover(committed): set next_offer and committed watermark on open so committed entries are never re-offered."
  - path: projects/relay/src/engine.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "publish_batch (group-commit append_many); recover the committed offset when a subject opens; persist the committed offset after ack / ack-batch."
  - path: projects/relay/src/config.rs
    action: modify
    section: config
    impl_mode: hand-written
    reason: "Default to durable, power-safe storage (data_dir set, fsync = Always); group commit makes the batch path cheap."
  - path: projects/relay/src/wire.rs
    action: modify
    section: schema
    impl_mode: hand-written
    reason: "PublishBatchItem / PublishBatchRequest / PublishBatchResponse DTOs."
  - path: projects/relay/src/server.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "POST /v1/{subject}/publish-batch handler (JSON + CBOR)."
  - path: projects/relay/src/openapi.rs
    action: modify
    section: rest-api
    impl_mode: hand-written
    reason: "Add the publish-batch path to the served OpenAPI."
  - path: projects/relay/tests/durable.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: "Tests: default config is durable, publish_batch group commit + dedupe, log recovery on reopen, and committed-offset crash recovery (resume after the committed offset)."
```
