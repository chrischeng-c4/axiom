---
id: mamba-core-test-coverage-spec
main_spec_ref: "cclab/specs/crates/mamba/testing/mamba-core-test-coverage-spec.md"
merge_strategy: new
filled_sections: [overview, requirements, scenarios, test-plan, changes]
fill_sections: [overview, requirements, scenarios, test-plan, changes]
create_complete: true
---

# Mamba Core Test Coverage Spec

## Overview

<!-- type: overview lang: markdown -->

Add comprehensive Rust unit and integration tests for four critically under-tested subsystems in `cclab-mamba`. Baseline: 434 tests, 7.1% line ratio. After this change: 800+ new tests across runtime, lower, resolve, and stdlib.

| Subsystem | LOC | Existing Tests | Target |
|-----------|-----|----------------|--------|
| `runtime/` | 35,483 | 49 integration | 500+ |
| `lower/` | 5,668 | 0 | 100+ |
| `resolve/` | 744 | 0 | 50+ |
| `stdlib/` top-10 | ~15,921 | 0 | 150+ |

**Test strategy**: Inline `#[cfg(test)]` modules co-located with source files. Cross-module scenarios go to `crates/mamba/tests/runtime_integration.rs`.

**Constraint**: No changes to non-test source logic. All tests must match Python 3.12 behavior (`cargo test -p mamba` passes with zero failures).

**In scope**: `runtime/value.rs`, `builtins.rs`, `class.rs`, `gc.rs`, `module.rs`, `string_ops.rs`, `list_ops.rs`, `dict_ops.rs`, `set_ops.rs`, `tuple_ops.rs`; `lower/ast_to_hir.rs`; `resolve/pass.rs`; `stdlib/` json, os, re, datetime, collections, pathlib, io, csv, hashlib, asyncio.

**Out of scope**: parser, types, codegen, lexer (deferred to follow-up).
## Requirements

<!-- type: requirements lang: markdown -->

### R1 — runtime/value.rs (target: 50+)

```yaml
id: R1
priority: high
```

- Round-trip tag encoding for every `MbValue` tag: INT, BOOL, NONE, FUNC, PTR, Float, Str, List, Dict, Set, Tuple, Bytes, Native
- BigInt overflow: values beyond i48 range promote to heap-allocated BigInt
- Edge cases: NaN, +Infinity, -Infinity, -0.0, i48::MIN, i48::MAX
- `mb_type()` returns correct type string for each tag

### R2 — runtime/builtins.rs (target: 100+)

```yaml
id: R2
priority: high
```

- Every builtin: `mb_print`, `mb_len`, `mb_int`, `mb_str`, `mb_float`, `mb_bool`, `mb_abs`, `mb_pow`, `mb_hash`, `mb_type`, `mb_isinstance`, `mb_getattr`/`setattr`/`delattr`, `mb_iter`/`next`, `mb_map`/`filter`/`zip`/`enumerate`, `mb_range`, `mb_sorted`/`reversed`
- Type coercion matrix: int↔float↔str↔bool for arithmetic and comparison builtins
- Error branches: TypeError, ValueError, AttributeError, StopIteration

### R3 — runtime/class.rs (target: 80+)

```yaml
id: R3
priority: high
```

- MRO C3 linearization: linear chains, diamond inheritance, multiple-inheritance
- `__init__` / `__new__` lifecycle ordering
- `super()` call dispatch: zero-arg and two-arg forms
- Descriptor protocol: `@property`, `@classmethod`, `@staticmethod`
- `__init_subclass__` hook invocation
- `__slots__` allocation and attribute restriction
- Metaclass `__call__` override

### R4 — runtime/gc.rs (target: 40+)

```yaml
id: R4
priority: medium
```

- Cycle detection: self-referential objects, mutual references, long reference chains
- Weak reference creation and invalidation after GC collection
- Destructor `__del__` ordering during cycle collection

### R5 — runtime/module.rs (target: 30+)

```yaml
id: R5
priority: medium
```

