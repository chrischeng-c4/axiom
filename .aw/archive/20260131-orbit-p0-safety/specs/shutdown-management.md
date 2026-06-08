---
id: shutdown-management
type: spec
title: "Shutdown Management"
version: 1
spec_type: algorithm
created_at: 2026-01-31T10:50:59.387089+00:00
updated_at: 2026-01-31T10:50:59.387089+00:00
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
      title: "Detailed Shutdown Flow"
history:
  - timestamp: 2026-01-31T10:50:59.387089+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Shutdown Management

## Overview

This specification defines the graceful shutdown mechanism for the PyLoop event loop. It coordinates the stopping of the main loop, draining of the task queue, and cleanup of background processors like the TimerWheel.

## Requirements

### R1 - Shutdown API

```yaml
id: R1
priority: medium
status: draft
```

Implement shutdown_with_timeout(timeout: Duration) which sets the 'stopped' flag and initiates task draining.

### R2 - Graceful Task Draining

```yaml
id: R2
priority: medium
status: draft
```

Modify process_tasks_internal to continue processing until the queue is empty during the shutdown phase, respecting the timeout.

### R3 - Lifecycle State Transitions

```yaml
id: R3
priority: medium
status: draft
```

Update the stop() and close() methods to ensure they are thread-safe and correctly transition the loop to a terminal state.

### R4 - TimerWheel Coordination

```yaml
id: R4
priority: medium
status: draft
```

Ensure the TimerWheel background task is cancelled and awaited before the loop is marked as 'closed'.

## Acceptance Criteria

### Scenario: Initiate Shutdown

- **GIVEN** PyLoop is running.
- **WHEN** shutdown_with_timeout is called.
- **THEN** The 'stopped' flag is set and the main loop is notified to exit after current poll.

### Scenario: Drain Pending Tasks

- **GIVEN** Shutdown is in progress.
- **WHEN** The loop enters the draining phase.
- **THEN** process_tasks_internal executes until all pending callbacks are handled or timeout expires.

### Scenario: TimerWheel Already Stopped

- **GIVEN** TimerWheel background thread is already stopped.
- **WHEN** JoinTimerWheel is executed.
- **THEN** The shutdown process continues without error.

### Scenario: Idempotent Shutdown

- **GIVEN** PyLoop is already shutting down.
- **WHEN** shutdown_with_timeout is called again.
- **THEN** The second call returns immediately without re-initiating the sequence.

## Diagrams

### Detailed Shutdown Flow

```mermaid
flowchart TB
    Start(PyLoop::shutdown_with_timeout())
    SetStopped[Set stopped = true]
    SignalMainLoop[Notify main loop via wakeup_notify]
    DrainTasks[Execute process_tasks_internal()]
    DecisionTimeout{Timeout?} 
    CancelTasks[Drop remaining callbacks]
    CheckEmpty[Queue empty?]
    DecisionDone{All tasks complete?} 
    JoinThreads[Join TimerWheel background task]
    CloseResources[Set closed = true]
    End(Loop Terminated)
    Start --> SetStopped
    SetStopped --> SignalMainLoop
    SignalMainLoop --> DrainTasks
    DrainTasks -->|Timeout reached?| DecisionTimeout
    DecisionTimeout -->|Yes| CancelTasks
    DecisionTimeout -->|No| CheckEmpty
    CheckEmpty -->|All Done?| DecisionDone
    DecisionDone -->|Yes| JoinThreads
    DecisionDone -->|No| DrainTasks
    CancelTasks --> JoinThreads
    JoinThreads --> CloseResources
    CloseResources --> End
```

</spec>
