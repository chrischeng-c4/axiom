---
id: push-receiver
main_spec_ref: "crates/cclab-fetch/scheduler/push-receiver.md"
merge_strategy: new
---

# Push Receiver

## Overview

Shared HTTP endpoint that receives scheduled trigger callbacks from both Cloud Scheduler (GCP) and K8s CronJob pods. Mounted as axum `Router` routes on the existing cclab server at `/scheduler/push/{task_name}` — no separate port or TLS config. Incoming POST requests are authenticated based on caller type: Cloud Scheduler requests carry OIDC bearer tokens validated against Google public keys (JWKS), K8s CronJob requests carry HMAC-SHA256 signatures validated by recomputing `X-Scheduler-Signature` header against the shared secret.

Request flow: extract `task_name` from URL path → authenticate caller (OIDC or HMAC) → deserialize `TaskMessage` from request body → record `actual_at` timestamp for schedule monitor → call `broker.publish(queue, message)` → return 200 OK on success, 401 on auth failure, 500 on processing error.

Authentication is pluggable via `PushAuthenticator` enum with two variants: `Oidc` (verifies JWT against Google JWKS endpoint, checks `audience` claim matches configured value, caches public keys with TTL) and `Hmac` (recomputes HMAC-SHA256 over request body, constant-time comparison against `X-Scheduler-Signature` header value). The authenticator is selected per-request based on header presence — `Authorization: Bearer` triggers OIDC path, `X-Scheduler-Signature` triggers HMAC path.

The push receiver holds references to `Arc<dyn Broker>` for downstream message publishing and `Arc<ScheduleMonitor>` (optional) for recording trigger timestamps. Configuration via `PushReceiverConfig` specifies OIDC audience, HMAC secret, enabled auth methods, and queue routing.

Source: `crates/cclab-queue/src/broker/mod.rs` (Broker trait, PushBroker pattern), `crates/cclab-queue/src/broker/cloudtasks.rs` (endpoint_path pattern), `crates/cclab-queue/src/scheduler/periodic.rs` (PeriodicTask, queue routing)
## Requirements

| ID | Title | Priority | Acceptance Criteria |
|----|-------|----------|---------------------|
| R1 | Axum router mount | P0 | `PushReceiver::router()` returns `axum::Router` with route `POST /scheduler/push/:task_name`. Router is mergeable into the existing cclab server via `app.merge(push_receiver.router())`. No separate port, listener, or TLS configuration |
| R2 | Cloud Scheduler OIDC authentication | P0 | When request contains `Authorization: Bearer <token>` header, validate JWT against Google JWKS endpoint (`https://www.googleapis.com/oauth2/v3/certs`). Verify: signature (RS256), `iss` is `https://accounts.google.com`, `aud` matches `PushReceiverConfig.oidc_audience`, token is not expired. Cache JWKS public keys with configurable TTL (default 1 hour). Reject with 401 if validation fails |
| R3 | K8s CronJob HMAC authentication | P0 | When request contains `X-Scheduler-Signature: sha256={hex_digest}` header, recompute HMAC-SHA256 over raw request body using `PushReceiverConfig.hmac_secret`. Compare using constant-time equality (`ring::constant_time::verify_slices_are_equal` or equivalent). Reject with 401 if signature is missing, malformed, or mismatched |
| R4 | Auth method auto-detection | P1 | Per-request auth selection based on header presence: `Authorization: Bearer` → OIDC path, `X-Scheduler-Signature` → HMAC path. If neither header present → reject 401. If both present → prefer `Authorization` (Cloud Scheduler always sets it). `PushReceiverConfig.enabled_auth_methods` restricts which methods are accepted (default: both enabled) |
| R5 | Task name extraction and routing | P0 | Extract `task_name` from URL path parameter `:task_name`. Look up target queue from `PushReceiverConfig.task_queue_map: HashMap<String, String>` (task_name → queue). If task_name not in map, use `PushReceiverConfig.default_queue`. If no default_queue and task_name not in map, return 404 |
| R6 | Request body parsing | P0 | Deserialize request body as JSON `TaskMessage`. Cloud Scheduler sends base64-encoded body in `httpTarget.body` — if `Content-Type` is `application/json`, parse directly; if body appears base64-encoded (from Cloud Scheduler httpTarget), decode first then parse. K8s CronJob pods send raw JSON. Return 400 on parse failure |
| R7 | Downstream broker publish | P0 | After successful auth and parsing, call `broker.publish(queue, task_message)` where `broker: Arc<dyn Broker>`. On `Ok(())` return HTTP 200 with empty body. On `Err(TaskError)` return HTTP 500 with error description in body |
| R8 | Schedule monitor integration | P1 | If `ScheduleMonitor` is configured (optional), call `monitor.record_trigger(task_name, Utc::now())` before `broker.publish()`. This records `actual_at` for missed/late detection. Monitor failure is logged but does not fail the request (best-effort) |
| R9 | Observability | P1 | Emit tracing spans: `push_receiver.handle` (per-request, includes task_name, auth_method, status_code). Increment Prometheus counter `scheduler_push_received_total{task_name, auth_method, status}` where status is `ok`, `auth_failed`, `parse_error`, `publish_error`. Record request latency in histogram `scheduler_push_duration_seconds{task_name}` |

