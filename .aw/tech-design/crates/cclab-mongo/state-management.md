# File: specs/state-management.md

---
id: state-management
type: spec
title: "State Management (StateTracker)"
version: 1
spec_type: algorithm
created_at: 2026-02-03T09:54:02.829595+00:00
updated_at: 2026-02-03T09:54:02.829595+00:00
requirements:
  total: 8
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
    - R7
    - R8
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "StateTracker State Flow"
history:
  - timestamp: 2026-02-03T09:54:02.829595+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
  - timestamp: 2026-02-03T09:54:14.990460+00:00
    agent: "codex:deep"
    tool: "revise_spec"
    action: "revised"
  - timestamp: 2026-02-03T09:54:37.129589+00:00
    agent: "codex:max"
    tool: "review_spec"
    action: "reviewed"---

<spec>

# State Management (StateTracker)

## Overview

Define the StateTracker behavior for copy-on-write change tracking when migrating Python state management to the Rust-backed PyStateTracker. The Python-facing StateTracker remains a thin wrapper that preserves the existing public API and semantics while delegating core tracking logic to Rust.

## Requirements

### R1 - Copy-On-Write Tracking

```yaml
id: R1
priority: medium
status: draft
```

StateTracker must track field changes using Copy-On-Write semantics: the first call to `track_change(field, original_value)` stores the original value for the field and marks it changed; subsequent calls for the same field do not update the stored original value.

### R2 - Top-Level Field Semantics

```yaml
id: R2
priority: medium
status: draft
```

For dotted field paths (e.g., `user.address.city`), StateTracker must treat only the top-level segment as the tracked field name for `track_change`, `has_changed`, and `changed_field_names`.

### R3 - Modification Queries

```yaml
id: R3
priority: medium
status: draft
```

StateTracker must provide `is_modified`, `has_changed`, `change_count`, and `changed_field_names` reflecting the current tracked change set with O(1) membership checks and no deep copies of the underlying data.

### R4 - Change Extraction

```yaml
id: R4
priority: medium
status: draft
```

`get_changes()` must return a dict of tracked fields mapped to their current values from the live data; tracked fields missing from current data must be omitted from the result.

### R5 - Rollback and Reset

```yaml
id: R5
priority: medium
status: draft
```

`rollback()` must restore tracked fields in the live data to their original values and clear tracking state. `reset()` must clear tracking state without modifying data.

### R6 - Original Value Access

```yaml
id: R6
priority: medium
status: draft
```

`get_original_value(field)` must return the original value for a tracked field or `None` if the field is not tracked. `compare_field(field)` must return `True` only when the field is tracked and the current value differs from the stored original value.

### R7 - Original Snapshot Reconstruction

```yaml
id: R7
priority: medium
status: draft
```

`get_all_original_data()` must return a full dict representing the pre-change document: original values for tracked fields and current values for untracked fields.

### R8 - Python API Parity

```yaml
id: R8
priority: medium
status: draft
```

The Python-facing `StateTracker` must preserve the existing public method signatures and return types, expose a stable `__repr__` that includes the modified flag and changed field names/count, and behave as a thin wrapper over the Rust-backed PyStateTracker.

## Acceptance Criteria

### Scenario: Track Change COW

- **GIVEN** a StateTracker with data {"name": "Alice"}
- **WHEN** track_change("name", "Alice") is called twice and the value is updated between calls
- **THEN** is_modified is True and get_original_value("name") remains "Alice" after the second call

### Scenario: Top-Level Field Tracking

- **GIVEN** a StateTracker with data {"user": {"name": "Alice"}}
- **WHEN** track_change("user.name", {"name": "Alice"}) is called and the user subdocument changes
- **THEN** has_changed("user.name") and has_changed("user") are True while changed_field_names contains only "user"

### Scenario: Get Changes From Live Data

- **GIVEN** a StateTracker with data {"name": "Alice", "age": 30}
- **WHEN** name and age are tracked then updated to "Bob" and 31
- **THEN** get_changes returns {"name": "Bob", "age": 31} and omits untracked fields

### Scenario: Rollback Restores Originals

- **GIVEN** a StateTracker with data {"name": "Alice", "age": 30}
- **WHEN** name and age are tracked, updated, and rollback() is called
- **THEN** data is restored to the original values and is_modified is False

### Scenario: Reset Accepts Current State

- **GIVEN** a StateTracker with data {"name": "Alice"}
- **WHEN** name is tracked, updated to "Bob", and reset() is called
- **THEN** data remains "Bob" and is_modified is False

### Scenario: Compare Field Behavior

- **GIVEN** a StateTracker with data {"name": "Alice"}
- **WHEN** compare_field("name") is called before and after track_change plus update
- **THEN** compare_field returns False before tracking and True after the value differs from the original

### Scenario: Original Snapshot Reconstruction

- **GIVEN** a StateTracker with data {"name": "Alice", "age": 30}
- **WHEN** name is tracked and updated to "Bob"
- **THEN** get_all_original_data returns {"name": "Alice", "age": 30}

### Scenario: Modification Queries Reflect Changes

- **GIVEN** a StateTracker with data {"name": "Alice", "age": 30}
- **WHEN** name and age are tracked and only name is updated
- **THEN** is_modified is True, has_changed("name") is True, has_changed("age") is True, change_count is 2, and changed_field_names contains ["name", "age"]

### Scenario: Python API Parity and Repr

- **GIVEN** a StateTracker wrapping a Rust-backed PyStateTracker
- **WHEN** track_change("name", "Alice") is called, the value is updated, and __repr__ is queried
- **THEN** the public methods return the same types as the prior Python implementation and __repr__ includes modified=True plus the changed field names/count

## Diagrams

### StateTracker State Flow

```mermaid
flowchart LR
    clean(Clean (no changes))
    track_first[track_change(first field)]
    dirty(Dirty (changes tracked))
    track_repeat[track_change(existing field)\n(no-op)]
    reset[reset()]
    rollback[rollback()]
    clean -->|first change| track_first
    track_first --> dirty
    dirty -->|new field change| track_first
    dirty -->|repeat field change| track_repeat
    track_repeat --> dirty
    dirty -->|accept changes| reset
    reset --> clean
    dirty -->|restore originals| rollback
    rollback --> clean
```

</spec>