- `mb_import`: successful import, cached re-import, missing module error
- `mb_import_from`: attribute extraction, `import *` behavior
- Circular import detection raises ImportError
- Relative imports: `.foo`, `..bar` forms

### R6 — runtime/{string,list,dict,set,tuple}_ops.rs (target: 200+)

```yaml
id: R6
priority: high
```

- Every method on each type: happy path, empty input, boundary values, Unicode (string), type-error branch
- All behavior must match Python 3.12 semantics

### R7 — lower/ast_to_hir.rs (target: 100+)

```yaml
id: R7
priority: high
```

- Every lowering rule: function def, class def, decorator application, async/await, generator/yield
- All comprehension forms: list, dict, set, generator expressions
- Control flow: match/case, try/except/finally, with statement
- Import forms: import, import-from
- Assignment forms: augmented assignment, walrus operator (`:=`)
- Verify HIR output: correct node type and field values for each AST pattern

### R8 — resolve/pass.rs (target: 50+)

```yaml
id: R8
priority: medium
```

- LEGB scope chain: local, enclosing, global, builtin resolution order
- `nonlocal` and `global` declarations
- Class scope: name invisible in nested functions without `nonlocal`
- Closure variable capture (late-binding semantics)
- Comprehension scope per PEP 709 (implicit function scope)
- Star import namespace handling

### R9 — stdlib top-10 (target: 150+)

```yaml
id: R9
priority: high
```

- Modules: json, os, re, datetime, collections, pathlib, io, csv, hashlib, asyncio
- Per function: happy path + exception case + edge case + type coercion
- All expected outputs verified against CPython 3.12 behavior

### Constraints

| # | Constraint |
|---|------------|
| C1 | All tests compile and pass with `cargo test -p mamba` |
| C2 | No modifications to non-test source logic |
| C3 | Python 3.12 conformance for all stdlib/runtime behavioral assertions |
| C4 | Existing tests must not regress |
| C5 | Coverage measurement via `cargo-llvm-cov` (tool setup out of scope) |
## Scenarios

<!-- type: scenarios lang: markdown -->

### Scenario: MbValue INT round-trip [R1]

- **GIVEN** integer value `42`
- **WHEN** encoded as `MbValue::int(42)` and decoded via `as_int()`
- **THEN** returns `Some(42)`; `mb_type()` returns `"int"`

### Scenario: MbValue BigInt promotion [R1]

- **GIVEN** an integer exceeding i48 range (e.g. `2_i64.pow(48)`)
- **WHEN** encoded as `MbValue`
- **THEN** tag is PTR referencing a heap-allocated BigInt; `mb_type()` returns `"int"`

### Scenario: MbValue NaN and -0.0 edge cases [R1]

- **GIVEN** float values `f64::NAN`, `f64::INFINITY`, `-0.0_f64`
- **WHEN** each is stored and retrieved from `MbValue`
- **THEN** NaN stays NaN; infinity sign preserved; -0.0 equals 0.0 per Python semantics

### Scenario: mb_len on empty and non-empty List [R2]

- **GIVEN** a List MbValue with 3 elements, and an empty List
- **WHEN** `mb_len` is called on each
- **THEN** returns `MbValue::int(3)` and `MbValue::int(0)` respectively

### Scenario: mb_isinstance type check [R2]

- **GIVEN** an object `x` of user-defined class `MyClass`
- **WHEN** `mb_isinstance(x, MyClass)` and `mb_isinstance(x, int)` are called
- **THEN** first returns `True`, second returns `False`

### Scenario: mb_range produces correct sequence [R2]

- **GIVEN** call `mb_range(1, 6, 2)`
- **WHEN** iterated via `mb_iter` + `mb_next`
- **THEN** yields `1, 3, 5` then raises `StopIteration`

### Scenario: MRO C3 diamond inheritance [R3]

- **GIVEN** classes `A`, `B(A)`, `C(A)`, `D(B, C)`
- **WHEN** MRO is computed for `D`
- **THEN** linearization is `[D, B, C, A, object]`

