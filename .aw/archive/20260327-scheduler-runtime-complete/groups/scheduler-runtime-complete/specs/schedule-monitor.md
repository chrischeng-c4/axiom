---
id: schedule-monitor
main_spec_ref: "crates/cclab-fetch/scheduler/schedule-monitor.md"
merge_strategy: new
fill_sections: [overview, requirements, scenarios, schema, changes]
filled_sections: [overview, requirements, scenarios, schema, changes]
create_complete: true
---

# Schedule Monitor

## Overview

Tracks expected trigger times vs actual trigger times for periodic tasks. Computes `expected_at` from the task's cron expression or interval schedule, records `actual_at` when the push receiver or self-hosted tick loop delivers a trigger callback, and classifies each fire as `on_time`, `late`, or `missed` based on a configurable per-task leeway duration.

Emits Prometheus metrics via existing cclab-queue metrics module: counter `scheduler_task_fire_total{task_name, status}` (status: `on_time`, `late`, `missed`), histogram `scheduler_task_latency_seconds{task_name}` (seconds between `expected_at` and `actual_at`). Supports configurable webhook URL per task for missed schedule alerts — sends POST with JSON payload containing task name, expected_at, and detection timestamp.

Hooks into both trigger paths uniformly: push receiver calls `monitor.record_trigger(task_name, actual_at)` on each callback (R8 in push-receiver spec); self-hosted tick loop calls the same method after enqueue. A background `check_missed` task runs on a configurable interval to detect tasks whose `expected_at` has passed beyond leeway without a corresponding `actual_at` recording, marking them as `missed` and invoking the webhook callback.

Owned as `Arc<ScheduleMonitor>` — shared across push receiver, periodic scheduler, and the background missed-check task. `ScheduleMonitor` requires `Send + Sync`.

Source: `crates/cclab-queue/src/scheduler/periodic.rs` (PeriodicScheduler, PeriodicTask, cron/interval schedules), push-receiver spec (R8, S11 — monitor integration point)
## Requirements

| ID | Title | Priority | Acceptance Criteria |
|----|-------|----------|---------------------|
| R1 | Expected time computation | P0 | `ScheduleMonitor::register_task(name, schedule)` accepts cron expression (5-field unix-cron) or `Duration` interval. Computes `expected_at` for the next fire using `cron::Schedule::upcoming()` for cron expressions, or `last_fire_at + interval` for duration-based schedules. Updates `expected_at` after each recorded trigger |
| R2 | Actual time recording | P0 | `record_trigger(task_name, actual_at)` records the actual fire timestamp. Computes latency as `actual_at - expected_at`. Classifies fire status: `on_time` if latency <= leeway, `late` if latency > leeway but trigger received, `missed` if no trigger received and detected by background check. Updates `expected_at` to next scheduled time after recording |
| R3 | Per-task leeway configuration | P0 | Each registered task has a configurable `leeway: Duration` (default from `ScheduleMonitorConfig.default_leeway`). Override per task via `register_task(name, schedule, Some(leeway))`. Leeway defines the threshold between `on_time` and `late` classification |
| R4 | Prometheus metrics emission | P0 | On each `record_trigger()` call, increment `scheduler_task_fire_total{task_name, status}` counter where status is `on_time` or `late`. Observe latency in `scheduler_task_latency_seconds{task_name}` histogram. On missed detection, increment `scheduler_task_fire_total{task_name, status="missed"}`. Uses existing cclab-queue metrics module (prometheus crate) |
| R5 | Missed schedule detection | P0 | Background tokio task runs on `ScheduleMonitorConfig.check_interval` (default 60s). For each registered task, if `Utc::now() > expected_at + leeway` and no `actual_at` recorded since `expected_at`, marks fire as `missed`. Emits missed metric and advances `expected_at` to next scheduled time |
| R6 | Webhook callback for missed alerts | P1 | `ScheduleMonitorConfig.webhook_url: Option<String>` — global webhook endpoint. Per-task override via `TaskMonitorEntry.webhook_url`. On missed detection, POST JSON `{ "task_name": "...", "expected_at": "...", "detected_at": "...", "status": "missed" }` to webhook URL. Non-blocking (tokio::spawn), failure logged at `warn` level, does not affect monitor operation |
| R7 | Thread-safe shared ownership | P0 | `ScheduleMonitor` is `Send + Sync`. Internal state protected by `Arc<RwLock<HashMap<String, TaskMonitorEntry>>>`. Shared as `Arc<ScheduleMonitor>` across push receiver, periodic scheduler, and background check task |
| R8 | Graceful lifecycle | P1 | `ScheduleMonitor::start()` spawns the background check task and returns `JoinHandle`. `ScheduleMonitor::stop()` signals shutdown via `tokio::sync::watch` channel and awaits task completion. Dropping `ScheduleMonitor` triggers shutdown signal |

