---
id: vortex-core-architecture
type: spec
title: "Vortex Core Architecture & Lifecycle"
version: 1
spec_type: algorithm
tags: [logic]
merge_strategy: replace
created_at: 2026-02-14T06:56:05.378532+00:00
updated_at: 2026-02-14T06:56:05.378532+00:00
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
      title: "Hybrid Fixed-Step Core Loop"
    - type: sequence
      title: "Core Loop Thread and Lifecycle Sequence"
history:
  - timestamp: 2026-02-14T06:56:05.378532+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Vortex Core Architecture & Lifecycle

## Overview

Define the executable contract for the Vortex runtime core: lifecycle interface (`init`, `run`, `shutdown`), deterministic simulation loop, thread ownership, Python/GIL interaction boundaries, and ordered interactions between simulation and rendering subsystems.

## Requirements

### R1 - VortexEngine Lifecycle Interface Contract

```yaml
id: R1
priority: high
status: draft
```

`VortexEngine` must expose three lifecycle operations with explicit state transitions and error boundaries: `init(config) -> Result<EngineHandle, InitError>`, `run() -> Result<RunSummary, RunError>`, and `shutdown(reason) -> Result<ShutdownReport, ShutdownError>`. `init` may be called exactly once from `Created` state, `run` only from `Initialized`, and `shutdown` must be idempotent from `Running|Initialized|Faulted`. Repeated or invalid calls must return typed errors without undefined behavior.

### R2 - Threading Model and GIL Boundary

```yaml
id: R2
priority: high
status: draft
```

The runtime must separate orchestration and simulation duties across threads: a main orchestration thread owns lifecycle and frame pacing; one or more background workers execute physics/ECS jobs; render submission stays on a render-capable thread. Python-backed logic must execute only inside explicit GIL acquisition windows, while the engine keeps long-running native simulation and rendering paths outside GIL-held regions. Cross-thread coordination must use lock-free queues or bounded channels with backpressure and clear ownership transfer of frame snapshots.

### R3 - Frame-Rate Independence with Hybrid Time Step

```yaml
id: R3
priority: high
status: draft
```

Simulation must use a fixed time step (`fixed_dt`, e.g. 1/120s) via an accumulator, while rendering uses variable frame delta (`frame_dt`) and interpolation factor `alpha = accumulator / fixed_dt`. Each frame: accumulate `frame_dt`, execute `N = floor(accumulator/fixed_dt)` simulation ticks, subtract `N * fixed_dt`, then render once with interpolation. The loop must enforce a configurable `max_substeps` clamp per frame to prevent spiral-of-death under load.

### R4 - Deterministic Tick Ordering and Snapshot Handoff

```yaml
id: R4
priority: medium
status: draft
```

Within each fixed simulation tick, execution order must be deterministic: input sampling -> ECS systems -> agent/behavior evaluation -> event publication -> state snapshot. Render consumes immutable snapshots produced by simulation workers and never mutates simulation state. Snapshot handoff must avoid blocking the simulation thread longer than one fixed tick budget.

### R5 - Graceful Shutdown and Worker Drain

```yaml
id: R5
priority: high
status: draft
```

`shutdown` must stop new frame scheduling, signal worker termination, drain in-flight simulation tasks up to a timeout, flush render queues, and release Python/runtime resources in dependency order. If drain timeout expires, shutdown must return a structured partial-cleanup report indicating unfinished workers and reclaimed resources.

## Acceptance Criteria

### Scenario: Initialize Engine and Enter Runnable State

- **GIVEN** The process is in `Created` state and valid runtime config is provided.
- **WHEN** `VortexEngine.init(config)` is invoked.
- **THEN** The engine validates config, allocates core services, starts background workers, transitions to `Initialized`, and returns an engine handle.

### Scenario: Run Loop Executes Fixed Simulation with Variable Rendering

- **GIVEN** The engine is `Initialized` and `fixed_dt`/`max_substeps` are configured.
- **WHEN** `VortexEngine.run()` processes a frame where `accumulator >= fixed_dt`.
- **THEN** The loop executes one or more fixed simulation ticks (up to `max_substeps`), updates accumulator, computes interpolation alpha, and renders exactly one frame using the latest immutable snapshot.

### Scenario: Python Logic Runs Only in Explicit GIL Windows

- **GIVEN** A simulation tick includes Python-backed agent logic.
- **WHEN** A worker reaches the Python execution stage.
- **THEN** The worker acquires GIL only around Python calls, releases it immediately after, and completes remaining native stages without holding the GIL.

### Scenario: Load Spike Triggers Substep Clamp

- **GIVEN** A frame delta large enough to imply more than `max_substeps` simulation ticks.
- **WHEN** The frame is processed by the accumulator loop.
- **THEN** Only `max_substeps` ticks are executed, excess time is capped or deferred per policy, and the engine records an overload metric instead of unbounded catch-up.

### Scenario: Idempotent Shutdown Drains Workers

- **GIVEN** The engine is `Running` with in-flight simulation jobs.
- **WHEN** `VortexEngine.shutdown(reason)` is called twice.
- **THEN** The first call performs stop/drain/flush/release and returns a shutdown report; the second call returns success with no duplicate teardown side effects.

## Diagrams

### Hybrid Fixed-Step Core Loop

```mermaid
flowchart TB
    start([Start run()])
    poll[Read monotonic clock + input]
    acc[accumulator += frame_dt]
    check{accumulator >= fixed_dt ?} 
    tick[Run simulation tick on worker(s)]
    sub[accumulator -= fixed_dt]
    clamp{substeps > max_substeps ?} 
    metric[Record overload + clamp catch-up]
    alpha[alpha = accumulator / fixed_dt]
    render[Render interpolated snapshot]
    running{running ?} 
    shutdown[Drain workers + flush + release]
    end([Exit run()])
    start --> poll
    poll --> acc
    acc --> check
    check -->|yes| tick
    tick --> sub
    sub --> clamp
    clamp -->|yes| metric
    clamp -->|no| check
    metric --> alpha
    check -->|no| alpha
    alpha --> render
    render --> running
    running -->|yes| poll
    running -->|no| shutdown
    shutdown --> end
```

### Core Loop Thread and Lifecycle Sequence

```mermaid
sequenceDiagram
    actor app as Host App
    participant engine as VortexEngine
    participant main as Main Loop Thread
    participant worker as Simulation Worker
    participant py as Python Runtime (GIL)
    participant render as Render Thread
    app->>engine: init(config)
    engine->>worker: spawn workers + init channels
    engine->>render: initialize renderer resources
    app->>engine: run()
    engine->>main: enter loop
    main-->>main: frame_dt = now - prev; accumulator += frame_dt
    main->>+worker: dispatch fixed tick(s) while accumulator >= fixed_dt
    worker->>py: acquire GIL for Python stage (if required)
    py->>worker: release GIL + return
    worker-->>worker: ECS -> agents -> events -> snapshot
    worker->>-main: publish immutable snapshot
    main->>render: render(snapshot, alpha, frame_dt)
    render->>main: present complete
    app->>engine: shutdown(reason)
    engine->>main: stop scheduling new frames
    main->>worker: drain and terminate workers
    engine->>render: flush and release GPU resources
    engine->>py: finalize Python runtime bindings
```

</spec>
