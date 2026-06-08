---
id: mamba-py312-p1-spec
main_spec_ref: cclab-mamba/testing/mamba-py312-conformance.md
merge_strategy: new
filled_sections: [overview, requirements, scenarios, test_plan, changes]
---

# Mamba Py312 P1 Spec

## Overview


**main_spec_ref**: `cclab-mamba/testing/mamba-py312-conformance.md`

**fill_sections**: overview, requirements, scenarios, test_plan, changes

This change implements P1 conformance tests and runtime fixes for three areas: data structure operations (#759), exception hierarchy (#755), and generator/iterator protocol (#756).

### Scope

1. **Data Structure Ops (#759)** — Conformance tests for list, dict, set, tuple, str operations. Focus on list/dict thoroughly with all major methods, then set/tuple/str with basic ops. Uses existing golden-file harness from P0.

2. **Exception Hierarchy (#755)** — Conformance tests for built-in exception classes, `except` matching via subclass, `raise from` chaining (`__cause__`, `__context__`), custom subclassing, `args` attribute. ExceptionGroup/`except*` (PEP 654) marked xfail.

3. **Generator & Iterator Protocol (#756)** — Conformance test fixtures for `yield`, `yield from`, `send()`, `throw()`, `close()`, `StopIteration.value`. Mark non-working features as xfail. Async generators xfail.

### Design Decisions

- **List/dict first**: Highest practical impact for conformance. Set/tuple/str follow with basic ops.
- **Fixtures first for generators**: Write all test fixtures, xfail what doesn't work. Fix runtime in this or next pass.
- **xfail PEP 654**: ExceptionGroup and `except*` are Py3.11+ features not yet in mamba — xfail entirely.
- **Reuse P0 harness**: All tests use `tests/fixtures/conformance/{category}/*.py` + `.expected` golden files with `# mamba-xfail` directive.
## Requirements


### R1: List Conformance Tests (#759)

| ID | Requirement | Priority |
|----|------------|----------|
| R1.1 | Mutation methods: append, extend, insert, pop, remove, sort, reverse, copy, clear | P1 |
| R1.2 | Slicing: `a[1:3]`, `a[::2]`, `a[::-1]`, negative indices | P1 |
| R1.3 | List comprehension edge cases | P1 |
| R1.4 | Comparison: `__contains__`, `__eq__`, `__lt__` (lexicographic) | P1 |

### R2: Dict Conformance Tests (#759)

| ID | Requirement | Priority |
|----|------------|----------|
| R2.1 | Lookup: get, setdefault | P1 |
| R2.2 | Mutation: update, pop, popitem, clear, copy | P1 |
| R2.3 | Views: keys(), values(), items() | P1 |
| R2.4 | `dict \| dict` merge (PEP 584) | P1 |
| R2.5 | Dict comprehension, insertion order preservation | P1 |

### R3: Set Conformance Tests (#759)

| ID | Requirement | Priority |
|----|------------|----------|
| R3.1 | Mutation: add, discard, remove, pop, clear | P1 |
| R3.2 | Set algebra: union, intersection, difference, symmetric_difference | P1 |
| R3.3 | Set comprehension, frozenset basics | P1 |

### R4: Tuple & String Conformance Tests (#759)

| ID | Requirement | Priority |
|----|------------|----------|
| R4.1 | Tuple: unpacking, `*` unpacking, hashing, lexicographic comparison | P1 |
| R4.2 | String: split, join, strip, replace, find, format, f-string edge cases | P1 |

### R5: Exception Hierarchy (#755)

| ID | Requirement | Priority |
|----|------------|----------|
| R5.1 | Built-in exception classes present (BaseException tree) | P1 |
| R5.2 | `except` matching: subclass catching, tuple of exceptions | P1 |
| R5.3 | `raise from` — exception chaining (`__cause__`, `__context__`) | P1 |
| R5.4 | Custom exception subclassing, `args` attribute | P1 |
| R5.5 | ExceptionGroup / `except*` (PEP 654) — xfail | P1 |

### R6: Generator & Iterator Protocol (#756)

| ID | Requirement | Priority |
|----|------------|----------|
| R6.1 | `yield` / `yield from` semantics | P1 |
| R6.2 | `generator.send(value)` — resume with value | P1 |
| R6.3 | `generator.throw(exc)` — inject exception | P1 |
| R6.4 | `generator.close()` — GeneratorExit handling | P1 |
| R6.5 | `StopIteration.value` for return from generator | P1 |
| R6.6 | Iterator protocol: `__iter__`, `__next__`, StopIteration | P1 |
| R6.7 | Async generators — xfail | P1 |

### Constraints

- All 1745+ existing tests must continue to pass
- Use existing conformance harness (`tests/fixtures/conformance/*.py` + `.expected`)
- `# mamba-xfail` for unimplemented features
- Golden files generated from CPython 3.12
## Scenarios


### S1: List mutation and query conformance

```
Given conformance test files for list append, extend, insert, pop, remove, sort, reverse
When cargo test runs the conformance suite
Then all list mutation outputs match CPython 3.12 golden files
```

### S2: List slicing conformance

```
Given test files with a[1:3], a[::2], a[::-1], negative index slicing
When cargo test runs
Then slice results match CPython 3.12
```

### S3: Dict operations conformance

```
Given test files for dict get, setdefault, update, pop, keys/values/items, dict | merge
When cargo test runs
Then all dict operation outputs match CPython 3.12
```

### S4: Set algebra conformance

```
Given test files for set union, intersection, difference, symmetric_difference
When cargo test runs
Then set algebra results match CPython 3.12
```

### S5: Exception hierarchy and matching

```
Given test files for except subclass catching, tuple of exceptions, raise from chaining
When cargo test runs
Then exception behavior matches CPython 3.12
And ExceptionGroup/except* tests are marked xfail
```

### S6: Generator yield and send

```
Given test files for yield, yield from, generator.send(), StopIteration.value
When cargo test runs
Then working generators match CPython 3.12
And non-working features are xfail (not FAIL)
```

### S7: Iterator protocol

```
Given test files for __iter__, __next__, StopIteration
When cargo test runs
Then iterator protocol matches CPython 3.12
```
## Diagrams

### Sequence Diagram
<!-- TODO -->

### Flowchart
<!-- TODO -->

### Class Diagram
<!-- TODO -->

### State Diagram
<!-- TODO -->

### ERD
<!-- TODO -->

## API Spec

### OpenAPI 3.1
<!-- TODO -->

### OpenRPC 1.3
<!-- TODO -->

### AsyncAPI 2.6
<!-- TODO -->

### Serverless Workflow 0.8
<!-- TODO -->

## Test Plan


### Data Structure Tests

| Test ID | Description | Type |
|---------|------------|------|
| T1.1 | list: append, extend, insert, pop, remove, clear, copy | conformance |
| T1.2 | list: sort, reverse with various element types | conformance |
| T1.3 | list: slicing (positive, negative, step) | conformance |
| T1.4 | list: comprehension edge cases | conformance |
| T1.5 | list: __contains__, __eq__, lexicographic comparison | conformance |
| T2.1 | dict: get, setdefault with defaults | conformance |
| T2.2 | dict: update, pop, popitem, clear, copy | conformance |
| T2.3 | dict: keys(), values(), items() iteration | conformance |
| T2.4 | dict: `\|` merge operator (PEP 584) | conformance |
| T2.5 | dict: comprehension, insertion order | conformance |
| T3.1 | set: add, discard, remove, pop, clear | conformance |
| T3.2 | set: union, intersection, difference, symmetric_difference | conformance |
| T3.3 | set: comprehension, frozenset | conformance |
| T4.1 | tuple: unpacking, * unpacking, hashing, comparison | conformance |
| T4.2 | str: split, join, strip, replace, find, format | conformance |

### Exception Tests

| Test ID | Description | Type |
|---------|------------|------|
| T5.1 | Built-in exception hierarchy (isinstance checks) | conformance |
| T5.2 | except subclass matching, tuple of exceptions | conformance |
| T5.3 | raise from — __cause__, __context__ | conformance |
| T5.4 | Custom exception subclass, args attribute | conformance |
| T5.5 | ExceptionGroup / except* (xfail) | conformance |

### Generator & Iterator Tests

| Test ID | Description | Type |
|---------|------------|------|
| T6.1 | yield basic, yield from delegation | conformance |
| T6.2 | generator.send(value) | conformance |
| T6.3 | generator.throw(exc) | conformance |
| T6.4 | generator.close(), GeneratorExit | conformance |
| T6.5 | StopIteration.value from generator return | conformance |
| T6.6 | Iterator __iter__, __next__, StopIteration | conformance |
| T6.7 | Async generators (xfail) | conformance |
## Changes


### New Files

| File | Purpose |
|------|---------|
| `tests/fixtures/conformance/data_structures/list_methods.py` | List mutation/query methods conformance |
| `tests/fixtures/conformance/data_structures/list_slicing.py` | List slicing conformance |
| `tests/fixtures/conformance/data_structures/dict_methods.py` | Dict operations conformance |
| `tests/fixtures/conformance/data_structures/dict_comprehension.py` | Dict comprehension + merge |
| `tests/fixtures/conformance/data_structures/set_ops.py` | Set operations conformance |
| `tests/fixtures/conformance/data_structures/tuple_ops.py` | Tuple operations conformance |
| `tests/fixtures/conformance/data_structures/string_methods.py` | String methods conformance |
| `tests/fixtures/conformance/exceptions/hierarchy.py` | Exception hierarchy conformance |
| `tests/fixtures/conformance/exceptions/matching.py` | except matching conformance |
| `tests/fixtures/conformance/exceptions/chaining.py` | raise from chaining conformance |
| `tests/fixtures/conformance/exceptions/custom.py` | Custom exception subclassing |
| `tests/fixtures/conformance/exceptions/exception_group.py` | ExceptionGroup (xfail) |
| `tests/fixtures/conformance/generators/basic_yield.py` | Basic yield conformance |
| `tests/fixtures/conformance/generators/yield_from.py` | yield from conformance |
| `tests/fixtures/conformance/generators/send_throw.py` | send/throw/close conformance |
| `tests/fixtures/conformance/generators/stopiteration.py` | StopIteration.value conformance |
| `tests/fixtures/conformance/iterators/protocol.py` | Iterator protocol conformance |
| All above `.py` files have matching `.expected` golden files | Generated from CPython 3.12 |

### Modified Files

| File | Change |
|------|--------|
| `src/runtime/list_ops.rs` | Fix any list operation bugs found during conformance |
| `src/runtime/dict_ops.rs` | Fix any dict operation bugs found during conformance |
| `src/runtime/set_ops.rs` | Fix any set operation bugs found during conformance |
| `src/runtime/exception.rs` | Fix exception hierarchy/chaining bugs found during conformance |
| `src/runtime/generator.rs` | Fix generator protocol bugs found during conformance |

### Spec Updates

| Spec | Change |
|------|--------|
| `cclab-mamba/testing/mamba-py312-conformance.md` | Add P1 test categories and requirements |
# Reviews