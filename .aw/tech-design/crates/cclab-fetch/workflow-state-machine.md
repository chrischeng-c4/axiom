---
id: workflow-state-machine
type: spec
title: "Workflow State Machine"
version: 1
spec_type: algorithm
created_at: 2026-01-31T11:39:34.896063+00:00
updated_at: 2026-02-01T15:00:00.000000+00:00
requirements:
  total: 10
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
    - R7
    - R8
    - R9
    - R10
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: true
  diagrams:
    - type: flowchart
      title: "Task State Machine Flow"
history:
  - timestamp: 2026-01-31T11:39:34.896063+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
  - timestamp: 2026-01-31T11:40:01.077852+00:00
    agent: "gemini-3-flash-preview"
    tool: "revise_spec"
    action: "revised"
    duration_secs: 351.51
  - timestamp: 2026-01-31T11:40:16.616724+00:00
    agent: "gpt-5.2-codex"
    tool: "review_spec"
    action: "reviewed"
    duration_secs: 15.54
  - timestamp: 2026-02-01T15:00:00.000000+00:00
    agent: "mcp"
    action: "merged"
---

<spec>

# Workflow State Machine

## Overview

This specification defines the comprehensive state machine for tasks in cclab-meteor, incorporating standard worker execution and offloaded execution (e.g., Kubernetes Jobs). It ensures consistent lifecycle management and reliable workflow continuation across distributed executors, with specific mechanisms for handling remote execution failures and state reconciliation.

## Requirements

### R1 - Core State Definitions

The system must support a defined set of states: PENDING, RECEIVED, STARTED, OFFLOADED, SUCCESS, FAILURE, RETRY, REVOKED, and REJECTED.

### R2 - Workflow State Derivation

Workflow state (Chains, Groups, Chords) must be derived from the terminal states of its constituent tasks. A workflow is only SUCCESS if all its terminal steps are SUCCESS.

### R3 - Atomic Persistence

All state transitions must be atomic and persisted in the ResultBackend to ensure consistency. Use Compare-and-Swap (CAS) if supported by the backend.

### R4 - Retry Lifecycle Support

The state machine must support retries by transitioning from STARTED or OFFLOADED to RETRY, and then back to PENDING.

### R5 - Cross-Executor Reporting

The system must provide an API or mechanism for external executors (like K8s Jobs) to report state transitions directly to the ResultBackend.

### R6 - Transition Validation

The system must enforce valid state transitions and prevent moving out of terminal states (SUCCESS, FAILURE, REVOKED, REJECTED). Any invalid transition attempt must return an error.

### R7 - Offloaded State Support

An OFFLOADED state must be used to represent tasks that are executing in an external environment without blocking a worker slot.

### R8 - Zombie Job Detection

The system must implement a periodic reconciliation loop to detect OFFLOADED tasks whose external jobs have disappeared or timed out, transitioning them to FAILURE.

### R9 - State Transition Semantics

Each state transition must be clearly defined with preconditions (valid source states) and side-effects (e.g. NATS ack, K8s API calls).

### R10 - Offloaded State Metadata

The OFFLOADED state must persist executor-specific metadata (e.g. K8s Job Name, Namespace, Cluster ID) in the ResultBackend to facilitate monitoring and reconciliation.

## Acceptance Criteria

### Scenario: Worker Happy Path

- **GIVEN** A task in PENDING state.
- **WHEN** The task is picked up and executed successfully by a worker.
- **THEN** The task transitions through RECEIVED to STARTED and finally to SUCCESS upon completion.

### Scenario: Offloaded Happy Path

- **GIVEN** A task marked for K8s execution.
- **WHEN** A worker offloads the task to a K8s Job.
- **THEN** The task transitions through RECEIVED and STARTED to OFFLOADED; then to SUCCESS when the K8s Job reports completion.

### Scenario: Retryable Failure

- **GIVEN** A task in STARTED state.
- **WHEN** The task fails with a retryable error.
- **THEN** The task transitions to RETRY and then back to PENDING for the next attempt.

### Scenario: External Revocation

- **GIVEN** A task in OFFLOADED state.
- **WHEN** The task is revoked via the revocation API.
- **THEN** The task transitions to REVOKED and the external job is ideally cleanup/cancelled.

### Scenario: Zombie Job Handling

- **GIVEN** A task in OFFLOADED state.
- **WHEN** The associated K8s Job is deleted externally without reporting a result.
- **THEN** The reconciliation loop detects the missing job and transitions the task to FAILURE with a 'JobLost' error.

### Scenario: Invalid Transition Prevention