### Constraints

- All error responses use structured JSON body: `{ "error": "<description>" }`
- Request body size limit: 1 MiB (configurable via `PushReceiverConfig.max_body_size`)
- HMAC secret must be at least 32 bytes
- OIDC JWKS cache is shared across all requests (not per-request fetch)
- `PushReceiver` requires `Send + Sync` for axum handler compatibility
- No dependency on any specific `SchedulerBackend` — push receiver is backend-agnostic
## Scenarios

### S1: Cloud Scheduler triggers push receiver with valid OIDC token (R1, R2, R6, R7)

**GIVEN** a `PushReceiver` mounted on the server with `oidc_audience = "https://app.example.com"`
**WHEN** Cloud Scheduler POSTs to `/scheduler/push/daily-cleanup` with `Authorization: Bearer <valid_jwt>` and JSON body containing `TaskMessage`
**THEN** JWT is validated against Google JWKS (RS256, iss=accounts.google.com, aud matches); `task_name = "daily-cleanup"` extracted from path; `TaskMessage` deserialized from body; `broker.publish("default", task_message)` called; returns HTTP 200

### S2: K8s CronJob triggers push receiver with valid HMAC signature (R1, R3, R6, R7)

**GIVEN** a `PushReceiver` with `hmac_secret = "my-secret-key"`
**WHEN** CronJob pod POSTs to `/scheduler/push/hourly-sync` with `X-Scheduler-Signature: sha256={hex(hmac_sha256("my-secret-key", body))}` and JSON body containing `TaskMessage`
**THEN** HMAC-SHA256 recomputed over raw body; constant-time compared with header value; `task_name = "hourly-sync"` extracted; `broker.publish(queue, task_message)` called; returns HTTP 200

### S3: Request with invalid OIDC token is rejected (R2)

**GIVEN** a `PushReceiver` with OIDC auth enabled
**WHEN** request arrives with `Authorization: Bearer <expired_or_invalid_jwt>`
**THEN** JWT validation fails (expired, wrong audience, invalid signature); returns HTTP 401 with `{ "error": "OIDC token validation failed: <reason>" }`; `broker.publish()` is NOT called

### S4: Request with invalid HMAC signature is rejected (R3)

**GIVEN** a `PushReceiver` with `hmac_secret = "correct-secret"`
**WHEN** request arrives with `X-Scheduler-Signature: sha256={hmac_computed_with_wrong_secret}`
**THEN** HMAC recomputation produces different digest; constant-time comparison fails; returns HTTP 401 with `{ "error": "HMAC signature validation failed" }`; `broker.publish()` is NOT called

### S5: Request with no auth headers is rejected (R4)

**GIVEN** a `PushReceiver` with both auth methods enabled
**WHEN** request arrives without `Authorization` or `X-Scheduler-Signature` headers
**THEN** returns HTTP 401 with `{ "error": "No authentication credentials provided" }`

