---
id: result-backend
main_spec_ref: "crates/cclab-queue/logic/result-backend.md"
merge_strategy: new
---

# Result Backend

## Overview

<!-- type: overview lang: markdown -->

`ResultBackend` is the async trait (`crates/cclab-queue/src/backend/mod.rs`) defining task result storage for cclab-queue. It exposes 11 async methods in 3 groups:

| Group | Methods | Purpose |
|-------|---------|--------|
| Task CRUD | `set_state`, `get_state`, `set_result`, `get_result`, `delete` | Store/retrieve task state and results with optional TTL |
| Query | `wait_for_result`, `get_many`, `health_check` | Blocking poll until terminal state, batch retrieval, liveness probe |
| Metadata | `set_metadata`, `get_metadata`, `delete_metadata` | Workflow tracking for chains/chords via `{prefix}:meta:{key}` keys |

Two feature-gated implementations exist:

| Backend | Module | Feature | Connection | Serialization |
|---------|--------|---------|------------|---------------|
| `RedisBackend` | `backend/redis.rs` | `redis` | `deadpool-redis` pool (async, cloneable) | `serde_json::to_string` |
| `IonBackend` | `backend/ion.rs` | `ion` | `Mutex<KvClient>` (single conn) | `serde_json::to_vec` |

Both backends use a `{prefix}:state:{task_id}` / `{prefix}:result:{task_id}` / `{prefix}:meta:{key}` key schema with configurable TTL. Redis defaults to `cclab-meteor` prefix with 24h TTL and 10-connection pool; Ion defaults to `meteor` prefix with 24h TTL and 5s connect timeout.

Existing unit tests cover config defaults, key generation, and serialization. Redis integration tests (marked `#[ignore]`) cover basic CRUD, `wait_for_result` success/timeout, `get_many`, `delete`, and `health_check`. Ion has no integration tests. This spec adds comprehensive test coverage for both backends across the full CRUD lifecycle, metadata API, batch operations, timeout/edge-case behavior, and error paths.
## Requirements
<!-- type: requirements lang: markdown -->

<!-- TODO -->

## Scenarios
<!-- type: scenarios lang: markdown -->

<!-- TODO -->

## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan

<!-- type: test-plan lang: markdown -->

Tests span two files: `crates/cclab-queue/src/backend/redis.rs` and `crates/cclab-queue/src/backend/ion.rs`, each as `#[cfg(test)] mod tests`. Unit tests run without external services; integration tests are `#[tokio::test] #[ignore]`.

### Redis Backend (`backend/redis.rs`)

**Unit tests (no Redis required):**

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| R1 | `config_defaults` | `RedisBackendConfig::default()` | url=`redis://localhost:6379`, key_prefix=`cclab-meteor`, default_ttl=86400s, pool_size=10 |
| R2 | `config_serde_round_trip` | Serialize/Deserialize derives | `serde_json::to_string` → `from_str` round-trips all fields |
| R3 | `state_serialization_round_trip` | TaskState JSON codec | `to_string(Success)` → `from_str` == `Success` |
| R4 | `result_serialization_round_trip` | TaskResult JSON codec | `to_string(result)` → `from_str` preserves task_id, state, result payload |

