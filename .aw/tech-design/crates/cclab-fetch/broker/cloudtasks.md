---
id: cloudtasks-broker
main_spec_ref: "crates/cclab-fetch/broker/cloudtasks.md"
merge_strategy: new
---

# Cloudtasks Broker

## Overview

Implements `Broker + PushBroker + DelayedBroker` traits backed by GCP Cloud Tasks REST API v2. Replaces the current stub `publish()` with actual HTTP API calls to create tasks via `POST https://cloudtasks.googleapis.com/v2/{queue_path}/tasks`. Push-based delivery model: Cloud Tasks dispatches HTTP POST requests to the worker endpoint at `/meteor/push/{queue}`. Supports native delayed task scheduling via the `scheduleTime` field on task creation. Parses incoming Cloud Tasks push requests using headers `x-cloudtasks-taskname` (delivery tag) and `x-cloudtasks-taskretrycount` (redelivery detection). Outbound API calls authenticated via OIDC bearer tokens (GCP metadata server in production, service account JSON key for local dev). Inbound push requests validated by checking the `Authorization` header OIDC JWT against configured service account. `BrokerCapabilities`: `delayed_tasks: true`, `dead_letter: true`, `priority: false`, `batching: false`, `max_delay: 30 days`. Feature-gated under `cloudtasks` Cargo feature.

Source: `crates/cclab-queue/src/broker/cloudtasks.rs` (existing stub), `crates/cclab-queue/src/broker/mod.rs` (trait definitions)
## Requirements

| ID | Title | Priority | Acceptance Criteria |
|----|-------|----------|---------------------|
| R1 | Broker trait — publish via Cloud Tasks API | P0 | `CloudTasksBroker` implements `Broker` trait. `publish(queue, message)` sends `POST https://cloudtasks.googleapis.com/v2/projects/{project}/locations/{location}/queues/{queue}/tasks` with `HttpRequest` body containing `url` (worker endpoint), `httpMethod: POST`, `body` (base64-encoded TaskMessage JSON), `oidcToken`. Replaces current stub that only logs. `connect()` validates config and initializes reqwest client. `disconnect()` is no-op. `health_check()` calls `GET /v2/projects/{project}/locations` to verify API reachability |
| R2 | PushBroker trait — parse incoming requests | P0 | `parse_push_request(headers, body)` deserializes body to `TaskMessage`, extracts `delivery_tag` from `x-cloudtasks-taskname` header, detects redelivery when `x-cloudtasks-taskretrycount > 0`. `ack_status_code()` returns 200. `nack_status_code()` returns 500 (triggers Cloud Tasks retry). `endpoint_path()` returns `/meteor/push/{queue}` |
| R3 | DelayedBroker trait — native delayed publishing | P0 | `publish_delayed(queue, message, delay)` creates a Cloud Tasks task with `scheduleTime = now + delay` (RFC 3339). `publish_at(queue, message, eta)` uses default trait implementation: if `eta <= now`, calls `publish()` immediately; otherwise converts to delay and calls `publish_delayed()`. Max delay: 30 days (Cloud Tasks limit) |
| R4 | OIDC authentication for outbound API calls | P0 | All Cloud Tasks REST API calls include `Authorization: Bearer <token>` header. Token obtained via: (1) GCP metadata server `http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token` in production, (2) service account JSON key file for local dev. Token cached in `OidcTokenCache` and refreshed 5 minutes before expiry |
| R5 | OIDC validation for inbound push requests | P1 | When `service_account_email` is configured, `parse_push_request` checks `Authorization: Bearer <jwt>` header. Validates JWT signature against Google public keys, checks audience matches `oidc_audience` or `worker_url`, checks email matches `service_account_email`. Returns `TaskError::Authentication` on failure |
| R6 | Cloud Tasks task creation HTTP payload | P0 | Task creation request body: `{ "task": { "httpRequest": { "url": "<worker_url>/meteor/push/<queue>", "httpMethod": "POST", "body": "<base64(TaskMessage JSON)>", "headers": { "Content-Type": "application/json" }, "oidcToken": { "serviceAccountEmail": "<email>", "audience": "<audience>" } }, "scheduleTime": "<RFC3339>" (optional, for delayed tasks) } }` |
| R7 | Error mapping | P1 | Cloud Tasks API HTTP errors map to `TaskError`: 404 → `TaskError::NotFound`, 401/403 → `TaskError::AuthenticationError`, 429 → `TaskError::RateLimited`, 409 → `TaskError::AlreadyExists`, 5xx → `TaskError::BackendError`. reqwest transport errors → `TaskError::ConnectionError` |
| R8 | Feature gate | P1 | Module conditionally compiled under `#[cfg(feature = "cloudtasks")]`. `CloudTasksBroker` and `CloudTasksConfig` only available when feature is enabled. Dependencies: `reqwest` (optional, with `json` feature), `base64` (optional). No GCP deps pulled when feature disabled |
| R9 | Retry and dispatch deadline configuration | P1 | `CloudTasksConfig` exposes `dispatch_deadline` (default 600s/10min), `max_retry_count` (optional). These map to Cloud Tasks API `dispatchDeadline` and retry config fields on task creation |