### Scenario: @property descriptor get/set/delete [R3]

- **GIVEN** a class with `@property` defining getter, setter, and deleter for `x`
- **WHEN** `obj.x` is read, assigned, and deleted
- **THEN** each operation dispatches to the corresponding descriptor function

### Scenario: GC detects mutual reference cycle [R4]

- **GIVEN** two objects `a` and `b` with `a.ref = b` and `b.ref = a`; both otherwise unreachable
- **WHEN** GC cycle detection runs
- **THEN** both objects are collected; `__del__` called exactly once per object

### Scenario: Circular import raises ImportError [R5]

- **GIVEN** module A that imports module B, and module B that imports module A
- **WHEN** `mb_import("A")` is executed
- **THEN** raises `ImportError` with message containing `"circular import"`

### Scenario: str.split empty string [R6]

- **GIVEN** empty string `""`
- **WHEN** `.split()` is called
- **THEN** returns `[]` matching CPython 3.12 behavior

### Scenario: dict.update merges correctly [R6]

- **GIVEN** `d = {"a": 1}` and update source `{"b": 2, "a": 99}`
- **WHEN** `d.update(...)` is called
- **THEN** `d == {"a": 99, "b": 2}`

### Scenario: AST→HIR function def lowering [R7]

- **GIVEN** AST node `FunctionDef(name="foo", args=[arg("x")], body=[Return(Name("x"))])`
- **WHEN** `ast_to_hir` processes the node
- **THEN** emits `HirFunctionDef` with name `"foo"`, one parameter `"x"`, body containing `HirReturn(HirName("x"))`

### Scenario: AST→HIR walrus operator [R7]

- **GIVEN** AST node `NamedExpr(target=Name("x"), value=Constant(10))`
- **WHEN** `ast_to_hir` processes the node
- **THEN** emits `HirAssign` for `x = 10` followed by `HirName("x")` referencing the assigned binding

### Scenario: AST→HIR list comprehension [R7]

- **GIVEN** AST `ListComp(elt=Name("x"), generators=[comprehension(Name("x"), List([1,2,3]))])`
- **WHEN** lowered to HIR
- **THEN** emits an implicit function scope with iteration over the generator target

### Scenario: Resolve LEGB local shadows global [R8]

- **GIVEN** function with local `x = 1` and module-level `x = 99`
- **WHEN** name `x` is resolved inside the function
- **THEN** resolves to the local binding (value `1`), not the global

### Scenario: Resolve nonlocal capture [R8]

- **GIVEN** nested functions where inner declares `nonlocal x` and outer defines `x = 0`
- **WHEN** `x` is resolved in the inner function
- **THEN** binding resolves to the enclosing scope slot, not a new local

### Scenario: Comprehension has implicit scope [R8]

- **GIVEN** list comprehension `[i for i in range(3)]` at module level
- **WHEN** resolve pass processes it
- **THEN** `i` is scoped to the comprehension's implicit function scope, not module scope

### Scenario: json.dumps / json.loads round-trip [R9]

- **GIVEN** dict `{"key": [1, 2, 3], "flag": True}`
- **WHEN** `json.dumps()` then `json.loads()` is called
- **THEN** returns the original dict; output matches CPython 3.12

### Scenario: re.findall with capture groups [R9]

- **GIVEN** pattern `r"(\d+)-(\w+)"` and input `"42-hello 7-world"`
- **WHEN** `re.findall()` is called
- **THEN** returns `[("42", "hello"), ("7", "world")]`

### Scenario: hashlib.sha256 NIST known-answer vector [R9]

- **GIVEN** input `b"abc"`
- **WHEN** `hashlib.sha256(b"abc").hexdigest()` is called
- **THEN** returns `"ba7816bf8f01cfea414140de5dae2ec73b00361bbef0469031f09144d8ccb79a"`

### Scenario: collections.Counter most_common [R9]

