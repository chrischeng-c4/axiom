---
id: scheduler-mode-selection
main_spec_ref: "crates/cclab-fetch/scheduler/scheduler-mode-selection.md"
merge_strategy: new
---

# Scheduler Mode Selection

## Overview

Extends `PeriodicScheduler::start()` with runtime mode selection based on the `SchedulerBackend` implementation. Adds a `SchedulingMode` enum (`SelfHosted`, `ExternalPush`) and a `fn scheduling_mode(&self) -> SchedulingMode` method to the `SchedulerBackend` trait. Self-hosted backends (Ion, Memory) return `SelfHosted` and run the existing leader election tick loop (acquire_leader → evaluate schedules → enqueue → renew_leader). External backends (CloudScheduler, K8sCronJob) return `ExternalPush` and start the HTTP push receiver server instead — no leader election, no tick loop.

`PeriodicScheduler::start()` queries `backend.scheduling_mode()` once at startup and branches: `SelfHosted` → spawns the existing `run_leader_loop` task; `ExternalPush` → constructs `PushReceiver` from config, mounts it on the server, registers all tasks with the external backend (creates Cloud Scheduler jobs or K8s CronJobs via `add_task()`), and optionally starts the `ScheduleMonitor` background check.

Task registration is unified through `PeriodicScheduler::add_task()` regardless of mode. In self-hosted mode, tasks are added to the in-memory task list (current behavior). In external push mode, `add_task()` additionally calls `backend.register_external_schedule(task)` to create the corresponding external resource (Cloud Scheduler job or K8s CronJob), and registers the task with `ScheduleMonitor` for expected_at tracking.

`PeriodicSchedulerConfig` gains optional fields for push receiver and monitor configuration: `push_receiver_config: Option<PushReceiverConfig>`, `monitor_config: Option<ScheduleMonitorConfig>`. These are required when `scheduling_mode() == ExternalPush` and ignored for `SelfHosted`.

Source: `crates/cclab-queue/src/scheduler/periodic.rs` (PeriodicScheduler, start()), `crates/cclab-queue/src/scheduler/backend.rs` (SchedulerBackend trait)
## Requirements

| ID | Title | Priority | Acceptance Criteria |
|----|-------|----------|---------------------|
| R1 | SchedulingMode enum | P0 | `SchedulingMode` enum with two variants: `SelfHosted` (backend manages scheduling internally via leader election tick loop) and `ExternalPush` (external system manages scheduling, triggers arrive via HTTP push receiver). Derives `Debug, Clone, Copy, PartialEq, Eq` |
| R2 | SchedulerBackend::scheduling_mode() trait method | P0 | Add `fn scheduling_mode(&self) -> SchedulingMode` to `SchedulerBackend` trait with default implementation returning `SchedulingMode::SelfHosted` (backward-compatible — existing backends unchanged). `IonSchedulerBackend` and `MemorySchedulerBackend` use default. `CloudSchedulerBackend` and `K8sCronJobBackend` override to return `ExternalPush` |
| R3 | PeriodicScheduler::start() mode branching | P0 | `start()` calls `self.backend.scheduling_mode()` once at startup. If `SelfHosted`: spawns existing `run_leader_loop` task (no behavior change). If `ExternalPush`: constructs `PushReceiver` from `config.push_receiver_config` (required — returns `TaskError::ConfigError` if `None`), starts `ScheduleMonitor` if `config.monitor_config` is `Some`, registers all tasks with external backend, does NOT start leader election loop |
| R4 | External task registration | P0 | In `ExternalPush` mode, `start()` iterates all registered tasks and calls `backend.register_external_schedule(task)` for each. This async trait method creates the corresponding external resource (Cloud Scheduler job or K8s CronJob). Default trait implementation returns `Ok(())` (no-op for self-hosted backends). If any registration fails, `start()` returns the error |
| R5 | PeriodicSchedulerConfig extension | P0 | Add `push_receiver_config: Option<PushReceiverConfig>` and `monitor_config: Option<ScheduleMonitorConfig>` to `PeriodicSchedulerConfig`. Both default to `None`. In `ExternalPush` mode, `push_receiver_config` is required. `monitor_config` is optional in both modes (monitor can track self-hosted tasks too) |
| R6 | Push receiver lifecycle in start() | P1 | In `ExternalPush` mode, `start()` constructs `Arc<PushReceiver>` and returns the `axum::Router` via a new `PeriodicScheduler::router()` method. Caller (server setup code) merges this router into the existing server. `start()` does NOT bind to a port — router is returned for external mounting. If `ScheduleMonitor` configured, it is passed to `PushReceiver` as `Option<Arc<ScheduleMonitor>>` |
| R7 | ScheduleMonitor integration in start() | P1 | If `monitor_config` is `Some`, `start()` creates `Arc<ScheduleMonitor>`, registers all tasks with it (using task schedule + default leeway), and calls `monitor.start()` to spawn the background missed-check task. Monitor is shared with PushReceiver (ExternalPush mode) or hooked into the tick loop after `broker.publish()` (SelfHosted mode) |
| R8 | register_external_schedule trait method | P1 | Add `async fn register_external_schedule(&self, task: &PeriodicTask) -> Result<(), TaskError>` to `SchedulerBackend` trait with default no-op implementation. `CloudSchedulerBackend` overrides to call `create_job()`. `K8sCronJobBackend` overrides to call `create_cronjob()`. Converts `PeriodicTask.schedule` to the backend-specific format (cron string or interval) |
| R9 | Graceful shutdown for both modes | P1 | `PeriodicScheduler::shutdown()` works for both modes. `SelfHosted`: cancels the tick loop via `CancellationToken` (current behavior). `ExternalPush`: stops the `ScheduleMonitor` background task if running. Push receiver routes remain active until the server shuts down (managed externally). Shutdown signals propagated via existing `CancellationToken` |