**Integration tests (`#[ignore]`, require Redis):**

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| R5 | `key_generation_state_format` | `state_key()` | output == `"{prefix}:state:{task_id}"` |
| R6 | `key_generation_result_format` | `result_key()` | output == `"{prefix}:result:{task_id}"` |
| R7 | `get_ttl_seconds_with_override` | `get_ttl_seconds(Some(60s))` | returns 60 |
| R8 | `get_ttl_seconds_default_fallback` | `get_ttl_seconds(None)` | returns config.default_ttl.as_secs() |
| R9 | `set_get_state_round_trip` | `set_state` + `get_state` | set Started → get returns Some(Started) |
| R10 | `get_state_absent_returns_none` | `get_state` miss | fresh TaskId → None |
| R11 | `set_state_overwrites` | `set_state` upsert | set Pending → set Started → get returns Some(Started) |
| R12 | `set_get_result_round_trip` | `set_result` + `get_result` | round-trip preserves task_id, state, result JSON |
| R13 | `get_result_absent_returns_none` | `get_result` miss | fresh TaskId → None |
| R14 | `set_result_custom_ttl` | `set_result` with `Some(Duration::from_secs(10))` | result retrievable; does not error |
| R15 | `set_result_zero_ttl_no_expiry` | `set_result` with `Some(Duration::ZERO)` | takes SET (no EX) branch; result retrievable |
| R16 | `set_result_writes_state_key` | `set_result` dual-key write | after set_result(Success), get_state returns Some(Success) |
| R17 | `wait_for_result_immediate_terminal` | `wait_for_result` fast path | set Success result → wait returns immediately |
| R18 | `wait_for_result_polls_until_done` | `wait_for_result` poll loop | spawn task sets result after 100ms; wait with 5s timeout succeeds |
| R19 | `wait_for_result_timeout` | `wait_for_result` timeout | set Pending only → wait with 200ms timeout → `TaskError::Timeout` |
| R20 | `wait_for_result_default_timeout` | `wait_for_result` timeout=None | timeout=None uses 3600s internally (verify no panic, cancel after 200ms via tokio::select) |
| R21 | `wait_for_result_terminal_no_result_key` | terminal state without result key | set_state(Success) only (no set_result) → wait → `TaskError::Backend("...no result found")` |
| R22 | `delete_removes_both_keys` | `delete` | set_result → delete → get_state==None && get_result==None |
| R23 | `delete_idempotent` | `delete` on absent | delete fresh TaskId → Ok(()) |
| R24 | `get_many_mixed` | `get_many` partial hits | 3 ids, 2 with results → vec[Some, Some, None] |
| R25 | `get_many_empty_input` | `get_many` empty slice | `get_many(&[])` → empty vec |
| R26 | `get_many_all_absent` | `get_many` all miss | 3 fresh ids → vec[None, None, None] |
| R27 | `health_check_ok` | `health_check` | returns Ok(()) |
| R28 | `set_get_metadata_round_trip` | `set_metadata` + `get_metadata` | set JSON value → get returns Some(same value) |
| R29 | `get_metadata_absent_returns_none` | `get_metadata` miss | fresh key → None |
| R30 | `set_metadata_custom_ttl` | `set_metadata` TTL override | set with Some(10s) → retrievable |
| R31 | `delete_metadata_removes_key` | `delete_metadata` | set → delete → get returns None |
| R32 | `delete_metadata_idempotent` | `delete_metadata` on absent | delete fresh key → Ok(()) |
| R33 | `metadata_key_format` | metadata key pattern | set with key="workflow-123" → get_metadata("workflow-123") returns stored value |
| R34 | `clone_shares_pool` | `Clone` impl | clone backend; both can set/get independently |

### Ion Backend (`backend/ion.rs`)

**Unit tests (no Ion required):**

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| I1 | `config_default_values` | `IonBackendConfig::default()` | url=`127.0.0.1:16380`, key_prefix=`meteor`, default_ttl=Some(86400s), connect_timeout=5s, request_timeout=30s |
| I2 | `key_format_state` | key pattern | `format!("{}:state:{}", prefix, id)` starts with `"meteor:state:"` |
| I3 | `key_format_result` | key pattern | starts with `"meteor:result:"` |
| I4 | `value_to_bytes_from_bytes` | `value_to_bytes` Bytes variant | `KvValue::Bytes(vec![1,2,3])` → `Ok(vec![1,2,3])` |
| I5 | `value_to_bytes_from_string` | `value_to_bytes` String variant | `KvValue::String("hello".into())` → `Ok(b"hello".to_vec())` |
| I6 | `value_to_bytes_unexpected_type` | `value_to_bytes` error path | `KvValue::Int(42)` → `Err(TaskError::Deserialization("Unexpected value type from Ion"))` |

