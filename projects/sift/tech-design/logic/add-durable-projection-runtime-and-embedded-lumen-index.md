---
id: "1660"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-projection-runtime
entry: mutation
nodes:
  mutation: { kind: start, label: "event or replay mutation" }
  command: { kind: process, label: "encode SiftCommandV1" }
  raft: { kind: process, label: "apply through the one SiftStateMachine" }
  raw: { kind: process, label: "fsync raw event or replay catalog transition" }
  notify: { kind: process, label: "notify asynchronous projection worker" }
  checkpoint: { kind: process, label: "load independent projection checkpoint and snapshot" }
  replay: { kind: process, label: "read ordered raw events after checkpoint" }
  apply: { kind: process, label: "idempotently apply projection batch" }
  lumen: { kind: process, label: "index allowlisted text keyword and range fields in embedded Lumen Engine" }
  persist: { kind: process, label: "atomically fsync snapshot plus checkpoint" }
  wake: { kind: terminal, label: "wake min cursor waiters" }
  query: { kind: start, label: "query with min_cursor" }
  caught_up: { kind: decision, label: "projection cursor reached minimum?" }
  lag: { kind: terminal, label: "projection_lag with current cursor and Retry-After" }
  rebuild: { kind: start, label: "POST /v1/replays" }
  fresh: { kind: process, label: "build fresh projection from raw cursor zero" }
  compare: { kind: decision, label: "fresh digest equals live digest at same cursor?" }
  swap: { kind: process, label: "atomically install rebuilt snapshot and checkpoint" }
  done: { kind: terminal, label: "durable replay job completed or failed" }
edges:
  - { from: mutation, to: command }
  - { from: command, to: raft }
  - { from: raft, to: raw }
  - { from: raw, to: notify }
  - { from: notify, to: checkpoint }
  - { from: checkpoint, to: replay }
  - { from: replay, to: apply }
  - { from: apply, to: lumen }
  - { from: lumen, to: persist }
  - { from: persist, to: wake }
  - { from: query, to: caught_up }
  - { from: caught_up, to: wake, label: "yes" }
  - { from: caught_up, to: lag, label: "timeout" }
  - { from: rebuild, to: fresh }
  - { from: fresh, to: compare }
  - { from: compare, to: swap, label: "yes" }
  - { from: compare, to: done, label: "no, record mismatch" }
  - { from: swap, to: done }
---
flowchart TD
    mutation([event or replay mutation]) --> command[encode SiftCommandV1]
    command --> raft[one SiftStateMachine]
    raft --> raw[durable raw or replay transition]
    raw --> notify[notify async projections]
    notify --> checkpoint[load projection state]
    checkpoint --> replay[read raw after cursor]
    replay --> apply[idempotent batch apply]
    apply --> lumen[embedded Lumen index]
    lumen --> persist[atomic snapshot and checkpoint]
    persist --> wake([wake min cursor waiters])
    query([query min_cursor]) --> caught_up{cursor reached?}
    caught_up -->|yes| wake
    caught_up -->|timeout| lag([projection_lag])
    rebuild([POST /v1/replays]) --> fresh[fresh projection from cursor zero]
    fresh --> compare{digest equality?}
    compare -->|yes| swap[atomic install]
    compare -->|no| done([durable failed status])
    swap --> done([durable completed status])
```
