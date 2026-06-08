---
id: mamba-async-runtime
type: spec
title: "Async/Await and Coroutine Scheduling (#313)"
version: 1
spec_type: integration
tags: [external]
created_at: 2026-02-14T09:32:18.512348+00:00
updated_at: 2026-02-14T09:32:18.512348+00:00
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
    - type: sequence
      title: "Async Coroutine Execution Flow"
history:
  - timestamp: 2026-02-14T09:32:18.512348+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Async/Await and Coroutine Scheduling (#313)

## Overview

This specification defines the Mamba async/await runtime integration with the cclab-orbit bridge. It details how Mamba coroutines (state machines) are scheduled on the Orbit event loop, how GIL release is handled during awaits, and the interaction between the Mamba runtime and the underlying Tokio executor.

## Requirements

### R1 - Async Function Compilation

```yaml
id: R1
priority: high
status: draft
```

Compile async functions into state machine objects capable of suspension and resumption.

### R2 - Orbit Loop Integration

```yaml
id: R2
priority: high
status: draft
```

Integrate with the Orbit Event Loop to schedule coroutines and register wakers for I/O and timers.

### R3 - GIL-safe Scheduling

```yaml
id: R3
priority: high
status: draft
```

Ensure the GIL is released during coroutine suspension and re-acquired upon resumption when executing Mamba/Python code.

### R4 - Future Interoperability

```yaml
id: R4
priority: medium
status: draft
```

Provide a mechanism to await external futures (e.g., Tokio futures) within Mamba coroutines.

## Acceptance Criteria

### Scenario: Suspend and Resume Coroutine

- **GIVEN** An async function 'async def f(): await asyncio.sleep(1)'.
- **WHEN** The function is called and awaited.
- **THEN** The coroutine should suspend, allowing the loop to run other tasks, and resume after 1 second.

### Scenario: Concurrent Task Execution

- **GIVEN** Multiple async tasks spawned via asyncio.gather.
- **WHEN** The tasks are scheduled on the Orbit loop.
- **THEN** Tasks should execute concurrently on the underlying Tokio runtime.

## Diagrams

### Async Coroutine Execution Flow

```mermaid
sequenceDiagram
    participant PythonCoro as Mamba Coroutine
    participant MambaRuntime as Mamba Runtime (Rust)
    participant OrbitLoop as Orbit Event Loop
    participant TokioRuntime as Tokio Executor
    PythonCoro->>MambaRuntime: await foo()
    MambaRuntime->>OrbitLoop: mb_coroutine_suspend()
    OrbitLoop->>TokioRuntime: tokio::spawn(coro)
    TokioRuntime->>OrbitLoop: Ready(result)
    OrbitLoop->>MambaRuntime: PythonWaker::wake()
    MambaRuntime->>PythonCoro: mb_coroutine_resume(result)
```

</spec>