### Constraints

- `scheduling_mode()` is not async and not fallible — mode is a static property of the backend type
- Default trait implementation of `scheduling_mode()` returns `SelfHosted` to ensure backward compatibility
- `PeriodicScheduler` remains generic over `B: Broker, S: SchedulerBackend` — no dynamic dispatch change
- `router()` method only available after `start()` in `ExternalPush` mode; returns `None` in `SelfHosted` mode
- Monitor integration in `SelfHosted` mode is best-effort — `record_trigger()` failure does not fail task enqueue
## Scenarios

### S1: Self-hosted backend starts leader election tick loop (R2, R3)

**GIVEN** a `PeriodicScheduler` with `IonSchedulerBackend` (or `MemorySchedulerBackend`)
**WHEN** `start()` is called
**THEN** `backend.scheduling_mode()` returns `SelfHosted`; spawns `run_leader_loop` task (existing behavior); no `PushReceiver` is created; `push_receiver_config` is ignored even if set

### S2: External backend starts push receiver mode (R2, R3, R6)

**GIVEN** a `PeriodicScheduler` with `CloudSchedulerBackend` and `push_receiver_config = Some(PushReceiverConfig { ... })`
**WHEN** `start()` is called
**THEN** `backend.scheduling_mode()` returns `ExternalPush`; `PushReceiver` is constructed from config; `router()` returns `Some(axum::Router)` with push receiver routes; no leader election loop is started; no `run_leader_loop` task is spawned

### S3: External mode without push_receiver_config returns error (R3, R5)

**GIVEN** a `PeriodicScheduler` with `K8sCronJobBackend` and `push_receiver_config = None`
**WHEN** `start()` is called
**THEN** returns `Err(TaskError::ConfigError("push_receiver_config required for ExternalPush mode"))`; no tasks are registered; no background tasks are spawned

### S4: Tasks registered with external backend on start (R4, R8)

**GIVEN** a `PeriodicScheduler` with `CloudSchedulerBackend` and 3 tasks: `daily-cleanup` (cron `0 2 * * *`), `hourly-sync` (interval 3600s), `weekly-report` (cron `0 9 * * 1`)
**WHEN** `start()` is called in `ExternalPush` mode
**THEN** `backend.register_external_schedule(task)` is called for each task; Cloud Scheduler creates 3 GCP jobs via API; all 3 tasks registered with `ScheduleMonitor` if configured

