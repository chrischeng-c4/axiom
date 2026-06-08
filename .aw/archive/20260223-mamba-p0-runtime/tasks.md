---
id: mamba-p0-runtime
change_id: mamba-p0-runtime
type: tasks
version: 1
created_at: 2026-02-15T17:35:00.000000+00:00
updated_at: 2026-02-15T17:35:00.000000+00:00
proposal_ref: mamba-p0-runtime
summary:
  total: 22
  completed: 22
  in_progress: 0
  blocked: 0
  pending: 0
layers:
  logic:
    task_count: 11
    estimated_files: 14
  integration:
    task_count: 1
    estimated_files: 1
  testing:
    task_count: 10
    estimated_files: 4
history:
  - timestamp: 2026-02-15T17:35:00.000000+00:00
    agent: "mainthread"
    tool: "manual_create"
    action: "created"
---

<tasks>

# Implementation Tasks

## Overview

This document outlines 22 implementation tasks for change `mamba-p0-runtime`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Logic Layer | 11 | ✅ Complete |
| Integration Layer | 1 | ✅ Complete |
| Testing Layer | 10 | ✅ Complete |

## 2. Logic Layer

### Task 2.1: Implement method-dispatch (mb_call_method)

```yaml
id: 2.1
action: MODIFY
status: done
file: crates/mamba/src/runtime/class.rs
spec_ref: method-dispatch:R1,R2,R3
```