**Integration tests (`#[ignore]`, require Ion):**

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| I7 | `set_get_state_round_trip` | `set_state` + `get_state` | set Started → get returns Some(Started) |
| I8 | `get_state_absent_returns_none` | `get_state` miss | fresh TaskId → None |
| I9 | `set_state_overwrites` | `set_state` upsert | Pending → Started → get == Some(Started) |
| I10 | `set_get_result_round_trip` | `set_result` + `get_result` | round-trip preserves task_id, state, result payload |
| I11 | `get_result_absent_returns_none` | `get_result` miss | fresh TaskId → None |
| I12 | `set_result_updates_state` | Ion `set_result` calls `self.set_state` internally | after set_result(Success), get_state returns Some(Success) |
| I13 | `set_result_custom_ttl` | `set_result` with `Some(Duration)` | result retrievable |
| I14 | `set_result_nil_ttl_uses_default` | `set_result` with `ttl=None` | uses config.default_ttl via `ttl.or(...)` |
| I15 | `wait_for_result_immediate_terminal` | `wait_for_result` fast path | set terminal result → wait returns immediately |
| I16 | `wait_for_result_timeout` | `wait_for_result` timeout | no result set → wait with 200ms timeout → `TaskError::Timeout` |
| I17 | `wait_for_result_polls_until_done` | `wait_for_result` poll loop | spawn sets result after 100ms; wait with 5s timeout succeeds |
| I18 | `wait_for_result_state_only_insufficient` | Ion polls `get_result` not `get_state` | set_state(Success) only → wait times out (no result key to find) |
| I19 | `delete_removes_both_keys` | `delete` | set_result → delete → get_state==None && get_result==None |
| I20 | `delete_idempotent` | `delete` on absent | fresh TaskId → Ok(()) |
| I21 | `get_many_mixed` | `get_many` via mget | 3 ids, 2 with results → vec[Some, Some, None] |
| I22 | `get_many_empty_input` | `get_many` empty slice | `get_many(&[])` → empty Vec |
| I23 | `health_check_ping` | `health_check` via client.ping() | returns Ok(()) |
| I24 | `set_get_metadata_round_trip` | metadata CRUD | set JSON → get returns Some(same value) |
| I25 | `get_metadata_absent_returns_none` | `get_metadata` miss | fresh key → None |
| I26 | `set_metadata_custom_ttl` | metadata TTL override | set with Some(10s) → retrievable |
| I27 | `delete_metadata_removes_key` | `delete_metadata` | set → delete → get returns None |
| I28 | `delete_metadata_idempotent` | `delete_metadata` on absent | fresh key → Ok(()) |

### Cross-Backend Behavioral Differences

| Behavior | Redis | Ion | Test Coverage |
|----------|-------|-----|---------------|
| `wait_for_result` poll target | checks `get_state` → if terminal → `get_result` | checks `get_result` directly → returns when `state.is_terminal()` | R21 vs I18 |
| `wait_for_result` default timeout | `None` → 3600s | `None` → polls indefinitely | R20 |
| `set_result` state write | two separate SET_EX calls for state+result keys | writes result key then calls `self.set_state()` | R16, I12 |
| TTL=0 handling | `ttl > 0` branch → SET without EX | always passes TTL to `client.set()` | R15 |
| Serialization format | `serde_json::to_string/from_str` (String) | `serde_json::to_vec/from_slice` (Bytes) | I4, I5 |
| Connection model | `deadpool-redis` pool (cloneable) | `Mutex<KvClient>` (single conn) | R34 |

### Totals

| File | Unit | Integration | Total |
|------|------|-------------|-------|
| `backend/redis.rs` | 4 | 30 | 34 |
| `backend/ion.rs` | 6 | 22 | 28 |
| **Total** | **10** | **52** | **62** |
## Changes

<!-- type: changes lang: yaml -->

