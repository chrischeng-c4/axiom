---
id: "1660"
summary: Add the one-process asynchronous projection runtime, atomically durable per-projection state/checkpoints, replay jobs through SiftStateMachine, min-cursor waits, and a rebuildable embedded Lumen event index.
capability_refs:
  - id: materialized-observability-stores
    role: primary
    gap: projection-runtime-and-checkpoints
    claim: projection-runtime-and-checkpoints
    coverage: full
    rationale: This slice establishes the independent durable projection lifecycle consumed by every later domain store.
  - id: raw-event-journal-and-archive
    role: primary
    gap: replayable-view-rebuild
    claim: replayable-view-rebuild
    coverage: full
    rationale: Rebuild jobs consume only ordered canonical raw events and prove fresh/live digest equality before atomic install.
  - id: shard-aware-hot-storage
    role: contributes
    gap: rebuildable-hot-index
    claim: rebuildable-hot-index
    coverage: full
    rationale: Lumen is linked as an in-process derived index with no service, network, WAL, or Raft ownership.
  - id: schema-governance
    role: contributes
    gap: projection-index-allowlist
    claim: projection-index-allowlist
    coverage: full
    rationale: The embedded event index declares a fixed field allowlist and never indexes arbitrary payload or attribute keys.
  - id: query-tail-and-replay
    role: contributes
    gap: replay-cursor-and-view-rebuild
    claim: replay-cursor-and-view-rebuild
    coverage: partial
    rationale: This slice owns durable replay start/status and projection-lag primitives; the typed query/tail CLI and API remain #1671.
fill_sections: [logic, schema, rest-api, unit-test, changes]
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

## Schema
<!-- type: schema lang: yaml -->

```yaml
constants:
  projection_batch_size: 1000
  projection_wait_timeout_ms: 2000
  projection_retry_after_seconds: 1
  projection_state_format_version: 1
  command_format_version: 1
schemas:
  - name: SiftCommandV1
    kind: tagged_enum
    tag: kind
    variants:
      - name: append_event
        fields:
          - { name: event, type: OperationalEventV2, required: true }
      - name: upsert_replay_job
        fields:
          - { name: job, type: ReplayJob, required: true }
    compatibility: Bare OperationalEventV1/V2 Raft commands remain readable and upcast to append_event.
  - name: AppendResult
    fields:
      - { name: event_id, type: String, required: true }
      - { name: cursor, type: u64, required: true, description: Retained compatibility alias for raw_cursor. }
      - { name: raw_cursor, type: u64, required: true }
      - { name: commit_index, type: u64, required: true }
      - { name: duplicate, type: bool, required: true }
  - name: SiftControlState
    persistence: sift-control-state.json via service_durability atomic_write Always
    fields:
      - { name: format_version, type: u16, required: true }
      - { name: applied_index, type: u64, required: true }
      - { name: replay_jobs, type: "BTreeMap<String, ReplayJob>", required: true }
  - name: ProjectionDescriptor
    fields:
      - { name: name, type: String, required: true }
      - { name: schema_version, type: u32, required: true }
      - { name: retention, type: String, required: true }
  - name: ProjectionCheckpoint
    fields:
      - { name: projection, type: String, required: true }
      - { name: schema_version, type: u32, required: true }
      - { name: cursor, type: u64, required: true }
      - { name: event_id, type: String, required: false }
      - { name: state_sha256, type: String, required: true }
      - { name: updated_at, type: String, required: true, constraints: RFC3339 }
  - name: ProjectionStateEnvelope
    persistence: projections/<name>/state.json via one atomic fsync
    fields:
      - { name: format_version, type: u16, required: true }
      - { name: checkpoint, type: ProjectionCheckpoint, required: true }
      - { name: state_encoding, type: String, required: true, constraints: base64 }
      - { name: state_base64, type: String, required: true }
  - name: ReplayState
    kind: enum
    values: [pending, running, completed, failed]
  - name: ReplayJob
    fields:
      - { name: id, type: String, required: true }
      - { name: projection, type: String, required: true }
      - { name: state, type: ReplayState, required: true }
      - { name: requested_at, type: String, required: true }
      - { name: started_at, type: String, required: false }
      - { name: completed_at, type: String, required: false }
      - { name: source_cursor, type: u64, required: true }
      - { name: rebuilt_cursor, type: u64, required: false }
      - { name: live_digest, type: String, required: false }
      - { name: rebuilt_digest, type: String, required: false }
      - { name: equal, type: bool, required: false }
      - { name: error, type: String, required: false }
  - name: ProjectionLag
    fields:
      - { name: error, type: String, required: true, constraints: projection_lag }
      - { name: projection, type: String, required: true }
      - { name: required_cursor, type: u64, required: true }
      - { name: current_cursor, type: u64, required: true }
      - { name: retryable, type: bool, required: true, constraints: true }
      - { name: retry_after_seconds, type: u64, required: true }
projection_trait:
  required_methods: [descriptor, apply_idempotent, snapshot, restore]
  factory_requirement: Every registered projection provides a fresh factory for isolated rebuild.
embedded_lumen:
  crate: apps/lumen default-features=false
  collection: sift_operational_events_v1
  fields:
    body: { type: text, analyzer: whitespace_lower }
    project: { type: keyword }
    environment: { type: keyword }
    signal: { type: keyword }
    severity: { type: keyword }
    trace_id: { type: keyword }
    session_id: { type: keyword }
    occurred_at: { type: keyword }
    cursor: { type: number }
  deny: Arbitrary payload and attribute keys are never promoted to fields.
durability_order:
  append_event: governance -> Raft/local state-machine commit index -> blob/raw segment fsync -> control applied-index fsync -> acknowledgement
  replay_job: Raft/local state-machine commit index -> replay catalog transition fsync -> acknowledgement
  projection_batch: apply in memory -> serialize -> hash -> atomic state plus checkpoint fsync -> publish cursor -> notify waiters
```

