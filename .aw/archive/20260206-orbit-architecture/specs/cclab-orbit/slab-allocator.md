---
id: slab-allocator
type: spec
title: "Custom Slab Allocator"
version: 1
spec_type: algorithm
spec_group: cclab-orbit
created_at: 2026-02-05T16:14:24.680451+00:00
updated_at: 2026-02-05T16:14:24.680451+00:00
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
      title: "Slab Allocation Flow"
history:
  - timestamp: 2026-02-05T16:14:24.680451+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Custom Slab Allocator

## Overview

Implement a custom slab allocator for fixed-size allocations to reduce heap allocation pressure in hot paths. Target structures: TimerEntry (~48 bytes), Handle (~16 bytes), ScheduledCallback (~64 bytes). The slab pre-allocates memory and reuses slots, providing O(1) allocation and deallocation.

## Requirements

### R1 - Slab<T> implementation

```yaml
id: R1
priority: high
status: draft
```

Generic Slab<T> struct with pre-allocated slots, free list tracking, and O(1) insert/remove operations

### R2 - Thread safety

```yaml
id: R2
priority: high
status: draft
```

Slab must be thread-safe using interior mutability (Mutex or lock-free design)

### R3 - Growth strategy

```yaml
id: R3
priority: medium
status: draft
```

Slab should grow when full, doubling capacity up to a configurable maximum

### R4 - Timer wheel integration

```yaml
id: R4
priority: high
status: draft
```

Integrate Slab<TimerEntry> into TimerWheel, gated behind slab-allocator feature flag

### R5 - Handle allocation

```yaml
id: R5
priority: low
status: draft
```

Optionally use Slab for Handle allocation to reduce allocation overhead

## Acceptance Criteria

### Scenario: Slab allocation cycle

- **GIVEN** Empty slab with capacity 1024
- **WHEN** Insert and remove 1000 items
- **THEN** All operations succeed in O(1), no heap allocations after initial

### Scenario: Slab growth

- **GIVEN** Full slab at capacity
- **WHEN** Insert new item
- **THEN** Slab grows to accommodate, existing items remain valid

### Scenario: Concurrent access

- **GIVEN** Slab shared across threads
- **WHEN** Multiple threads insert/remove concurrently
- **THEN** No data races, all operations succeed

### Scenario: Timer wheel with slab

- **GIVEN** TimerWheel using Slab<TimerEntry>
- **WHEN** Schedule 10k timers
- **THEN** Fewer heap allocations than BTreeMap approach

## Diagrams

### Slab Allocation Flow

```mermaid
flowchart TB
    request([Allocation Request])
    check_free{Free slot available?} 
    pop_free[Pop from free list]
    check_cap{At capacity?} 
    grow[Grow slab (2x)]
    use_next[Use next slot]
    return([Return SlabKey])
    request --> check_free
    check_free -->|Yes| pop_free
    check_free -->|No| check_cap
    check_cap -->|Yes| grow
    check_cap -->|No| use_next
    grow --> use_next
    pop_free --> return
    use_next --> return
```

</spec>
