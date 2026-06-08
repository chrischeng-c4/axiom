---
id: mpsc-task-queue
type: spec
title: "Lock-free MPSC Task Queue"
version: 1
spec_type: algorithm
created_at: 2026-02-05T04:29:08.666573+00:00
updated_at: 2026-02-05T04:29:08.666573+00:00
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
      title: "MPSC Queue Architecture"
history:
  - timestamp: 2026-02-05T04:29:08.666573+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Lock-free MPSC Task Queue

## Overview

Replace the mutex-protected task queue with a lock-free MPSC (multi-producer, single-consumer) queue using crossbeam channels. This eliminates lock contention under high task creation rates and improves scalability when multiple threads submit tasks concurrently. The implementation maintains ordering guarantees while reducing latency for task scheduling.

## Requirements

### R1 - Lock-free queue implementation

```yaml
id: R1
priority: high
status: draft
```

Use crossbeam::channel::unbounded() or equivalent lock-free MPSC queue to replace Mutex<UnboundedReceiver>.

### R2 - Ordering guarantees

```yaml
id: R2
priority: high
status: draft
```

Maintain FIFO ordering for tasks submitted from the same thread. Cross-thread ordering follows happens-before semantics.

### R3 - High concurrency support

```yaml
id: R3
priority: high
status: draft
```

Support 100k+ concurrent task submissions without significant contention or performance degradation.

### R4 - Batch receive

```yaml
id: R4
priority: medium
status: draft
```

Support efficient batch receiving of multiple tasks in a single operation to reduce per-task overhead.

### R5 - Backpressure handling

```yaml
id: R5
priority: low
status: draft
```

Provide optional bounded queue mode with backpressure signaling for memory-constrained environments.

## Acceptance Criteria

### Scenario: Single producer fast path

- **GIVEN** A single thread submitting tasks
- **WHEN** Tasks are submitted rapidly
- **THEN** No lock contention occurs and tasks are queued in order

### Scenario: Multi-producer scaling

- **GIVEN** 10 threads each submitting 10k tasks
- **WHEN** All threads submit concurrently
- **THEN** Total throughput scales linearly with producer count

### Scenario: Batch receive efficiency

- **GIVEN** 100 tasks pending in queue
- **WHEN** Consumer calls batch_receive(50)
- **THEN** 50 tasks are returned in a single operation without per-task overhead

### Scenario: Empty queue handling

- **GIVEN** An empty task queue
- **WHEN** Consumer attempts to receive
- **THEN** Returns immediately with empty result, no blocking

### Scenario: Stress test

- **GIVEN** 100k tasks submitted from 100 threads
- **WHEN** All tasks are processed
- **THEN** No tasks are lost and ordering within each thread is preserved

## Diagrams

### MPSC Queue Architecture

```mermaid
flowchart LR
    p1(Producer 1)
    p2(Producer 2)
    pn(Producer N)
    queue[(Lock-free MPSC Queue)]
    consumer(Event Loop Consumer)
    batch[Batch Process]
    p1 -->|send()| queue
    p2 -->|send()| queue
    pn -->|send()| queue
    queue -->|try_recv()| consumer
    consumer -->|batch| batch
```

</spec>
