---
id: mamba-p2
change_id: mamba-p2
type: tasks
version: 1
created_at: 2026-02-22T11:23:57.940381+00:00
updated_at: 2026-02-22T11:23:57.940381+00:00
proposal_ref: mamba-p2
summary:
  total: 28
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 28
layers:
  logic:
    task_count: 14
    estimated_files: 14
  testing:
    task_count: 14
    estimated_files: 14
history:
  - timestamp: 2026-02-22T11:23:57.940381+00:00
    agent: "task-generator"
    tool: "create_tasks"
    action: "created"
---

<tasks>

# Implementation Tasks

## Overview

This document outlines 28 implementation tasks for change `mamba-p2`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Logic Layer | 14 | 🔲 Pending |
| Testing Layer | 14 | 🔲 Pending |

## 2. Logic Layer

### Task 2.1: Create mamba-type-extensions-p2.rs

```yaml
id: 2.1
action: CREATE
status: pending
file: src/logic/mamba-type-extensions-p2.rs
spec_ref: mamba-type-extensions-p2:*
```

Implement Type extensions: frozenset, enum, dataclasses covering:
- R1: FrozenSet ObjData variant
- R2: frozenset() constructor and dispatch
- R6: @dataclass decorator

### Task 2.2: Create mamba-runtime-core-p2.rs

```yaml
id: 2.2
action: CREATE
status: pending
file: src/logic/mamba-runtime-core-p2.rs
spec_ref: mamba-runtime-core-p2:*
depends_on: [2.1]
```

Implement Runtime core P2: unpacking, __format__, __del__, except*, __slots__ covering:
- R1: Dict unpacking expressions
- R2: List unpacking expressions
- R6: ExceptionGroup class

### Task 2.3: Create mamba-stdlib-utils.rs

```yaml
id: 2.3
action: CREATE
status: pending
file: src/logic/mamba-stdlib-utils.rs
spec_ref: mamba-stdlib-utils:*
depends_on: [2.2]
```

Implement Stdlib: contextlib, copy, operator, weakref covering:
- R2: copy module
- R1: contextlib module
- R3: operator module

### Task 2.4: Create mamba-stdlib-numeric.rs

```yaml
id: 2.4
action: CREATE
status: pending
file: src/logic/mamba-stdlib-numeric.rs
spec_ref: mamba-stdlib-numeric:*
depends_on: [2.3]
```

Implement Stdlib: random, decimal, fractions covering:
- R1: random module registration
- R2: random core functions
- R3: decimal module

### Task 2.5: Create mamba-stdlib-iteration.rs

```yaml
id: 2.5
action: CREATE
status: pending
file: src/logic/mamba-stdlib-iteration.rs
spec_ref: mamba-stdlib-iteration:*
depends_on: [2.4]
```

Implement Stdlib: itertools and functools covering:
- R1: itertools module registration
- R2: itertools.chain(a, b)
- R3: itertools.islice(iterable, stop) / islice(iterable, start, stop)

### Task 2.6: Create stdlib-modules.rs

```yaml
id: 2.6
action: CREATE
status: pending
file: src/logic/stdlib-modules.rs
spec_ref: stdlib-modules:*
depends_on: [2.5]
```

Implement Standard Library Modules Implementation covering:
- R1: Regular Expressions
- R2: Date and Time
- R3: Collections

### Task 2.7: Create builtin-types.rs

```yaml
id: 2.7
action: CREATE
status: pending
file: src/logic/builtin-types.rs
spec_ref: builtin-types:*
depends_on: [2.6]
```

Implement Built-in Types covering:
- R1: Bytes Type
- R2: ByteArray Type
- R3: Frozenset Type

### Task 2.8: Create mamba-stdlib-diagnostics.rs

```yaml
id: 2.8
action: CREATE
status: pending
file: src/logic/mamba-stdlib-diagnostics.rs
spec_ref: mamba-stdlib-diagnostics:*
depends_on: [2.7]
```

