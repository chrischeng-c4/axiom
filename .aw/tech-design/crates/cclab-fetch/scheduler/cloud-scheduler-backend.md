---
id: cloud-scheduler-backend
main_spec_ref: "crates/cclab-fetch/scheduler/cloud-scheduler-backend.md"
merge_strategy: new
---

# Cloud Scheduler Backend

## Overview

Implements `SchedulerBackend` trait backed by GCP Cloud Scheduler service. Replaces the self-hosted leader election loop (acquire_leader → evaluate → enqueue → renew) with GCP-managed cron/interval scheduling. Cloud Scheduler creates HTTP-target jobs that POST to the application's task endpoint, offloading schedule evaluation, leader election, and distributed coordination to GCP infrastructure. Task state (enabled, last_run_at, total_run_count) is tracked locally via `get_task_state`/`set_task_state` backed by an in-memory store with optional persistence. Schedule CRUD (create, update, delete, pause, resume) maps to Cloud Scheduler REST API v1 operations. Authentication uses OIDC tokens via GCP metadata server or service account credentials. Feature-gated under `cloud-scheduler` Cargo feature.

Source: `crates/cclab-queue/src/scheduler/backend.rs` (SchedulerBackend trait)
## Requirements

| ID | Title | Priority | Acceptance Criteria |
|----|-------|----------|---------------------|
| R1 | SchedulerBackend trait implementation | P0 | `CloudSchedulerBackend` implements `SchedulerBackend` trait: `acquire_leader(ttl)` returns `Ok(true)` always (no leader election needed — GCP manages scheduling), `renew_leader(ttl)` returns `Ok(true)` (no-op), `release_leader()` returns `Ok(())` (no-op). Leader election is unnecessary because GCP Cloud Scheduler is the single authoritative scheduler |
| R2 | Task state management | P0 | `get_task_state(name)` and `set_task_state(name, state)` track `TaskScheduleState { enabled, last_run_at, total_run_count }` locally. Default state for unknown tasks: `{ enabled: true, last_run_at: None, total_run_count: 0 }`. `record_task_run(name)` increments `total_run_count` and sets `last_run_at` to `Utc::now()` |
| R3 | Schedule CRUD via Cloud Scheduler REST API | P0 | Full lifecycle management of Cloud Scheduler jobs: `create_job(job)` → POST, `update_job(job)` → PATCH, `delete_job(name)` → DELETE, `get_job(name)` → GET, `list_jobs()` → GET, `pause_job(name)` → POST, `resume_job(name)` → POST. All operations target `https://cloudscheduler.googleapis.com/v1/projects/{project}/locations/{location}/jobs` |
| R4 | OIDC authentication | P0 | All REST API calls include `Authorization: Bearer <token>` header. Token obtained via: (1) GCP metadata server `http://metadata.google.internal/...` in production, (2) service account JSON key file for local dev. Token is cached and refreshed before expiry (5-minute buffer). `CloudSchedulerConfig` accepts `oidc_service_account_email` for target audience |
| R5 | HTTP-target job configuration | P1 | Cloud Scheduler jobs use `httpTarget` with: `uri` = application task endpoint URL, `httpMethod` = POST, `body` = base64-encoded `TaskMessage` JSON, `headers` = `{"Content-Type": "application/json"}`, `oidcToken` = `{ serviceAccountEmail, audience }`. Supports both cron (`schedule` field) and interval-based scheduling |
| R6 | Feature gate | P1 | Entire module conditionally compiled under `#[cfg(feature = "cloud-scheduler")]`. `CloudSchedulerBackend` and `CloudSchedulerConfig` only available when feature is enabled. No GCP dependencies pulled otherwise |
| R7 | Pause/resume mapping | P1 | `pause_task(name)` calls Cloud Scheduler `pause_job` API AND sets local `TaskScheduleState.enabled = false`. `resume_task(name)` calls `resume_job` API AND sets local `enabled = true`. Both local state and GCP state stay in sync |
| R8 | Error mapping | P1 | GCP REST API errors map to `TaskError`: 404 → `TaskError::NotFound`, 401/403 → `TaskError::AuthenticationError`, 429 → `TaskError::RateLimited`, 5xx → `TaskError::BackendError`. reqwest transport errors → `TaskError::ConnectionError` |

