---
id: orbit-internal-components
type: spec
title: "Orbit Internal Components"
version: 1
spec_type: utility
created_at: 2026-01-27T16:54:00.828209+00:00
updated_at: 2026-01-27T16:54:00.828209+00:00
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
design_elements:
  has_mermaid: false
  has_json_schema: true
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-01-27T16:54:00.828209+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Orbit Internal Components

## Overview

This specification details the internal utility components and data models required for the optimized Orbit engine, focusing on the Python-to-Tokio Waker bridge and the hierarchical timer wheel internals.

## Requirements

### R1 - Hashed Wheel Internal Structure

```yaml
id: R1
priority: high
status: draft
```

The timer wheel must utilize a 4-level hierarchical structure with 64 slots per level, using bit-shifting for rapid bucket indexing. This replaces the BTreeMap to achieve O(1) performance.

### R2 - Python-to-Tokio Waker Bridge

```yaml
id: R2
priority: high
status: draft
```

A specialized Waker implementation that allows Python's 'add_done_callback' or equivalent mechanisms to wake up the associated Rust Task awaiting in the Tokio runtime.

### R3 - Lock-Free MPSC Internals

```yaml
id: R3
priority: high
status: draft
```

The task queue must implement a lock-free Multi-Producer Single-Consumer (MPSC) pattern, optimized for low-latency delivery of ScheduledCallback objects to the event loop.

## Acceptance Criteria

### Scenario: Hierarchical Bucket Placement

- **GIVEN** A timer is scheduled for 150ms in the future.
- **WHEN** register() is called with the specific Instant.
- **THEN** The insertion algorithm must calculate the correct level and slot index using bitwise masks and place the timer in O(1) time.

### Scenario: Waker Trigger Flow Orientation

- **GIVEN** A Rust task is awaiting the completion of a Python coroutine.
- **WHEN** The Python-side callback is triggered.
- **THEN** The PythonWaker must hold a reference to the Tokio Waker and correctly signal it when the Python-side dependency is resolved.

### Scenario: MPSC Contention Handling

- **GIVEN** Multiple producers are sending callbacks to the event loop.
- **WHEN** Concurrently scheduling more than 10,000 tasks per second.
- **THEN** The queue must accept all items without blocking and allow the consumer (event loop) to drain the items in a single atomic operation where possible.

## Data Model

```json
{
  "\"$schema\"": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "PythonWaker": {
      "description": "Bridge between Python's awaitable callback and Rust's Waker API",
      "properties": {
        "callback_handle": {
          "description": "Atomic pointer to the Python callback",
          "type": "string"
        },
        "waker": {
          "description": "Tokio Waker instance",
          "type": "string"
        }
      },
      "required": [
        "waker"
      ],
      "type": "object"
    },
    "TimerWheelHashed": {
      "description": "Hashed hierarchical timer wheel structure",
      "properties": {
        "current_tick": {
          "description": "Current monotonic tick count",
          "type": "integer"
        },
        "levels": {
          "description": "Array of levels, each containing 64 slots (hashed buckets)",
          "items": {
            "items": {
              "type": "string"
            },
            "type": "array"
          },
          "maxItems": 4,
          "minItems": 4,
          "type": "array"
        }
      },
      "required": [
        "levels",
        "current_tick"
      ],
      "type": "object"
    }
  },
  "type": "object"
}
```

</spec>