### Constraints

- `cron` crate for cron expression parsing (same as existing PeriodicScheduler)
- `reqwest::Client` for webhook HTTP calls (already a dependency)
- Metrics registered once at construction, not per-call
- `record_trigger()` must be non-blocking for the caller — metric emission and state update are synchronous on the RwLock (write lock held briefly)
- Background check task must not panic — all errors caught and logged
## Scenarios

### S1: Register a task with cron schedule and record on-time trigger (R1, R2, R3, R4)

**GIVEN** a `ScheduleMonitor` with `default_leeway = 30s`
**WHEN** `register_task("daily-cleanup", Cron("0 2 * * *"), None)` is called
**THEN** `expected_at` is computed as next `02:00:00` UTC from `cron::Schedule::upcoming()`; `leeway` defaults to 30s
**WHEN** `record_trigger("daily-cleanup", 2026-03-28T02:00:05Z)` is called (5s after expected)
**THEN** latency = 5s; status = `on_time` (5s <= 30s leeway); `scheduler_task_fire_total{task_name="daily-cleanup", status="on_time"}` incremented; `scheduler_task_latency_seconds{task_name="daily-cleanup"}` observes 5.0; `expected_at` advanced to next `02:00:00` UTC

### S2: Record a late trigger (R2, R3, R4)

**GIVEN** a task `hourly-sync` registered with `interval = 3600s`, `leeway = 60s`, `expected_at = 2026-03-28T10:00:00Z`
**WHEN** `record_trigger("hourly-sync", 2026-03-28T10:02:30Z)` is called (150s after expected)
**THEN** latency = 150s; status = `late` (150s > 60s leeway); `scheduler_task_fire_total{task_name="hourly-sync", status="late"}` incremented; `scheduler_task_latency_seconds{task_name="hourly-sync"}` observes 150.0; `expected_at` advanced to `2026-03-28T11:00:00Z`

### S3: Background check detects missed schedule (R5, R4, R6)

**GIVEN** a task `daily-cleanup` with `expected_at = 2026-03-28T02:00:00Z`, `leeway = 30s`, no `actual_at` recorded since expected
**WHEN** background check runs at `2026-03-28T02:01:00Z` (60s after expected, beyond 30s leeway)
**THEN** status = `missed`; `scheduler_task_fire_total{task_name="daily-cleanup", status="missed"}` incremented; `expected_at` advanced to next scheduled time; webhook POST sent with `{ "task_name": "daily-cleanup", "expected_at": "2026-03-28T02:00:00Z", "detected_at": "2026-03-28T02:01:00Z", "status": "missed" }`

### S4: Per-task leeway overrides default (R3)

**GIVEN** a `ScheduleMonitor` with `default_leeway = 30s`
**WHEN** `register_task("critical-job", Cron("*/5 * * * *"), Some(Duration::from_secs(10)))` is called
**THEN** task `critical-job` uses 10s leeway (not 30s default)
**WHEN** `record_trigger("critical-job", expected_at + 15s)` is called
**THEN** status = `late` (15s > 10s leeway)

### S5: Webhook failure does not affect monitor operation (R6)

**GIVEN** a `ScheduleMonitor` with `webhook_url = "https://hooks.example.com/alert"` and webhook endpoint is unreachable
**WHEN** missed detection triggers webhook POST
**THEN** reqwest returns connection error; error logged at `warn` level; monitor continues operating; metrics still emitted correctly; next check cycle runs on schedule

### S6: Push receiver integration records actual_at (R2, R7)

**GIVEN** `PushReceiver` holds `Arc<ScheduleMonitor>` and task `hourly-sync` is registered
**WHEN** authenticated push request arrives for `/scheduler/push/hourly-sync`
**THEN** push receiver calls `monitor.record_trigger("hourly-sync", Utc::now())` before `broker.publish()`; monitor updates state under write lock; concurrent push requests for different tasks do not block each other (lock granularity is per-entry via RwLock on HashMap)

### S7: Record trigger for unregistered task is no-op (R2)

