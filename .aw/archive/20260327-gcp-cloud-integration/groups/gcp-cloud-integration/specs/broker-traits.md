---
id: broker-traits
main_spec_ref: "crates/cclab-fetch/broker/broker-traits.md"
merge_strategy: new
fill_sections: [overview, requirements, scenarios, schema]
filled_sections: [overview, requirements, scenarios, schema]
create_complete: true
---

# Broker Traits

## Overview

Defines the trait hierarchy for message brokers in cclab-queue. The `Broker` trait provides the core interface (connect, disconnect, publish, health_check, delivery_model, capabilities). Three extension traits specialize delivery patterns: `PullBroker` (worker-fetched subscriptions with ack/nack), `PushBroker` (HTTP push with request parsing and status codes), and `DelayedBroker` (native delayed/scheduled task publishing). `BrokerCapabilities` advertises feature support (delayed_tasks, dead_letter, priority, batching, max_delay). `DeliveryModel` enum distinguishes Pull vs Push brokers. `BrokerMessage` carries decoded payloads with delivery metadata. Implementations are feature-gated per backend: `nats`, `pubsub`, `pubsub-push`, `cloudtasks`.

Source: `crates/cclab-queue/src/broker/mod.rs`
## Requirements

| ID | Title | Priority | Acceptance Criteria |
|----|-------|----------|---------------------|
| R1 | Broker core trait | P0 | `Broker` trait defines async methods: `connect()`, `disconnect()`, `publish(queue, message)`, `health_check()`, and sync methods: `delivery_model()` → `DeliveryModel`, `capabilities()` → `BrokerCapabilities`. All broker implementations must implement this trait. Requires `Send + Sync + 'static` |
| R2 | PullBroker trait | P1 | `PullBroker: Broker` adds `subscribe(queue, handler)` → `SubscriptionHandle`, `ack(delivery_tag)`, `nack(delivery_tag, requeue)`. Used by NATS and Pub/Sub pull backends |
| R3 | PushBroker trait | P0 | `PushBroker: Broker` adds `parse_push_request(headers, body)` → `BrokerMessage`, `ack_status_code()` (default 200), `nack_status_code()` (default 500), `endpoint_path()`. Used by Cloud Tasks and Pub/Sub push backends |
| R4 | DelayedBroker trait | P0 | `DelayedBroker: Broker` adds `publish_delayed(queue, message, delay: Duration)` and `publish_at(queue, message, eta: DateTime<Utc>)`. `publish_at` has default implementation that converts to delay or calls `publish()` immediately if eta is past |
| R5 | BrokerCapabilities struct | P1 | Declares broker feature support: `delayed_tasks: bool`, `dead_letter: bool`, `priority: bool`, `batching: bool`, `max_delay: Option<Duration>`. Returns `Default` (all false/None) as baseline |
| R6 | DeliveryModel enum | P1 | Two variants: `Pull` (worker fetches messages) and `Push` (broker sends HTTP to worker). Each broker returns exactly one model |
| R7 | BrokerMessage struct | P1 | Carries `delivery_tag: String`, `payload: TaskMessage`, `headers: HashMap<String, String>`, `timestamp: DateTime<Utc>`, `redelivered: bool`. Produced by `PushBroker::parse_push_request` and `PullBroker::subscribe` handler |
| R8 | Feature-gated backends | P1 | Each backend module is conditionally compiled: `nats` → NatsBroker, `pubsub` → PubSubPullBroker, `pubsub-push` → PubSubPushBroker, `cloudtasks` → CloudTasksBroker. No GCP/NATS dependencies pulled unless opted in |

### Constraints

- All trait methods returning `Result` use `TaskError` as the error type
- `Broker` requires `async_trait` macro for async method dispatch
- `PushBroker` is NOT async (sync parse + sync status codes) — avoids lifetime issues in HTTP handlers
- `SubscriptionHandle` owns a `CancellationToken` for cooperative cancellation
## Scenarios

### S1: Push broker receives and parses HTTP task delivery (R1, R3, R7)

**GIVEN** a CloudTasksBroker implementing Broker + PushBroker
**WHEN** an HTTP POST arrives with `x-cloudtasks-taskname` header and JSON body
**THEN** `parse_push_request()` returns a `BrokerMessage` with `delivery_tag` from the header, deserialized `TaskMessage` payload, and `redelivered = true` if `x-cloudtasks-taskretrycount > 0`

