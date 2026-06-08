---
id: exception
type: spec
title: "Exception Hierarchy"
version: 1
spec_type: utility
files:
  - runtime/exception.rs
---

# Exception Hierarchy

## Overview

Implements Python exception hierarchy as Mamba class objects. BaseException is the root, Exception inherits from it, and concrete exceptions (ValueError, TypeError, KeyError, etc.) inherit from Exception. Includes ExceptionGroup (PEP 654) with `except*` syntax for handling multiple exceptions. Raise/except uses isinstance-based matching against the class hierarchy.

Renamed from exception-hierarchy.md with updated frontmatter.

## Source Files

| File | LOC | Responsibility |
|------|-----|----------------|
| `runtime/exception.rs` | 419 | Exception classes, instantiation, matching, thread-local state, ExceptionGroup |

## Requirements

### R1 - Exception class hierarchy

```yaml
id: R1
priority: high
```

Create pre-built MbClass objects for BaseException, Exception, ValueError, TypeError, KeyError, IndexError, AttributeError, FileNotFoundError, StopIteration, RuntimeError, ZeroDivisionError, NameError. Each with correct MRO inheritance.

### R2 - Exception instantiation

```yaml
id: R2
priority: high
```

`mb_exception_new(class_id, message) -> ObjData::Exception` instance with class reference, message string, and optional cause/traceback.

### R3 - Exception matching

```yaml
id: R3
priority: high
```

`mb_exception_matches(exception, class) -> bool`. Check if exception is instance of class using MRO chain. Used by except clause matching.

### R4 - Thread-local exception state

```yaml
id: R4
priority: high
```

Thread-local storage for current exception state:
- `mb_raise(exception)` -- set current exception
- `mb_get_exception() -> Option<MbValue>` -- retrieve current exception
- `mb_clear_exception()` -- clear exception state

### R5 - ExceptionGroup (PEP 654)

```yaml
id: R5
priority: high
```

ExceptionGroup wrapping `Vec<MbValue>` of sub-exceptions. Methods:
- `mb_exception_group_new(message, exceptions)` -- create group
- `mb_exception_group_split(group, predicate)` -- split into matching/non-matching
- `mb_exception_group_subgroup(group, predicate)` -- filter matching only
- `mb_exception_group_exceptions(group)` -- access sub-exceptions

### R6 - except* syntax support

```yaml
id: R6
priority: high
```

Add ExceptStar variant to AST. Parse `except* Type as e:` syntax. Lower to HIR/MIR with multi-handler matching that splits ExceptionGroup by type.

### R7 - Symbol registration

```yaml
id: R7
priority: high
```

Register exception functions and class constants in symbols.rs. All exception classes accessible as builtin names.

## Acceptance Criteria

### Scenario: Raise ValueError

- **WHEN** `raise ValueError('bad input')`
- **THEN** Creates Exception instance with class=ValueError, message='bad input'

### Scenario: Except matching via inheritance

- **WHEN** try/except Exception catches ValueError
- **THEN** `mb_exception_matches` returns True since ValueError inherits Exception

### Scenario: Except specific mismatch

- **WHEN** try/except TypeError does not catch ValueError
- **THEN** `mb_exception_matches` returns False

### Scenario: except* catches matching exceptions

- **WHEN** ExceptionGroup raised with [ValueError, TypeError], except* ValueError handler
- **THEN** Handler catches ValueError, remaining TypeError propagates