```yaml
_sdd:
  id: result-backend-changes
  refs:
    - $ref: "#result-backend-async-api"
    - $ref: "error-types#error-types-schema"
changes:
  - path: crates/cclab-queue/src/backend/redis.rs
    action: modify
    description: >-
      Expand #[cfg(test)] mod tests with 30 new tests (4 unit + 26 integration #[ignore]).
      Unit: config serde round-trip.
      Integration: state CRUD edge cases (absent/overwrite), result TTL variants (custom/zero),
      set_result dual-key write, wait_for_result edge cases (immediate terminal, default timeout,
      terminal-no-result-key), delete idempotency, get_many (empty/all-absent),
      full metadata API (roundtrip/absent/custom-ttl/delete/idempotent/key-format), clone pool sharing.
  - path: crates/cclab-queue/src/backend/ion.rs
    action: modify
    description: >-
      Expand #[cfg(test)] mod tests with 26 new tests (4 unit + 22 integration #[ignore]).
      Unit: value_to_bytes for Bytes/String/unexpected variants, config custom values.
      Integration: full ResultBackend trait surface — state CRUD, result CRUD with TTL variants,
      set_result→set_state coupling, wait_for_result (immediate/timeout/poll/state-only-insufficient),
      delete/get_many/health_check, full metadata API.
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->


## Async API

<!-- type: async-api lang: yaml -->

```yaml
asyncapi: '2.6.0'
info:
  title: ResultBackend Async Trait API
  version: 0.1.0
  description: >
    Async trait defining task result storage for cclab-queue.
    Two feature-gated implementations: RedisBackend (redis) and IonBackend (ion).
  x-sdd:
    id: result-backend-async-api
    refs:
      - $ref: "error-types#error-types-schema"

defaultContentType: application/json

channels:
  task-state:
    description: Task state storage — key pattern {prefix}:state:{task_id}
    parameters:
      prefix:
        description: Backend key prefix
        schema:
          type: string
          default: cclab-meteor
      task_id:
        description: UUID v4 task identifier
        schema:
          type: string
          format: uuid
    publish:
      operationId: setState
      summary: Upsert task state with default TTL
      message:
        $ref: '#/components/messages/TaskStateMessage'
    subscribe:
      operationId: getState
      summary: Retrieve task state; returns null if key absent
      message:
        $ref: '#/components/messages/TaskStateMessage'

  task-result:
    description: Task result storage — key pattern {prefix}:result:{task_id}
    parameters:
      prefix:
        schema:
          type: string
      task_id:
        schema:
          type: string
          format: uuid
    publish:
      operationId: setResult
      summary: Write result + state keys (best-effort atomic); accepts optional TTL override
      bindings:
        redis:
          method: SET_EX
        ion:
          method: SET
      message:
        $ref: '#/components/messages/TaskResultMessage'
    subscribe:
      operationId: getResult
      summary: Retrieve task result; returns null if key absent
      message:
        $ref: '#/components/messages/TaskResultMessage'

  task-result-poll:
    description: Blocking poll channel — calls getState in loop until is_terminal(), then getResult
    subscribe:
      operationId: waitForResult
      summary: Poll until terminal state reached or timeout expires
      bindings:
        poll:
          interval_param: poll_interval
          timeout_param: timeout
          default_timeout_redis: 3600s
          default_timeout_ion: caller-supplied
      message:
        $ref: '#/components/messages/TaskResultMessage'

  task-batch:
    description: Batch result retrieval — Redis MGET / Ion mget
    subscribe:
      operationId: getMany
      summary: Fetch multiple results in single round-trip; empty input returns empty vec
      message:
        $ref: '#/components/messages/TaskResultBatchMessage'

  task-lifecycle:
    description: Task data deletion — removes both state and result keys
    publish:
      operationId: delete
      summary: Delete state + result keys for a task
      message:
        name: TaskDeleteRequest
        payload:
          type: object
          required: [task_id]
          properties:
            task_id:
              type: string
              format: uuid

  metadata:
    description: Workflow metadata storage — key pattern {prefix}:meta:{key}
    parameters:
      prefix:
        schema:
          type: string
      key:
        description: Arbitrary metadata key (e.g. chain_id, chord_id)
        schema:
          type: string
    publish:
      operationId: setMetadata
      summary: Store workflow tracking value with optional TTL
      message:
        $ref: '#/components/messages/MetadataMessage'
    subscribe:
      operationId: getMetadata
      summary: Retrieve metadata; returns null if key absent
      message:
        $ref: '#/components/messages/MetadataMessage'

  metadata-lifecycle:
    description: Metadata deletion
    publish:
      operationId: deleteMetadata
      summary: Remove metadata key
      message:
        name: MetadataDeleteRequest
        payload:
          type: object
          required: [key]
          properties:
            key:
              type: string

  health:
    description: Backend liveness probe
    subscribe:
      operationId: healthCheck
      summary: Redis GET __health_check__ / Ion ping
      message:
        name: HealthCheckResponse
        payload:
          type: object
          properties:
            ok:
              type: boolean