Implement Stdlib: traceback, warnings, inspect covering:
- R1: traceback module
- R2: warnings module
- R3: inspect module

### Task 2.9: Create mamba-stdlib-binary.rs

```yaml
id: 2.9
action: CREATE
status: pending
file: src/logic/mamba-stdlib-binary.rs
spec_ref: mamba-stdlib-binary:*
depends_on: [2.8]
```

Implement Stdlib: io/struct, hashlib, base64 covering:
- R1: io module
- R2: struct module
- R3: hashlib module

### Task 2.10: Create mamba-stdlib-re.rs

```yaml
id: 2.10
action: CREATE
status: pending
file: src/logic/mamba-stdlib-re.rs
spec_ref: mamba-stdlib-re:*
depends_on: [2.9]
```

Implement Stdlib: re (regular expressions) covering:
- R1: re module registration
- R2: re.search(pattern, string)
- R3: re.match(pattern, string)

### Task 2.11: Create mamba-stdlib-fs.rs

```yaml
id: 2.11
action: CREATE
status: pending
file: src/logic/mamba-stdlib-fs.rs
spec_ref: mamba-stdlib-fs:*
depends_on: [2.10]
```

Implement Stdlib: pathlib, shutil, tempfile, glob covering:
- R1: pathlib module registration
- R2: Path methods using std::path
- R3: shutil module

### Task 2.12: Create runtime-features.rs

```yaml
id: 2.12
action: CREATE
status: pending
file: src/logic/runtime-features.rs
spec_ref: runtime-features:*
depends_on: [2.11]
```

Implement Runtime Features covering:
- R1: Context Managers
- R2: Unpacking
- R3: Exception Groups

### Task 2.13: Create mamba-stdlib-collections.rs

```yaml
id: 2.13
action: CREATE
status: pending
file: src/logic/mamba-stdlib-collections.rs
spec_ref: mamba-stdlib-collections:*
depends_on: [2.12]
```

Implement Stdlib: collections (defaultdict, Counter, deque, OrderedDict) covering:
- R1: collections module registration
- R2: defaultdict
- R3: Counter

### Task 2.14: Create mamba-stdlib-datetime.rs

```yaml
id: 2.14
action: CREATE
status: pending
file: src/logic/mamba-stdlib-datetime.rs
spec_ref: mamba-stdlib-datetime:*
depends_on: [2.13]
```

Implement Stdlib: datetime and time covering:
- R1: datetime module registration
- R2: datetime.now()
- R3: datetime constructor

## 4. Testing Layer

### Task 4.1: Add tests for Type extensions: frozenset, enum, dataclasses

```yaml
id: 4.1
action: CREATE
status: pending
file: tests/mamba-type-extensions-p2_test.rs
spec_ref: mamba-type-extensions-p2:*
depends_on: [2.1]
```

Create unit tests for Type extensions: frozenset, enum, dataclasses covering all requirements and acceptance scenarios

### Task 4.2: Add tests for Runtime core P2: unpacking, __format__, __del__, except*, __slots__

```yaml
id: 4.2
action: CREATE
status: pending
file: tests/mamba-runtime-core-p2_test.rs
spec_ref: mamba-runtime-core-p2:*
depends_on: [2.2]
```

Create unit tests for Runtime core P2: unpacking, __format__, __del__, except*, __slots__ covering all requirements and acceptance scenarios

### Task 4.3: Add tests for Stdlib: contextlib, copy, operator, weakref

```yaml
id: 4.3
action: CREATE
status: pending
file: tests/mamba-stdlib-utils_test.rs
spec_ref: mamba-stdlib-utils:*
depends_on: [2.3]
```

Create unit tests for Stdlib: contextlib, copy, operator, weakref covering all requirements and acceptance scenarios

### Task 4.4: Add tests for Stdlib: random, decimal, fractions