- **GIVEN** `Counter(["a", "b", "a", "c", "a", "b"])`
- **WHEN** `.most_common(2)` is called
- **THEN** returns `[("a", 3), ("b", 2)]` matching CPython 3.12
## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan

<!-- type: test-plan lang: markdown -->

### Verification Steps

| Step | Command | Pass Criterion |
|------|---------|----------------|
| 1 | `cargo test -p mamba` | 0 failures; all new tests pass |
| 2 | `cargo test -p mamba 2>&1 \| grep -c 'test.*ok'` | Count ≥ 1234 (434 existing + 800+ new) |
| 3 | `cargo llvm-cov --package cclab-mamba` | Line ratio improves from 7.1% baseline |
| 4 | Per-subsystem count check (see table below) | All targets met |

### Per-Subsystem Test Count Targets

| Subsystem | File(s) | Minimum New Tests |
|-----------|---------|------------------|
| runtime/value | `value.rs` | 50 |
| runtime/builtins | `builtins.rs` | 100 |
| runtime/class | `class.rs` | 80 |
| runtime/gc | `gc.rs` | 40 |
| runtime/module | `module.rs` | 30 |
| runtime/ops | `string_ops`, `list_ops`, `dict_ops`, `set_ops`, `tuple_ops` | 200 |
| lower | `ast_to_hir.rs` | 100 |
| resolve | `pass.rs` | 50 |
| stdlib top-10 | 10 `*_mod.rs` files | 150 |

### CPython 3.12 Conformance

- For every test asserting a behavioral output, cross-reference against CPython 3.12
- Numeric results, string method outputs, and exception messages must match exactly
- Source of truth: CPython 3.12 REPL or `python3.12 -c "..."` invocation

### Regression Guard

- All 434 pre-existing tests must continue to pass
- No test moved to `#[ignore]` unless it was already ignored before this change
- `cargo test -p mamba -- --ignored` must show no newly ignored tests

### requirementDiagram

```mermaid
requirementDiagram
  requirement R1 { id: R1; text: "MbValue round-trips"; risk: medium; verifymethod: test }
  requirement R2 { id: R2; text: "builtins coverage"; risk: high; verifymethod: test }
  requirement R3 { id: R3; text: "class MRO/descriptors"; risk: high; verifymethod: test }
  requirement R4 { id: R4; text: "GC cycle detection"; risk: medium; verifymethod: test }
  requirement R5 { id: R5; text: "module import"; risk: medium; verifymethod: test }
  requirement R6 { id: R6; text: "ops methods"; risk: medium; verifymethod: test }
  requirement R7 { id: R7; text: "AST-to-HIR lowering"; risk: high; verifymethod: test }
  requirement R8 { id: R8; text: "resolve scope chain"; risk: medium; verifymethod: test }
  requirement R9 { id: R9; text: "stdlib top-10"; risk: medium; verifymethod: test }
  element cargo_test { type: "cargo test"; docref: "crates/cclab-mamba" }
  cargo_test - verifies -> R1
  cargo_test - verifies -> R2
  cargo_test - verifies -> R3
  cargo_test - verifies -> R4
  cargo_test - verifies -> R5
  cargo_test - verifies -> R6
  cargo_test - verifies -> R7
  cargo_test - verifies -> R8
  cargo_test - verifies -> R9
```
## Changes

<!-- type: changes lang: yaml -->