## REST API
<!-- type: rest-api lang: yaml -->

```yaml
openapi: 3.1.0
info:
  title: Sift Projection Replay Slice
  version: 1.0.0
paths:
  /v1/replays:
    post:
      operationId: startReplay
      summary: Start an isolated raw-journal rebuild for one registered projection.
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [projection]
              properties:
                projection: { type: string }
      responses:
        "202":
          description: Replay mutation is durable and asynchronous rebuild is scheduled.
          content:
            application/json:
              schema: { $ref: "#/components/schemas/ReplayJob" }
        "400": { description: Unknown or invalid projection. }
        "503": { description: Mutation could not pass the shared state-machine durability boundary. }
  /v1/replays/{id}:
    get:
      operationId: getReplay
      summary: Read durable pending/running/completed/failed replay state.
      parameters:
        - { name: id, in: path, required: true, schema: { type: string } }
      responses:
        "200":
          description: Durable replay state.
          content:
            application/json:
              schema: { $ref: "#/components/schemas/ReplayJob" }
        "404": { description: Replay id not found. }
components:
  schemas:
    ReplayJob:
      type: object
      required: [id, projection, state, requested_at, source_cursor]
      properties:
        id: { type: string }
        projection: { type: string }
        state: { type: string, enum: [pending, running, completed, failed] }
        requested_at: { type: string, format: date-time }
        started_at: { type: [string, "null"], format: date-time }
        completed_at: { type: [string, "null"], format: date-time }
        source_cursor: { type: integer, format: uint64 }
        rebuilt_cursor: { type: [integer, "null"], format: uint64 }
        live_digest: { type: [string, "null"] }
        rebuilt_digest: { type: [string, "null"] }
        equal: { type: [boolean, "null"] }
        error: { type: [string, "null"] }
x-sift-shared-query-contract:
  min_cursor:
    behavior: Store query handlers call ProjectionRuntime.wait_for before reading a view.
    timeout_status: 503
    error: projection_lag
    headers: { Retry-After: "1" }
    body_fields: [projection, required_cursor, current_cursor, retryable, retry_after_seconds]
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: sift-projection-runtime-verification
requirements:
  retry_idempotency: { id: R1, text: "restart and retry never double apply", kind: reliability, risk: critical, verify: test }
  atomic_checkpoint: { id: R2, text: "checkpoint and projection snapshot publish atomically", kind: reliability, risk: critical, verify: test }
  min_cursor: { id: R3, text: "min cursor wait succeeds or returns typed lag", kind: functional, risk: high, verify: test }
  rebuild_equality: { id: R4, text: "fresh raw rebuild equals live projection", kind: reliability, risk: critical, verify: test }
  command_order: { id: R5, text: "event and replay commands share one state machine while commit and raw cursors remain distinct", kind: compatibility, risk: critical, verify: test }
  replay_restart: { id: R6, text: "replay start and status survive process restart", kind: reliability, risk: critical, verify: test }
  replay_api_state: { id: R7, text: "API emits durable pending running completed or failed state", kind: functional, risk: high, verify: test }
  lumen_queries: { id: R8, text: "text keyword and range queries resolve external event ids", kind: functional, risk: high, verify: test }
  index_allowlist: { id: R9, text: "only declared fields are indexed", kind: security, risk: critical, verify: test }
  lumen_restore: { id: R10, text: "snapshot restore and raw rebuild are semantically equal", kind: reliability, risk: critical, verify: test }
  single_service: { id: R11, text: "Sift links no second Lumen server WAL or Raft runtime", kind: architecture, risk: critical, verify: test }
elements:
  projection_runtime_test: { kind: test, type: "rs/#[tokio::test]" }
  replay_api_test: { kind: test, type: "rs/#[tokio::test]" }
  embedded_lumen_test: { kind: test, type: "rs/#[test]" }
relations:
  - { from: projection_runtime_test, to: R1, type: verifies }
  - { from: projection_runtime_test, to: R2, type: verifies }
  - { from: projection_runtime_test, to: R3, type: verifies }
  - { from: projection_runtime_test, to: R4, type: verifies }
  - { from: replay_api_test, to: R5, type: verifies }
  - { from: replay_api_test, to: R6, type: verifies }
  - { from: replay_api_test, to: R7, type: verifies }
  - { from: embedded_lumen_test, to: R8, type: verifies }
  - { from: embedded_lumen_test, to: R9, type: verifies }
  - { from: embedded_lumen_test, to: R10, type: verifies }
  - { from: embedded_lumen_test, to: R11, type: verifies }
---
graph LR
    projection_runtime_test --> R1
    projection_runtime_test --> R2
    projection_runtime_test --> R3
    projection_runtime_test --> R4
    replay_api_test --> R5
    replay_api_test --> R6
    replay_api_test --> R7
    embedded_lumen_test --> R8
    embedded_lumen_test --> R9
    embedded_lumen_test --> R10
    embedded_lumen_test --> R11
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/sift/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    gap: sift-embedded-lumen-dependency
    tracker: "1660"
    description: Link the lumen crate with default features disabled; do not link its server, operator, WAL, or Raft features.
  - path: projects/sift/src/projection/mod.rs
    action: create
    section: logic
    impl_mode: hand-written
    gap: sift-projection-module
    tracker: "1660"
    description: Export projection contracts, runtime, replay state, and the embedded Lumen adapter.
  - path: projects/sift/src/projection/model.rs
    action: create
    section: schema
    impl_mode: hand-written
    gap: sift-projection-model
    tracker: "1660"
    description: Define descriptors, checkpoints, state envelopes, replay jobs, and projection-lag errors.
  - path: projects/sift/src/projection/runtime.rs
    action: create
    section: logic
    impl_mode: hand-written
    gap: sift-projection-runtime
    tracker: "1660"
    description: Register factories, restore atomic states, catch up asynchronously, wait for cursors, and rebuild from raw.
  - path: projects/sift/src/projection/lumen.rs
    action: create
    section: logic
    impl_mode: hand-written
    gap: sift-embedded-lumen-adapter
    tracker: "1660"
    description: Wrap lumen Engine and RdbSnapshot for fixed-field indexing/search without a second service or durable log.
  - path: projects/sift/src/durability.rs
    action: modify
    section: schema
    impl_mode: hand-written
    gap: sift-framed-journal-state-machine
    tracker: "1605"
    description: Add backward-compatible SiftCommandV1, durable control state, replay transitions, and separate commit/raw cursor semantics.
  - path: projects/sift/src/lib.rs
    action: modify
    section: rest-api
    impl_mode: hand-written
    gap: sift-service-core
    tracker: "1576"
    description: Route all HTTP mutations through one state machine, start the projection worker, expose replay APIs, and map projection_lag.
  - path: projects/sift/src/bin/sift.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: sift-domain-v1-projection-worker
    tracker: "1660"
    description: Start and gracefully stop the in-process projection worker with the Sift service.
  - path: projects/sift/tests/projection_runtime.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    gap: sift-projection-runtime-tests
    tracker: "1660"
    description: Verify idempotency, atomic restart, lag waits, and fresh/live rebuild equality.
  - path: projects/sift/tests/replay_api.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    gap: sift-replay-api-tests
    tracker: "1660"
    description: Verify one-state-machine event/replay ordering, durable job restart, and API lifecycle.
  - path: projects/sift/tests/embedded_lumen_projection.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    gap: sift-embedded-lumen-tests
    tracker: "1660"
    description: Verify fixed-field text/keyword/range search, snapshot restore, and absence of a second service boundary.
```
