---
id: scheduler-cloud-backend
main_spec_ref: "crates/cclab-queue/logic/scheduler-cloud-backend.md"
merge_strategy: new
---

# Scheduler Cloud Backend

## Overview

<!-- type: overview lang: markdown -->

`CloudSchedulerBackend` (`crates/cclab-queue/src/scheduler/cloud_scheduler_backend.rs`) implements the `SchedulerBackend` trait using GCP Cloud Scheduler as the external scheduling authority. It operates in `ExternalPush` mode — leader election is a no-op since GCP manages scheduling.

| Component | Type | Purpose |
|-----------|------|--------|
| `CloudSchedulerConfig` | struct | GCP project, location, OIDC SA email, target URL, timezone, optional credentials path |
| `CloudSchedulerBackend` | struct | `reqwest::Client` + `OidcTokenCache` + in-memory `HashMap<String, TaskScheduleState>` |
| `OidcTokenCache` | struct (internal) | Cached bearer token with 5-minute refresh buffer |
| `CloudSchedulerJob` | struct (pub) | GCP Cloud Scheduler Job representation (camelCase serde) |
| `HttpTarget` | struct (pub) | HTTP target config with optional body, headers, OIDC token |
| `OidcTokenTarget` | struct (pub) | OIDC token target for httpTarget (service account + audience) |
| `ListJobsResponse` | struct (internal) | Paginated list response with `jobs` vec and `next_page_token` |
| `MetadataTokenResponse` | struct (internal) | GCP metadata server token endpoint response |

The backend exposes 6 REST API methods (`create_job`, `update_job`, `delete_job`, `get_job`, `list_jobs`, `pause_job_api`, `resume_job_api`) that require GCP network access, plus trait methods for local task state management. `register_external_schedule` converts `PeriodicSchedule::Cron` (6-field → 5-field) and `PeriodicSchedule::Interval` (seconds → cron or `every Xs` notation) to Cloud Scheduler job format.

Existing tests (42 tests in `cloud_scheduler_backend_tests.rs`) cover: leader election no-ops (S1), config helpers and job serialization (S2), task state CRUD (S4), pause/resume local state (S3/S7), OIDC token cache validity (S5), GCP error mapping (S6), delete local state cleanup (S8), construction, Send+Sync bounds, HttpTarget serialization, and MetadataTokenResponse deserialization. This spec documents the complete test surface and async API contract.
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

All tests in `crates/cclab-queue/src/scheduler/cloud_scheduler_backend_tests.rs`, included via `#[cfg(test)] #[path = "cloud_scheduler_backend_tests.rs"] mod tests`. All tests are unit tests (no GCP network required) — REST API methods are tested indirectly through local state management, serialization, and error mapping.

### S1: Leader Election No-Op (R1)

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| S1a | `test_acquire_leader_returns_true` | `acquire_leader` | returns `Ok(true)` regardless of TTL |
| S1b | `test_renew_leader_returns_true` | `renew_leader` | returns `Ok(true)` |
| S1c | `test_release_leader_returns_ok` | `release_leader` | returns `Ok(())` |
| S1d | `test_leader_election_full_cycle` | acquire→renew→release→acquire | all succeed; re-acquire after release still returns true |

### S2: Config Helpers & Job Serialization (R3, R5)

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| S2a | `test_config_jobs_parent` | `CloudSchedulerConfig::jobs_parent()` | `"projects/my-project/locations/us-central1"` |
| S2b | `test_config_job_name` | `CloudSchedulerConfig::job_name()` | `"{parent}/jobs/daily-cleanup"` |
| S2c | `test_config_default` | `CloudSchedulerConfig::default()` | location=`us-central1`, time_zone=`UTC`, credentials_path=None, project_id empty |
| S2d | `test_job_serialization_camel_case` | `CloudSchedulerJob` serde | JSON keys are camelCase (`httpTarget`, `timeZone`); None fields omitted |
| S2e | `test_job_deserialization_from_gcp_response` | full GCP JSON → struct | all fields including optional `userUpdateTime`, `lastAttemptTime`, `status` |
| S2f | `test_job_deserialization_minimal` | minimal JSON → struct | `timeZone` defaults to `UTC`; optional fields are None/empty |
| S2g | `test_job_roundtrip_serialization` | serialize → deserialize | `name`, `schedule`, `http_target.uri` preserved |
| S2h | `test_list_jobs_response_deserialization` | `ListJobsResponse` JSON | 2 jobs parsed, `nextPageToken` = `"abc123"` |
| S2i | `test_list_jobs_response_empty` | empty JSON `{}` | `jobs` defaults to empty vec, `nextPageToken` is None |