### Constraints

- All trait methods returning `Result` use `TaskError` as the error type
- `CloudSchedulerBackend` requires `Send + Sync` (per `SchedulerBackend` trait bound)
- HTTP client is `reqwest::Client` with connection pooling
- Cloud Scheduler API v1 only (no beta/alpha endpoints)
- Job names follow pattern: `projects/{project}/locations/{location}/jobs/{task_name}`
- Schedule expressions use standard unix-cron format (5 fields, not 6-field with seconds)
## Scenarios

### S1: Leader election is no-op for cloud-managed backend (R1)

**GIVEN** a `CloudSchedulerBackend` instance
**WHEN** `acquire_leader(Duration::from_secs(15))` is called
**THEN** returns `Ok(true)` immediately without network calls; `renew_leader()` returns `Ok(true)`; `release_leader()` returns `Ok(())`

### S2: Create a Cloud Scheduler job for a periodic task (R3, R5)

**GIVEN** a configured `CloudSchedulerBackend` with project=`my-project`, location=`us-central1`
**WHEN** `create_job()` is called with name=`daily-cleanup`, schedule=`0 2 * * *`, target_uri=`https://app.example.com/tasks/cleanup`
**THEN** sends POST to `https://cloudscheduler.googleapis.com/v1/projects/my-project/locations/us-central1/jobs` with JSON body containing `name`, `schedule`, `timeZone: "UTC"`, `httpTarget: { uri, httpMethod: "POST", body: <base64>, oidcToken: { serviceAccountEmail } }`; returns `Ok(Job)` on 200

### S3: Pause and resume a scheduled task (R7, R2)

**GIVEN** a Cloud Scheduler job `daily-cleanup` in ENABLED state
**WHEN** `pause_task("daily-cleanup")` is called
**THEN** sends POST to `.../jobs/daily-cleanup:pause`; local `TaskScheduleState.enabled` set to `false`; subsequent `is_task_enabled("daily-cleanup")` returns `Ok(false)`
**WHEN** `resume_task("daily-cleanup")` is called
**THEN** sends POST to `.../jobs/daily-cleanup:resume`; local `enabled` set to `true`

### S4: Record task run updates local state (R2)

**GIVEN** a task `hourly-sync` with `total_run_count = 5` and `last_run_at = 2026-03-26T10:00:00Z`
**WHEN** `record_task_run("hourly-sync")` is called
**THEN** `total_run_count` becomes `6`; `last_run_at` updated to current `Utc::now()`

### S5: OIDC token acquisition and caching (R4)

**GIVEN** a `CloudSchedulerBackend` running in GCP environment
**WHEN** the first API call is made
**THEN** token is fetched from metadata server `http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token`; cached in-memory; reused for subsequent calls until 5 minutes before expiry

### S6: GCP API error maps to TaskError (R8)

**GIVEN** a `CloudSchedulerBackend` calling `get_job("nonexistent")`
**WHEN** GCP returns HTTP 404
**THEN** returns `Err(TaskError::NotFound("nonexistent"))`
**WHEN** GCP returns HTTP 403
**THEN** returns `Err(TaskError::AuthenticationError(...))`
**WHEN** GCP returns HTTP 500
**THEN** returns `Err(TaskError::BackendError(...))`

### S7: Feature gate excludes cloud-scheduler module (R6)

**GIVEN** a Cargo.toml without `cloud-scheduler` feature
**WHEN** the crate is compiled
**THEN** `CloudSchedulerBackend` and `CloudSchedulerConfig` are not available; no reqwest or GCP-related dependencies are pulled

### S8: Delete a Cloud Scheduler job (R3)

