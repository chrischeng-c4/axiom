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
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: relay-core-durable-log#schema
title: Relay Core Durable Log Types
description: >
  Core in-process data model for the relay broker: a durable ordered log per
  (subject, shard) plus the per-model delivery state that reads from it. The
  message payload reuses the cclab-queue message model unchanged; relay owns
  only the log, sequencing, dedupe, subscriber cursors, and work-queue leases.

definitions:
  Subject:
    type: string
    $id: Subject
    description: "Logical channel a producer publishes to and consumers subscribe on."

  ShardId:
    type: integer
    $id: ShardId
    minimum: 0
    description: "Partition of a subject's log; ordering and sequencing are per (subject, shard)."

  Seq:
    type: integer
    $id: Seq
    minimum: 0
    description: "Monotonic, gap-free position assigned on append within one (subject, shard). The replay and ack cursors are expressed in this space."

  MessageId:
    type: string
    $id: MessageId
    description: "Deterministic id derived from producer key + content, used as the idempotency/dedupe key so an at-least-once retry maps to the same log entry."

  DeliveryModel:
    type: string
    $id: DeliveryModel
    x-rust-derive: ["Debug", "Clone", "Copy", "PartialEq", "Eq", "Hash", "Serialize", "Deserialize"]
    enum:
      - singlecast
      - multicast
      - broadcast
      - work_queue
    description: >
      How a subject's appended messages are delivered. `broadcast`/`multicast`
      fan out every message to every (group) subscriber, replayable from a seq.
      `work_queue` leases each message to exactly one competing consumer.
      `singlecast` is the degenerate one-consumer case of work_queue.

  Payload:
    type: object
    $id: Payload
    x-rust-type: "cclab_queue::TaskMessage"
    description: "Reused cclab-queue message model. relay stores it verbatim as the durable payload and never reinterprets its fields."

  LogEntry:
    type: object
    $id: LogEntry
    x-rust-derive: ["Debug", "Clone", "PartialEq", "Serialize", "Deserialize"]
    required: [seq, message_id, subject, shard, payload, appended_at]
    description: "One durable record in the ordered log; the unit of both broadcast replay and work-queue lease."
    properties:
      seq:
        $ref: "#/definitions/Seq"
        description: "Monotonic position within (subject, shard)."
      message_id:
        $ref: "#/definitions/MessageId"
      subject:
        $ref: "#/definitions/Subject"
      shard:
        $ref: "#/definitions/ShardId"
      payload:
        $ref: "#/definitions/Payload"
      headers:
        type: object
        additionalProperties: { type: string }
        description: "Opaque routing/trace headers carried with the entry."
      appended_at:
        type: string
        format: date-time
        description: "Server time the entry was durably appended."

  AppendOutcome:
    type: object
    $id: AppendOutcome
    x-rust-derive: ["Debug", "Clone", "Copy", "PartialEq", "Eq", "Serialize", "Deserialize"]
    required: [seq, deduped]
    description: "Result of a publish/append; idempotent on MessageId."
    properties:
      seq:
        $ref: "#/definitions/Seq"
        description: "Seq of the (new or pre-existing) entry."
      deduped:
        type: boolean
        description: "True when the id was already present and no new entry was written."

  SubscriberCursor:
    type: object
    $id: SubscriberCursor
    x-rust-derive: ["Debug", "Clone", "PartialEq", "Eq", "Serialize", "Deserialize"]
    required: [subscriber_id, subject, shard, from_seq, position]
    description: "Broadcast/fan-out read position; each subscriber advances independently and may replay from any seq."
    properties:
      subscriber_id:
        type: string
        description: "Stable id of a broadcast subscriber (or member of a multicast group)."
      subject:
        $ref: "#/definitions/Subject"
      shard:
        $ref: "#/definitions/ShardId"
      from_seq:
        $ref: "#/definitions/Seq"
        description: "Seq the subscription (re)started replay from."
      position:
        $ref: "#/definitions/Seq"
        description: "Seq of the last entry delivered to this subscriber."

  Lease:
    type: object
    $id: Lease
    x-rust-derive: ["Debug", "Clone", "PartialEq", "Eq", "Serialize", "Deserialize"]
    required: [lease_id, seq, subject, shard, consumer_id, granted_at, expires_at, attempt]
    description: "Work-queue grant of one entry to exactly one consumer until it acks or the lease expires."
    properties:
      lease_id:
        type: string
        description: "Unique id for this grant; required to ack/extend."
      seq:
        $ref: "#/definitions/Seq"
        description: "Leased entry position."
      subject:
        $ref: "#/definitions/Subject"
      shard:
        $ref: "#/definitions/ShardId"
      consumer_id:
        type: string
        description: "Consumer the entry is currently leased to."
      granted_at:
        type: string
        format: date-time
      expires_at:
        type: string
        format: date-time
        description: "On expiry the entry becomes eligible for redelivery to another consumer."
      attempt:
        type: integer
        minimum: 1
        description: "1-based delivery attempt; drives retry / revocation policy."

  CommittedOffset:
    type: object
    $id: CommittedOffset
    x-rust-derive: ["Debug", "Clone", "Copy", "PartialEq", "Eq", "Serialize", "Deserialize"]
    required: [subject, shard, committed_seq]
    description: "Work-queue durable progress: every entry at or below committed_seq has been acked."
    properties:
      subject:
        $ref: "#/definitions/Subject"
      shard:
        $ref: "#/definitions/ShardId"
      committed_seq:
        $ref: "#/definitions/Seq"
```
## Config
<!-- type: config lang: yaml -->

```yaml
# RelayCoreConfig — in-process broker core engine settings.
# All durability/retention is local to this core; transport, sharding fan-out,
# and HA live in the server issues (#115 / #109) and are out of scope here.

# Durable ordered log substrate (RAM ring + disk segments).
data_dir: "./relay-data"        # root directory for durable disk segments
segment_bytes: 134217728        # roll to a new disk segment at 128 MiB
ram_ring_entries: 65536         # hot in-memory entries retained per (subject, shard) for low-latency fan-out / replay
fsync: "interval"               # durability policy: always | interval | os
fsync_interval_ms: 50           # flush cadence when fsync = interval
default_shards: 1               # shards per subject unless the subject overrides it

# Idempotent at-least-once append: how long a MessageId is remembered for dedupe.
dedupe:
  window_entries: 1048576       # MessageIds retained per shard for duplicate detection
  ttl_secs: 3600                # also evict dedupe keys older than this

# Work-queue / competing-consumer delivery (reuses the cclab-queue retry / revocation model).
work_queue:
  lease_ttl_ms: 30000           # lease duration before an unacked entry is redelivery-eligible
  max_attempts: 5               # redelivery attempts before revocation / dead-letter
  redeliver_backoff_ms: 1000    # base backoff between delivery attempts

# Broadcast / fan-out delivery (replayable from any retained seq).
broadcast:
  max_replay_entries: 0         # 0 = replay from any retained seq; >0 caps replay depth

# Retention / pruning of the durable log.
retention:
  max_age_secs: 604800          # prune fully-acked / aged segments after 7 days
  max_bytes_per_shard: 0        # 0 = unbounded; else prune oldest segments past this size
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