### S3/S7: Pause & Resume Local State (R7)

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| S3a | `test_pause_resume_via_local_state` | local pause/resume simulation | set enabled=false → `is_task_enabled`==false; set enabled=true → `is_task_enabled`==true |
| S3b | `test_is_task_enabled_default_true` | `is_task_enabled` for unknown task | returns true (default `TaskScheduleState`) |

### S4: Task State Management (R2)

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| S4a | `test_default_task_state_for_unknown_task` | `get_task_state` miss | enabled=true, last_run_at=None, total_run_count=0 |
| S4b | `test_set_and_get_task_state` | `set_task_state` + `get_task_state` | roundtrip preserves enabled, last_run_at, total_run_count |
| S4c | `test_set_task_state_overwrites` | upsert semantics | second set overwrites first; count changes from 1→99 |
| S4d | `test_record_task_run_increments_count` | `record_task_run` | first run → count=1; second run → count=2 |
| S4e | `test_record_task_run_updates_last_run_at` | `record_task_run` timestamp | last_run_at updated to ~now (between before/after timestamps) |
| S4f | `test_record_task_run_preserves_enabled` | `record_task_run` enabled | default task stays enabled=true after record |
| S4g | `test_multiple_tasks_isolated` | task state isolation | task-a(2 runs), task-b(1 run), task-c(0 runs) — counts independent |

### S5: OIDC Token Cache Validity (R4)

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| S5a | `test_oidc_cache_new_is_invalid` | `OidcTokenCache::new()` | `is_valid()==false`, both fields None |
| S5b | `test_oidc_cache_valid_token` | token with 1h expiry | `is_valid()==true` |
| S5c | `test_oidc_cache_expired_token` | token expired 60s ago | `is_valid()==false` |
| S5d | `test_oidc_cache_within_refresh_buffer` | token expires in 4min (< 5min buffer) | `is_valid()==false` |
| S5e | `test_oidc_cache_outside_refresh_buffer` | token expires in 6min (> 5min buffer) | `is_valid()==true` |
| S5f | `test_oidc_cache_token_none_with_expiry` | access_token=None, expiry set | `is_valid()==false` |
| S5g | `test_oidc_cache_token_present_no_expiry` | access_token=Some, expires_at=None | `is_valid()==false` |

### S6: GCP API Error Mapping (R8)

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| S6a | `test_map_gcp_error_404_not_found` | 404 → `TaskNotFound` | message = body text |
| S6b | `test_map_gcp_error_401_authentication` | 401 → `Authentication` | contains status code and body |
| S6c | `test_map_gcp_error_403_authentication` | 403 → `Authentication` | contains `"403"` |
| S6d | `test_map_gcp_error_429_rate_limited` | 429 → `RateLimited(60s)` | duration = 60 seconds |
| S6e | `test_map_gcp_error_500_backend` | 500 → `Backend` | contains `"500"` |
| S6f | `test_map_gcp_error_503_backend` | 503 → `Backend` | contains `"503"` |
| S6g | `test_map_gcp_error_502_backend` | 502 → `Backend` | contains `"502"` |
| S6h | `test_map_gcp_error_unknown_status` | 400 → `Backend` (catch-all) | contains `"400"` and body |

### S8: Delete Job Local State Cleanup (R3)

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| S8a | `test_delete_removes_local_state_directly` | local state removal after delete | record_task_run → remove from HashMap → get_task_state returns default |

### Construction & Trait Bounds

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| C1 | `test_backend_new_success` | `CloudSchedulerBackend::new()` | returns `Ok` with valid config |
| C2 | `test_backend_is_send_sync` | `Send + Sync` bounds | `assert_send_sync::<CloudSchedulerBackend>()` compiles |

### HttpTarget Serialization (R5)

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| H1 | `test_http_target_serialization` | full HttpTarget → JSON | camelCase keys; body, headers, oidcToken present |
| H2 | `test_http_target_empty_headers_omitted` | skip_serializing_if | empty headers/None body/None oidcToken omitted from JSON |

### MetadataTokenResponse (R4)

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| M1 | `test_metadata_token_response_deserialization` | JSON → `MetadataTokenResponse` | access_token, expires_in, token_type parsed correctly |

### Requirements Traceability

| Req | Description | Tests |
|-----|-------------|-------|
| R1 | Leader election no-op | S1a-S1d |
| R2 | In-memory task state (HashMap) | S4a-S4g |
| R3 | Config helpers (jobs_parent, job_name) + delete cleanup | S2a-S2c, S8a |
| R4 | OIDC token cache with 5-min refresh buffer | S5a-S5g, M1 |
| R5 | camelCase serde for GCP API types | S2d-S2i, H1-H2 |
| R7 | Pause/resume syncs local state | S3a-S3b |
| R8 | HTTP status → TaskError mapping | S6a-S6h |

### Totals

