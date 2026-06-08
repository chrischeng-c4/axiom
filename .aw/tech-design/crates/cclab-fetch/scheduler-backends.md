---
id: scheduler-backends
type: spec
title: "Meteor Scheduler Backends"
version: 1
spec_type: integration
spec_group: cclab-meteor
created_at: 2026-02-03T08:35:15.650162+00:00
updated_at: 2026-02-03T08:35:15.650162+00:00
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
  has_api_spec: false
  has_semantic_diagrams: true
  diagrams:
    - type: sequence
      title: "Scheduler and Backend Interaction"
history:
  - timestamp: 2026-02-03T08:35:15.650162+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Meteor Scheduler Backends

## Overview

This specification defines the interfaces for the Meteor scheduler backends and the implementation for the Ion backend. It allows for different storage and locking mechanisms while maintaining a consistent interface for the periodic scheduler.

## Requirements

### R1 - SchedulerBackend Trait

```yaml
id: R1
priority: medium
status: draft
```

Define a trait to abstract away the leader election and state persistence.

### R2 - Ion Backend Implementation

```yaml
id: R2
priority: medium
status: draft
```

Provide a production-ready implementation using cclab-ion for distributed locking.

### R3 - InMemory Backend for Testing

```yaml
id: R3
priority: medium
status: draft
```

Include an in-memory backend for unit testing without external dependencies.

## Acceptance Criteria

### Scenario: Ion Leader Acquisition

- **GIVEN** An IonSchedulerBackend.
- **WHEN** acquire_leader() is called.
- **THEN** The backend uses the Ion lock command to attempt acquisition.

### Scenario: Task State Persistence

- **GIVEN** A scheduler instance using Ion backend.
- **WHEN** set_task_state() is called for a specific task.
- **THEN** The state is stored in Ion and survives instance restarts.

## Diagrams

### Scheduler and Backend Interaction

```mermaid
sequenceDiagram
    participant Scheduler as PeriodicScheduler
    participant Backend as SchedulerBackend
    Scheduler->>Backend: acquire_leader(ttl)
    Backend->>Scheduler: Result<bool>
    Scheduler->>Backend: get_task_state(name)
    Backend->>Scheduler: Result<TaskScheduleState>
```

<semantic-data>

```json
{
  "messages": [
    {
      "from": "Scheduler",
      "text": "acquire_leader(ttl)",
      "to": "Backend"
    },
    {
      "from": "Backend",
      "text": "Result<bool>",
      "to": "Scheduler"
    },
    {
      "from": "Scheduler",
      "text": "get_task_state(name)",
      "to": "Backend"
    },
    {
      "from": "Backend",
      "text": "Result<TaskScheduleState>",
      "to": "Scheduler"
    }
  ],
  "participants": [
    {
      "id": "Scheduler",
      "label": "PeriodicScheduler",
      "type": "participant"
    },
    {
      "id": "Backend",
      "label": "SchedulerBackend",
      "type": "participant"
    }
  ]
}
```

</semantic-data>

</spec>
