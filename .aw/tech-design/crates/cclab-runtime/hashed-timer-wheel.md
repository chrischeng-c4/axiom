---
id: hashed-timer-wheel
type: spec
title: "Hashed Hierarchical Timer Wheel Integration"
version: 1
spec_type: algorithm
created_at: 2026-02-05T04:29:24.461982+00:00
updated_at: 2026-02-05T04:29:24.461982+00:00
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
      title: "Hierarchical Timer Wheel Structure"
history:
  - timestamp: 2026-02-05T04:29:24.461982+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Hashed Hierarchical Timer Wheel Integration

## Overview

Integrate the existing timer_wheel_hashed.rs implementation into the main event loop, replacing the BTreeMap-based timer with O(1) hashed hierarchical wheel. The wheel uses 4 levels with 64 slots each, providing efficient timer scheduling with 1ms resolution at the finest level. This improves timer operations from O(log n) to O(1) and reduces lock contention through better cache locality.

## Requirements

### R1 - O(1) timer operations

```yaml
id: R1
priority: high
status: draft
```

Timer insertion, cancellation, and expiry checks must be O(1) amortized complexity.

### R2 - Hierarchical wheel structure

```yaml
id: R2
priority: high
status: draft
```

Implement 4-level hierarchy: Level 0 (1ms), Level 1 (64ms), Level 2 (4096ms), Level 3 (262144ms) for handling timers from milliseconds to minutes.

### R3 - API compatibility

```yaml
id: R3
priority: high
status: draft
```

Maintain compatibility with existing call_later, call_at, and cancel_handle APIs.

### R4 - Cascade efficiency

```yaml
id: R4
priority: medium
status: draft
```

Timer cascade from higher to lower levels must be efficient and not cause latency spikes.

### R5 - Memory efficiency

```yaml
id: R5
priority: medium
status: draft
```

Use linked lists or similar structures to avoid pre-allocating memory for all possible timer slots.

## Acceptance Criteria

### Scenario: Short timer fast path

- **GIVEN** A timer with 5ms delay
- **WHEN** Timer is scheduled
- **THEN** Inserted into Level 0 in O(1) time

### Scenario: Long timer scheduling

- **GIVEN** A timer with 30 second delay
- **WHEN** Timer is scheduled
- **THEN** Inserted into appropriate higher level wheel

### Scenario: Timer cascade

- **GIVEN** Level 1 slot expires with multiple timers
- **WHEN** Wheel advances to that slot
- **THEN** Timers cascade down to Level 0 efficiently

### Scenario: Timer cancellation

- **GIVEN** A scheduled timer
- **WHEN** Cancel is called before expiry
- **THEN** Timer is removed in O(1) time

### Scenario: High timer volume

- **GIVEN** 10000 timers scheduled within same second
- **WHEN** All timers expire
- **THEN** Callbacks execute without latency spikes

## Diagrams

### Hierarchical Timer Wheel Structure

```mermaid
flowchart TB
    l3[Level 3: 262s resolution]
    l2[Level 2: 4s resolution]
    l1[Level 1: 64ms resolution]
    l0[Level 0: 1ms resolution]
    fire(Fire callbacks)
    l3 -->|cascade| l2
    l2 -->|cascade| l1
    l1 -->|cascade| l0
    l0 -->|expire| fire
```

</spec>