| Category | Count |
|----------|-------|
| Leader election | 4 |
| Config & serialization | 9 |
| Pause/resume | 2 |
| Task state CRUD | 7 |
| OIDC cache | 7 |
| Error mapping | 8 |
| Delete cleanup | 1 |
| Construction | 2 |
| HttpTarget | 2 |
| MetadataTokenResponse | 1 |
| **Total** | **43** |
## Changes

<!-- type: changes lang: yaml -->

```yaml
_sdd:
  id: scheduler-cloud-backend-changes
  refs:
    - $ref: "#cloud-scheduler-backend-async-api"
    - $ref: "error-types#task-error-schema"
changes:
  - path: crates/cclab-queue/src/scheduler/cloud_scheduler_backend_tests.rs
    action: existing
    description: >-
      Tests already exist (43 tests). No new tests needed — existing coverage is comprehensive.
      Covers: leader election no-ops (4), config helpers + job serde (9), pause/resume local state (2),
      task state CRUD + record_task_run (7), OIDC token cache validity (7), GCP error mapping (8),
      delete local state cleanup (1), construction + Send+Sync (2), HttpTarget serde (2),
      MetadataTokenResponse deserialization (1).
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
  title: CloudSchedulerBackend Async API
  version: 0.1.0
  description: >
    GCP Cloud Scheduler backend implementing SchedulerBackend trait.
    Leader election is no-op; scheduling authority delegated to GCP.
  x-sdd:
    id: cloud-scheduler-backend-async-api
    refs:
      - $ref: "error-types#task-error-schema"

defaultContentType: application/json

channels:
  scheduler-job-crud:
    description: GCP Cloud Scheduler REST API v1 job operations
    parameters:
      project_id:
        schema:
          type: string
      location:
        schema:
          type: string
          default: us-central1
    publish:
      operationId: createJob
      summary: POST {base}/{parent}/jobs — create Cloud Scheduler job
      message:
        $ref: '#/components/messages/CloudSchedulerJobMessage'
    subscribe:
      operationId: getJob
      summary: GET {base}/{jobName} — retrieve single job
      message:
        $ref: '#/components/messages/CloudSchedulerJobMessage'

  scheduler-job-update:
    description: Job update via PATCH
    publish:
      operationId: updateJob
      summary: PATCH {base}/{job.name} — update existing job
      message:
        $ref: '#/components/messages/CloudSchedulerJobMessage'

  scheduler-job-delete:
    description: Job deletion + local state cleanup
    publish:
      operationId: deleteJob
      summary: DELETE {base}/{jobName} — delete job and remove local task state
      message:
        name: DeleteJobRequest
        payload:
          type: object
          required: [name]
          properties:
            name:
              type: string
              description: Fully qualified job name or short job ID

  scheduler-job-list:
    description: List all jobs under parent
    subscribe:
      operationId: listJobs
      summary: GET {base}/{parent}/jobs — list all jobs
      message:
        $ref: '#/components/messages/ListJobsMessage'

  scheduler-job-pause-resume:
    description: Pause/resume job via Cloud Scheduler API + local state sync
    publish:
      operationId: pauseTask
      summary: POST {base}/{jobName}:pause — pause GCP job and set local enabled=false
      message:
        $ref: '#/components/messages/CloudSchedulerJobMessage'
    subscribe:
      operationId: resumeTask
      summary: POST {base}/{jobName}:resume — resume GCP job and set local enabled=true
      message:
        $ref: '#/components/messages/CloudSchedulerJobMessage'

  leader-election:
    description: No-op leader election for cloud-managed backend
    publish:
      operationId: acquireLeader
      summary: Always returns true — GCP is the scheduling authority
      message:
        name: LeaderResult
        payload:
          type: boolean
          const: true
    subscribe:
      operationId: renewLeader
      summary: Always returns true — no lease to renew
      message:
        name: LeaderResult
        payload:
          type: boolean
          const: true

  task-state:
    description: In-memory task state store — HashMap<String, TaskScheduleState>
    publish:
      operationId: setTaskState
      summary: Insert/overwrite task state in local HashMap
      message:
        $ref: '#/components/messages/TaskScheduleStateMessage'
    subscribe:
      operationId: getTaskState
      summary: Retrieve task state; returns Default if absent
      message:
        $ref: '#/components/messages/TaskScheduleStateMessage'

  register-schedule:
    description: Convert PeriodicTask to Cloud Scheduler job and create it
    publish:
      operationId: registerExternalSchedule
      summary: >
        Convert PeriodicSchedule to Cloud Scheduler format:
        Cron(6-field) → 5-field unix-cron (strip seconds);
        Interval(secs) → */N * * * * or every Ns notation.
        Creates job with OIDC-authenticated HTTP POST target.
      message:
        $ref: '#/components/messages/PeriodicTaskMessage'

  oidc-token:
    description: OIDC bearer token caching from GCP metadata server
    subscribe:
      operationId: getOidcToken
      summary: Return cached token if valid (>5min to expiry), else fetch from metadata server
      bindings:
        metadata:
          url: http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token
          header: Metadata-Flavor=Google
          refresh_buffer_secs: 300
      message:
        name: OidcToken
        payload:
          type: string

components:
  messages:
    CloudSchedulerJobMessage:
      name: CloudSchedulerJob
      contentType: application/json
      payload:
        $ref: '#/components/schemas/CloudSchedulerJob'

    ListJobsMessage:
      name: ListJobsResponse
      contentType: application/json
      payload:
        type: object
        properties:
          jobs:
            type: array
            items:
              $ref: '#/components/schemas/CloudSchedulerJob'
          nextPageToken:
            type: string
            nullable: true

    TaskScheduleStateMessage:
      name: TaskScheduleState
      contentType: application/json
      payload:
        $ref: '#/components/schemas/TaskScheduleState'

    PeriodicTaskMessage:
      name: PeriodicTask
      contentType: application/json
      payload:
        $ref: '#/components/schemas/PeriodicTask'

  schemas:
    CloudSchedulerConfig:
      type: object
      required: [project_id, location, oidc_service_account_email, target_base_url, time_zone]
      properties:
        project_id:
          type: string
        location:
          type: string
          default: us-central1
        oidc_service_account_email:
          type: string
        target_base_url:
          type: string
        time_zone:
          type: string
          default: UTC
        credentials_path:
          type: string
          nullable: true
      x-sdd:
        id: cloud-scheduler-config
        source: crates/cclab-queue/src/scheduler/cloud_scheduler_backend.rs
        helpers:
          jobs_parent: "projects/{project_id}/locations/{location}"
          job_name: "{jobs_parent}/jobs/{job_id}"

    CloudSchedulerJob:
      type: object
      required: [name, schedule, httpTarget]
      properties:
        name:
          type: string
          description: "Fully qualified: projects/{project}/locations/{location}/jobs/{jobId}"
        schedule:
          type: string
          description: Unix-cron 5-field format
        timeZone:
          type: string
          default: UTC
        httpTarget:
          $ref: '#/components/schemas/HttpTarget'
        state:
          type: string
          enum: [ENABLED, PAUSED, DISABLED, UPDATE_FAILED]
          nullable: true
        userUpdateTime:
          type: string
          format: date-time
          nullable: true
        lastAttemptTime:
          type: string
          format: date-time
          nullable: true
        status:
          type: object
          nullable: true
      x-sdd:
        id: cloud-scheduler-job
        serde: camelCase

    HttpTarget:
      type: object
      required: [uri, httpMethod]
      properties:
        uri:
          type: string
          format: uri
        httpMethod:
          type: string
          enum: [POST, GET, PUT, DELETE, PATCH]
        body:
          type: string
          description: Base64-encoded request body
          nullable: true
        headers:
          type: object
          additionalProperties:
            type: string
          description: Omitted when empty (skip_serializing_if)
        oidcToken:
          $ref: '#/components/schemas/OidcTokenTarget'
          nullable: true
      x-sdd:
        id: http-target
        serde: camelCase

    OidcTokenTarget:
      type: object
      required: [serviceAccountEmail]
      properties:
        serviceAccountEmail:
          type: string
        audience:
          type: string
          nullable: true
      x-sdd:
        serde: camelCase

    OidcTokenCache:
      type: object
      description: Internal cached token with expiry tracking
      properties:
        access_token:
          type: string
          nullable: true
        expires_at:
          type: string
          format: date-time
          nullable: true
      x-sdd:
        validity_rule: "valid iff access_token.is_some() && expires_at > now + 300s"

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

    PeriodicTask:
      type: object
      required: [name, task_name, schedule, args, queue, enabled]
      properties:
        name:
          type: string
        task_name:
          type: string
        schedule:
          $ref: '#/components/schemas/PeriodicSchedule'
        args:
          type: object
        queue:
          type: string
        enabled:
          type: boolean

    PeriodicSchedule:
      oneOf:
        - type: object
          required: [Cron]
          properties:
            Cron:
              type: string
              description: "6-field cron (sec min hour dom month dow) — converted to 5-field for Cloud Scheduler"
        - type: object
          required: [Interval]
          properties:
            Interval:
              type: integer
              format: uint64
              description: "Seconds — converted to */N cron (if >=60 and divisible) or 'every Ns'"

    GcpErrorMapping:
      type: object
      description: HTTP status → TaskError variant mapping
      x-sdd:
        id: gcp-error-mapping
      properties:
        '404':
          const: TaskNotFound
        '401':
          const: Authentication
        '403':
          const: Authentication
        '429':
          const: RateLimited
          x-default-retry: 60s
        '500-599':
          const: Backend
        default:
          const: Backend
```