### Constraints

- All trait methods returning `Result` use `TaskError` as the error type
- `CloudTasksBroker` requires `Send + Sync + 'static` (per `Broker` trait bound)
- HTTP client is `reqwest::Client` with connection pooling (shared across calls)
- Cloud Tasks REST API v2 (not v1, not beta)
- Queue path format: `projects/{project}/locations/{location}/queues/{queue}`
- Task names auto-generated by Cloud Tasks API (not caller-specified)
- `PushBroker` methods are sync (not async) — avoids lifetime issues in HTTP handlers
- `scheduleTime` must be within 30 days of current time (Cloud Tasks limit)
## Scenarios

### S1: Publish a task via Cloud Tasks REST API (R1, R6)

**GIVEN** a connected `CloudTasksBroker` with `project_id=my-project`, `location=us-central1`, `worker_url=https://app.example.com`
**WHEN** `publish("default", task_message)` is called
**THEN** sends `POST https://cloudtasks.googleapis.com/v2/projects/my-project/locations/us-central1/queues/default/tasks` with JSON body `{ "task": { "httpRequest": { "url": "https://app.example.com/meteor/push/default", "httpMethod": "POST", "body": "<base64(task_message JSON)>", "headers": { "Content-Type": "application/json" }, "oidcToken": { "serviceAccountEmail": "...", "audience": "..." } } } }`; returns `Ok(())` on HTTP 200

### S2: Parse incoming Cloud Tasks push request (R2)

**GIVEN** a `CloudTasksBroker` receiving an HTTP POST
**WHEN** `parse_push_request(headers, body)` is called with `headers = { "x-cloudtasks-taskname": "task-123", "x-cloudtasks-taskretrycount": "0" }` and valid TaskMessage JSON body
**THEN** returns `BrokerMessage { delivery_tag: "task-123", payload: <deserialized TaskMessage>, headers: <original>, timestamp: <now>, redelivered: false }`

### S3: Detect redelivered push request (R2)

**GIVEN** a Cloud Tasks push request with `x-cloudtasks-taskretrycount: 3`
**WHEN** `parse_push_request(headers, body)` is called
**THEN** returns `BrokerMessage` with `redelivered: true`

### S4: Publish delayed task with scheduleTime (R3, R6)

**GIVEN** a connected `CloudTasksBroker`
**WHEN** `publish_delayed("default", task_message, Duration::from_secs(300))` is called
**THEN** sends task creation request with `scheduleTime` set to `Utc::now() + 300s` in RFC 3339 format; Cloud Tasks holds the task until `scheduleTime` before dispatching

### S5: Publish at past ETA falls back to immediate publish (R3)

**GIVEN** a `CloudTasksBroker`
**WHEN** `publish_at("default", task_message, eta)` is called with `eta <= Utc::now()`
**THEN** default trait implementation calls `self.publish("default", task_message)` immediately without `scheduleTime`

### S6: OIDC token acquisition and caching for outbound calls (R4)

**GIVEN** a `CloudTasksBroker` running in GCP environment with no cached token
**WHEN** `publish()` is called (first API call)
**THEN** fetches token from `http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token` with header `Metadata-Flavor: Google`; caches token in `OidcTokenCache`; uses cached token for subsequent calls until 5 minutes before expiry

### S7: OIDC inbound validation rejects invalid token (R5)

**GIVEN** a `CloudTasksBroker` with `service_account_email` configured
**WHEN** `parse_push_request()` is called with missing or invalid `Authorization` header
**THEN** returns `Err(TaskError::Authentication("Missing Authorization header"))` or `Err(TaskError::Authentication("Invalid Authorization header format"))`

### S8: Health check verifies Cloud Tasks API reachability (R1)

**GIVEN** a connected `CloudTasksBroker`
**WHEN** `health_check()` is called
**THEN** sends `GET https://cloudtasks.googleapis.com/v2/projects/{project}/locations` with OIDC bearer token; returns `Ok(())` on HTTP 200; returns `Err(TaskError::ConnectionError(...))` on transport failure

