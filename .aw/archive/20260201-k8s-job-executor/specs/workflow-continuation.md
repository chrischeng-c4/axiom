---
id: workflow-continuation
type: spec
title: "Workflow Continuation"
version: 1
spec_type: algorithm
created_at: 2026-01-31T11:31:39.959610+00:00
updated_at: 2026-01-31T11:31:39.959610+00:00
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
  has_semantic_diagrams: true
  diagrams:
    - type: flowchart
      title: "Workflow Continuation Logic Flow"
history:
  - timestamp: 2026-01-31T11:31:39.959610+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Workflow Continuation

## Overview

This specification defines the logic for continuing task workflows (chains) after a task completes. This logic must be centralized so it can be invoked by both standard workers and standalone task runners (like K8s Jobs). It ensures that sequential tasks are triggered with the correct results passed along.

## Requirements

### R1 - Workflow Engine Component

```yaml
id: R1
priority: high
status: draft
```

Implement a WorkflowEngine or similar component that handles advancing chains based on task results.

### R2 - Chain Metadata Loading

```yaml
id: R2
priority: medium
status: draft
```

The engine must load ChainMeta from the backend using the task's root_id.

### R3 - Chain Advancement and Dispatch

```yaml
id: R3
priority: high
status: draft
```

Upon task success, the engine must advance the chain, prepend the current result to the next task's arguments, and publish the next task to NATS.

### R4 - Cross-executor Compatibility

```yaml
id: R4
priority: medium
status: draft
```

The engine must be usable by TaskExecutor and the new run-once CLI command.

## Acceptance Criteria

### Scenario: Continue chain upon success

- **GIVEN** a completed task that is part of a chain.
- **WHEN** the WorkflowEngine is triggered with TaskSuccess.
- **THEN** the next task in the chain is published to NATS with the previous task's result as an argument.

### Scenario: Complete chain

- **GIVEN** the last task in a chain.
- **WHEN** the last task succeeds.
- **THEN** the chain is marked as complete and no further tasks are published.

## Diagrams

### Workflow Continuation Logic Flow

```mermaid
flowchart TB
    Start(Task Success(result, root_id))
    LoadMeta[Load ChainMeta from Backend]
    AdvanceChain[Advance index & Append result]
    SaveMeta[Save Updated ChainMeta]
    CheckNextTask{Is there a next task?} 
    PublishNextTask[Publish Next Task to NATS]
    End(End Workflow Continuation)
    Start --> LoadMeta
    LoadMeta --> AdvanceChain
    LoadMeta --> End
    AdvanceChain --> SaveMeta
    SaveMeta --> CheckNextTask
    CheckNextTask -->|Has Next| PublishNextTask
    CheckNextTask -->|Done| End
    PublishNextTask --> End
```

<semantic-data>

```json
{
  "edges": [
    {
      "from": "LoadMeta",
      "semantic": {
        "condition": "meta.is_some()"
      },
      "to": "AdvanceChain"
    },
    {
      "from": "LoadMeta",
      "semantic": {
        "condition": "meta.is_none()",
        "is_error_path": true
      },
      "to": "End"
    },
    {
      "from": "CheckNextTask",
      "semantic": {
        "condition": "next_task.is_some()"
      },
      "to": "PublishNextTask"
    },
    {
      "from": "CheckNextTask",
      "semantic": {
        "condition": "next_task.is_none()"
      },
      "to": "End"
    }
  ],
  "metadata": null,
  "nodes": [
    {
      "id": "Start",
      "semantic": {
        "type": "start"
      }
    },
    {
      "id": "LoadMeta",
      "semantic": {
        "operation": "SELECT",
        "table": "metadata",
        "type": "db_query"
      }
    },
    {
      "id": "AdvanceChain",
      "semantic": {
        "type": "transform"
      }
    },
    {
      "id": "SaveMeta",
      "semantic": {
        "operation": "UPDATE",
        "table": "metadata",
        "type": "db_mutation"
      }
    },
    {
      "id": "CheckNextTask",
      "semantic": {
        "type": "condition"
      }
    },
    {
      "id": "PublishNextTask",
      "semantic": {
        "type": "api_call",
        "url": "nats://publish"
      }
    },
    {
      "id": "End",
      "semantic": {
        "type": "end"
      }
    }
  ]
}
```

</semantic-data>

</spec>