**GIVEN** an existing Cloud Scheduler job `daily-cleanup`
**WHEN** `delete_job("daily-cleanup")` is called
**THEN** sends DELETE to `.../jobs/daily-cleanup`; returns `Ok(())` on 200; local task state is removed
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
  - file: crates/cclab-queue/src/scheduler/cloud_scheduler_backend.rs
    action: create
    description: |
      New file implementing CloudSchedulerBackend and CloudSchedulerConfig.
      Implements SchedulerBackend trait with no-op leader election and
      local task state management. Includes OIDC token caching, Cloud Scheduler
      REST API client methods (create/update/delete/get/list/pause/resume job),
      HTTP error-to-TaskError mapping.
    structs:
      - CloudSchedulerBackend
      - CloudSchedulerConfig
      - OidcTokenCache
      - CloudSchedulerJob (serde deserialize for API responses)
      - HttpTarget (serde serialize/deserialize)
    trait_impls:
      - "SchedulerBackend for CloudSchedulerBackend"
    methods:
      - "async fn acquire_leader(&self, _ttl: Duration) -> Result<bool, TaskError>  # returns Ok(true)"
      - "async fn renew_leader(&self, _ttl: Duration) -> Result<bool, TaskError>  # returns Ok(true)"
      - "async fn release_leader(&self) -> Result<(), TaskError>  # no-op"
      - "async fn get_task_state(&self, name: &str) -> Result<TaskScheduleState, TaskError>"
      - "async fn set_task_state(&self, name: &str, state: &TaskScheduleState) -> Result<(), TaskError>"
      - "async fn pause_task(&self, name: &str) -> Result<(), TaskError>  # overrides default: calls GCP + local"
      - "async fn resume_task(&self, name: &str) -> Result<(), TaskError>  # overrides default: calls GCP + local"
      - "async fn create_job(&self, job: &CloudSchedulerJob) -> Result<CloudSchedulerJob, TaskError>"
      - "async fn update_job(&self, job: &CloudSchedulerJob) -> Result<CloudSchedulerJob, TaskError>"
      - "async fn delete_job(&self, name: &str) -> Result<(), TaskError>"
      - "async fn get_job(&self, name: &str) -> Result<CloudSchedulerJob, TaskError>"
      - "async fn list_jobs(&self) -> Result<Vec<CloudSchedulerJob>, TaskError>"
      - "async fn get_oidc_token(&self) -> Result<String, TaskError>  # cached token fetch"
      - "fn map_gcp_error(status: StatusCode, body: &str) -> TaskError  # error mapping"

  - file: crates/cclab-queue/src/scheduler/mod.rs
    action: modify
    description: |
      Add conditional module declaration and re-export for cloud_scheduler_backend.
    additions:
      - '#[cfg(feature = "cloud-scheduler")] pub mod cloud_scheduler_backend;'
      - '#[cfg(feature = "cloud-scheduler")] pub use cloud_scheduler_backend::{CloudSchedulerBackend, CloudSchedulerConfig};'

  - file: crates/cclab-queue/Cargo.toml
    action: modify
    description: |
      Add cloud-scheduler feature flag and conditional dependencies.
    additions:
      - 'cloud-scheduler feature: ["dep:reqwest", "dep:base64"]'
      - 'reqwest dependency (optional): { version = "0.12", features = ["json"], optional = true }'
      - 'base64 dependency (optional): { version = "0.22", optional = true }'
    notes: |
      reqwest may already be a dependency for cloudtasks feature — share the dependency,
      add cloud-scheduler to its feature activation list.
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
  "title": "Cloud Scheduler Backend — Data Models",
  "$defs": {
    "CloudSchedulerConfig": {
      "$id": "meteor://scheduler/cloud-scheduler-config",
      "type": "object",
      "description": "Configuration for CloudSchedulerBackend. Feature-gated under cloud-scheduler.",
      "properties": {
        "project_id": {
          "type": "string",
          "description": "GCP project ID"
        },
        "location": {
          "type": "string",
          "description": "GCP region (e.g., us-central1)"
        },
        "oidc_service_account_email": {
          "type": "string",
          "format": "email",
          "description": "Service account email for OIDC token in httpTarget"
        },
        "target_base_url": {
          "type": "string",
          "format": "uri",
          "description": "Base URL of the application task endpoint (e.g., https://app.example.com/tasks)"
        },
        "time_zone": {
          "type": "string",
          "default": "UTC",
          "description": "IANA time zone for schedule evaluation"
        },
        "credentials_path": {
          "oneOf": [
            { "type": "string", "description": "Path to service account JSON key file (local dev)" },
            { "type": "null" }
          ],
          "default": null,
          "description": "If null, uses GCP metadata server for authentication"
        }
      },
      "required": ["project_id", "location", "oidc_service_account_email", "target_base_url"]
    },
    "CloudSchedulerBackend": {
      "$id": "meteor://scheduler/cloud-scheduler-backend",
      "type": "object",
      "description": "SchedulerBackend implementation backed by GCP Cloud Scheduler. Requires Send + Sync.",
      "properties": {
        "config": {
          "$ref": "meteor://scheduler/cloud-scheduler-config"
        },
        "client": {
          "type": "string",
          "const": "reqwest::Client",
          "description": "HTTP client with connection pooling"
        },
        "token_cache": {
          "$ref": "meteor://scheduler/oidc-token-cache",
          "description": "Cached OIDC access token with expiry"
        },
        "task_states": {
          "type": "string",
          "const": "Arc<RwLock<HashMap<String, TaskScheduleState>>>",
          "description": "In-memory task state store"
        }
      },
      "required": ["config", "client", "token_cache", "task_states"]
    },
    "OidcTokenCache": {
      "$id": "meteor://scheduler/oidc-token-cache",
      "type": "object",
      "description": "Cached OIDC bearer token with expiry tracking. Refreshed 5 minutes before expiry.",
      "properties": {
        "access_token": {
          "oneOf": [
            { "type": "string" },
            { "type": "null" }
          ],
          "description": "Current bearer token (null if not yet fetched)"
        },
        "expires_at": {
          "oneOf": [
            { "type": "string", "format": "date-time" },
            { "type": "null" }
          ],
          "description": "Token expiry timestamp (null if not yet fetched)"
        }
      },
      "required": ["access_token", "expires_at"]
    },
    "CloudSchedulerJob": {
      "$id": "meteor://scheduler/cloud-scheduler-job",
      "type": "object",
      "description": "GCP Cloud Scheduler Job representation (subset of fields used by this backend).",
      "properties": {
        "name": {
          "type": "string",
          "pattern": "^projects/[^/]+/locations/[^/]+/jobs/[^/]+$",
          "description": "Fully qualified job name"
        },
        "schedule": {
          "type": "string",
          "description": "Unix-cron format schedule (5 fields)"
        },
        "timeZone": {
          "type": "string",
          "default": "UTC"
        },
        "httpTarget": {
          "$ref": "meteor://scheduler/http-target"
        },
        "state": {
          "type": "string",
          "enum": ["ENABLED", "PAUSED", "DISABLED", "UPDATE_FAILED"],
          "description": "GCP-managed job state"
        },
        "userUpdateTime": {
          "type": "string",
          "format": "date-time",
          "description": "Last user-initiated update time"
        },
        "lastAttemptTime": {
          "type": "string",
          "format": "date-time",
          "description": "Last execution attempt time"
        },
        "status": {
          "type": "object",
          "description": "Execution status from last attempt"
        }
      },
      "required": ["name", "schedule", "httpTarget"]
    },
    "HttpTarget": {
      "$id": "meteor://scheduler/http-target",
      "type": "object",
      "description": "HTTP target configuration for Cloud Scheduler job.",
      "properties": {
        "uri": {
          "type": "string",
          "format": "uri",
          "description": "Full URL of the task endpoint"
        },
        "httpMethod": {
          "type": "string",
          "enum": ["POST", "GET", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"],
          "default": "POST"
        },
        "body": {
          "type": "string",
          "contentEncoding": "base64",
          "description": "Base64-encoded TaskMessage JSON"
        },
        "headers": {
          "type": "object",
          "additionalProperties": { "type": "string" },
          "default": { "Content-Type": "application/json" }
        },
        "oidcToken": {
          "type": "object",
          "properties": {
            "serviceAccountEmail": { "type": "string", "format": "email" },
            "audience": { "type": "string", "format": "uri" }
          },
          "required": ["serviceAccountEmail"]
        }
      },
      "required": ["uri", "httpMethod", "oidcToken"]
    },
    "GcpErrorMapping": {
      "$id": "meteor://scheduler/gcp-error-mapping",
      "type": "object",
      "description": "HTTP status code to TaskError variant mapping.",
      "properties": {
        "404": { "const": "TaskError::NotFound" },
        "401": { "const": "TaskError::AuthenticationError" },
        "403": { "const": "TaskError::AuthenticationError" },
        "429": { "const": "TaskError::RateLimited" },
        "5xx": { "const": "TaskError::BackendError" },
        "transport": { "const": "TaskError::ConnectionError" }
      }
    },
    "FeatureGate": {
      "$id": "meteor://scheduler/feature-gate",
      "type": "object",
      "description": "Cargo feature gate for cloud-scheduler module.",
      "properties": {
        "cloud-scheduler": { "const": "CloudSchedulerBackend, CloudSchedulerConfig" }
      }
    }
  }
}
```


## REST API

```yaml
openapi: 3.0.3
info:
  title: GCP Cloud Scheduler REST API v1 (subset used by CloudSchedulerBackend)
  version: v1
