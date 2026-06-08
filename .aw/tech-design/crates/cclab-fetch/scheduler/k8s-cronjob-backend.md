---
id: k8s-cronjob-backend
main_spec_ref: "crates/cclab-fetch/scheduler/k8s-cronjob-backend.md"
merge_strategy: new
---

# K8s Cronjob Backend

## Overview

Implements `SchedulerBackend` trait backed by Kubernetes CronJob resources. Replaces the self-hosted leader election loop (acquire_leader → evaluate → enqueue → renew) with K8s-managed cron scheduling. CronJob pods run a minimal HTTP client container that POSTs to the shared push receiver endpoint (`/scheduler/push/{task_name}`), routing all triggers through the unified push path consistent with the Cloud Scheduler pattern.

Task state (enabled, last_run_at, total_run_count) is tracked locally via `get_task_state`/`set_task_state` backed by an in-memory store with `Arc<RwLock<HashMap>>`. Schedule CRUD (create, update, delete, suspend, resume) maps to K8s CronJob API operations via kube-rs `Api<CronJob>` client. Authentication uses shared HMAC-SHA256 secret — CronJob pods sign requests with `X-Scheduler-Signature` header, push receiver validates by recomputing HMAC. Secret injected via K8s Secret as environment variable.

Feature-gated under `k8s-scheduler` Cargo feature. Follows the same registration pattern as `CloudSchedulerBackend` — `#[cfg(feature = "k8s-scheduler")] pub mod k8s_cronjob_backend` in `scheduler/mod.rs`. Reuses kube-rs client setup patterns from existing `K8sJobExecutor` in `executor/k8s.rs`.

Source: `crates/cclab-queue/src/scheduler/backend.rs` (SchedulerBackend trait), `crates/cclab-queue/src/executor/k8s.rs` (kube-rs patterns)
## Requirements

| ID | Title | Priority | Acceptance Criteria |
|----|-------|----------|---------------------|
| R1 | SchedulerBackend trait implementation | P0 | `K8sCronJobBackend` implements `SchedulerBackend` trait: `acquire_leader(ttl)` returns `Ok(true)` always (no leader election needed — K8s CronJob controller is the authoritative scheduler), `renew_leader(ttl)` returns `Ok(true)` (no-op), `release_leader()` returns `Ok(())` (no-op) |
| R2 | Task state management | P0 | `get_task_state(name)` and `set_task_state(name, state)` track `TaskScheduleState { enabled, last_run_at, total_run_count }` locally. Default state for unknown tasks: `{ enabled: true, last_run_at: None, total_run_count: 0 }`. `record_task_run(name)` increments `total_run_count` and sets `last_run_at` to `Utc::now()` |
| R3 | CronJob CRUD via K8s API | P0 | Full lifecycle management of K8s CronJob resources via kube-rs `Api<CronJob>`: `create_cronjob(spec)` creates CronJob, `update_cronjob(spec)` patches CronJob, `delete_cronjob(name)` deletes CronJob, `get_cronjob(name)` gets CronJob, `list_cronjobs()` lists CronJobs in configured namespace. Namespace configurable via `K8sCronJobConfig` |
| R4 | CronJob container spec | P1 | CronJob pod runs a minimal HTTP client container that POSTs to the push receiver endpoint (`{target_base_url}/scheduler/push/{task_name}`). Request body contains `TaskMessage` JSON. Request signed with HMAC-SHA256 using shared secret in `X-Scheduler-Signature` header. Container image configurable via `K8sCronJobConfig.trigger_image`. Resource limits (CPU, memory) configurable per CronJob |
| R5 | HMAC authentication | P0 | CronJob pods sign requests: `X-Scheduler-Signature: sha256={hex(hmac_sha256(secret, body))}`. Secret injected as K8s Secret mounted as env var `SCHEDULER_HMAC_SECRET`. Push receiver validates signature by recomputing HMAC over request body with the same secret. Rejects requests with missing or invalid signature |
| R6 | Feature gate | P1 | Entire module conditionally compiled under `#[cfg(feature = "k8s-scheduler")]`. `K8sCronJobBackend` and `K8sCronJobConfig` only available when feature is enabled. kube-rs dependency only pulled when `k8s-scheduler` feature is active |
| R7 | Suspend/resume mapping | P1 | `pause_task(name)` patches CronJob `spec.suspend = true` AND sets local `TaskScheduleState.enabled = false`. `resume_task(name)` patches `spec.suspend = false` AND sets local `enabled = true`. Both local state and K8s state stay in sync |
| R8 | Error mapping | P1 | kube-rs API errors map to `TaskError`: 404 → `TaskError::NotFound`, 401/403 → `TaskError::AuthenticationError`, 409 → `TaskError::AlreadyExists`, 5xx → `TaskError::BackendError`. kube-rs transport errors → `TaskError::ConnectionError` |