### S6: Auth method auto-detection selects correct validator (R4)

**GIVEN** a `PushReceiver` with both OIDC and HMAC enabled
**WHEN** request contains `Authorization: Bearer <token>` header
**THEN** OIDC validation path is used (not HMAC)
**WHEN** request contains only `X-Scheduler-Signature` header
**THEN** HMAC validation path is used
**WHEN** request contains both headers
**THEN** OIDC path is preferred (`Authorization` takes precedence)

### S7: Task name routing to configured queue (R5)

**GIVEN** a `PushReceiverConfig` with `task_queue_map = { "daily-cleanup": "maintenance", "hourly-sync": "sync" }` and `default_queue = "default"`
**WHEN** authenticated request arrives for `/scheduler/push/daily-cleanup`
**THEN** `broker.publish("maintenance", task_message)` is called
**WHEN** authenticated request arrives for `/scheduler/push/unknown-task`
**THEN** `broker.publish("default", task_message)` is called (falls back to default_queue)

### S8: Unknown task with no default queue returns 404 (R5)

**GIVEN** a `PushReceiverConfig` with `task_queue_map = { "daily-cleanup": "maintenance" }` and `default_queue = None`
**WHEN** authenticated request arrives for `/scheduler/push/unknown-task`
**THEN** returns HTTP 404 with `{ "error": "Unknown task: unknown-task" }`

### S9: Malformed request body returns 400 (R6)

**GIVEN** a `PushReceiver` with valid auth
**WHEN** authenticated request arrives with body that is not valid `TaskMessage` JSON
**THEN** returns HTTP 400 with `{ "error": "Failed to parse TaskMessage: <serde error>" }`

### S10: Broker publish failure returns 500 (R7)

**GIVEN** a `PushReceiver` with `broker` that returns `Err(TaskError::ConnectionError(...))`
**WHEN** authenticated request is processed and `broker.publish()` fails
**THEN** returns HTTP 500 with `{ "error": "Failed to publish task: <error>" }`

### S11: Schedule monitor records trigger timestamp (R8)

**GIVEN** a `PushReceiver` with `ScheduleMonitor` configured
**WHEN** authenticated request for task `daily-cleanup` is successfully processed
**THEN** `monitor.record_trigger("daily-cleanup", actual_at)` is called before `broker.publish()`; if monitor call fails, error is logged at `warn` level but request processing continues; `broker.publish()` still executes

### S12: Metrics are emitted on each request (R9)

