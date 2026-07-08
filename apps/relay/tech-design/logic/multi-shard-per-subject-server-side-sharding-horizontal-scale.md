---
id: relay-multi-shard
summary: Server-side multi-shard per subject — key engine state by (subject, shard), route publish by crc32(message_id) % default_shards, lease scans shards (whole subject drains exactly-once), ack/heartbeat/release route by scanning shards for the lease_id. default_shards=1 is identical to today (backward compatible); >1 gives independent per-shard locks/logs for horizontal scale. Standalone.
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
    label: "ack / heartbeat / release: scan shards for the one owning the lease_id; per-shard epoch fencing unchanged"
  reconcile:
    kind: process
    label: "reconcile: sweep every (subject, shard) independently for expired leases"
  scale:
    kind: terminal
    label: "Different shards => different mutexes/logs => concurrent across cores. default_shards=1 => all shard 0 => identical to today"
edges:
  - { from: op, to: which }
  - { from: which, to: publish, label: "publish" }
  - { from: which, to: lease, label: "lease" }
  - { from: which, to: ackhb, label: "ack/heartbeat/release" }
  - { from: which, to: reconcile, label: "reconcile" }
  - { from: publish, to: scale }
  - { from: lease, to: scale }
  - { from: ackhb, to: scale }
  - { from: reconcile, to: scale }
---
flowchart TD
    op([engine op on subject]) --> which{which op?}
    which -->|publish| publish[shard = crc32 id % shards; append to that shard]
    which -->|lease| lease[scan shards for next ready]
    which -->|ack/heartbeat/release| ackhb[scan shards for lease_id's owner]
    which -->|reconcile| reconcile[sweep every subject,shard]
    publish --> scale([per-shard locks => scale; shards=1 => identical])
    lease --> scale
    ackhb --> scale
    reconcile --> scale
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-multi-shard-test-plan
entry: suite
nodes:
  suite: { kind: start, label: "multi-shard tests" }
  t_drain: { kind: process, label: "default_shards=4; publish 200 distinct ids; lease+ack until empty" }
  a_drain: { kind: terminal, label: "assert log_len=200; each (shard,seq) leased exactly once; messages spread across >1 shard; whole subject drained" }
  t_compat: { kind: process, label: "default_shards=1; publish 3, lease+ack all" }
  a_compat: { kind: terminal, label: "assert every lease.shard=0, seqs 0,1,2, committed_offset=2 — identical to single-shard" }
edges:
  - { from: suite, to: t_drain }
  - { from: t_drain, to: a_drain }
  - { from: suite, to: t_compat }
  - { from: t_compat, to: a_compat }
---
flowchart TD
    suite([multi-shard suite]) --> t_drain[shards=4, publish 200, drain]
    t_drain --> a_drain([spread across shards; exactly-once drain])
    suite --> t_compat[shards=1]
    t_compat --> a_compat([identical to single-shard])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/relay/src/engine.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Key subjects by (subject, shard); store shards = config.default_shards. publish/publish_batch route by crc32(message_id) % shards (reuse shard::shard_for); lease/lease_batch scan shards from a rotating start (whole subject drains exactly-once); ack/ack_batch/heartbeat/release route by scanning shards for the shard owning the lease_id; reconcile sweeps every (subject,shard) independently. committed_offset reads shard 0 (offsets are per-shard). default_shards=1 => shard 0 only => identical behavior."
  - path: apps/relay/tests/multi_shard.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: "Tests: publish spread across shards with whole-subject exactly-once drain through lease/ack, and default_shards=1 parity with single-shard semantics."
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] (subject,shard) keying + crc32(message_id) routing gives independent per-shard logs/locks; lease scans shards, ack/heartbeat/release route by scanning shards for the lease_id's owner, reconcile sweeps all (subject,shard). default_shards=1 collapses to shard 0 -> identical. Coherent horizontal-scale model for a single-cast work queue.
- [unit-test] routing spread + exactly-once drain across shards, and shards=1 parity.
- [changes] engine.rs + a new test; reuses default_shards + shard::shard_for.