Implement type-tagged method dispatch (#380 foundation):
- R1: Create mb_call_method(receiver, method_name, args, argc) dispatch function
- R2: Handle primitive types (TAG_INT, TAG_BOOL, TAG_NONE) without heap deref
- R3: MRO fallback for ObjData::Instance

### Task 2.2: Register method-dispatch symbols

```yaml
id: 2.2
action: MODIFY
status: done
file: crates/mamba/src/runtime/symbols.rs
spec_ref: method-dispatch:R4
depends_on: [2.1]
```

Register mb_call_method in symbols.rs as MirExtern entry for JIT/AOT linking.

### Task 2.3: Implement string methods (#375)

```yaml
id: 2.3
action: MODIFY
status: done
file: crates/mamba/src/runtime/string_ops.rs
spec_ref: string-methods:R1,R2,R3,R4,R5
depends_on: [2.1]
```

Add string methods to string_ops.rs:
- R1: split, join
- R2: strip, lstrip, rstrip
- R3: replace, find
- R4: startswith, endswith, isdigit, isalpha, count
- R5: upper, lower

### Task 2.4: Implement list methods (#376)

```yaml
id: 2.4
action: CREATE
status: done
file: crates/mamba/src/runtime/list_ops.rs
spec_ref: list-methods:R1,R2,R3,R4
depends_on: [2.1]
```

Create list_ops.rs with list methods:
- R1: append, insert, remove, clear, extend
- R2: pop, index
- R3: sort, reverse
- R4: copy, count

### Task 2.5: Implement dict methods (#377)

```yaml
id: 2.5
action: CREATE
status: done
file: crates/mamba/src/runtime/dict_ops.rs
spec_ref: dict-methods:R1,R2,R3,R4
depends_on: [2.1]
```

Create dict_ops.rs with dict methods:
- R1: get, setdefault
- R2: keys, values, items
- R3: update, pop, clear
- R4: copy

### Task 2.6: Implement core builtins (#378)

```yaml
id: 2.6
action: MODIFY
status: done
file: crates/mamba/src/runtime/builtins.rs
spec_ref: core-builtins:R1,R2,R3,R4
depends_on: [2.1]
```

Extend builtins.rs with new builtins:
- R1: enumerate, zip, reversed (iterator builtins)
- R2: min, max, sum, sorted (aggregation)
- R3: isinstance (type checking)
- R4: repr, hash, id, input, chr, ord, hex, oct, bin

### Task 2.7: Add iterator variants for builtins

```yaml
id: 2.7
action: MODIFY
status: done
file: crates/mamba/src/runtime/iter.rs
spec_ref: core-builtins:R1
depends_on: [2.6]
```

Add EnumerateIterator, ZipIterator, ReversedIterator variants to MbIterator enum.

### Task 2.8: Implement exception hierarchy (#381)

```yaml
id: 2.8
action: MODIFY
status: done
file: crates/mamba/src/runtime/exception.rs
spec_ref: exception-hierarchy:R1,R2,R3,R4
depends_on: [2.1]
```

Implement class-based exceptions:
- R1: Pre-built MbClass objects for BaseException, Exception, ValueError, TypeError, etc.
- R2: mb_exception_new(class_id, message)
- R3: mb_exception_matches(exception, class) for except clause matching
- R4: Update thread-local integration

### Task 2.9: Implement magic method dispatch (#380)

```yaml
id: 2.9
action: MODIFY
status: done
file: crates/mamba/src/runtime/class.rs
spec_ref: magic-methods:R1,R2,R3,R4
depends_on: [2.1, 2.8]
```

Implement dunder method dispatch:
- R1: mb_binop for arithmetic operators (__add__, __sub__, etc.)
- R2: mb_compare for comparison operators (__eq__, __lt__, etc.)
- R3: mb_call_str, mb_call_repr, mb_call_bool
- R4: mb_call_len, mb_call_iter, mb_call_next, mb_call_contains

### Task 2.10: Implement file I/O (#379)

```yaml
id: 2.10
action: CREATE
status: done
file: crates/mamba/src/runtime/file_io.rs
spec_ref: file-io:R1,R2,R3,R4,R5
depends_on: [2.1, 2.8]
```

Create file_io.rs with:
- R1: ObjData::File variant in rc.rs
- R2: mb_open(path, mode)
- R3: mb_file_read, mb_file_readline, mb_file_readlines
- R4: mb_file_write, mb_file_writelines
- R5: mb_file_close

### Task 2.11: Register all new symbols

```yaml
id: 2.11
action: MODIFY
status: done
file: crates/mamba/src/runtime/symbols.rs
spec_ref: string-methods:R6, list-methods:R5, dict-methods:R5, core-builtins:R5, exception-hierarchy:R5, magic-methods:R5, file-io:R6
depends_on: [2.3, 2.4, 2.5, 2.6, 2.8, 2.9, 2.10]
```

Register ALL new runtime functions in symbols.rs:
- String methods (split, join, strip, etc.)
- List methods (append, pop, sort, etc.)
- Dict methods (get, keys, values, etc.)
- Core builtins (enumerate, zip, min, max, etc.)
- Exception functions (new, matches, etc.)
- Magic method dispatch (binop, compare, etc.)
- File I/O functions (open, read, write, etc.)
- Add pub mod list_ops, dict_ops, file_io to runtime/mod.rs

## 3. Integration Layer

### Task 3.1: Wire method dispatch into lowering and codegen

```yaml
id: 3.1
action: MODIFY
status: done
file: crates/mamba/src/lower/hir_to_mir.rs
spec_ref: method-dispatch:R1, magic-methods:R5
depends_on: [2.1, 2.9, 2.11]
```

Update HIR-to-MIR lowering:
- Lower method calls (x.method()) to CallExtern(mb_call_method)
- Lower operators to CallExtern(mb_binop/mb_compare)
- Lower conversion calls (str/repr/bool) to CallExtern dispatch

## 4. Testing Layer

### Task 4.1: Test method dispatch

```yaml
id: 4.1
action: MODIFY
status: done
file: crates/mamba/tests/pipeline_tests.rs
spec_ref: method-dispatch:S1,S2,S3,S4
depends_on: [2.1, 2.2, 3.1]
```

Test type-tagged dispatch for string, int, user class, and AttributeError cases.

### Task 4.2: Test string methods

```yaml
id: 4.2
action: MODIFY
status: done
file: crates/mamba/tests/pipeline_tests.rs
spec_ref: string-methods:S1,S2,S3,S4,S5,S6
depends_on: [2.3, 2.11]
```

Test split, join, strip, replace, find, upper, lower.

### Task 4.3: Test list methods

```yaml
id: 4.3
action: MODIFY
status: done
file: crates/mamba/tests/pipeline_tests.rs
spec_ref: list-methods:S1,S2,S3,S4
depends_on: [2.4, 2.11]
```

Test append, pop, sort, remove ValueError, extend.

### Task 4.4: Test dict methods

```yaml
id: 4.4
action: MODIFY
status: done
file: crates/mamba/tests/pipeline_tests.rs
spec_ref: dict-methods:S1,S2,S3,S4
depends_on: [2.5, 2.11]
```

Test get with default, keys, pop KeyError, update.

### Task 4.5: Test core builtins

```yaml
id: 4.5
action: MODIFY
status: done
file: crates/mamba/tests/pipeline_tests.rs
spec_ref: core-builtins:S1,S2,S3,S4
depends_on: [2.6, 2.7, 2.11]
```

Test enumerate, min/max, isinstance, sorted.

### Task 4.6: Test exception hierarchy

```yaml
id: 4.6
action: MODIFY
status: done
file: crates/mamba/tests/pipeline_tests.rs
spec_ref: exception-hierarchy:S1,S2,S3,S4
depends_on: [2.8, 2.11]
```

Test raise ValueError, except matching, except specific, exception message.

### Task 4.7: Test magic methods

```yaml
id: 4.7
action: MODIFY
status: done
file: crates/mamba/tests/pipeline_tests.rs
spec_ref: magic-methods:S1,S2,S3,S4
depends_on: [2.9, 2.11, 3.1]
```

Test int addition, user class __add__, str conversion, len protocol.

### Task 4.8: Test file I/O

```yaml
id: 4.8
action: MODIFY
status: done
file: crates/mamba/tests/pipeline_tests.rs
spec_ref: file-io:S1,S2,S3,S4
depends_on: [2.10, 2.11]
```

Test read file, write file, FileNotFoundError, read after close.

### Task 4.9: Unit tests for string/list/dict ops

```yaml
id: 4.9
action: CREATE
status: done
file: crates/mamba/tests/runtime_tests.rs
spec_ref: string-methods:*, list-methods:*, dict-methods:*
depends_on: [2.3, 2.4, 2.5]
```

Direct unit tests for runtime functions (bypass codegen, call extern C functions directly).

### Task 4.10: Unit tests for exception and builtins

```yaml
id: 4.10
action: CREATE
status: done
file: crates/mamba/tests/runtime_tests.rs
spec_ref: exception-hierarchy:*, core-builtins:*
depends_on: [2.6, 2.8]
```

Direct unit tests for exception hierarchy matching and builtin functions.

</tasks>