**GIVEN** a `ScheduleMonitor` with only `daily-cleanup` registered
**WHEN** `record_trigger("unknown-task", Utc::now())` is called
**THEN** returns `Ok(())` without error; no metrics emitted; logged at `debug` level as unmonitored task

### S8: Monitor lifecycle start and stop (R8)

**GIVEN** a `ScheduleMonitor` with `check_interval = 60s`
**WHEN** `monitor.start()` is called
**THEN** background tokio task is spawned; returns `JoinHandle`; task checks all registered tasks every 60s
**WHEN** `monitor.stop()` is called
**THEN** shutdown signal sent via watch channel; background task exits gracefully within one check interval; `JoinHandle` resolves to `Ok(())`

### S9: Interval-based task computes expected_at correctly (R1)

**GIVEN** a task `every-5m` registered with `interval = 300s` and first `record_trigger()` at `2026-03-28T10:00:00Z`
**WHEN** first trigger is recorded
**THEN** `expected_at` advances to `2026-03-28T10:05:00Z`
**WHEN** second trigger arrives at `2026-03-28T10:05:02Z`
**THEN** latency = 2s; `expected_at` advances to `2026-03-28T10:10:00Z`
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
  - file: crates/cclab-queue/src/scheduler/schedule_monitor.rs
    action: create
    description: |
      New file implementing ScheduleMonitor, ScheduleMonitorConfig,
      TaskMonitorEntry, TaskSchedule, FireStatus, and WebhookPayload.
      Core logic: register_task() adds entry with computed expected_at,
      record_trigger() classifies fire status and emits metrics,
      check_missed() background loop detects missed fires.
      Webhook callback via reqwest. Prometheus metrics via IntCounterVec
      and HistogramVec.
    structs:
      - ScheduleMonitor
      - ScheduleMonitorConfig
      - TaskMonitorEntry
      - WebhookPayload
      - MonitorMetrics
    enums:
      - TaskSchedule
      - FireStatus
    methods:
      - "fn new(config: ScheduleMonitorConfig) -> Result<Self, TaskError>  # creates monitor, registers metrics"
      - "fn register_task(&self, name: &str, schedule: TaskSchedule, leeway: Option<Duration>, webhook_url: Option<String>) -> Result<(), TaskError>  # adds task entry, computes initial expected_at"
      - "fn record_trigger(&self, task_name: &str, actual_at: DateTime<Utc>) -> Result<Option<FireStatus>, TaskError>  # classifies fire, emits metrics, advances expected_at. Returns None for unregistered tasks"
      - "fn start(&self) -> JoinHandle<()>  # spawns background check_missed task"
      - "fn stop(&self)  # signals shutdown via watch channel"
      - "async fn check_missed(&self)  # iterates registered tasks, detects missed, emits metrics + webhook"
      - "async fn send_webhook(&self, url: &str, payload: &WebhookPayload)  # POST JSON payload, logs errors"
      - "fn compute_next_expected(schedule: &TaskSchedule, after: DateTime<Utc>) -> Option<DateTime<Utc>>  # computes next expected_at from cron or interval"
      - "fn classify_fire(latency: Duration, leeway: Duration) -> FireStatus  # on_time if latency <= leeway, else late"

  - file: crates/cclab-queue/src/scheduler/mod.rs
    action: modify
    description: |
      Add module declaration and re-export for schedule_monitor.
    additions:
      - 'pub mod schedule_monitor;'
      - 'pub use schedule_monitor::{ScheduleMonitor, ScheduleMonitorConfig, FireStatus};'
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
  "title": "Schedule Monitor — Data Models",
  "$defs": {
    "ScheduleMonitorConfig": {
      "$id": "meteor://scheduler/schedule-monitor-config",
      "type": "object",
      "description": "Configuration for ScheduleMonitor.",
      "properties": {
        "default_leeway_secs": {
          "type": "integer",
          "default": 30,
          "minimum": 1,
          "description": "Default leeway in seconds for on_time vs late classification"
        },
        "check_interval_secs": {
          "type": "integer",
          "default": 60,
          "minimum": 5,
          "description": "Interval in seconds between missed-schedule background checks"
        },
        "webhook_url": {
          "oneOf": [
            { "type": "string", "format": "uri" },
            { "type": "null" }
          ],
          "default": null,
          "description": "Global webhook URL for missed schedule alerts. POST JSON payload on missed detection"
        },
        "webhook_timeout_secs": {
          "type": "integer",
          "default": 10,
          "minimum": 1,
          "description": "Timeout in seconds for webhook HTTP calls"
        }
      }
    },
    "ScheduleMonitor": {
      "$id": "meteor://scheduler/schedule-monitor",
      "type": "object",
      "description": "Tracks expected vs actual trigger times. Shared as Arc<ScheduleMonitor>. Requires Send + Sync.",
      "properties": {
        "config": {
          "$ref": "meteor://scheduler/schedule-monitor-config"
        },
        "tasks": {
          "type": "string",
          "const": "Arc<RwLock<HashMap<String, TaskMonitorEntry>>>",
          "description": "Per-task monitoring state"
        },
        "metrics": {
          "$ref": "meteor://scheduler/monitor-metrics",
          "description": "Prometheus metric handles registered at construction"
        },
        "http_client": {
          "type": "string",
          "const": "reqwest::Client",
          "description": "HTTP client for webhook calls"
        },
        "shutdown_tx": {
          "type": "string",
          "const": "tokio::sync::watch::Sender<bool>",
          "description": "Shutdown signal sender for background task"
        }
      },
      "required": ["config", "tasks", "metrics", "shutdown_tx"]
    },
    "TaskMonitorEntry": {
      "$id": "meteor://scheduler/task-monitor-entry",
      "type": "object",
      "description": "Per-task monitoring state tracked by ScheduleMonitor.",
      "properties": {
        "task_name": {
          "type": "string",
          "description": "Unique task identifier"
        },
        "schedule": {
          "$ref": "meteor://scheduler/task-schedule",
          "description": "Cron expression or interval duration"
        },
        "leeway": {
          "type": "string",
          "const": "Duration",
          "description": "Threshold between on_time and late. From per-task override or config default"
        },
        "expected_at": {
          "oneOf": [
            { "type": "string", "format": "date-time" },
            { "type": "null" }
          ],
          "description": "Next expected trigger time. Null before first computation"
        },
        "last_actual_at": {
          "oneOf": [
            { "type": "string", "format": "date-time" },
            { "type": "null" }
          ],
          "description": "Most recent actual trigger timestamp"
        },
        "webhook_url": {
          "oneOf": [
            { "type": "string", "format": "uri" },
            { "type": "null" }
          ],
          "default": null,
          "description": "Per-task webhook URL override. If null, uses global webhook_url from config"
        }
      },
      "required": ["task_name", "schedule", "leeway"]
    },
    "TaskSchedule": {
      "$id": "meteor://scheduler/task-schedule",
      "type": "object",
      "description": "Tagged union: cron expression or fixed interval.",
      "oneOf": [
        {
          "type": "object",
          "properties": {
            "type": { "const": "cron" },
            "expression": { "type": "string", "description": "5-field unix-cron expression" },
            "parsed": { "type": "string", "const": "cron::Schedule", "description": "Parsed cron schedule for upcoming() iteration" }
          },
          "required": ["type", "expression"]
        },
        {
          "type": "object",
          "properties": {
            "type": { "const": "interval" },
            "duration": { "type": "string", "const": "Duration", "description": "Fixed interval between fires" }
          },
          "required": ["type", "duration"]
        }
      ]
    },
    "FireStatus": {
      "$id": "meteor://scheduler/fire-status",
      "type": "string",
      "enum": ["on_time", "late", "missed"],
      "description": "Classification of a task fire relative to expected_at and leeway."
    },
    "WebhookPayload": {
      "$id": "meteor://scheduler/webhook-payload",
      "type": "object",
      "description": "JSON payload POSTed to webhook URL on missed detection.",
      "properties": {
        "task_name": { "type": "string" },
        "expected_at": { "type": "string", "format": "date-time" },
        "detected_at": { "type": "string", "format": "date-time" },
        "status": { "const": "missed" }
      },
      "required": ["task_name", "expected_at", "detected_at", "status"]
    },
    "MonitorMetrics": {
      "$id": "meteor://scheduler/monitor-metrics",
      "type": "object",
      "description": "Prometheus metric handles for schedule monitoring.",
      "properties": {
        "scheduler_task_fire_total": {
          "type": "string",
          "const": "IntCounterVec<task_name, status>",
          "description": "Total task fires by status (on_time, late, missed)"
        },
        "scheduler_task_latency_seconds": {
          "type": "string",
          "const": "HistogramVec<task_name>",
          "description": "Latency between expected_at and actual_at in seconds"
        }
      }
    }
  }
}
```

# Reviews
