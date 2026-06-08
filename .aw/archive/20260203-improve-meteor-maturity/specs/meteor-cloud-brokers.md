---
id: meteor-cloud-brokers
type: spec
title: "Meteor Cloud Brokers Specification"
version: 1
spec_type: http-api
created_at: 2026-01-30T03:53:45.667517+00:00
updated_at: 2026-01-30T03:53:45.667517+00:00
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: true
  has_semantic_diagrams: false
  api_spec_type: openapi-3.1
  diagrams:
    - type: sequence
      title: "Cloud Tasks Push Broker Flow"
history:
  - timestamp: 2026-01-30T03:53:45.667517+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Meteor Cloud Brokers Specification

## Overview

Implementation details for GCP Cloud Tasks and Pub/Sub Push brokers in cclab-meteor. These brokers utilize a push-based delivery model where the broker sends HTTP requests to the worker.

## Requirements

### R1 - Cloud Tasks Broker

```yaml
id: R1
priority: high
status: draft
```

Implement CloudTasksBroker with publish and push-parsing capabilities.

### R2 - Pub/Sub Push Broker

```yaml
id: R2
priority: high
status: draft
```

Implement PubSubPushBroker for handling push notifications from GCP Pub/Sub.

### R3 - Push Handler Service

```yaml
id: R3
priority: high
status: draft
```

Develop an Axum-based HTTP handler for processing push requests from brokers.

## Acceptance Criteria

### Scenario: Cloud Tasks Dispatch

- **WHEN** A task is published via CloudTasksBroker and dispatched by GCP.
- **THEN** Meteor worker receives the HTTP request and executes the task.

### Scenario: Pub/Sub Push Delivery

- **WHEN** GCP Pub/Sub pushes a message to the configured Meteor endpoint.
- **THEN** The message is parsed and executed by the Meteor worker.

## Diagrams

### Cloud Tasks Push Broker Flow

```mermaid
sequenceDiagram
    actor Client as Meteor Client
    participant CTBroker as CloudTasksBroker
    participant GCP_CT_API as GCP Cloud Tasks API
    participant MeteorWorker as Meteor Worker (Axum)
    Client->>CTBroker: publish(queue, msg)
    CTBroker->>GCP_CT_API: POST /v2/projects/.../tasks
    GCP_CT_API->>CTBroker: 200 OK
    GCP_CT_API->>MeteorWorker: POST /meteor/push/queue
    MeteorWorker->>GCP_CT_API: 200 OK (ACK)
```

## API Specification (OpenAPI 3.1)

```yaml
info:
  title: Meteor Push API
  version: 1.0.0
openapi: 3.1.0
paths:
  /meteor/push/{queue}:
    post:
      parameters:
      - in: path
        name: queue
        required: true
        schema:
          type: string
      responses:
        '200':
          description: Task acknowledged
        '500':
          description: Task failed, retry later
```

</spec>