### S9: Cloud Tasks API error mapping (R7)

**GIVEN** a `CloudTasksBroker` calling `publish()`
**WHEN** Cloud Tasks API returns HTTP 404
**THEN** returns `Err(TaskError::NotFound(...))`
**WHEN** returns HTTP 429
**THEN** returns `Err(TaskError::RateLimited(...))`
**WHEN** returns HTTP 500
**THEN** returns `Err(TaskError::BackendError(...))`

### S10: Connect validates configuration (R1)

**GIVEN** a `CloudTasksBroker` with empty `project_id`
**WHEN** `connect()` is called
**THEN** returns `Err(TaskError::Configuration("project_id is required"))`

### S11: Feature gate excludes cloudtasks module (R8)

**GIVEN** a Cargo.toml without `cloudtasks` feature
**WHEN** the crate is compiled
**THEN** `CloudTasksBroker` and `CloudTasksConfig` are not available; no reqwest or base64 dependencies pulled

### S12: Ack/nack status codes for push delivery (R2)

**GIVEN** a `CloudTasksBroker` implementing `PushBroker`
**WHEN** task processing succeeds and `ack_status_code()` is called
**THEN** returns 200 (Cloud Tasks marks task complete)
**WHEN** task processing fails and `nack_status_code()` is called
**THEN** returns 500 (Cloud Tasks retries the task according to retry config)
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
  - file: crates/cclab-queue/src/broker/cloudtasks.rs
    action: modify
    description: |
      Replace stub publish() with actual Cloud Tasks REST API v2 call.
      Add DelayedBroker trait implementation. Add reqwest::Client field
      and OidcTokenCache for authentication. Enhance connect() to
      initialize HTTP client. Add health_check() API call.
    structs:
      - CloudTasksConfig (extend: add credentials_path, max_retry_count fields)
      - CloudTasksBroker (extend: add client: reqwest::Client, token_cache: Arc<RwLock<OidcTokenCache>>)
      - OidcTokenCache (new: access_token, expires_at)
      - CreateTaskRequest (new: serde serialize for API request body)
      - CloudTask (new: serde deserialize for API response)
      - CloudTasksHttpRequest (new: serde serialize for httpRequest field)
    trait_impls:
      - "Broker for CloudTasksBroker (modify: replace stub publish, add real health_check)"
      - "PushBroker for CloudTasksBroker (modify: implement OIDC validation in parse_push_request)"
      - "DelayedBroker for CloudTasksBroker (new)"
    methods:
      - "async fn publish(&self, queue: &str, message: TaskMessage) -> Result<(), TaskError>  # POST to Cloud Tasks API"
      - "async fn health_check(&self) -> Result<(), TaskError>  # GET /v2/projects/{project}/locations"
      - "async fn connect(&self) -> Result<(), TaskError>  # validate config, init client"
      - "async fn publish_delayed(&self, queue: &str, message: TaskMessage, delay: Duration) -> Result<(), TaskError>  # POST with scheduleTime"
      - "fn parse_push_request(&self, headers, body) -> Result<BrokerMessage, TaskError>  # add OIDC JWT validation"
      - "async fn get_oidc_token(&self) -> Result<String, TaskError>  # cached token fetch from metadata server or SA key"
      - "fn map_gcp_error(status: StatusCode, body: &str) -> TaskError  # HTTP status to TaskError mapping"
      - "fn build_create_task_request(&self, queue: &str, message: &TaskMessage, schedule_time: Option<DateTime<Utc>>) -> CreateTaskRequest"

  - file: crates/cclab-queue/src/broker/mod.rs
    action: modify
    description: |
      No changes needed — cloudtasks module and re-exports already present
      under #[cfg(feature = "cloudtasks")].
    notes: |
      Already has: pub mod cloudtasks; pub use cloudtasks::{CloudTasksBroker, CloudTasksConfig};
      DelayedBroker trait already defined and exported.

  - file: crates/cclab-queue/Cargo.toml
    action: modify
    description: |
      Add reqwest and base64 as optional dependencies activated by cloudtasks feature.
    additions:
      - 'cloudtasks feature: ["dep:reqwest", "dep:base64"]'
      - 'reqwest dependency (optional): { version = "0.12", features = ["json"], optional = true }'
      - 'base64 dependency (optional): { version = "0.22", optional = true }'
    notes: |
      reqwest may already exist as optional dep for other features (pubsub).
      If so, add cloudtasks to its feature activation list rather than adding a new entry.
      base64 needed for encoding TaskMessage body in httpRequest.
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
  "title": "CloudTasks Broker — Data Models",
  "$defs": {
    "CloudTasksConfig": {
      "$id": "meteor://broker/cloudtasks-config",
      "type": "object",
      "description": "Configuration for CloudTasksBroker. Feature-gated under cloudtasks.",
      "properties": {
        "project_id": {
          "type": "string",
          "description": "GCP project ID"
        },
        "location": {
          "type": "string",
          "default": "us-central1",
          "description": "GCP region (e.g., us-central1)"
        },
        "worker_url": {
          "type": "string",
          "format": "uri",
          "description": "Worker endpoint URL for task dispatch (e.g., https://app.example.com)"
        },
        "service_account_email": {
          "oneOf": [
            { "type": "string", "format": "email" },
            { "type": "null" }
          ],
          "default": null,
          "description": "Service account email for OIDC auth. If null, OIDC validation on inbound requests is skipped"
        },
        "oidc_audience": {
          "oneOf": [
            { "type": "string", "format": "uri" },
            { "type": "null" }
          ],
          "default": null,
          "description": "OIDC audience for token validation. Defaults to worker_url if null"
        },
        "default_queue": {
          "type": "string",
          "default": "default",
          "description": "Default queue name when not specified"
        },
        "dispatch_deadline": {
          "type": "integer",
          "default": 600,
          "minimum": 15,
          "maximum": 1800,
          "description": "Task dispatch deadline in seconds (Cloud Tasks limit: 15s-30min)"
        },
        "max_retry_count": {
          "oneOf": [
            { "type": "integer", "minimum": 0 },
            { "type": "null" }
          ],
          "default": null,
          "description": "Max retry attempts. Null = Cloud Tasks default (unlimited)"
        },
        "credentials_path": {
          "oneOf": [
            { "type": "string" },
            { "type": "null" }
          ],
          "default": null,
          "description": "Path to service account JSON key file for local dev. If null, uses GCP metadata server"
        }
      },
      "required": ["project_id", "location", "worker_url", "default_queue"]
    },
    "CloudTasksBroker": {
      "$id": "meteor://broker/cloudtasks-broker",
      "type": "object",
      "description": "Broker + PushBroker + DelayedBroker implementation backed by GCP Cloud Tasks. Requires Send + Sync + 'static.",
      "properties": {
        "config": {
          "$ref": "meteor://broker/cloudtasks-config"
        },
        "client": {
          "type": "string",
          "const": "reqwest::Client",
          "description": "HTTP client with connection pooling for Cloud Tasks API calls"
        },
        "token_cache": {
          "$ref": "meteor://broker/oidc-token-cache",
          "description": "Cached OIDC access token for outbound API authentication"
        },
        "connected": {
          "type": "boolean",
          "default": false,
          "description": "Connection state flag"
        }
      },
      "required": ["config", "client", "token_cache", "connected"]
    },
    "OidcTokenCache": {
      "$id": "meteor://broker/oidc-token-cache",
      "type": "object",
      "description": "Cached OIDC bearer token with expiry tracking. Thread-safe via Arc<RwLock<>>. Refreshed 5 minutes before expiry.",
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
    "CreateTaskRequest": {
      "$id": "meteor://broker/cloudtasks-create-task-request",
      "type": "object",
      "description": "Request body for Cloud Tasks v2 tasks.create API.",
      "properties": {
        "task": {
          "$ref": "meteor://broker/cloudtasks-task"
        }
      },
      "required": ["task"]
    },
    "CloudTask": {
      "$id": "meteor://broker/cloudtasks-task",
      "type": "object",
      "description": "GCP Cloud Tasks task representation (subset of fields used by broker).",
      "properties": {
        "name": {
          "type": "string",
          "pattern": "^projects/[^/]+/locations/[^/]+/queues/[^/]+/tasks/[^/]+$",
          "description": "Fully qualified task name (auto-generated by API if omitted)"
        },
        "httpRequest": {
          "$ref": "meteor://broker/cloudtasks-http-request"
        },
        "scheduleTime": {
          "oneOf": [
            { "type": "string", "format": "date-time" },
            { "type": "null" }
          ],
          "description": "Earliest time the task may be dispatched (RFC 3339). Null = immediate. Max 30 days from now"
        },
        "dispatchDeadline": {
          "type": "string",
          "pattern": "^[0-9]+s$",
          "description": "Timeout for task dispatch in seconds format (e.g., '600s')"
        },
        "createTime": {
          "type": "string",
          "format": "date-time",
          "description": "Task creation time (read-only, set by API)"
        },
        "dispatchCount": {
          "type": "integer",
          "description": "Number of dispatches (read-only)"
        },
        "responseCount": {
          "type": "integer",
          "description": "Number of responses received (read-only)"
        }
      },
      "required": ["httpRequest"]
    },
    "CloudTasksHttpRequest": {
      "$id": "meteor://broker/cloudtasks-http-request",
      "type": "object",
      "description": "HTTP request target configuration for a Cloud Tasks task.",
      "properties": {
        "url": {
          "type": "string",
          "format": "uri",
          "description": "Full URL of the task handler endpoint"
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
      "required": ["url", "httpMethod", "oidcToken"]
    },
    "GcpErrorMapping": {
      "$id": "meteor://broker/cloudtasks-error-mapping",
      "type": "object",
      "description": "HTTP status code to TaskError variant mapping for Cloud Tasks API.",
      "properties": {
        "404": { "const": "TaskError::NotFound" },
        "401": { "const": "TaskError::AuthenticationError" },
        "403": { "const": "TaskError::AuthenticationError" },
        "409": { "const": "TaskError::AlreadyExists" },
        "429": { "const": "TaskError::RateLimited" },
        "5xx": { "const": "TaskError::BackendError" },
        "transport": { "const": "TaskError::ConnectionError" }
      }
    },
    "FeatureGate": {
      "$id": "meteor://broker/cloudtasks-feature-gate",
      "type": "object",
      "description": "Cargo feature gate for cloudtasks broker module.",
      "properties": {
        "cloudtasks": { "const": "CloudTasksBroker, CloudTasksConfig" }
      }
    }
  }
}
```


## REST API

```yaml
openapi: 3.0.3
info:
  title: GCP Cloud Tasks REST API v2 (subset used by CloudTasksBroker)
  version: v2
servers:
  - url: https://cloudtasks.googleapis.com/v2
paths:
  /projects/{project}/locations/{location}/queues/{queue}/tasks:
    post:
      operationId: createTask
      summary: Create a task in the specified queue
      description: Used by CloudTasksBroker::publish() and publish_delayed()
      parameters:
        - name: project
          in: path
          required: true
          schema: { type: string }
        - name: location
          in: path
          required: true
          schema: { type: string }
        - name: queue
          in: path
          required: true
          schema: { type: string }
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateTaskRequest'
      responses:
        '200':
          description: Task created
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Task'
        '409': { description: Task already exists (name conflict) }
        '429': { description: Rate limited }
  /projects/{project}/locations:
    get:
      operationId: listLocations
      summary: List available locations
      description: Used by CloudTasksBroker::health_check() to verify API reachability
      parameters:
        - name: project
          in: path
          required: true
          schema: { type: string }
      responses:
        '200':
          description: Locations list
          content:
            application/json:
              schema:
                type: object
                properties:
                  locations:
                    type: array
                    items:
                      type: object
                      properties:
                        name: { type: string }
                        locationId: { type: string }
components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
      description: OIDC bearer token from GCP metadata server or service account
  schemas:
    CreateTaskRequest:
      type: object
      properties:
        task:
          $ref: '#/components/schemas/Task'
      required: [task]
    Task:
      type: object
      properties:
        name:
          type: string
          description: 'Fully qualified: projects/{project}/locations/{location}/queues/{queue}/tasks/{taskId}'
        httpRequest:
          type: object
          properties:
            url: { type: string, format: uri, description: Worker handler URL }
            httpMethod: { type: string, enum: [POST, GET, PUT, DELETE, PATCH, HEAD, OPTIONS], default: POST }
            body: { type: string, description: Base64-encoded request body }
            headers: { type: object, additionalProperties: { type: string } }
            oidcToken:
              type: object
              properties:
                serviceAccountEmail: { type: string }
                audience: { type: string }
              required: [serviceAccountEmail]
          required: [url, httpMethod]
        scheduleTime:
          type: string
          format: date-time
          description: Earliest dispatch time (RFC 3339). Max 30 days from now
        dispatchDeadline:
          type: string
          description: 'Task timeout in duration format (e.g., "600s"). Range: 15s-1800s'
        createTime:
          type: string
          format: date-time
          description: Read-only creation timestamp
        dispatchCount:
          type: integer
          description: Read-only dispatch attempt count
        responseCount:
          type: integer
          description: Read-only response count
        firstAttempt:
          type: object
          description: Details of first dispatch attempt
        lastAttempt:
          type: object
          description: Details of most recent dispatch attempt
      required: [httpRequest]
security:
  - bearerAuth: []
```