### S5: External task registration failure aborts start (R4)

**GIVEN** a `PeriodicScheduler` with `K8sCronJobBackend` and task `daily-cleanup`
**WHEN** `start()` calls `backend.register_external_schedule("daily-cleanup")` and K8s API returns 403 Forbidden
**THEN** `start()` returns `Err(TaskError::AuthenticationError(...))` immediately; subsequent tasks are NOT registered; push receiver is NOT started

### S6: ScheduleMonitor integrates with both modes (R7)

**GIVEN** a `PeriodicScheduler` with `monitor_config = Some(ScheduleMonitorConfig { default_leeway_secs: 30, check_interval_secs: 60, .. })` in `SelfHosted` mode
**WHEN** `start()` is called
**THEN** `ScheduleMonitor` is created and started; all tasks registered with monitor; after each `broker.publish()` in the tick loop, `monitor.record_trigger(task_name, Utc::now())` is called (best-effort — failure logged, does not fail enqueue)

### S7: ScheduleMonitor in ExternalPush mode receives triggers via push receiver (R7, R6)

**GIVEN** a `PeriodicScheduler` in `ExternalPush` mode with `ScheduleMonitor` configured
**WHEN** push receiver handles an authenticated request for task `daily-cleanup`
**THEN** `PushReceiver` calls `monitor.record_trigger("daily-cleanup", actual_at)` before `broker.publish()`; monitor classifies fire as on_time/late based on leeway; metrics emitted

### S8: Default scheduling_mode() is backward-compatible (R2)

**GIVEN** an existing `SchedulerBackend` implementation that does NOT override `scheduling_mode()`
**WHEN** `scheduling_mode()` is called
**THEN** returns `SchedulingMode::SelfHosted` (default implementation); existing behavior is completely unchanged

### S9: Shutdown in ExternalPush mode stops monitor (R9)

**GIVEN** a `PeriodicScheduler` in `ExternalPush` mode with `ScheduleMonitor` running
**WHEN** `shutdown()` is called
**THEN** `CancellationToken` is cancelled; `ScheduleMonitor::stop()` is called (signals background check task to exit); push receiver routes remain active until server process exits (not managed by scheduler)

### S10: Shutdown in SelfHosted mode unchanged (R9)

**GIVEN** a `PeriodicScheduler` in `SelfHosted` mode running the leader election loop
**WHEN** `shutdown()` is called
**THEN** `CancellationToken` is cancelled; leader loop exits and calls `backend.release_leader()`; if `ScheduleMonitor` was configured, `monitor.stop()` is also called

### S11: router() returns None in SelfHosted mode (R6)

**GIVEN** a `PeriodicScheduler` in `SelfHosted` mode after `start()` is called
**WHEN** `router()` is called
**THEN** returns `None`; no push receiver routes exist

### S12: K8sCronJobBackend overrides scheduling_mode (R2)

**GIVEN** a `K8sCronJobBackend` instance
**WHEN** `scheduling_mode()` is called
**THEN** returns `SchedulingMode::ExternalPush`; this drives `PeriodicScheduler::start()` to skip leader election and use push receiver mode instead
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

<!-- TODO -->

## Changes