- **GIVEN** A task in SUCCESS state.
- **WHEN** An attempt is made to transition the task back to STARTED.
- **THEN** The system returns an error (e.g., InvalidTransition) and the state remains SUCCESS.

## Diagrams

### Task State Machine Flow

```mermaid
flowchart TB
    Pending[PENDING (In Broker)]
    Received[RECEIVED (By Worker)]
    Started[STARTED (Executing)]
    Offloaded[OFFLOADED (In K8s)]
    Success[SUCCESS (Terminal)]
    Failure[FAILURE (Terminal)]
    Retry[RETRY (Waiting)]
    Revoked[REVOKED (Terminal)]
    Rejected[REJECTED (Terminal)]
    Pending -->|Receive message| Received
    Received -->|Start execution| Started
    Received -->|Task not found/Invalid| Rejected
    Started -->|Success| Success
    Started -->|Failure| Failure
    Started -->|Request retry| Retry
    Started -->|Offload to K8s| Offloaded
    Offloaded -->|Job Success| Success
    Offloaded -->|Job Failure| Failure
    Offloaded -->|Job Retry request| Retry
    Offloaded -->|Job timeout/missing| Failure
    Retry -->|Re-publish| Pending
    Pending -->|Revoke| Revoked
    Received -->|Revoke| Revoked
    Started -->|Revoke| Revoked
    Offloaded -->|Revoke/Delete Job| Revoked
```

<semantic-data>

```json
{
  "edges": [
    {
      "from": "Pending",
      "semantic": {
        "condition": "message_received == true"
      },
      "to": "Received"
    },
    {
      "from": "Received",
      "semantic": {
        "condition": "can_start == true"
      },
      "to": "Started"
    },
    {
      "from": "Received",
      "semantic": {
        "condition": "registry.get(name).is_none() == true",
        "is_error_path": true
      },
      "to": "Rejected"
    },
    {
      "from": "Started",
      "semantic": {
        "condition": "outcome == Success"
      },
      "to": "Success"
    },
    {
      "from": "Started",
      "semantic": {
        "condition": "outcome == Failure && !retryable",
        "is_error_path": true
      },
      "to": "Failure"
    },
    {
      "from": "Started",
      "semantic": {
        "condition": "outcome == Retry || (outcome == Failure && retryable)"
      },
      "to": "Retry"
    },
    {
      "from": "Started",
      "semantic": {
        "condition": "executor == 'k8s-job'"
      },
      "to": "Offloaded"
    },
    {
      "from": "Offloaded",
      "semantic": {
        "condition": "job_status == Succeeded"
      },
      "to": "Success"
    },
    {
      "from": "Offloaded",
      "semantic": {
        "condition": "job_status == Failed && !retryable",
        "is_error_path": true
      },
      "to": "Failure"
    },
    {
      "from": "Offloaded",
      "semantic": {
        "condition": "job_status == Failed && retryable"
      },
      "to": "Retry"
    },
    {
      "from": "Offloaded",
      "semantic": {
        "condition": "job_not_found == true || timeout == true",
        "is_error_path": true
      },
      "to": "Failure"
    },
    {
      "from": "Retry",
      "semantic": {
        "code_pattern": "broker.publish(queue, message.with_retry())"
      },
      "to": "Pending"
    },
    {
      "from": "Pending",
      "semantic": {
        "is_error_path": true
      },
      "to": "Revoked"
    },
    {
      "from": "Received",
      "semantic": {
        "is_error_path": true
      },
      "to": "Revoked"
    },
    {
      "from": "Started",
      "semantic": {
        "is_error_path": true
      },
      "to": "Revoked"
    },
    {
      "from": "Offloaded",
      "semantic": {
        "is_error_path": true
      },
      "to": "Revoked"
    }
  ],
  "metadata": null,
  "nodes": [
    {
      "id": "Pending",
      "semantic": {
        "type": "start"
      }
    },
    {
      "id": "Received",
      "semantic": {
        "type": "assign"
      }
    },
    {
      "id": "Started",
      "semantic": {
        "type": "assign"
      }
    },
    {
      "id": "Offloaded",
      "semantic": {
        "type": "assign"
      }
    },
    {
      "id": "Success",
      "semantic": {
        "type": "end"
      }
    },
    {
      "id": "Failure",
      "semantic": {
        "type": "end"
      }
    },
    {
      "id": "Retry",
      "semantic": {
        "type": "assign"
      }
    },
    {
      "id": "Revoked",
      "semantic": {
        "type": "end"
      }
    },
    {
      "id": "Rejected",
      "semantic": {
        "type": "end"
      }
    }
  ]
}
```

</semantic-data>

</spec>
