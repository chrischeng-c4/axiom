---
id: fuzz-targets
type: spec
title: "Fuzz Testing Infrastructure"
version: 1
spec_type: utility
spec_group: cclab-orbit
created_at: 2026-02-05T16:13:52.494761+00:00
updated_at: 2026-02-05T16:13:52.494761+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-05T16:13:52.494761+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Fuzz Testing Infrastructure

## Overview

Set up cargo-fuzz infrastructure for the orbit crate with fuzz targets for TimerWheel, PythonWaker, and Handle operations. Fuzz targets must isolate pure-Rust logic from PyO3 dependencies since fuzz testing cannot run Python runtime.

## Requirements

### R1 - Fuzz infrastructure setup

```yaml
id: R1
priority: high
status: draft
```

Create fuzz/ directory with Cargo.toml configured for cargo-fuzz, including arbitrary crate for structured input generation

### R2 - TimerWheel fuzz target

```yaml
id: R2
priority: high
status: draft
```

Fuzz target for timer wheel operations: registration with arbitrary times, cancellation sequences, expiration processing. Must use mock callbacks instead of PyObject.

### R3 - Waker fuzz target

```yaml
id: R3
priority: medium
status: draft
```

Fuzz target for PythonWaker operations: wake/reset sequences, concurrent wake from multiple threads, pool get/put cycles

### R4 - Handle fuzz target

```yaml
id: R4
priority: medium
status: draft
```

Fuzz target for Handle and TimerHandle: cancellation flag operations, clone/drop sequences

## Acceptance Criteria

### Scenario: Timer registration with edge times

- **GIVEN** Empty timer wheel
- **WHEN** Register timers with times from past to far future
- **THEN** All registrations succeed without panic

### Scenario: Rapid cancel after register

- **GIVEN** Timer wheel with active timers
- **WHEN** Cancel immediately after registration
- **THEN** Cancellation succeeds, timer does not fire

### Scenario: Concurrent waker operations

- **GIVEN** PythonWaker instance
- **WHEN** Multiple threads call wake() simultaneously
- **THEN** No data races, is_triggered() eventually returns true

### Scenario: Waker pool stress

- **GIVEN** WakerPool with capacity 256
- **WHEN** Get and put 10k wakers concurrently
- **THEN** Pool maintains consistency, no memory leaks

</spec>
