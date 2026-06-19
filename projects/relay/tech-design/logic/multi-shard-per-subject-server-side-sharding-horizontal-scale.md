---
id: relay-multi-shard
summary: Server-side multi-shard per subject — key engine state by (subject, shard), route publish by crc32(message_id) % default_shards, lease across shards, ack/heartbeat to the lease's shard, subscribe across shards. default_shards=1 is identical to today (backward compatible); >1 gives independent per-shard locks/logs for horizontal scale. Standalone.
fill_sections: [logic, unit-test, changes]
---

# relay multi-shard per subject (server-side sharding, horizontal scale)

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-multi-shard-flow
entry: op
nodes:
  op:
    kind: start
    label: "engine op on a subject (state keyed by (subject, shard); default_shards shards)"
  which:
    kind: decision
    label: "which op?"
  publish:
    kind: process
    label: "publish: shard = crc32(message_id) % default_shards; append to (subject, shard)'s own log (own seq space + lock)"
  lease:
    kind: process
    label: "lease: scan shards from a rotating start; return the first ready entry (whole subject drains across shards)"
  ackhb:
    kind: process
    label: "ack / heartbeat: route to the shard named in the lease_id; per-shard epoch fencing unchanged"
  sub:
    kind: process
    label: "subscribe: register a cursor on every shard; poll merges new entries from all shards (per-shard order preserved)"
  scale:
    kind: terminal
    label: "Different shards => different mutexes/logs => concurrent across cores. default_shards=1 => all shard 0 => identical to today"
edges:
  - { from: op, to: which }
  - { from: which, to: publish, label: "publish" }
  - { from: which, to: lease, label: "lease" }
  - { from: which, to: ackhb, label: "ack/heartbeat" }
  - { from: which, to: sub, label: "subscribe/poll" }
  - { from: publish, to: scale }
  - { from: lease, to: scale }
  - { from: ackhb, to: scale }
  - { from: sub, to: scale }
---
flowchart TD
    op([engine op on subject]) --> which{which op?}
    which -->|publish| publish[shard = crc32 id % shards; append to that shard]
    which -->|lease| lease[scan shards for next ready]
    which -->|ack/heartbeat| ackhb[route to lease_id's shard]
    which -->|subscribe| sub[cursor per shard; poll merges shards]
    publish --> scale([per-shard locks => scale; shards=1 => identical])
    lease --> scale
    ackhb --> scale
    sub --> scale
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-multi-shard-test-plan
entry: suite
nodes:
  suite: { kind: start, label: "multi-shard tests" }
  t_route: { kind: process, label: "default_shards=8; publish many distinct ids" }
  a_route: { kind: terminal, label: "assert messages spread across >1 shard (sum of per-shard lens = total; not all in one)" }
  t_drain: { kind: process, label: "default_shards=4; publish N; lease+ack until empty across shards" }
  a_drain: { kind: terminal, label: "assert every message delivered exactly once (whole subject drains across shards)" }
  t_bcast: { kind: process, label: "default_shards=4; publish N; subscribe from 0; poll" }
  a_bcast: { kind: terminal, label: "assert all N delivered (merged across shards), per-shard order preserved" }
  t_compat: { kind: process, label: "default_shards=1; publish 3, lease+ack, broadcast" }
  a_compat: { kind: terminal, label: "assert single shard 0, seqs 0,1,2, committed_seq=2 — identical to single-shard" }
edges:
  - { from: suite, to: t_route }
  - { from: t_route, to: a_route }
  - { from: suite, to: t_drain }
  - { from: t_drain, to: a_drain }
  - { from: suite, to: t_bcast }
  - { from: t_bcast, to: a_bcast }
  - { from: suite, to: t_compat }
  - { from: t_compat, to: a_compat }
---
flowchart TD
    suite([multi-shard suite]) --> t_route[shards=8, publish many]
    t_route --> a_route([spread across shards])
    suite --> t_drain[shards=4, lease+ack all]
    t_drain --> a_drain([exactly-once across shards])
    suite --> t_bcast[shards=4, subscribe 0]
    t_bcast --> a_bcast([all delivered, merged])
    suite --> t_compat[shards=1]
    t_compat --> a_compat([identical to single-shard])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/relay/src/engine.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Key subjects by (subject, shard); store shards = config.default_shards. publish/publish_batch route by crc32(message_id) % shards (reuse shard::shard_for); lease/lease_batch scan shards from a rotating start; ack/ack_batch/heartbeat route to the shard parsed from the lease_id (scan fallback); subscribe registers a cursor per shard and poll merges all shards; reconcile sweeps every (subject,shard). committed_offset/log_len aggregate over the subject's shards. default_shards=1 => shard 0 only => identical behavior."
  - path: projects/relay/tests/multi_shard.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: "Tests: publish spread across shards, whole-subject exactly-once drain across shards, broadcast merge across shards, and default_shards=1 parity with single-shard semantics."
```
