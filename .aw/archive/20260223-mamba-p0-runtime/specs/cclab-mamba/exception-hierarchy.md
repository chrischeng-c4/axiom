---
id: exception-hierarchy
type: spec
title: "Class-Based Exception Hierarchy"
version: 1
spec_type: utility
spec_group: cclab-mamba
created_at: 2026-02-15T17:31:34.758482+00:00
updated_at: 2026-02-15T17:31:34.758482+00:00
requirements:
  total: 5
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
depends:
  - method-dispatch
changes:
  - file: crates/mamba/src/runtime/exception.rs
    action: MODIFY
    description: "Add class hierarchy, mb_exception_new, mb_exception_matches"
  - file: crates/mamba/src/runtime/rc.rs
    action: MODIFY
    description: "Update ObjData::Exception to reference class"
  - file: crates/mamba/src/runtime/symbols.rs
    action: MODIFY
    description: "Register exception functions and class constants"
history:
  - timestamp: 2026-02-15T17:31:34.758482+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Class-Based Exception Hierarchy

## Overview

Implements Python exception hierarchy as Mamba class objects. BaseException is the root, Exception inherits from it, and concrete exceptions (ValueError, TypeError, KeyError, IndexError, AttributeError, FileNotFoundError, StopIteration, RuntimeError, ZeroDivisionError, NameError) inherit from Exception. Each exception is an ObjData::Class with proper MRO. Raise/except uses isinstance-based matching against the class hierarchy. Thread-local current exception stores both the class and instance.

## Requirements

### R1 - Exception class hierarchy

```yaml
id: R1
priority: high
status: draft
```

Create pre-built MbClass objects for BaseException, Exception, ValueError, TypeError, KeyError, IndexError, AttributeError, FileNotFoundError, StopIteration, RuntimeError, ZeroDivisionError, NameError. Each with correct MRO inheritance.

### R2 - Exception instantiation

```yaml
id: R2
priority: high
status: draft
```

mb_exception_new(class_id, message) -> ObjData::Exception instance with class reference, message string, and optional cause/traceback.

### R3 - Exception matching

```yaml
id: R3
priority: high
status: draft
```

mb_exception_matches(exception, class) -> bool. Check if exception is instance of class using MRO chain. Used by except clause matching.

### R4 - Thread-local integration

```yaml
id: R4
priority: high
status: draft
```

Update mb_raise/mb_get_exception/mb_clear_exception to use class-based exceptions while preserving thread-local storage.

### R5 - Symbol registration

```yaml
id: R5
priority: high
status: draft
```

Register exception functions in symbols.rs. Register exception class constants for name resolution.

## Acceptance Criteria

### Scenario: Raise ValueError

- **WHEN** raise ValueError('bad input')
- **THEN** Creates Exception instance with class=ValueError, message='bad input'

### Scenario: Except matching

- **WHEN** try/except Exception catches ValueError
- **THEN** mb_exception_matches returns True since ValueError inherits Exception

### Scenario: Except specific

- **WHEN** try/except TypeError does not catch ValueError
- **THEN** mb_exception_matches returns False

### Scenario: Exception message

- **WHEN** str(ValueError('x'))
- **THEN** Returns 'x'

</spec>