servers:
  - url: https://cloudscheduler.googleapis.com/v1
paths:
  /projects/{project}/locations/{location}/jobs:
    post:
      operationId: createJob
      summary: Create a Cloud Scheduler job
      parameters:
        - name: project
          in: path
          required: true
          schema: { type: string }
        - name: location
          in: path
          required: true
          schema: { type: string }
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/Job'
      responses:
        '200': { description: Job created, content: { application/json: { schema: { $ref: '#/components/schemas/Job' } } } }
        '409': { description: Job already exists }
    get:
      operationId: listJobs
      summary: List all jobs in a location
      parameters:
        - name: project
          in: path
          required: true
          schema: { type: string }
        - name: location
          in: path
          required: true
          schema: { type: string }
        - name: pageSize
          in: query
          schema: { type: integer, default: 500 }
        - name: pageToken
          in: query
          schema: { type: string }
      responses:
        '200':
          description: List of jobs
          content:
            application/json:
              schema:
                type: object
                properties:
                  jobs: { type: array, items: { $ref: '#/components/schemas/Job' } }
                  nextPageToken: { type: string }
  /projects/{project}/locations/{location}/jobs/{jobId}:
    get:
      operationId: getJob
      summary: Get a single job
      responses:
        '200': { description: Job details, content: { application/json: { schema: { $ref: '#/components/schemas/Job' } } } }
        '404': { description: Job not found }
    patch:
      operationId: updateJob
      summary: Update an existing job
      parameters:
        - name: updateMask
          in: query
          schema: { type: string }
          description: Comma-separated list of fields to update
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/Job'
      responses:
        '200': { description: Updated job, content: { application/json: { schema: { $ref: '#/components/schemas/Job' } } } }
    delete:
      operationId: deleteJob
      summary: Delete a job
      responses:
        '200': { description: Job deleted }
        '404': { description: Job not found }
  /projects/{project}/locations/{location}/jobs/{jobId}:pause:
    post:
      operationId: pauseJob
      summary: Pause a job (stops future executions)
      responses:
        '200': { description: Job paused, content: { application/json: { schema: { $ref: '#/components/schemas/Job' } } } }
  /projects/{project}/locations/{location}/jobs/{jobId}:resume:
    post:
      operationId: resumeJob
      summary: Resume a paused job
      responses:
        '200': { description: Job resumed, content: { application/json: { schema: { $ref: '#/components/schemas/Job' } } } }
components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
      description: OIDC bearer token from GCP metadata server or service account
  schemas:
    Job:
      type: object
      properties:
        name: { type: string, description: 'Fully qualified: projects/{project}/locations/{location}/jobs/{jobId}' }
        schedule: { type: string, description: 'Unix-cron format (5 fields)' }
        timeZone: { type: string, default: UTC }
        httpTarget:
          type: object
          properties:
            uri: { type: string, format: uri }
            httpMethod: { type: string, enum: [POST, GET, PUT, DELETE, PATCH, HEAD, OPTIONS], default: POST }
            body: { type: string, description: Base64-encoded request body }
            headers: { type: object, additionalProperties: { type: string } }
            oidcToken:
              type: object
              properties:
                serviceAccountEmail: { type: string }
                audience: { type: string }
              required: [serviceAccountEmail]
          required: [uri, httpMethod, oidcToken]
        state: { type: string, enum: [ENABLED, PAUSED, DISABLED, UPDATE_FAILED] }
        userUpdateTime: { type: string, format: date-time }
        lastAttemptTime: { type: string, format: date-time }
        status: { type: object }
      required: [name, schedule, httpTarget]
security:
  - bearerAuth: []
```