**GIVEN** a `PushReceiver` handling requests
**WHEN** request for `daily-cleanup` succeeds with OIDC auth
**THEN** `scheduler_push_received_total{task_name="daily-cleanup", auth_method="oidc", status="ok"}` incremented; `scheduler_push_duration_seconds{task_name="daily-cleanup"}` observed
**WHEN** request fails with auth error
**THEN** `scheduler_push_received_total{..., status="auth_failed"}` incremented
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
  - file: crates/cclab-queue/src/scheduler/push_receiver.rs
    action: create
    description: |
      New file implementing PushReceiver, PushReceiverConfig, PushAuthenticator,
      OidcValidator, HmacValidator, and JwksCache. Provides axum Router with
      POST /scheduler/push/:task_name route. Handles auth detection, JWT/HMAC
      validation, TaskMessage parsing, schedule monitor integration, and
      broker.publish() downstream call. Emits tracing spans and Prometheus metrics.
    structs:
      - PushReceiver
      - PushReceiverConfig
      - PushAuthenticator
      - OidcValidator
      - HmacValidator
      - JwksCache
      - PushErrorResponse
    enums:
      - AuthMethod
    methods:
      - "fn new(config: PushReceiverConfig, broker: Arc<dyn Broker>, monitor: Option<Arc<ScheduleMonitor>>) -> Result<Self, TaskError>  # constructs PushReceiver with initialized authenticator"
      - "fn router(self: Arc<Self>) -> axum::Router  # returns axum Router with POST /scheduler/push/:task_name"
      - "async fn handle_push(State(receiver): State<Arc<PushReceiver>>, Path(task_name): Path<String>, headers: HeaderMap, body: Bytes) -> impl IntoResponse  # main request handler"
      - "async fn authenticate(&self, headers: &HeaderMap, body: &[u8]) -> Result<AuthMethod, (StatusCode, Json<PushErrorResponse>)>  # auto-detect and validate auth"
      - "fn resolve_queue(&self, task_name: &str) -> Result<String, (StatusCode, Json<PushErrorResponse>)>  # task_name → queue lookup"
      - "fn parse_task_message(body: &[u8]) -> Result<TaskMessage, (StatusCode, Json<PushErrorResponse>)>  # deserialize body as TaskMessage"

  - file: crates/cclab-queue/src/scheduler/push_auth.rs
    action: create
    description: |
      Separated auth module for OidcValidator and HmacValidator implementations.
      OidcValidator: fetch_jwks() from Google endpoint, validate_token() verifying
      RS256 signature + iss + aud + exp claims, JWKS cache with TTL refresh.
      HmacValidator: validate_signature() with constant-time comparison using
      hmac + sha2 crates.
    structs:
      - OidcValidator
      - HmacValidator
      - JwksCache
    methods:
      - "async fn OidcValidator::validate_token(&self, token: &str) -> Result<(), TaskError>  # full JWT validation"
      - "async fn OidcValidator::fetch_jwks(&self) -> Result<Vec<DecodingKey>, TaskError>  # fetch and parse Google JWKS"
      - "fn OidcValidator::is_cache_valid(&self) -> bool  # check if cached keys are within TTL"
      - "fn HmacValidator::validate_signature(&self, body: &[u8], signature_header: &str) -> Result<(), TaskError>  # HMAC-SHA256 verify"
      - "fn HmacValidator::compute_signature(&self, body: &[u8]) -> String  # compute sha256={hex_digest}"
    dependencies:
      - "jsonwebtoken (JWT decode + verification)"
      - "hmac + sha2 (HMAC-SHA256)"
      - "reqwest (JWKS fetch)"

  - file: crates/cclab-queue/src/scheduler/mod.rs
    action: modify
    description: |
      Add module declarations and re-exports for push_receiver and push_auth.
    additions:
      - 'pub mod push_receiver;'
      - 'pub mod push_auth;'
      - 'pub use push_receiver::{PushReceiver, PushReceiverConfig};'

  - file: crates/cclab-queue/Cargo.toml
    action: modify
    description: |
      Add dependencies for push receiver auth: jsonwebtoken, hmac, sha2.
      These are always-on dependencies (not feature-gated) since push receiver
      is used by both cloud-scheduler and k8s-scheduler backends.
    additions:
      - 'jsonwebtoken = "9"'
      - 'hmac = "0.12"'
      - 'sha2 = "0.10"'
    notes: |
      reqwest is likely already a dependency (used by cloud-scheduler).
      axum and tokio are already dependencies of cclab-queue.
      hex crate may be needed for HMAC hex encoding — check if it exists already.
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
  "title": "Push Receiver — Data Models",
  "$defs": {
    "PushReceiverConfig": {
      "$id": "meteor://scheduler/push-receiver-config",
      "type": "object",
      "description": "Configuration for the push receiver HTTP endpoint.",
      "properties": {
        "oidc_audience": {
          "type": "string",
          "format": "uri",
          "description": "Expected audience claim in OIDC JWT tokens from Cloud Scheduler"
        },
        "oidc_issuer": {
          "type": "string",
          "default": "https://accounts.google.com",
          "description": "Expected issuer claim in OIDC JWT tokens"
        },
        "oidc_jwks_url": {
          "type": "string",
          "format": "uri",
          "default": "https://www.googleapis.com/oauth2/v3/certs",
          "description": "URL to fetch Google JWKS public keys for JWT verification"
        },
        "oidc_jwks_cache_ttl_secs": {
          "type": "integer",
          "default": 3600,
          "minimum": 60,
          "description": "TTL in seconds for cached JWKS public keys"
        },
        "hmac_secret": {
          "type": "string",
          "minLength": 32,
          "description": "Shared HMAC-SHA256 secret for K8s CronJob request validation"
        },
        "enabled_auth_methods": {
          "type": "array",
          "items": {
            "type": "string",
            "enum": ["oidc", "hmac"]
          },
          "default": ["oidc", "hmac"],
          "description": "Which authentication methods are accepted"
        },
        "task_queue_map": {
          "type": "object",
          "additionalProperties": { "type": "string" },
          "default": {},
          "description": "Mapping of task_name to target queue name"
        },
        "default_queue": {
          "oneOf": [
            { "type": "string" },
            { "type": "null" }
          ],
          "default": "default",
          "description": "Fallback queue when task_name is not in task_queue_map. If null, unknown tasks return 404"
        },
        "max_body_size": {
          "type": "integer",
          "default": 1048576,
          "minimum": 1024,
          "description": "Maximum request body size in bytes (default 1 MiB)"
        }
      },
      "required": []
    },
    "PushReceiver": {
      "$id": "meteor://scheduler/push-receiver",
      "type": "object",
      "description": "HTTP push receiver that handles Cloud Scheduler and K8s CronJob callbacks. Produces an axum Router. Requires Send + Sync.",
      "properties": {
        "config": {
          "$ref": "meteor://scheduler/push-receiver-config"
        },
        "broker": {
          "type": "string",
          "const": "Arc<dyn Broker>",
          "description": "Downstream broker for publishing received task messages"
        },
        "monitor": {
          "type": "string",
          "const": "Option<Arc<ScheduleMonitor>>",
          "description": "Optional schedule monitor for recording trigger timestamps"
        },
        "authenticator": {
          "$ref": "meteor://scheduler/push-authenticator",
          "description": "Authentication handler (auto-detects OIDC vs HMAC per request)"
        }
      },
      "required": ["config", "broker", "authenticator"]
    },
    "PushAuthenticator": {
      "$id": "meteor://scheduler/push-authenticator",
      "type": "object",
      "description": "Authentication handler for push receiver. Holds state for both OIDC and HMAC validation.",
      "properties": {
        "oidc_validator": {
          "oneOf": [
            { "$ref": "meteor://scheduler/oidc-validator" },
            { "type": "null" }
          ],
          "description": "OIDC JWT validator (null if OIDC auth is disabled)"
        },
        "hmac_validator": {
          "oneOf": [
            { "$ref": "meteor://scheduler/hmac-validator" },
            { "type": "null" }
          ],
          "description": "HMAC signature validator (null if HMAC auth is disabled)"
        }
      }
    },
    "OidcValidator": {
      "$id": "meteor://scheduler/oidc-validator",
      "type": "object",
      "description": "OIDC JWT validator that verifies tokens from Google Cloud Scheduler.",
      "properties": {
        "audience": {
          "type": "string",
          "description": "Expected aud claim"
        },
        "issuer": {
          "type": "string",
          "description": "Expected iss claim"
        },
        "jwks_url": {
          "type": "string",
          "format": "uri",
          "description": "Google JWKS endpoint URL"
        },
        "jwks_cache": {
          "type": "string",
          "const": "Arc<RwLock<JwksCache>>",
          "description": "Cached JWKS keys with expiry"
        }
      },
      "required": ["audience", "issuer", "jwks_url", "jwks_cache"]
    },
    "JwksCache": {
      "$id": "meteor://scheduler/jwks-cache",
      "type": "object",
      "description": "Cached Google JWKS public keys with TTL-based refresh.",
      "properties": {
        "keys": {
          "type": "string",
          "const": "Vec<jsonwebtoken::DecodingKey>",
          "description": "Parsed RSA public keys from Google JWKS"
        },
        "fetched_at": {
          "type": "string",
          "format": "date-time",
          "description": "Timestamp when keys were last fetched"
        },
        "ttl_secs": {
          "type": "integer",
          "description": "Cache TTL in seconds"
        }
      },
      "required": ["keys", "fetched_at", "ttl_secs"]
    },
    "HmacValidator": {
      "$id": "meteor://scheduler/hmac-validator",
      "type": "object",
      "description": "HMAC-SHA256 signature validator for K8s CronJob requests.",
      "properties": {
        "secret": {
          "type": "string",
          "const": "hmac::Hmac<sha2::Sha256>",
          "description": "Pre-initialized HMAC key"
        },
        "header_name": {
          "type": "string",
          "const": "X-Scheduler-Signature",
          "description": "Header containing the HMAC signature"
        },
        "signature_prefix": {
          "type": "string",
          "const": "sha256=",
          "description": "Prefix before hex-encoded digest in header value"
        }
      },
      "required": ["secret"]
    },
    "AuthMethod": {
      "$id": "meteor://scheduler/auth-method",
      "type": "string",
      "enum": ["oidc", "hmac"],
      "description": "Authentication method detected from request headers."
    },
    "PushErrorResponse": {
      "$id": "meteor://scheduler/push-error-response",
      "type": "object",
      "description": "Structured error response from push receiver endpoint.",
      "properties": {
        "error": {
          "type": "string",
          "description": "Human-readable error description"
        }
      },
      "required": ["error"]
    },
    "PushMetrics": {
      "$id": "meteor://scheduler/push-metrics",
      "type": "object",
      "description": "Prometheus metrics emitted by push receiver.",
      "properties": {
        "scheduler_push_received_total": {
          "type": "string",
          "const": "Counter<task_name, auth_method, status>",
          "description": "Total push requests received. status: ok, auth_failed, parse_error, publish_error"
        },
        "scheduler_push_duration_seconds": {
          "type": "string",
          "const": "Histogram<task_name>",
          "description": "Request processing latency in seconds"
        }
      }
    }
  }
}
```


## REST API

```yaml
openapi: 3.0.3
info:
  title: Scheduler Push Receiver
  version: 1.0.0
  description: |
    HTTP endpoint receiving scheduled trigger callbacks from Cloud Scheduler (OIDC)
    and K8s CronJob pods (HMAC). Mounted on existing server at /scheduler/push/{task_name}.
