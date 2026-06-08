---
id: shutdown-management
type: spec
title: "Shutdown Management"
version: 1
spec_type: algorithm
created_at: 2026-01-31T10:50:59.387089+00:00
updated_at: 2026-01-31T10:50:59.387089+00:00
main_spec_ref: "crates/cclab-server/src"
merge_strategy: new
fill_sections: [overview, requirements, scenarios, logic, changes]
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

# Shutdown Management

## Overview
<!-- type: overview lang: markdown -->

Graceful shutdown coordinates the PyLoop event loop, task queue draining, and
background processors such as the TimerWheel. Shutdown sets a stopped flag,
notifies the main loop, drains pending callbacks within a timeout, joins
background tasks, and marks the loop closed.

The old file lived at `.aw/tech-design/crates/cclab-server/shutdown-management.md`.
The canonical TD now lives under `logic/`.

## Requirements
<!-- type: requirements lang: mermaid -->

```mermaid
---
id: shutdown-management-requirements
entry: R1
---
requirementDiagram
    requirement R1 {
        id: R1
        text: shutdown_with_timeout starts graceful shutdown
        risk: high
        verifymethod: test
    }
    requirement R2 {
        id: R2
        text: Pending callbacks drain until empty or timeout
        risk: high
        verifymethod: test
    }
    requirement R3 {
        id: R3
        text: stop and close transitions are thread safe and terminal
        risk: high
        verifymethod: test
    }
    requirement R4 {
        id: R4
        text: TimerWheel task is cancelled and awaited before closed
        risk: medium
        verifymethod: test
    }
```

### R1: Shutdown API

`shutdown_with_timeout(timeout: Duration)` sets the stopped flag and initiates
task draining.

### R2: Graceful Task Draining

`process_tasks_internal` continues processing until the queue is empty during
shutdown, while respecting the configured timeout.

### R3: Lifecycle State Transitions

`stop()` and `close()` must be thread-safe and transition the loop to a terminal
state exactly once.

### R4: TimerWheel Coordination

The TimerWheel background task must be cancelled and awaited before the loop is
marked closed.

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: S1
    requirement: R1
    given: PyLoop is running
    when: shutdown_with_timeout is called
    then: The stopped flag is set and the main loop is notified to exit after the current poll
  - id: S2
    requirement: R2
    given: Shutdown is in progress
    when: The loop enters the draining phase
    then: process_tasks_internal executes until pending callbacks are handled or timeout expires
  - id: S3
    requirement: R4
    given: The TimerWheel background thread is already stopped
    when: TimerWheel join is executed
    then: Shutdown continues without error
  - id: S4
    requirement: R3
    given: PyLoop is already shutting down
    when: shutdown_with_timeout is called again
    then: The second call returns without re-initiating shutdown
```

## Detailed Shutdown Flow
<!-- type: logic lang: mermaid -->

```mermaid
---
id: shutdown-management-detailed-flow
entry: Start
---
flowchart TB
    Start[PyLoop::shutdown_with_timeout] --> SetStopped[Set stopped true]
    SetStopped --> SignalMainLoop[Notify main loop via wakeup_notify]
    SignalMainLoop --> DrainTasks[Execute process_tasks_internal]
    DrainTasks --> DecisionTimeout{Timeout reached?}
    DecisionTimeout -- Yes --> CancelTasks[Drop remaining callbacks]
    DecisionTimeout -- No --> CheckEmpty{Queue empty?}
    CheckEmpty -- No --> DrainTasks
    CheckEmpty -- Yes --> JoinThreads[Join TimerWheel background task]
    CancelTasks --> JoinThreads
    JoinThreads --> CloseResources[Set closed true]
    CloseResources --> End[Loop terminated]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: .aw/tech-design/crates/cclab-server/logic/shutdown-management.md
    action: MODIFY
    impl_mode: hand-written
    desc: Move shutdown management TD under logic and normalize sections.
  - path: crates/cclab-server/src
    action: MODIFY
    impl_mode: hand-written
    desc: Implement graceful shutdown timeout task draining and TimerWheel coordination.
```
