---
id: scheduler-memory-backend
main_spec_ref: "crates/cclab-queue/logic/scheduler-memory-backend.md"
merge_strategy: new
fill_sections: [overview, async-api, test-plan]
filled_sections: [overview, async-api, test-plan, changes]
create_complete: true
---

# Scheduler Memory Backend

## Overview

<!-- type: overview lang: markdown -->

`MemorySchedulerBackend` (`crates/cclab-queue/src/scheduler/memory_backend.rs`) implements the `SchedulerBackend` trait as a test-only in-memory backend. It uses `SelfHosted` scheduling mode with single-instance leader election assumption — no distributed coordination.

| Component | Type | Purpose |
|-----------|------|--------|
| `MemorySchedulerBackend` | struct | `RwLock<HashMap<String, TaskScheduleState>>` + `RwLock<bool>` for leadership |
| `TaskScheduleState` | struct (shared) | enabled flag, last_run_at timestamp, total_run_count |

### Trait Method Implementations (5 required + 5 inherited defaults)

| Method | Impl | Behavior |
|--------|------|----------|
| `acquire_leader(ttl)` | override | Sets `is_leader=true`, returns `Ok(true)` — TTL ignored |
| `renew_leader(ttl)` | override | Returns current `is_leader` value — TTL ignored |
| `release_leader()` | override | Sets `is_leader=false` |
| `get_task_state(name)` | override | Returns cloned state or `Default` if absent |
| `set_task_state(name, state)` | override | Upserts into HashMap |
| `scheduling_mode()` | default | Returns `SelfHosted` |
| `register_external_schedule(task)` | default | No-op `Ok(())` |
| `record_task_run(name)` | default | get→mutate last_run_at/count→set |
| `pause_task(name)` | default | get→set enabled=false |
| `resume_task(name)` | default | get→set enabled=true |
| `is_task_enabled(name)` | default | get→return enabled |

Also implements `Default` (delegates to `new()`). No feature gates — always available.

Existing tests (2 in `memory_backend.rs::tests`): leader acquire→renew→release cycle, and task state CRUD (default, pause, resume, record_task_run). This spec adds comprehensive tests covering all trait methods, edge cases, and Send+Sync bounds.
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

All tests in `crates/cclab-queue/src/scheduler/memory_backend.rs` under `#[cfg(test)] mod tests`. All are async unit tests (no external services). Existing 2 tests retained; 18 new tests added.

### S1: Leader Election (R1)

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| S1a | `test_acquire_leader_returns_true` | `acquire_leader` | returns `Ok(true)` |
| S1b | `test_renew_leader_after_acquire` | `renew_leader` after acquire | returns `Ok(true)` |
| S1c | `test_renew_leader_before_acquire` | `renew_leader` without prior acquire | returns `Ok(false)` (is_leader starts false) |
| S1d | `test_release_leader` | `release_leader` | returns `Ok(())` |
| S1e | `test_renew_leader_after_release` | `renew_leader` after release | returns `Ok(false)` |
| S1f | `test_leader_full_cycle` | acquire→renew→release→renew→acquire | true→true→ok→false→true |
| S1g | `test_acquire_leader_ignores_ttl` | `acquire_leader(0s)` and `acquire_leader(1000s)` | both return `Ok(true)` |

### S2: Task State CRUD (R2)

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| S2a | `test_get_task_state_default` | `get_task_state` for unknown task | enabled=true, last_run_at=None, total_run_count=0 |
| S2b | `test_set_and_get_task_state` | `set_task_state` + `get_task_state` | roundtrip preserves enabled, last_run_at, total_run_count |
| S2c | `test_set_task_state_upsert` | upsert semantics | second set overwrites first; count changes from 1→99 |
| S2d | `test_multiple_tasks_isolated` | task state isolation | task-a(2 runs), task-b(1 run), task-c(0 runs) — counts independent |

### S3: Pause & Resume (R3)

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| S3a | `test_pause_task` | `pause_task` | enabled becomes false |
| S3b | `test_resume_task` | `resume_task` after pause | enabled becomes true |
| S3c | `test_is_task_enabled_default` | `is_task_enabled` for unknown task | returns `Ok(true)` |
| S3d | `test_is_task_enabled_after_pause` | `is_task_enabled` after pause | returns `Ok(false)` |

### S4: Record Task Run (R4)

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| S4a | `test_record_task_run_increments_count` | `record_task_run` multiple | count=1 after first, count=2 after second |
| S4b | `test_record_task_run_updates_last_run_at` | `record_task_run` timestamp | last_run_at is Some and ~now (between before/after timestamps) |
| S4c | `test_record_task_run_preserves_enabled` | `record_task_run` enabled | default task stays enabled=true after record |

### S5: Construction & Trait Bounds (R5)

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| S5a | `test_new_and_default_equivalent` | `new()` vs `Default::default()` | both produce empty state (get_task_state returns default) |
| S5b | `test_is_send_sync` | `Send + Sync` bounds | `assert_send_sync::<MemorySchedulerBackend>()` compiles |

### Requirements Traceability