servers:
  - url: '{base_url}'
    variables:
      base_url:
        default: http://localhost:8080
paths:
  /scheduler/push/{task_name}:
    post:
      operationId: handleScheduledTrigger
      summary: Receive a scheduled task trigger callback
      description: |
        Authenticates the caller (OIDC or HMAC based on headers),
        deserializes TaskMessage from body, records trigger for monitoring,
        and publishes to the configured broker queue.
      parameters:
        - name: task_name
          in: path
          required: true
          schema:
            type: string
          description: Name of the scheduled task being triggered
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/TaskMessage'
      responses:
        '200':
          description: Task trigger accepted and published to broker
        '400':
          description: Invalid request body (not valid TaskMessage JSON)
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorResponse'
        '401':
          description: Authentication failed (invalid/missing OIDC token or HMAC signature)
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorResponse'
        '404':
          description: Unknown task_name and no default_queue configured
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorResponse'
        '413':
          description: Request body exceeds max_body_size
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorResponse'
        '500':
          description: Internal error (broker.publish failed)
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorResponse'
      security:
        - oidcAuth: []
        - hmacAuth: []
components:
  securitySchemes:
    oidcAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
      description: |
        OIDC bearer token from Google Cloud Scheduler.
        JWT validated against Google JWKS (RS256), iss=accounts.google.com, aud=configured audience.
    hmacAuth:
      type: apiKey
      in: header
      name: X-Scheduler-Signature
      description: |
        HMAC-SHA256 signature from K8s CronJob pods.
        Format: sha256={hex(hmac_sha256(secret, body))}
  schemas:
    TaskMessage:
      type: object
      description: Task message payload (defined in cclab-queue core)
      properties:
        id:
          type: string
          format: uuid
        task_name:
          type: string
        args:
          type: object
        kwargs:
          type: object
        queue:
          type: string
        eta:
          type: string
          format: date-time
          nullable: true
        expires:
          type: string
          format: date-time
          nullable: true
        retries:
          type: integer
          default: 0
      required: [id, task_name]
    ErrorResponse:
      type: object
      properties:
        error:
          type: string
          description: Human-readable error description
      required: [error]
```