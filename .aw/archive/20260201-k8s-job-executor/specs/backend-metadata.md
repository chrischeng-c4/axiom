---
id: backend-metadata
type: spec
title: "Result Backend Metadata"
version: 1
spec_type: utility
created_at: 2026-01-31T11:31:24.461821+00:00
updated_at: 2026-01-31T11:31:24.461821+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Backend Metadata Data Flow"
history:
  - timestamp: 2026-01-31T11:31:24.461821+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Result Backend Metadata

## Overview

This specification defines the extension of the ResultBackend trait to support generic metadata storage. This is necessary for tracking workflow state (like chains and chords) across different task executions, especially when some tasks are executed by external systems like Kubernetes Jobs. The metadata API provides a way to store and retrieve state information associated with a root task ID.

## Requirements

### R1 - Backend Metadata Trait Methods

```yaml
id: R1
priority: high
status: draft
```

Add set_metadata(key, value, ttl) and get_metadata(key) to the ResultBackend trait.

### R2 - Redis Metadata Implementation

```yaml
id: R2
priority: medium
status: draft
```

Implement metadata support in RedisBackend using Redis strings with TTL support.

### R3 - Ion Metadata Implementation

```yaml
id: R3
priority: medium
status: draft
```

Implement metadata support in IonBackend using its internal KV store and TTL logic.

### R4 - Metadata Expiration Support

```yaml
id: R4
priority: high
status: draft
```

Metadata must support expiration (TTL) to prevent stale state from accumulating in the backends.

## Acceptance Criteria

### Scenario: Store and Retrieve Metadata

- **GIVEN** A connected result backend (Redis or Ion).
- **WHEN** A metadata key-value pair is set.
- **THEN** The value is correctly stored and can be retrieved using the same key.

### Scenario: Metadata Expiration

- **GIVEN** A metadata entry with a short TTL.
- **WHEN** The TTL elapsed.
- **THEN** The metadata is no longer available after the TTL expires.

## Diagrams

### Backend Metadata Data Flow

```mermaid
flowchart LR
    Client(Caller (Worker/WorkflowEngine))
    ResultBackend{{ResultBackend Trait}}
    RedisImplementation[RedisBackend Impl]
    IonImplementation[IonBackend Impl]
    RedisStorage[(Redis Server)]
    IonStorage[(Ion Storage Backend)]
    Client -->|set_metadata(key, value, ttl)| ResultBackend
    Client -->|get_metadata(key)| ResultBackend
    ResultBackend --> RedisImplementation
    ResultBackend --> IonImplementation
    RedisImplementation -->|SETEX key ttl value| RedisStorage
    IonImplementation -->|put(key, value) + ttl check| IonStorage
```

</spec>