### Constraints

- All trait methods returning `Result` use `TaskError` as the error type
- `K8sCronJobBackend` requires `Send + Sync` (per `SchedulerBackend` trait bound)
- Uses kube-rs `Client` with default kubeconfig (local) or in-cluster config (production)
- CronJob names follow K8s naming constraints: lowercase alphanumeric + hyphens, max 52 chars
- Schedule expressions use standard unix-cron format (5 fields, not 6-field with seconds)
- CronJob `concurrencyPolicy` defaults to `Forbid` (prevent overlapping trigger executions)
- CronJob `successfulJobsHistoryLimit` and `failedJobsHistoryLimit` configurable, default 1 and 3
## Scenarios

### S1: Leader election is no-op for K8s-managed backend (R1)

**GIVEN** a `K8sCronJobBackend` instance
**WHEN** `acquire_leader(Duration::from_secs(15))` is called
**THEN** returns `Ok(true)` immediately without network calls; `renew_leader()` returns `Ok(true)`; `release_leader()` returns `Ok(())`

### S2: Create a K8s CronJob for a periodic task (R3, R4)

**GIVEN** a configured `K8sCronJobBackend` with namespace=`scheduler`, target_base_url=`https://app.example.com`
**WHEN** `create_cronjob()` is called with name=`daily-cleanup`, schedule=`0 2 * * *`, trigger_image=`gcr.io/project/scheduler-trigger:latest`
**THEN** creates a K8s CronJob in namespace `scheduler` with: `spec.schedule = "0 2 * * *"`, `spec.concurrencyPolicy = "Forbid"`, `spec.jobTemplate.spec.template.spec.containers[0].image = trigger_image`, container command = `POST https://app.example.com/scheduler/push/daily-cleanup` with HMAC-signed body containing `TaskMessage` JSON; returns `Ok(CronJob)`

### S3: Suspend and resume a scheduled task (R7, R2)

**GIVEN** a K8s CronJob `daily-cleanup` with `spec.suspend = false`
**WHEN** `pause_task("daily-cleanup")` is called
**THEN** patches CronJob `spec.suspend = true` via K8s API; local `TaskScheduleState.enabled` set to `false`; subsequent `get_task_state("daily-cleanup")` returns `enabled: false`
**WHEN** `resume_task("daily-cleanup")` is called
**THEN** patches CronJob `spec.suspend = false`; local `enabled` set to `true`

### S4: Record task run updates local state (R2)

**GIVEN** a task `hourly-sync` with `total_run_count = 5` and `last_run_at = 2026-03-26T10:00:00Z`
**WHEN** `record_task_run("hourly-sync")` is called
**THEN** `total_run_count` becomes `6`; `last_run_at` updated to current `Utc::now()`

### S5: CronJob pod sends HMAC-signed request to push receiver (R4, R5)

**GIVEN** a CronJob pod with `SCHEDULER_HMAC_SECRET=my-secret-key` env var
**WHEN** the pod container executes and POSTs to `/scheduler/push/daily-cleanup`
**THEN** request includes `X-Scheduler-Signature: sha256={hex(hmac_sha256("my-secret-key", body))}` header; push receiver recomputes HMAC and validates; on match, processes `TaskMessage` from body and calls `broker.publish()`

### S6: K8s API error maps to TaskError (R8)

**GIVEN** a `K8sCronJobBackend` calling `get_cronjob("nonexistent")`
**WHEN** K8s API returns 404 Not Found
**THEN** returns `Err(TaskError::NotFound("nonexistent"))`
**WHEN** K8s API returns 403 Forbidden
**THEN** returns `Err(TaskError::AuthenticationError(...))`
**WHEN** K8s API returns 500 Internal Server Error
**THEN** returns `Err(TaskError::BackendError(...))`

### S7: Feature gate excludes k8s-scheduler module (R6)

**GIVEN** a Cargo.toml without `k8s-scheduler` feature
**WHEN** the crate is compiled
**THEN** `K8sCronJobBackend` and `K8sCronJobConfig` are not available; no kube-rs dependencies are pulled; existing Ion, InMemory, and CloudScheduler backends compile normally

### S8: Delete a K8s CronJob (R3)

