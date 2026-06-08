---
id: scheduler-architecture
type: spec
title: "Meteor Scheduler Architecture"
version: 1
spec_type: algorithm
spec_group: cclab-meteor
created_at: 2026-02-03T08:34:55.688677+00:00
updated_at: 2026-02-03T08:34:55.688677+00:00
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
    - type: flowchart
      title: "Scheduler Leader Election and Evaluation Flow"
history:
  - timestamp: 2026-02-03T08:34:55.688677+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Meteor Scheduler Architecture

## Overview

This specification defines the core architecture for the Meteor periodic scheduler, focusing on distributed leader election and schedule evaluation logic. It ensures that in a distributed environment, only one scheduler instance acts as the leader to prevent duplicate task execution.

## Requirements

### R1 - Distributed Leader Election

```yaml
id: R1
priority: medium
status: draft
```

Support distributed leader election using Ion locks to ensure only one active scheduler.

### R2 - Flexible Schedule Triggers

```yaml
id: R2
priority: medium
status: draft
```

Support both Crontab and simple Interval based scheduling.

### R3 - Pluggable Backend Architecture

```yaml
id: R3
priority: medium
status: draft
```

Decouple schedule evaluation from the backend storage.

## Acceptance Criteria

### Scenario: Leader Election Success

- **GIVEN** Two scheduler instances S1 and S2.
- **WHEN** S1 acquires the leader lock 'meteor:leader' first.
- **THEN** S1 becomes the leader, S2 becomes a follower.

### Scenario: Task Enqueue When Due

- **GIVEN** A task scheduled every 5 minutes.
- **WHEN** The current time matches the cron expression and the task is active.
- **THEN** The task is enqueued to the broker.

### Scenario: Skip Paused Tasks

- **GIVEN** A task marked as 'paused' in the backend.
- **WHEN** The scheduler evaluates the due tasks.
- **THEN** The task is NOT enqueued even if it is due.

## Diagrams

### Scheduler Leader Election and Evaluation Flow

```mermaid
flowchart TB
    ElectionLoop((Election Loop Start))
    AcquireLeader[Acquire Leader Lock]
    EvaluateSchedules[Evaluate Due Schedules]
    Sleep5s([Sleep 5s (Follower)])
    CheckTaskState[Check Task Paused State]
    EnqueueTask[Enqueue Task to Broker]
    UpdateLastRunTime[Update Last Run Time]
    RenewLeader[Renew Leader Lock]
    ElectionLoop --> AcquireLeader
    AcquireLeader -->|Leader acquired| EvaluateSchedules
    AcquireLeader -->|Follower| Sleep5s
    EvaluateSchedules --> CheckTaskState
    CheckTaskState -->|Due & Active| EnqueueTask
    EnqueueTask --> UpdateLastRunTime
    UpdateLastRunTime --> RenewLeader
    RenewLeader --> ElectionLoop
    Sleep5s --> ElectionLoop
```

<semantic-data>

```json
{
  "edges": [
    {
      "from": "ElectionLoop",
      "to": "AcquireLeader"
    },
    {
      "from": "AcquireLeader",
      "label": "Leader acquired",
      "to": "EvaluateSchedules"
    },
    {
      "from": "AcquireLeader",
      "label": "Follower",
      "to": "Sleep5s"
    },
    {
      "from": "EvaluateSchedules",
      "to": "CheckTaskState"
    },
    {
      "from": "CheckTaskState",
      "label": "Due & Active",
      "to": "EnqueueTask"
    },
    {
      "from": "EnqueueTask",
      "to": "UpdateLastRunTime"
    },
    {
      "from": "UpdateLastRunTime",
      "to": "RenewLeader"
    },
    {
      "from": "RenewLeader",
      "to": "ElectionLoop"
    },
    {
      "from": "Sleep5s",
      "to": "ElectionLoop"
    }
  ],
  "nodes": [
    {
      "id": "ElectionLoop",
      "semantic": {
        "type": "start"
      }
    },
    {
      "id": "AcquireLeader",
      "semantic": {
        "operation": "UPSERT",
        "table": "meteor:leader",
        "type": "db_mutation"
      }
    },
    {
      "id": "EvaluateSchedules",
      "semantic": {
        "type": "transform"
      }
    },
    {
      "id": "Sleep5s",
      "semantic": {
        "type": "transform"
      }
    },
    {
      "id": "CheckTaskState",
      "semantic": {
        "operation": "SELECT",
        "table": "meteor:schedule:state:*",
        "type": "db_query"
      }
    },
    {
      "id": "EnqueueTask",
      "semantic": {
        "type": "api_call",
        "url": "broker://enqueue"
      }
    },
    {
      "id": "UpdateLastRunTime",
      "semantic": {
        "operation": "UPDATE",
        "table": "meteor:schedule:state:*",
        "type": "db_mutation"
      }
    },
    {
      "id": "RenewLeader",
      "semantic": {
        "operation": "UPDATE",
        "table": "meteor:leader",
        "type": "db_mutation"
      }
    }
  ]
}
```

</semantic-data>

</spec>
