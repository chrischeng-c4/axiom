---
id: nebula-rust-state-pyo3-spec
type: spec
title: "PyO3 StateTracker Bindings"
version: 1
spec_type: utility
created_at: 2026-02-01T14:27:37.894039+00:00
updated_at: 2026-02-01T14:27:37.894039+00:00
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-01T14:27:37.894039+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# PyO3 StateTracker Bindings

## Overview

This specification defines the PyO3 bindings for StateTracker, enabling Python to use the Rust implementation for change tracking. It covers the PyO3 class definition, method mappings, and data conversion between Python objects and BSON.

## Requirements

### R1 - PyO3 Class Definition

```yaml
id: R1
priority: medium
status: draft
```

Define a PyO3 class `StateTracker` that wraps the internal Rust `StateTracker` struct.

### R2 - Method Mappings

```yaml
id: R2
priority: medium
status: draft
```

Expose `track_change`, `get_changes`, `rollback`, `is_modified`, `has_changed`, `reset`, and `get_all_original_data` to Python.

### R3 - Data Conversion

```yaml
id: R3
priority: medium
status: draft
```

Handle BSON conversion between Python dictionaries/objects and Rust `bson::Document`/`bson::Bson`. Use `pythonize` or similar for efficient conversion.

## Acceptance Criteria

### Scenario: Python-Rust interaction

- **GIVEN** A Python dictionary and a Rust-backed StateTracker.
- **WHEN** tracker.track_change("field", value) is called from Python.
- **THEN** The Rust StateTracker correctly records the change and can return it to Python as a dictionary.

### Scenario: Nested field tracking

- **GIVEN** A Python dictionary with nested values.
- **WHEN** tracker.track_change("nested.field", value) is called (if supported) or parent field is tracked.
- **THEN** The tracker marks the top-level field as dirty when a nested value is tracked.

## Flow Diagram

```mermaid
classDiagram
    class PyStateTracker {
        -inner: StateTracker
        +track_change(field: String, value: PyObject)
        +get_changes(current_data: PyDict) PyDict
        +rollback(document: PyDict)
        +is_modified() bool
        +has_changed(field: String) bool
        +reset()
        +get_all_original_data(current_data: PyDict) PyDict
    }

sequenceDiagram
    participant P as Python
    participant R as Rust (PyO3)
    participant S as StateTracker (Rust)
    P->>R: tracker.track_change("field", val)
    R->>R: Convert PyObject to bson::Bson
    R->>S: inner.track_change("field", bson)
    S-->>R: ok
    R-->>P: ok
```

</spec>