### S2: Push broker returns ack/nack status codes (R3)

**GIVEN** a PushBroker implementation
**WHEN** the HTTP handler calls `ack_status_code()` after successful processing
**THEN** returns 200; when processing fails and `nack_status_code()` is called, returns 500 (triggering Cloud Tasks retry)

### S3: Delayed broker publishes task with native delay (R1, R4)

**GIVEN** a broker implementing Broker + DelayedBroker with `capabilities().delayed_tasks == true`
**WHEN** `publish_delayed(queue, message, Duration::from_secs(300))` is called
**THEN** the task is scheduled for delivery 300 seconds in the future via the broker's native delay mechanism

### S4: Delayed broker publish_at with past ETA falls back to immediate publish (R4)

**GIVEN** a DelayedBroker implementation
**WHEN** `publish_at(queue, message, eta)` is called with `eta <= Utc::now()`
**THEN** the default implementation calls `self.publish(queue, message)` immediately without delay

### S5: Pull broker subscription and ack lifecycle (R1, R2)

**GIVEN** a NatsBroker implementing Broker + PullBroker
**WHEN** `subscribe(queue, handler)` is called
**THEN** returns a `SubscriptionHandle` with a `CancellationToken`; messages arrive via `handler.handle()`; calling `ack(delivery_tag)` confirms processing; calling `handle.cancel()` stops the subscription

### S6: Broker capabilities declare feature support (R1, R5, R6)

**GIVEN** a CloudTasksBroker
**WHEN** `capabilities()` is called
**THEN** returns `BrokerCapabilities { delayed_tasks: true, dead_letter: true, priority: false, batching: false, max_delay: Some(30 days) }`; `delivery_model()` returns `DeliveryModel::Push`

### S7: Feature gate excludes unused backend dependencies (R8)

**GIVEN** a Cargo.toml with only `features = ["cloudtasks"]` enabled
**WHEN** the crate is compiled
**THEN** only `CloudTasksBroker` and `CloudTasksConfig` are available; NATS and Pub/Sub modules are excluded from compilation
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
<!-- type: changes lang: yaml -->

