---
id: cclab-mamba-fix-xfail-spec
main_spec_ref: cclab-mamba/testing/conformance.md
merge_strategy: append
filled_sections: [overview, requirements, scenarios, test_plan, changes]
---

# Cclab Mamba Fix Xfail Spec

## Overview

Fix all 7 remaining xfail conformance tests by implementing three major compiler/runtime features:

1. **Minimal class system** (#754) — `class Foo(Bar):`, `__init__`, instance creation, single inheritance, method dispatch, `super().__init__()`. Enough to support custom exception subclassing and custom iterators.

2. **Generator state machine** (#756) — Compile `yield` functions to heap-allocated state machines in Cranelift JIT. Support `__next__`, `send(value)`, `throw(exc)`, `close()`, `StopIteration.value`, `yield from` delegation.

3. **ExceptionGroup / except*** (#755) — PEP 654 `ExceptionGroup` class, `except*` syntax in parser/HIR/MIR/codegen, group splitting by type.

4. **Iterator protocol for user classes** (#756) — Custom `__iter__`/`__next__` methods dispatched through the class system.

Additionally close already-passing issues: #752 (test harness), #753 (arithmetic), #758 (builtins), #759 (data structures).

### Affected Layers

| Layer | Files | Change |
|-------|-------|--------|
| Parser | `stmt.rs`, `stmt_compound.rs` | `class` statement, `except*` syntax |
| AST | `ast.rs` | ClassDef node, ExceptStar handler variant |
| HIR | `hir/mod.rs` | HIR ClassDef, Yield, YieldFrom nodes |
| AST→HIR | `ast_to_hir.rs` | Lower class defs, generator detection |
| HIR→MIR | `hir_to_mir.rs` | Generator state machine transform, class MIR, except* lowering |
| MIR | `mir/mod.rs` | Generator state/resume instructions |
| Codegen | `cranelift/mod.rs`, `jit.rs` | Generator frame allocation, yield/resume, class construction |
| Runtime | `class.rs` | MbClass creation, MRO, instance creation, method dispatch |
| Runtime | `generator.rs` | MbGenerator state machine, send/throw/close |
| Runtime | `exception.rs` | ExceptionGroup, except* split/subgroup |
| Runtime | `iter.rs` | Custom iterator protocol dispatch |
| Runtime | `symbols.rs` | Register new mb_* functions |
## Requirements

### R1 - Minimal Class System

```yaml
id: R1
priority: high
affects: parser/stmt.rs, ast.rs, hir/mod.rs, lower/ast_to_hir.rs, lower/hir_to_mir.rs, codegen/cranelift/mod.rs, codegen/cranelift/jit.rs, runtime/class.rs
```

Parse `class Name(Base):` with method definitions and `__init__`. Lower to HIR ClassDef → MIR class construction sequence. At runtime, create `MbClass` with methods dict, single-inheritance MRO, and `mb_class_instantiate(class) -> Instance`. Support `super().__init__()` via MRO lookup.

Scope: single inheritance only, no metaclasses, no descriptors, no `__slots__`, no multiple inheritance.

### R2 - Generator State Machine Compilation

```yaml
id: R2
priority: high
affects: hir/mod.rs, lower/hir_to_mir.rs, mir/mod.rs, codegen/cranelift/mod.rs, codegen/cranelift/jit.rs, runtime/generator.rs
```

Functions containing `yield` are compiled to generator functions. Calling returns a generator object (no body execution). The generator body is transformed into a state machine with states for each yield point. Local variables are stored in a heap-allocated generator frame. `__next__()` resumes execution until the next yield or return.

Generator states: Created → Running → Suspended → Completed.

### R3 - Generator send/throw/close Protocol

```yaml
id: R3
priority: high
affects: runtime/generator.rs, codegen/cranelift/mod.rs
```

- `send(value)` — resume generator, yield expression evaluates to `value`
- `throw(exc)` — raise exception at yield point; if caught, resume to next yield
- `close()` — throw GeneratorExit; if generator yields after, raise RuntimeError
- `StopIteration.value` — carry return value from generator

### R4 - yield from Delegation

```yaml
id: R4
priority: high
affects: lower/hir_to_mir.rs, runtime/generator.rs
```

`yield from iterable` delegates to a sub-iterator. Values from the sub-iterator are yielded directly. `send()` and `throw()` are forwarded to the sub-iterator. The sub-iterator's return value becomes the value of the `yield from` expression.

### R5 - Custom Iterator Protocol

```yaml
id: R5
priority: high
affects: runtime/iter.rs, runtime/class.rs
```

User classes with `__iter__` and `__next__` methods participate in the iteration protocol. `mb_iter_new()` calls `__iter__` via MRO dispatch. `mb_iter_next()` calls `__next__` via MRO dispatch. For-loops work with custom iterators.

### R6 - Custom Exception Subclassing

```yaml
id: R6
priority: high
affects: runtime/exception.rs, runtime/class.rs
```

`class MyError(ValueError):` creates a custom exception class. Exception matching (`except MyError`) uses isinstance via MRO. Custom exception instances support `super().__init__(message)` and custom attributes (e.g., `self.code = code`).

### R7 - ExceptionGroup and except* Syntax

```yaml
id: R7
priority: high
affects: parser/stmt.rs, ast.rs, hir/mod.rs, lower/hir_to_mir.rs, runtime/exception.rs
```

PEP 654 support:
- `ExceptionGroup(message, [exc1, exc2, ...])` — groups multiple exceptions
- `except* TypeError as eg:` — matches sub-exceptions by type, splits group
- `eg.exceptions` — access matched sub-exceptions
- Unmatched exceptions propagate as a new ExceptionGroup
## Scenarios

### Scenario: Custom exception class with inheritance

- **GIVEN** `class MyError(ValueError): def __init__(self, msg, code): super().__init__(msg); self.code = code`
- **WHEN** `raise MyError("bad", 42)` is caught by `except ValueError as e:`
- **THEN** Handler executes, `e.code == 42`, `str(e) == "bad"`

### Scenario: Basic generator yield

- **GIVEN** `def gen(): yield 1; yield 2; yield 3`
- **WHEN** `list(gen())` is called
- **THEN** Returns `[1, 2, 3]`

### Scenario: Generator send value

- **GIVEN** `def echo(): while True: v = yield; print(v)`
- **WHEN** `g = echo(); next(g); g.send("hello")`
- **THEN** Prints `hello`

### Scenario: Generator throw exception

- **GIVEN** A generator suspended at yield
- **WHEN** `g.throw(ValueError, "bad")`
- **THEN** ValueError is raised at the yield point; if caught by try/except in generator, resumes

### Scenario: StopIteration.value from return

- **GIVEN** `def gen(): yield 1; return "done"`
- **WHEN** Generator exhausted
- **THEN** `StopIteration.value == "done"`

### Scenario: yield from delegation

- **GIVEN** `def outer(): result = yield from inner()`
- **WHEN** `inner()` yields values then returns
- **THEN** `outer()` yields same values, `result` is inner's return value

### Scenario: Custom iterator protocol

- **GIVEN** `class Counter: def __init__(self, n): self.n = n; self.i = 0; def __iter__(self): return self; def __next__(self): if self.i >= self.n: raise StopIteration; self.i += 1; return self.i`
- **WHEN** `list(Counter(3))` is called
- **THEN** Returns `[1, 2, 3]`

### Scenario: ExceptionGroup with except*

- **GIVEN** `raise ExceptionGroup("errors", [TypeError("t"), ValueError("v")])`
- **WHEN** `except* ValueError as eg:` handler
- **THEN** Handler receives ExceptionGroup with ValueError, TypeError propagates

### Scenario: xfail markers removed

- **GIVEN** All 7 xfail conformance test fixtures
- **WHEN** `# mamba-xfail` directives removed
- **THEN** All 33 tests pass (26 existing + 7 newly passing)
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

### Conformance Tests (xfail removal)

Remove `# mamba-xfail` from all 7 fixture files and verify they pass:

```bash
cargo test -p mamba --test conformance_tests
```

| Test | Validates | Requirements |
|------|-----------|-------------|
| `exceptions/custom.py` | Custom exception classes, inheritance, super().__init__(), custom attrs | R1, R6 |
| `exceptions/exception_group.py` | ExceptionGroup, except*, group splitting | R7 |
| `generators/basic_yield.py` | Basic yield, generator iteration, lazy evaluation | R2 |
| `generators/send_throw.py` | send(), throw(), close() protocol | R3 |
| `generators/stopiteration.py` | StopIteration.value, generator return | R2, R3 |
| `generators/yield_from.py` | yield from delegation, sub-iterator forwarding | R4 |
| `iterators/protocol.py` | Custom __iter__/__next__, for-loop integration | R1, R5 |

### Regression

```bash
cargo test -p mamba
```

All 26 currently-passing conformance tests must continue to pass.
## Changes

### New Files

| File | Responsibility | Requirement |
|------|---------------|-------------|
| `src/runtime/generator.rs` | Generator state machine, MbGenerator struct, send/throw/close | R2, R3, R4 |

### Modified Files

| File | Change | Requirement |
|------|--------|-------------|
| `src/parser/stmt.rs` | Parse `class Name(Base):` statements, `except*` syntax | R1, R7 |
| `src/ast.rs` | Add ClassDef, ExceptStar AST nodes | R1, R7 |
| `src/hir/mod.rs` | Add HIR ClassDef, Yield, YieldFrom nodes | R1, R2, R4 |
| `src/lower/ast_to_hir.rs` | Lower class defs, detect generator functions | R1, R2 |
| `src/lower/hir_to_mir.rs` | Generator state machine transform, class construction MIR, except* lowering | R1, R2, R4, R7 |
| `src/mir/mod.rs` | Add generator state/resume MIR instructions | R2 |
| `src/codegen/cranelift/mod.rs` | Emit generator frame alloc, yield/resume, class construction calls | R1, R2 |
| `src/codegen/cranelift/jit.rs` | Wire new mb_* symbols for generators, classes, ExceptionGroup | R1, R2, R7 |
| `src/runtime/class.rs` | MbClass creation for user classes, single-inheritance MRO, instance creation, super() | R1, R5, R6 |
| `src/runtime/exception.rs` | ExceptionGroup class, except* split/subgroup, custom exception support | R6, R7 |
| `src/runtime/iter.rs` | Custom iterator dispatch via __iter__/__next__ MRO lookup | R5 |
| `src/runtime/symbols.rs` | Register mb_generator_*, mb_class_*, mb_exception_group_* symbols | R1-R7 |
| `tests/fixtures/conformance/exceptions/custom.py` | Remove xfail marker | R6 |
| `tests/fixtures/conformance/exceptions/exception_group.py` | Remove xfail marker | R7 |
| `tests/fixtures/conformance/generators/basic_yield.py` | Remove xfail marker | R2 |
| `tests/fixtures/conformance/generators/send_throw.py` | Remove xfail marker | R3 |
| `tests/fixtures/conformance/generators/stopiteration.py` | Remove xfail marker | R2, R3 |
| `tests/fixtures/conformance/generators/yield_from.py` | Remove xfail marker | R4 |
| `tests/fixtures/conformance/iterators/protocol.py` | Remove xfail marker | R5 |
# Reviews