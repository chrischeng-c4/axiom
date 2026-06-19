---
id: relay-ha-replication
summary: HA via async leader/follower log replication — a follower tails the leader's broadcast subscribe stream per subject and re-applies each entry to its own durable log (idempotent on message_id), staying caught up and promotable. Async primary-backup; reqwest becomes a normal dep. Standalone (no axiom-project dep).
fill_sections: [logic, unit-test, changes]
---

# relay HA via leader/follower log replication (async primary-backup)

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-ha-flow
entry: spawn
nodes:
  spawn:
    kind: start
    label: "spawn_follower(local, leader_url, subjects): one task per subject"
  connect:
    kind: process
    label: "GET leader /v1/{subject}/subscribe (h2c) -> live stream of length-prefixed CBOR LogEntry frames"
  apply:
    kind: process
    label: "Decode frames; for each, local.publish(subject, message_id, payload, headers, appended_at)"
  idem:
    kind: decision
    label: "message_id already present locally?"
  skip:
    kind: terminal
    label: "Deduped (idempotent) -> follower mirrors the leader; reconnects/replay are safe"
  store:
    kind: terminal
    label: "Applied to the local durable log (deterministic routing => same shard/seq); follower stays caught up & promotable"
  reconnect:
    kind: process
    label: "On stream end / error: back off and reconnect (dedupe absorbs the replay)"
edges:
  - { from: spawn, to: connect }
  - { from: connect, to: apply, label: "frames" }
  - { from: apply, to: idem }
  - { from: idem, to: skip, label: "yes" }
  - { from: idem, to: store, label: "no" }
  - { from: connect, to: reconnect, label: "stream ended / error" }
  - { from: reconnect, to: connect }
---
flowchart TD
    spawn([spawn_follower per subject]) --> connect[GET leader subscribe stream h2c]
    connect --> apply[decode frames -> local.publish by message_id]
    apply --> idem{message_id present?}
    idem -->|yes| skip([deduped; mirror stays consistent])
    idem -->|no| store([applied to local durable log; promotable])
    connect --> reconnect[stream end/error: backoff + reconnect]
    reconnect --> connect
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-ha-test-plan
entry: suite
nodes:
  suite: { kind: start, label: "HA replication tests (real h2c leader)" }
  t_conv: { kind: process, label: "publish N to a leader; point a follower's replicator at it; wait" }
  a_conv: { kind: terminal, label: "assert the follower converges to the same N entries (same message_ids) and can serve poll/log_len" }
  t_live: { kind: process, label: "publish more to the leader after the follower is connected" }
  a_live: { kind: terminal, label: "assert the new entries replicate to the follower" }
  t_idem: { kind: process, label: "let the follower keep tailing / re-apply" }
  a_idem: { kind: terminal, label: "assert no duplicates (dedupe by message_id keeps log_len == N)" }
edges:
  - { from: suite, to: t_conv }
  - { from: t_conv, to: a_conv }
  - { from: suite, to: t_live }
  - { from: t_live, to: a_live }
  - { from: suite, to: t_idem }
  - { from: t_idem, to: a_idem }
---
flowchart TD
    suite([HA suite]) --> t_conv[leader N; follower replicates]
    t_conv --> a_conv([follower converges, serves reads])
    suite --> t_live[publish more after connect]
    t_live --> a_live([new entries replicate])
    suite --> t_idem[keep tailing]
    t_idem --> a_idem([no duplicates])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/relay/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Move reqwest from dev-dependencies to dependencies (the follower is an HTTP/2 client at runtime)."
  - path: projects/relay/src/replication.rs
    action: create
    section: logic
    impl_mode: hand-written
    reason: "spawn_follower(local, leader_url, subjects) -> FollowerHandle: one tokio task per subject that tails the leader's subscribe stream over h2c, decodes length-prefixed CBOR LogEntry frames, and re-applies each via local.publish (idempotent on message_id); reconnects with backoff on stream end/error. FollowerHandle aborts the tasks on stop/drop."
  - path: projects/relay/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Declare and re-export the replication module (spawn_follower, FollowerHandle)."
  - path: projects/relay/tests/replication.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: "Real-h2c HA test: a follower replicator converges to a leader's N entries, replicates entries published after connect, serves reads, and does not duplicate on continued tailing."
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Per-subject follower tails the leader subscribe stream and re-applies by message_id (idempotent => mirror + safe reconnect); deterministic routing keeps shard/seq aligned; promotable. Async primary-backup, clearly scoped vs full Raft. Applicable.
- [unit-test] Convergence to N, live replication after connect, serve reads, no-duplicate on retail. Applicable.
- [changes] reqwest -> normal dep, replication.rs follower + handle, lib re-export, a real-h2c test. Applicable.