<!-- TODO -->

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
  "title": "Broker Traits — Data Models and Trait Interfaces",
  "$defs": {
    "DeliveryModel": {
      "$id": "meteor://broker/delivery-model",
      "type": "string",
      "enum": ["Pull", "Push"],
      "description": "Pull = worker fetches messages from broker (NATS, Pub/Sub pull). Push = broker sends HTTP to worker (Cloud Tasks, Pub/Sub push)."
    },
    "BrokerCapabilities": {
      "$id": "meteor://broker/capabilities",
      "type": "object",
      "description": "Advertised feature support for a broker implementation. Default: all false/None.",
      "properties": {
        "delayed_tasks": {
          "type": "boolean",
          "default": false,
          "description": "Supports native delayed/scheduled task delivery"
        },
        "dead_letter": {
          "type": "boolean",
          "default": false,
          "description": "Supports dead-letter queues for failed messages"
        },
        "priority": {
          "type": "boolean",
          "default": false,
          "description": "Supports message priority levels"
        },
        "batching": {
          "type": "boolean",
          "default": false,
          "description": "Supports message batching"
        },
        "max_delay": {
          "oneOf": [
            { "type": "integer", "minimum": 0, "description": "Maximum delay in seconds" },
            { "type": "null" }
          ],
          "default": null,
          "description": "Maximum delay duration in seconds (if delayed_tasks is true)"
        }
      },
      "required": ["delayed_tasks", "dead_letter", "priority", "batching"]
    },
    "BrokerMessage": {
      "$id": "meteor://broker/message",
      "type": "object",
      "description": "Message received from the broker, wrapping TaskMessage with delivery metadata.",
      "properties": {
        "delivery_tag": {
          "type": "string",
          "description": "Unique tag for acknowledgment (e.g., x-cloudtasks-taskname or NATS message ID)"
        },
        "payload": {
          "$ref": "meteor://task-message",
          "description": "The deserialized TaskMessage"
        },
        "headers": {
          "type": "object",
          "additionalProperties": { "type": "string" },
          "description": "Original message headers (HTTP headers for push, NATS headers for pull)"
        },
        "timestamp": {
          "type": "string",
          "format": "date-time",
          "description": "UTC timestamp when message was received"
        },
        "redelivered": {
          "type": "boolean",
          "description": "True if this is a redelivery (retry count > 0)"
        }
      },
      "required": ["delivery_tag", "payload", "headers", "timestamp", "redelivered"]
    },
    "SubscriptionHandle": {
      "$id": "meteor://broker/subscription-handle",
      "type": "object",
      "description": "Handle for managing a pull subscription. Owns a CancellationToken for cooperative shutdown.",
      "properties": {
        "queue": {
          "type": "string",
          "description": "Queue name this subscription is bound to"
        }
      },
      "required": ["queue"]
    },
    "BrokerTrait": {
      "$id": "meteor://broker/trait/broker",
      "type": "object",
      "description": "Core broker interface. All implementations require Send + Sync + 'static. Uses async_trait.",
      "properties": {
        "methods": {
          "type": "object",
          "properties": {
            "connect": { "description": "async fn connect(&self) -> Result<(), TaskError>" },
            "disconnect": { "description": "async fn disconnect(&self) -> Result<(), TaskError>" },
            "publish": { "description": "async fn publish(&self, queue: &str, message: TaskMessage) -> Result<(), TaskError>" },
            "health_check": { "description": "async fn health_check(&self) -> Result<(), TaskError>" },
            "delivery_model": { "description": "fn delivery_model(&self) -> DeliveryModel" },
            "capabilities": { "description": "fn capabilities(&self) -> BrokerCapabilities" }
          }
        }
      }
    },
    "PullBrokerTrait": {
      "$id": "meteor://broker/trait/pull-broker",
      "type": "object",
      "description": "Extension trait for pull-based brokers (worker fetches messages). Extends Broker.",
      "properties": {
        "methods": {
          "type": "object",
          "properties": {
            "subscribe": { "description": "async fn subscribe<H: MessageHandler>(&self, queue: &str, handler: Arc<H>) -> Result<SubscriptionHandle, TaskError>" },
            "ack": { "description": "async fn ack(&self, delivery_tag: &str) -> Result<(), TaskError>" },
            "nack": { "description": "async fn nack(&self, delivery_tag: &str, requeue: bool) -> Result<(), TaskError>" }
          }
        }
      }
    },
    "PushBrokerTrait": {
      "$id": "meteor://broker/trait/push-broker",
      "type": "object",
      "description": "Extension trait for push-based brokers (broker sends HTTP to worker). Extends Broker. NOT async — sync methods only.",
      "properties": {
        "methods": {
          "type": "object",
          "properties": {
            "parse_push_request": { "description": "fn parse_push_request(&self, headers: &HashMap<String,String>, body: &[u8]) -> Result<BrokerMessage, TaskError>" },
            "ack_status_code": { "description": "fn ack_status_code(&self) -> u16  [default: 200]" },
            "nack_status_code": { "description": "fn nack_status_code(&self) -> u16  [default: 500]" },
            "endpoint_path": { "description": "fn endpoint_path(&self) -> &str" }
          }
        }
      }
    },
    "DelayedBrokerTrait": {
      "$id": "meteor://broker/trait/delayed-broker",
      "type": "object",
      "description": "Extension trait for brokers with native delayed task support. Extends Broker. Uses async_trait.",
      "properties": {
        "methods": {
          "type": "object",
          "properties": {
            "publish_delayed": { "description": "async fn publish_delayed(&self, queue: &str, message: TaskMessage, delay: Duration) -> Result<(), TaskError>" },
            "publish_at": { "description": "async fn publish_at(&self, queue: &str, message: TaskMessage, eta: DateTime<Utc>) -> Result<(), TaskError>  [default: converts to delay or immediate]" }
          }
        }
      }
    },
    "FeatureGateMapping": {
      "$id": "meteor://broker/feature-gates",
      "type": "object",
      "description": "Cargo feature → backend module mapping",
      "properties": {
        "nats": { "const": "NatsBroker, NatsBrokerConfig" },
        "pubsub": { "const": "PubSubPullBroker, PubSubPullConfig" },
        "pubsub-push": { "const": "PubSubPushBroker, PubSubPushConfig" },
        "cloudtasks": { "const": "CloudTasksBroker, CloudTasksConfig" }
      }
    }
  }
}
```

# Reviews
