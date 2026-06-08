---
id: orbit-core-optimization
type: spec
title: "Orbit Core Engine Optimization"
version: 1
spec_type: algorithm
created_at: 2026-01-27T16:53:50.808276+00:00
updated_at: 2026-01-27T16:53:50.808276+00:00
requirements:
  total: 5
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Optimized Event Loop Flow"
history:
  - timestamp: 2026-01-27T16:53:50.808276+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Orbit Core Engine Optimization

## Overview

This specification defines the core architectural optimizations for the cclab-orbit event loop. It replaces the legacy busy-waiting and Mutex-heavy implementation with high-performance, async-native structures to achieve sub-microsecond latency and high concurrent throughput.

## Requirements

### R1 - Async-Native Coroutine Polling

```yaml
id: R1
priority: high
status: draft
```

Replace the current std::thread::sleep(10ms) in task.rs with a Waker-driven polling mechanism. Implement a TaskFuture that yields to Tokio and is re-scheduled only when its associated Python waker is triggered.

### R2 - Hashed Hierarchical Timer Wheel

```yaml
id: R2
priority: high
status: draft
```

Replace the Mutex<BTreeMap> in timer_wheel.rs with a hashed hierarchical timer wheel implementation. Use bit-manipulation for bucket identification to ensure O(1) insertion and expiration performance.

### R3 - Lock-Free Task Processing

```yaml
id: R3
priority: high
status: draft
```

Replace the current Mutex-protected UnboundedReceiver with a high-concurrency MPSC queue. Ensure the event loop can drain the queue in adaptive batches to balance latency and throughput.

### R4 - Tokio Notify Integration

```yaml
id: R4
priority: medium
status: draft
```

Replace the synchronous Condvar wakeup mechanism with tokio::sync::Notify. This ensures the event loop integrates perfectly with Tokio's reactor for minimal wakeup latency.

### R5 - Adaptive GIL Batching

```yaml
id: R5
priority: medium
status: draft
```

Optimize the process_tasks_internal function to minimize GIL acquisition frequency by using adaptive batching based on queue depth and processing time.

## Acceptance Criteria

### Scenario: Zero-CPU Suspension

- **GIVEN** A Python coroutine is suspended awaiting an external future.
- **WHEN** The coroutine yields Control back to Orbit.
- **THEN** The coroutine's Rust Task must be idle (not polling) and consume zero CPU cycles until the future completes and triggers the waker.

### Scenario: High-Volume Timer Registration

- **GIVEN** 100,000 timers are scheduled with varied expiration times.
- **WHEN** Multiple threads register timers concurrently.
- **THEN** The system must maintain O(1) performance for insertions and should not experience lock contention stalls between producers and the background timer processor.

### Scenario: Adaptive Batching Efficiency

- **GIVEN** The task queue contains 5,000 pending callbacks.
- **WHEN** The event loop begins a processing iteration.
- **THEN** The event loop should process them in optimally-sized batches (e.g., up to 256 per GIL acquisition) to maximize throughput while preventing starvation of the timer wheel.

## Diagrams

### Optimized Event Loop Flow

```mermaid
flowchart TB
    Start(Event Loop Iteration Start)
    CheckQueue_LF{Check Lock-Free Task Queue} 
    BatchTasks[Extract Batch (Adaptive Size)]
    AcquireGIL[Acquire Python GIL]
    ExecCallbacks[Execute Python Callbacks / Step Tasks]
    ReleaseGIL[Release Python GIL]
    CheckTimers_HWS{Check Hierarchical Timer Wheel} 
    ScheduleTimers[Move Expired Timers to Queue]
    CalculateWait[Calculate Next Timeout]
    WaitNotify_Tokio[/Wait for Notify (tokio::sync::Notify)\]
    Start --> CheckQueue_LF
    CheckQueue_LF -->|Has Tasks| BatchTasks
    BatchTasks --> AcquireGIL
    AcquireGIL --> ExecCallbacks
    ExecCallbacks --> ReleaseGIL
    ReleaseGIL --> CheckTimers_HWS
    CheckQueue_LF -->|Empty| CheckTimers_HWS
    CheckTimers_HWS -->|Expired Timers| ScheduleTimers
    ScheduleTimers --> BatchTasks
    CheckTimers_HWS -->|No Timers| CalculateWait
    CalculateWait --> WaitNotify_Tokio
    WaitNotify_Tokio -->|Woken Up| CheckQueue_LF
```

</spec>
