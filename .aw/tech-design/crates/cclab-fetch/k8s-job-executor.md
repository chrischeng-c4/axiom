---
id: k8s-job-executor
type: spec
title: "K8s Job Executor"
version: 1
spec_type: algorithm
created_at: 2026-01-31T11:32:35.897330+00:00
updated_at: 2026-02-01T15:00:00.000000+00:00
requirements:
  total: 6
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: true
  diagrams:
    - type: flowchart
      title: "K8s Job Execution Flow"
history:
  - timestamp: 2026-01-31T11:32:35.897330+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
  - timestamp: 2026-02-01T15:00:00.000000+00:00
    agent: "mcp"
    action: "merged"
---

<spec>

# K8s Job Executor

## Overview

This specification defines the implementation of a Kubernetes Job Executor for cclab-meteor. It enables workers to delegate resource-intensive tasks to isolated Kubernetes Jobs, supporting advanced resource requirements like GPUs and TPUs. The executor handles the non-blocking hand-off of tasks from the worker to the Kubernetes API and ensures the job correctly reports its results back to the system.

## Requirements

### R1 - Executor Marker Support

The Task trait and TaskMessage must support an executor field to identify tasks requiring Kubernetes execution.

### R2 - Kube-rs Integration

The K8sJobExecutor must use kube-rs to communicate with the Kubernetes API.

### R3 - Rich Resource Configuration

The executor must support configurable resource limits (CPU, memory), requests (GPU, TPU), nodeSelector, and tolerations per task.

### R4 - Non-blocking Spawning

Spawning a K8s Job must be non-blocking for the worker; the worker should ack the task message once the Job is successfully created.

### R5 - Single-task Container Execution

The K8s Job container must be able to run a single task using a run-once command, reporting results to the Ion result backend.

### R6 - Workflow Continuation Support

The K8s Job must trigger chain continuation via NATS if the task is part of a larger workflow.

## Acceptance Criteria

### Scenario: Successfully spawn a GPU task

- **GIVEN** a task marked with executor='k8s-job' and GPU requirements.
- **WHEN** the worker receives the task.
- **THEN** a K8s Job is created with the specified GPU resources and the worker acknowledges the original message.

### Scenario: K8s Job execution and result reporting

- **GIVEN** a running K8s Job spawned by the executor.
- **WHEN** the task completes successfully inside the job pod.
- **THEN** the result is written to the Ion backend and the task state is updated to SUCCESS.

### Scenario: Chain continuation from K8s Job

- **GIVEN** a task that is the first part of a chain and is executed as a K8s Job.
- **WHEN** the task inside the K8s Job finishes.
- **THEN** the K8s Job triggers the next task in the chain by publishing to NATS before exiting.

## Diagrams

### K8s Job Execution Flow

```mermaid
flowchart TB
    subgraph WorkerSide["Worker (Non-blocking hand-off)"]
        ReceiveTask(Worker Receives Task Message)
        CreateK8sJob[Create Kubernetes Job Pod]
        UpdateStateOffloaded[Set Task State to OFFLOADED]
        AckMessage[Acknowledge Message to Broker]
        EndWorkerPart(Worker slot released)
        RaiseError{{Error creating K8s Job}}
    end
    subgraph K8sSide["K8s Job Pod (Execution)"]
        K8sJobStart(K8s Pod Starts Execution)
        RunOnceTask[Execute Task (run-once)]
        ReportResult[Write Result to Backend]
        TriggerContinuation[Trigger Workflow Continuation]
        EndJobPart(K8s Job Pod Exits)
    end
    ReceiveTask --> CreateK8sJob
    CreateK8sJob -->|Success| UpdateStateOffloaded
    CreateK8sJob -->|Failure| RaiseError
    UpdateStateOffloaded --> AckMessage
    AckMessage --> EndWorkerPart
    K8sJobStart --> RunOnceTask
    RunOnceTask --> ReportResult
    ReportResult --> TriggerContinuation
    TriggerContinuation --> EndJobPart
```

<semantic-data>

```json
{
  "edges": [
    {
      "from": "CreateK8sJob",
      "semantic": {
        "condition": "job_created == true"
      },
      "to": "UpdateStateOffloaded"
    },
    {
      "from": "CreateK8sJob",
      "semantic": {
        "condition": "job_created == false",
        "is_error_path": true
      },
      "to": "RaiseError"
    }
  ],
  "metadata": null,
  "nodes": [
    {
      "id": "ReceiveTask",
      "semantic": {
        "type": "start"
      }
    },
    {
      "id": "CreateK8sJob",
      "semantic": {
        "type": "api_call",
        "url": "kubernetes/api/v1/jobs"
      }
    },
    {
      "id": "UpdateStateOffloaded",
      "semantic": {
        "operation": "UPDATE",
        "table": "task_state",
        "type": "db_mutation"
      }
    },
    {
      "id": "AckMessage",
      "semantic": {
        "type": "api_call",
        "url": "nats/ack"
      }
    },
    {
      "id": "EndWorkerPart",
      "semantic": {
        "type": "end"
      }
    },
    {
      "id": "K8sJobStart",
      "semantic": {
        "type": "start"
      }
    },
    {
      "id": "RunOnceTask",
      "semantic": {
        "type": "transform"
      }
    },
    {
      "id": "ReportResult",
      "semantic": {
        "operation": "INSERT",
        "table": "task_results",
        "type": "db_mutation"
      }
    },
    {
      "id": "TriggerContinuation",
      "semantic": {
        "type": "api_call",
        "url": "workflow_engine/trigger"
      }
    },
    {
      "id": "EndJobPart",
      "semantic": {
        "type": "end"
      }
    },
    {
      "id": "RaiseError",
      "semantic": {
        "error": {
          "code": 500,
          "message": "Failed to create K8s Job: {{error}}"
        },
        "type": "raise_error"
      }
    }
  ]
}
```

</semantic-data>

</spec>