```yaml
id: 4.4
action: CREATE
status: pending
file: tests/mamba-stdlib-numeric_test.rs
spec_ref: mamba-stdlib-numeric:*
depends_on: [2.4]
```

Create unit tests for Stdlib: random, decimal, fractions covering all requirements and acceptance scenarios

### Task 4.5: Add tests for Stdlib: itertools and functools

```yaml
id: 4.5
action: CREATE
status: pending
file: tests/mamba-stdlib-iteration_test.rs
spec_ref: mamba-stdlib-iteration:*
depends_on: [2.5]
```

Create unit tests for Stdlib: itertools and functools covering all requirements and acceptance scenarios

### Task 4.6: Add tests for Standard Library Modules Implementation

```yaml
id: 4.6
action: CREATE
status: pending
file: tests/stdlib-modules_test.rs
spec_ref: stdlib-modules:*
depends_on: [2.6]
```

Create unit tests for Standard Library Modules Implementation covering all requirements and acceptance scenarios

### Task 4.7: Add tests for Built-in Types

```yaml
id: 4.7
action: CREATE
status: pending
file: tests/builtin-types_test.rs
spec_ref: builtin-types:*
depends_on: [2.7]
```

Create unit tests for Built-in Types covering all requirements and acceptance scenarios

### Task 4.8: Add tests for Stdlib: traceback, warnings, inspect

```yaml
id: 4.8
action: CREATE
status: pending
file: tests/mamba-stdlib-diagnostics_test.rs
spec_ref: mamba-stdlib-diagnostics:*
depends_on: [2.8]
```

Create unit tests for Stdlib: traceback, warnings, inspect covering all requirements and acceptance scenarios

### Task 4.9: Add tests for Stdlib: io/struct, hashlib, base64

```yaml
id: 4.9
action: CREATE
status: pending
file: tests/mamba-stdlib-binary_test.rs
spec_ref: mamba-stdlib-binary:*
depends_on: [2.9]
```

Create unit tests for Stdlib: io/struct, hashlib, base64 covering all requirements and acceptance scenarios

### Task 4.10: Add tests for Stdlib: re (regular expressions)

```yaml
id: 4.10
action: CREATE
status: pending
file: tests/mamba-stdlib-re_test.rs
spec_ref: mamba-stdlib-re:*
depends_on: [2.10]
```

Create unit tests for Stdlib: re (regular expressions) covering all requirements and acceptance scenarios

### Task 4.11: Add tests for Stdlib: pathlib, shutil, tempfile, glob

```yaml
id: 4.11
action: CREATE
status: pending
file: tests/mamba-stdlib-fs_test.rs
spec_ref: mamba-stdlib-fs:*
depends_on: [2.11]
```

Create unit tests for Stdlib: pathlib, shutil, tempfile, glob covering all requirements and acceptance scenarios

### Task 4.12: Add tests for Runtime Features

```yaml
id: 4.12
action: CREATE
status: pending
file: tests/runtime-features_test.rs
spec_ref: runtime-features:*
depends_on: [2.12]
```

Create unit tests for Runtime Features covering all requirements and acceptance scenarios

### Task 4.13: Add tests for Stdlib: collections (defaultdict, Counter, deque, OrderedDict)

```yaml
id: 4.13
action: CREATE
status: pending
file: tests/mamba-stdlib-collections_test.rs
spec_ref: mamba-stdlib-collections:*
depends_on: [2.13]
```

Create unit tests for Stdlib: collections (defaultdict, Counter, deque, OrderedDict) covering all requirements and acceptance scenarios

### Task 4.14: Add tests for Stdlib: datetime and time

```yaml
id: 4.14
action: CREATE
status: pending
file: tests/mamba-stdlib-datetime_test.rs
spec_ref: mamba-stdlib-datetime:*
depends_on: [2.14]
```

Create unit tests for Stdlib: datetime and time covering all requirements and acceptance scenarios

</tasks>