**GIVEN** an existing K8s CronJob `daily-cleanup`
**WHEN** `delete_cronjob("daily-cleanup")` is called
**THEN** deletes the CronJob resource from K8s API; returns `Ok(())`; local task state is removed
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
  - file: crates/cclab-queue/src/scheduler/k8s_cronjob_backend.rs
    action: create
    description: |
      New file implementing K8sCronJobBackend and K8sCronJobConfig.
      Implements SchedulerBackend trait with no-op leader election and
      local task state management. Includes kube-rs CronJob CRUD operations
      (create/update/delete/get/list/suspend/resume), CronJob resource
      construction with trigger pod container spec, HMAC secret mounting,
      and K8s API error-to-TaskError mapping.
    structs:
      - K8sCronJobBackend
      - K8sCronJobConfig
      - TriggerPodResources
    trait_impls:
      - "SchedulerBackend for K8sCronJobBackend"
    methods:
      - "async fn acquire_leader(&self, _ttl: Duration) -> Result<bool, TaskError>  # returns Ok(true)"
      - "async fn renew_leader(&self, _ttl: Duration) -> Result<bool, TaskError>  # returns Ok(true)"
      - "async fn release_leader(&self) -> Result<(), TaskError>  # no-op"
      - "async fn get_task_state(&self, name: &str) -> Result<TaskScheduleState, TaskError>"
      - "async fn set_task_state(&self, name: &str, state: &TaskScheduleState) -> Result<(), TaskError>"
      - "async fn pause_task(&self, name: &str) -> Result<(), TaskError>  # patches suspend=true + local state"
      - "async fn resume_task(&self, name: &str) -> Result<(), TaskError>  # patches suspend=false + local state"
      - "async fn create_cronjob(&self, name: &str, schedule: &str, task_message: &TaskMessage) -> Result<CronJob, TaskError>"
      - "async fn update_cronjob(&self, name: &str, schedule: &str) -> Result<CronJob, TaskError>"
      - "async fn delete_cronjob(&self, name: &str) -> Result<(), TaskError>"
      - "async fn get_cronjob(&self, name: &str) -> Result<CronJob, TaskError>"
      - "async fn list_cronjobs(&self) -> Result<Vec<CronJob>, TaskError>"
      - "fn build_cronjob_spec(&self, name: &str, schedule: &str, task_message: &TaskMessage) -> CronJob  # constructs K8s CronJob resource"
      - "fn map_kube_error(err: kube::Error) -> TaskError  # error mapping"

  - file: crates/cclab-queue/src/scheduler/mod.rs
    action: modify
    description: |
      Add conditional module declaration and re-export for k8s_cronjob_backend.
      Follows same pattern as cloud_scheduler_backend registration.
    additions:
      - '#[cfg(feature = "k8s-scheduler")] pub mod k8s_cronjob_backend;'
      - '#[cfg(feature = "k8s-scheduler")] pub use k8s_cronjob_backend::{K8sCronJobBackend, K8sCronJobConfig};'

  - file: crates/cclab-queue/Cargo.toml
    action: modify
    description: |
      Add k8s-scheduler feature flag and conditional kube-rs dependencies.
    additions:
      - 'k8s-scheduler feature: ["dep:kube", "dep:k8s-openapi"]'
    notes: |
      kube and k8s-openapi may already be optional dependencies for the k8s-executor feature.
      If so, k8s-scheduler should share these deps (both features activate the same optional deps).
      Do NOT duplicate dependency entries — add k8s-scheduler to existing activation lists.
      k8s-openapi requires a version feature (e.g., v1_28) matching the target K8s cluster version.
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
  "title": "K8s CronJob Backend — Data Models",
  "$defs": {
    "K8sCronJobConfig": {
      "$id": "meteor://scheduler/k8s-cronjob-config",
      "type": "object",
      "description": "Configuration for K8sCronJobBackend. Feature-gated under k8s-scheduler.",
      "properties": {
        "namespace": {
          "type": "string",
          "default": "default",
          "description": "K8s namespace for CronJob resources"
        },
        "target_base_url": {
          "type": "string",
          "format": "uri",
          "description": "Base URL of the push receiver endpoint (e.g., https://app.example.com)"
        },
        "trigger_image": {
          "type": "string",
          "description": "Container image for the CronJob trigger pod (minimal HTTP client that POSTs to push receiver)"
        },
        "hmac_secret_name": {
          "type": "string",
          "default": "scheduler-hmac-secret",
          "description": "K8s Secret name containing the HMAC signing key"
        },
        "hmac_secret_key": {
          "type": "string",
          "default": "hmac-key",
          "description": "Key within the K8s Secret that holds the HMAC value"
        },
        "concurrency_policy": {
          "type": "string",
          "enum": ["Allow", "Forbid", "Replace"],
          "default": "Forbid",
          "description": "CronJob concurrencyPolicy — Forbid prevents overlapping trigger executions"
        },
        "successful_jobs_history_limit": {
          "type": "integer",
          "default": 1,
          "minimum": 0,
          "description": "Number of successful finished CronJob pods to retain"
        },
        "failed_jobs_history_limit": {
          "type": "integer",
          "default": 3,
          "minimum": 0,
          "description": "Number of failed finished CronJob pods to retain"
        },
        "default_resources": {
          "$ref": "meteor://scheduler/trigger-pod-resources",
          "description": "Default resource limits/requests for trigger pods"
        },
        "kubeconfig_path": {
          "oneOf": [
            { "type": "string", "description": "Path to kubeconfig file (local dev)" },
            { "type": "null" }
          ],
          "default": null,
          "description": "If null, uses in-cluster config or default kubeconfig"
        }
      },
      "required": ["target_base_url", "trigger_image"]
    },
    "K8sCronJobBackend": {
      "$id": "meteor://scheduler/k8s-cronjob-backend",
      "type": "object",
      "description": "SchedulerBackend implementation backed by K8s CronJob resources. Requires Send + Sync.",
      "properties": {
        "config": {
          "$ref": "meteor://scheduler/k8s-cronjob-config"
        },
        "client": {
          "type": "string",
          "const": "kube::Client",
          "description": "kube-rs client for K8s API access"
        },
        "cronjob_api": {
          "type": "string",
          "const": "Api<CronJob>",
          "description": "Namespaced CronJob API handle"
        },
        "task_states": {
          "type": "string",
          "const": "Arc<RwLock<HashMap<String, TaskScheduleState>>>",
          "description": "In-memory task state store"
        }
      },
      "required": ["config", "client", "cronjob_api", "task_states"]
    },
    "TriggerPodResources": {
      "$id": "meteor://scheduler/trigger-pod-resources",
      "type": "object",
      "description": "Resource limits and requests for the CronJob trigger pod container.",
      "properties": {
        "cpu_limit": {
          "type": "string",
          "default": "100m",
          "description": "CPU limit (e.g., 100m, 0.5)"
        },
        "memory_limit": {
          "type": "string",
          "default": "64Mi",
          "description": "Memory limit (e.g., 64Mi, 128Mi)"
        },
        "cpu_request": {
          "type": "string",
          "default": "50m",
          "description": "CPU request"
        },
        "memory_request": {
          "type": "string",
          "default": "32Mi",
          "description": "Memory request"
        }
      }
    },
    "K8sErrorMapping": {
      "$id": "meteor://scheduler/k8s-error-mapping",
      "type": "object",
      "description": "kube-rs API error to TaskError variant mapping.",
      "properties": {
        "404": { "const": "TaskError::NotFound" },
        "401": { "const": "TaskError::AuthenticationError" },
        "403": { "const": "TaskError::AuthenticationError" },
        "409": { "const": "TaskError::AlreadyExists" },
        "5xx": { "const": "TaskError::BackendError" },
        "transport": { "const": "TaskError::ConnectionError" }
      }
    },
    "HmacSignature": {
      "$id": "meteor://scheduler/hmac-signature",
      "type": "object",
      "description": "HMAC-SHA256 signature scheme for CronJob trigger authentication.",
      "properties": {
        "header_name": {
          "type": "string",
          "const": "X-Scheduler-Signature"
        },
        "format": {
          "type": "string",
          "const": "sha256={hex_digest}",
          "description": "Header value format: sha256= prefix + hex-encoded HMAC-SHA256 digest"
        },
        "algorithm": {
          "type": "string",
          "const": "HMAC-SHA256"
        },
        "secret_env_var": {
          "type": "string",
          "const": "SCHEDULER_HMAC_SECRET",
          "description": "Environment variable name in CronJob pod containing the shared secret"
        }
      }
    },
    "FeatureGate": {
      "$id": "meteor://scheduler/k8s-feature-gate",
      "type": "object",
      "description": "Cargo feature gate for k8s-scheduler module.",
      "properties": {
        "k8s-scheduler": { "const": "K8sCronJobBackend, K8sCronJobConfig" }
      }
    }
  }
}
```