```yaml
changes:
  - file: crates/cclab-queue/src/scheduler/backend.rs
    action: modify
    description: |
      Add SchedulingMode enum and two new methods to SchedulerBackend trait.
      Both methods have default implementations for backward compatibility.
    additions:
      - 'pub enum SchedulingMode { SelfHosted, ExternalPush }  # derives Debug, Clone, Copy, PartialEq, Eq'
      - 'fn scheduling_mode(&self) -> SchedulingMode { SchedulingMode::SelfHosted }  # default impl'
      - 'async fn register_external_schedule(&self, task: &PeriodicTask) -> Result<(), TaskError> { Ok(()) }  # default no-op'
    imports:
      - 'use super::periodic::PeriodicTask'
    notes: |
      SchedulingMode must be defined before the trait (or in same file).
      The PeriodicTask import creates a circular reference between periodic.rs and backend.rs.
      If circular, move PeriodicTask to backend.rs or a shared types module.

  - file: crates/cclab-queue/src/scheduler/periodic.rs
    action: modify
    description: |
      Extend PeriodicSchedulerConfig with push_receiver_config and monitor_config.
      Modify PeriodicScheduler to hold optional PushReceiver and ScheduleMonitor.
      Rewrite start() to branch on backend.scheduling_mode().
      Add router() method returning Option<axum::Router>.
      Add monitor integration to run_leader_loop (best-effort record_trigger after publish).
    structs_modified:
      - PeriodicSchedulerConfig
      - PeriodicScheduler
    methods_modified:
      - "pub async fn start(&self) -> Result<(), TaskError>  # adds mode branching"
      - "async fn run_leader_loop(...)  # adds optional monitor.record_trigger() after broker.publish()"
    methods_added:
      - "pub fn router(&self) -> Option<axum::Router>  # returns push receiver router in ExternalPush mode"
      - "async fn start_external_push(&self) -> Result<(), TaskError>  # ExternalPush mode startup logic"
      - "async fn register_all_tasks_external(&self) -> Result<(), TaskError>  # iterates tasks, calls register_external_schedule"
    fields_added:
      - "push_receiver: Option<Arc<PushReceiver>>  # created in ExternalPush mode"
      - "monitor: Option<Arc<ScheduleMonitor>>  # created when monitor_config is set"
    notes: |
      PeriodicScheduler fields push_receiver and monitor must use interior mutability
      (e.g., OnceCell or RwLock) since start() takes &self.
      Consider Arc<OnceCell<Arc<PushReceiver>>> to set once during start().

  - file: crates/cclab-queue/src/scheduler/cloud_scheduler_backend.rs
    action: modify
    description: |
      Override scheduling_mode() to return ExternalPush.
      Override register_external_schedule() to create Cloud Scheduler job.
    additions:
      - 'fn scheduling_mode(&self) -> SchedulingMode { SchedulingMode::ExternalPush }'
      - 'async fn register_external_schedule(&self, task: &PeriodicTask) -> Result<(), TaskError>  # calls create_job() with converted schedule'
    notes: |
      Converts PeriodicTask.schedule to Cloud Scheduler format:
      Cron(expr) → schedule field in Cloud Scheduler API.
      Interval(secs) → convert to cron expression or use Cloud Scheduler's
      appEngineHttpTarget.schedule with interval notation.

  - file: crates/cclab-queue/src/scheduler/mod.rs
    action: modify
    description: |
      Re-export SchedulingMode from backend module.
    additions:
      - 'pub use backend::SchedulingMode;'
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


## Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Scheduler Mode Selection — Data Models",
  "$defs": {
    "SchedulingMode": {
      "$id": "meteor://scheduler/scheduling-mode",
      "type": "string",
      "enum": ["self_hosted", "external_push"],
      "description": "Runtime scheduling mode. SelfHosted runs leader election tick loop internally. ExternalPush delegates scheduling to external system (Cloud Scheduler, K8s CronJob) and receives triggers via HTTP push receiver."
    },
    "PeriodicSchedulerConfig": {
      "$id": "meteor://scheduler/periodic-scheduler-config",
      "type": "object",
      "description": "Extended configuration for PeriodicScheduler with optional push receiver and monitor settings.",
      "properties": {
        "leader_ttl": {
          "type": "string",
          "const": "Duration",
          "default": "15s",
          "description": "TTL for leader lock (SelfHosted mode only)"
        },
        "follower_sleep": {
          "type": "string",
          "const": "Duration",
          "default": "5s",
          "description": "How long followers sleep before retrying (SelfHosted mode only)"
        },
        "leader_renew_interval": {
          "type": "string",
          "const": "Duration",
          "default": "5s",
          "description": "How often to renew leader lock (SelfHosted mode only)"
        },
        "push_receiver_config": {
          "oneOf": [
            { "$ref": "meteor://scheduler/push-receiver-config" },
            { "type": "null" }
          ],
          "default": null,
          "description": "Push receiver configuration. Required when scheduling_mode is ExternalPush. Ignored for SelfHosted"
        },
        "monitor_config": {
          "oneOf": [
            { "$ref": "meteor://scheduler/schedule-monitor-config" },
            { "type": "null" }
          ],
          "default": null,
          "description": "Schedule monitor configuration. Optional for both modes. Enables expected_at vs actual_at tracking"
        }
      }
    },
    "PeriodicScheduler": {
      "$id": "meteor://scheduler/periodic-scheduler",
      "type": "object",
      "description": "Extended PeriodicScheduler with mode-aware start() and optional push receiver/monitor. Generic over B: Broker, S: SchedulerBackend.",
      "properties": {
        "tasks": {
          "type": "string",
          "const": "Vec<PeriodicTask>",
          "description": "Registered periodic tasks"
        },
        "broker": {
          "type": "string",
          "const": "Arc<B>",
          "description": "Broker for publishing task messages"
        },
        "backend": {
          "type": "string",
          "const": "Arc<S>",
          "description": "Scheduler backend (determines mode via scheduling_mode())"
        },
        "config": {
          "$ref": "meteor://scheduler/periodic-scheduler-config"
        },
        "shutdown": {
          "type": "string",
          "const": "CancellationToken",
          "description": "Shutdown signal for both modes"
        },
        "push_receiver": {
          "oneOf": [
            { "type": "string", "const": "Arc<PushReceiver>" },
            { "type": "null" }
          ],
          "description": "Push receiver instance, created during start() in ExternalPush mode"
        },
        "monitor": {
          "oneOf": [
            { "type": "string", "const": "Arc<ScheduleMonitor>" },
            { "type": "null" }
          ],
          "description": "Schedule monitor instance, created during start() if monitor_config is set"
        }
      },
      "required": ["tasks", "broker", "backend", "config", "shutdown"]
    },
    "SchedulerBackendTraitExtension": {
      "$id": "meteor://scheduler/backend-trait-extension",
      "type": "object",
      "description": "New methods added to SchedulerBackend trait for mode selection and external registration.",
      "properties": {
        "scheduling_mode": {
          "type": "object",
          "description": "fn scheduling_mode(&self) -> SchedulingMode. Default returns SelfHosted.",
          "properties": {
            "signature": { "const": "fn scheduling_mode(&self) -> SchedulingMode" },
            "default_return": { "const": "SchedulingMode::SelfHosted" },
            "is_async": { "const": false }
          }
        },
        "register_external_schedule": {
          "type": "object",
          "description": "async fn register_external_schedule(&self, task: &PeriodicTask) -> Result<(), TaskError>. Default is no-op.",
          "properties": {
            "signature": { "const": "async fn register_external_schedule(&self, task: &PeriodicTask) -> Result<(), TaskError>" },
            "default_return": { "const": "Ok(())" },
            "is_async": { "const": true }
          }
        }
      }
    },
    "ModeSelectionDecisionTable": {
      "$id": "meteor://scheduler/mode-decision-table",
      "type": "array",
      "description": "Maps backend type to scheduling mode and start() behavior.",
      "items": {
        "type": "object",
        "properties": {
          "backend": { "type": "string" },
          "mode": { "$ref": "meteor://scheduler/scheduling-mode" },
          "start_behavior": { "type": "string" },
          "leader_election": { "type": "boolean" },
          "push_receiver": { "type": "boolean" }
        }
      },
      "default": [
        { "backend": "IonSchedulerBackend", "mode": "self_hosted", "start_behavior": "run_leader_loop", "leader_election": true, "push_receiver": false },
        { "backend": "MemorySchedulerBackend", "mode": "self_hosted", "start_behavior": "run_leader_loop", "leader_election": true, "push_receiver": false },
        { "backend": "CloudSchedulerBackend", "mode": "external_push", "start_behavior": "start_push_receiver + register_tasks", "leader_election": false, "push_receiver": true },
        { "backend": "K8sCronJobBackend", "mode": "external_push", "start_behavior": "start_push_receiver + register_tasks", "leader_election": false, "push_receiver": true }
      ]
    }
  }
}
```