```yaml
files:
  # runtime/ — inline #[cfg(test)] additions
  - path: crates/mamba/src/runtime/value.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: 50+ tests for every MbValue tag round-trip, BigInt promotion, NaN/Inf/-0 edge cases"

  - path: crates/mamba/src/runtime/builtins.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: 100+ tests for every builtin function, type coercion matrix, TypeError/ValueError/StopIteration error branches"

  - path: crates/mamba/src/runtime/class.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: 80+ tests for MRO C3 linearization, __init__/__new__, super(), @property/@classmethod/@staticmethod, __init_subclass__, __slots__, metaclass __call__"

  - path: crates/mamba/src/runtime/gc.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: 40+ tests for cycle detection (self-ref, mutual-ref, long chain), weak ref invalidation, destructor ordering"

  - path: crates/mamba/src/runtime/module.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: 30+ tests for mb_import, mb_import_from, module cache, circular import detection, relative imports"

  - path: crates/mamba/src/runtime/string_ops.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: every str method (split/join/strip/replace/find/format/encode/...), empty string, unicode, boundary edge cases, Py3.12 semantics"

  - path: crates/mamba/src/runtime/list_ops.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: every list method (append/extend/insert/remove/pop/sort/reverse/...), empty list, boundary values, Py3.12 semantics"

  - path: crates/mamba/src/runtime/dict_ops.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: every dict method (get/update/pop/keys/values/items/setdefault/...), empty dict, key-error branches, Py3.12 semantics"

  - path: crates/mamba/src/runtime/set_ops.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: every set method (add/discard/union/intersection/difference/issubset/...), empty set, type-error branches, Py3.12 semantics"

  - path: crates/mamba/src/runtime/tuple_ops.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: every tuple method (count/index), unpacking, empty tuple, boundary values, Py3.12 semantics"

  # lower/ — inline #[cfg(test)] additions
  - path: crates/mamba/src/lower/ast_to_hir.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: 100+ tests for every AST→HIR lowering rule — function/class/decorator/async/await/generator/yield/comprehensions/match/try/with/import/augmented-assign/walrus"

  # resolve/ — inline #[cfg(test)] additions
  - path: crates/mamba/src/resolve/pass.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: 50+ tests for LEGB scope chain, nonlocal/global declarations, class scope quirks, closure capture, comprehension scope (PEP 709), star import"

  # stdlib top-10 — inline #[cfg(test)] additions
  - path: crates/mamba/src/runtime/stdlib/json_mod.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: json.dumps/loads round-trips, indent/sort_keys options, ValueError on invalid input, Py3.12 conformance"

  - path: crates/mamba/src/runtime/stdlib/os_mod.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: os.path.join/exists/basename/dirname, os.getcwd, os.environ get/set, error branches, Py3.12 conformance"

  - path: crates/mamba/src/runtime/stdlib/re_mod.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: re.match/search/findall/sub/compile, capture groups, flags (IGNORECASE/MULTILINE), error on bad pattern"

  - path: crates/mamba/src/runtime/stdlib/datetime_mod.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: datetime/date/time/timedelta construction, arithmetic, strftime/strptime, timezone handling, Py3.12 conformance"

  - path: crates/mamba/src/runtime/stdlib/collections_mod.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: deque (append/appendleft/rotate), defaultdict, OrderedDict (move_to_end), Counter (most_common/+/-), namedtuple — all methods and edge cases"

  - path: crates/mamba/src/runtime/stdlib/pathlib_mod.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: Path construction via / operator, .name/.stem/.suffix, .exists()/.is_file()/.is_dir(), Py3.12 semantics"

  - path: crates/mamba/src/runtime/stdlib/io_mod.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: BytesIO and StringIO read/write/seek/tell, closed-state error, context manager protocol, Py3.12 conformance"

  - path: crates/mamba/src/runtime/stdlib/csv_mod.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: reader/writer round-trips, dialect options, quoting modes, special character escaping, Py3.12 conformance"

  - path: crates/mamba/src/runtime/stdlib/hashlib_mod.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: md5/sha1/sha256/sha512 with NIST known-answer vectors, hexdigest/digest output, update() chaining"

  - path: crates/mamba/src/runtime/stdlib/asyncio_mod.rs
    action: MODIFY
    desc: "Add #[cfg(test)] module: asyncio.run, gather, sleep, create_task — basic async task execution and ordering tests"

  # integration tests
  - path: crates/mamba/tests/runtime_integration.rs
    action: CREATE
    desc: "Cross-module integration tests: end-to-end scenarios exercising runtime value system + stdlib + GC together (e.g., JSON encode/decode of complex objects, regex over unicode strings)"
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->

# Reviews