components:
  messages:
    TaskStateMessage:
      name: TaskState
      contentType: application/json
      payload:
        $ref: '#/components/schemas/TaskState'

    TaskResultMessage:
      name: TaskResult
      contentType: application/json
      payload:
        $ref: '#/components/schemas/TaskResult'

    TaskResultBatchMessage:
      name: TaskResultBatch
      contentType: application/json
      payload:
        type: array
        items:
          oneOf:
            - $ref: '#/components/schemas/TaskResult'
            - type: 'null'

    MetadataMessage:
      name: Metadata
      contentType: application/json
      payload:
        $ref: '#/components/schemas/MetadataEntry'

  schemas:
    TaskState:
      type: string
      enum: [Pending, Received, Started, Offloaded, Success, Failure, Retry, Revoked, Rejected]
      x-terminal-states: [Success, Failure, Revoked, Rejected]

    TaskResult:
      type: object
      required: [task_id, state, retries]
      properties:
        task_id:
          type: string
          format: uuid
        state:
          $ref: '#/components/schemas/TaskState'
        result:
          description: Arbitrary JSON return value
        error:
          type: string
          nullable: true
        traceback:
          type: string
          nullable: true
        started_at:
          type: string
          format: date-time
          nullable: true
        completed_at:
          type: string
          format: date-time
          nullable: true
        runtime_ms:
          type: integer
          format: uint64
          nullable: true
        retries:
          type: integer
          format: uint32
          default: 0
        worker_id:
          type: string
          nullable: true

    MetadataEntry:
      type: object
      required: [key, value]
      properties:
        key:
          type: string
        value:
          description: Arbitrary JSON value for workflow tracking
        ttl:
          type: integer
          format: uint64
          description: TTL in seconds; null uses backend default
          nullable: true

    TaskError:
      type: string
      enum: [Backend, Serialization, Deserialization, Timeout, Connection, NotConnected]
      description: Error variants relevant to ResultBackend operations

    RedisBackendConfig:
      type: object
      required: [url, key_prefix, default_ttl, pool_size]
      properties:
        url:
          type: string
          default: "redis://localhost:6379"
        key_prefix:
          type: string
          default: cclab-meteor
        default_ttl:
          type: integer
          format: uint64
          description: Seconds; 0 = no expiry
          default: 86400
        pool_size:
          type: integer
          format: usize
          default: 10

    IonBackendConfig:
      type: object
      required: [url, key_prefix, connect_timeout, request_timeout]
      properties:
        url:
          type: string
          default: "127.0.0.1:16380"
        key_prefix:
          type: string
          default: meteor
        default_ttl:
          type: integer
          format: uint64
          description: Seconds; null = no expiry
          nullable: true
          default: 86400
        connect_timeout:
          type: integer
          format: uint64
          description: Seconds
          default: 5
        request_timeout:
          type: integer
          format: uint64
          description: Seconds
          default: 30

    KeySchema:
      type: object
      description: Key patterns per backend
      properties:
        state:
          type: string
          pattern: "{prefix}:state:{task_id}"
        result:
          type: string
          pattern: "{prefix}:result:{task_id}"
        metadata:
          type: string
          pattern: "{prefix}:meta:{key}"

  serverBindings:
    redis:
      description: deadpool-redis connection pool (async, cloneable)
      serialization: serde_json::to_string / from_str
    ion:
      description: Mutex<KvClient> single connection
      serialization: serde_json::to_vec / from_slice
```