| Req | Description | Tests |
|-----|-------------|-------|
| R1 | RwLock<bool> leader election | S1a-S1g |
| R2 | RwLock<HashMap> task state CRUD + upsert | S2a-S2d |
| R3 | Default trait pause/resume methods | S3a-S3d |
| R4 | Default trait record_task_run (get→mutate→set) | S4a-S4c |
| R5 | Construction, Default impl, Send+Sync | S5a-S5b |

### Totals

| Category | Count |
|----------|-------|
| Leader election | 7 |
| Task state CRUD | 4 |
| Pause/resume | 4 |
| Record task run | 3 |
| Construction & bounds | 2 |
| **Total** | **20** |
## Changes

<!-- type: changes lang: yaml -->

```yaml
_sdd:
  id: scheduler-memory-backend-changes
  refs:
    - $ref: "#memory-scheduler-backend-async-api"
    - $ref: "error-types#task-error-schema"
changes:
  - path: crates/cclab-queue/src/scheduler/memory_backend.rs
    action: modify
    description: >-
      Expand #[cfg(test)] mod tests from 2 to 20 tests.
      New tests: leader election edge cases (renew before acquire returns false,
      renew after release returns false, TTL ignored, full cycle), task state
      upsert semantics, multi-task isolation, is_task_enabled direct calls
      (default true, after pause false), record_task_run multi-increment +
      timestamp validation + enabled preservation, Default impl equivalence,
      Send+Sync bounds assertion.
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
  title: MemorySchedulerBackend Async API
  version: 0.1.0
  description: >
    In-memory test backend implementing SchedulerBackend trait.
    Single-instance leader election, RwLock-based task state store.
  x-sdd:
    id: memory-scheduler-backend-async-api
    refs:
      - $ref: "error-types#task-error-schema"

defaultContentType: application/json

channels:
  leader-election:
    description: Single-instance leader election via RwLock<bool>
    publish:
      operationId: acquireLeader
      summary: Sets is_leader=true, returns Ok(true) — TTL parameter ignored
      message:
        name: LeaderAcquireResult
        payload:
          type: boolean
          const: true
    subscribe:
      operationId: renewLeader
      summary: Returns current is_leader value — true if acquired, false if released
      message:
        name: LeaderRenewResult
        payload:
          type: boolean

  leader-release:
    description: Leadership release
    publish:
      operationId: releaseLeader
      summary: Sets is_leader=false, returns Ok(())
      message:
        name: LeaderReleaseResult
        payload:
          type: 'null'

  task-state:
    description: In-memory task state store — RwLock<HashMap<String, TaskScheduleState>>
    publish:
      operationId: setTaskState
      summary: Insert/overwrite task state in HashMap
      message:
        $ref: '#/components/messages/TaskScheduleStateMessage'
    subscribe:
      operationId: getTaskState
      summary: Retrieve task state; returns Default if absent
      message:
        $ref: '#/components/messages/TaskScheduleStateMessage'

  task-state-derived:
    description: Default trait methods using get_task_state + set_task_state
    publish:
      operationId: recordTaskRun
      summary: get → set last_run_at=now, total_run_count+=1 → set
      message:
        $ref: '#/components/messages/TaskScheduleStateMessage'
    subscribe:
      operationId: isTaskEnabled
      summary: get → return state.enabled
      message:
        name: TaskEnabledResult
        payload:
          type: boolean

  task-pause-resume:
    description: Pause/resume via default trait methods
    publish:
      operationId: pauseTask
      summary: get → set enabled=false → set
      message:
        $ref: '#/components/messages/TaskScheduleStateMessage'
    subscribe:
      operationId: resumeTask
      summary: get → set enabled=true → set
      message:
        $ref: '#/components/messages/TaskScheduleStateMessage'

components:
  messages:
    TaskScheduleStateMessage:
      name: TaskScheduleState
      contentType: application/json
      payload:
        $ref: '#/components/schemas/TaskScheduleState'

  schemas:
    MemorySchedulerBackend:
      type: object
      description: In-memory scheduler backend struct
      properties:
        task_states:
          type: object
          description: RwLock<HashMap<String, TaskScheduleState>>
          additionalProperties:
            $ref: '#/components/schemas/TaskScheduleState'
        is_leader:
          type: boolean
          description: RwLock<bool>, initially false
          default: false
      x-sdd:
        id: memory-scheduler-backend
        source: crates/cclab-queue/src/scheduler/memory_backend.rs
        concurrency: tokio::sync::RwLock
        send_sync: true

    TaskScheduleState:
      type: object
      required: [enabled, total_run_count]
      properties:
        enabled:
          type: boolean
          default: true
        last_run_at:
          type: string
          format: date-time
          nullable: true
        total_run_count:
          type: integer
          format: uint64
          default: 0

    LeaderElectionBehavior:
      type: object
      description: Leader election state transitions
      x-sdd:
        id: memory-leader-behavior
      properties:
        initial:
          type: boolean
          const: false
          description: is_leader starts false after new()
        after_acquire:
          type: boolean
          const: true
          description: acquire_leader always sets true
        after_release:
          type: boolean
          const: false
          description: release_leader sets false
        renew_returns:
          type: boolean
          description: Returns current is_leader value
```

# Reviews
