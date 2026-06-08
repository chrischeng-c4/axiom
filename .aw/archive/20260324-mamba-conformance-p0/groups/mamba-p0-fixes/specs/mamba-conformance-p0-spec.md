---
id: mamba-conformance-p0-spec
main_spec_ref: "crates/mamba/testing/mamba-py312-conformance.md"
merge_strategy: new
filled_sections: [overview, requirements, scenarios, test_plan, changes]
fill_sections: [overview, requirements, scenarios, test_plan, changes]
---

# Mamba Conformance P0 Spec

## Overview

<!-- type: overview lang: markdown -->

Fix 6 P0 conformance gaps that block real Python programs on the Mamba JIT backend. Three are SIGBUS crashes in Cranelift codegen, one is stdlib functions returning None due to incomplete module dispatch, and two are scope resolution bugs.

| ID | Bug | Root Cause | File |
|----|-----|-----------|------|
| P0-R1 | Lambda SIGBUS | Closure codegen in cranelift/mod.rs doesn't handle anonymous function signatures | codegen/cranelift/mod.rs |
| P0-R2 | With-statement SIGBUS | `__enter__`/`__exit__` calling convention mismatch in codegen | codegen/cranelift/mod.rs |
| P0-R3 | @decorator SIGBUS | Stacked decorator application emits invalid Cranelift IR | codegen/cranelift/mod.rs |
| P0-R4 | Stdlib functions return None | (a) Module Dict callable dispatch incomplete in class.rs; (b) CallExtern return slot discarded in hir_to_mir.rs | runtime/class.rs, lower/hir_to_mir.rs |
| P0-R5 | Comprehension scope leaks | resolve/pass.rs doesn't create isolated scope for list/dict/set comprehensions | resolve/pass.rs |
| P0-R6 | Walrus := wrong scope | NamedExpr in comprehensions binds to comprehension scope instead of enclosing function scope | resolve/pass.rs |

All 6 fixes are independent — no ordering dependency. Acceptance: all targeted xfail fixtures pass with `cargo test -p mamba --test conformance_tests`.
## Requirements

<!-- type: requirements lang: markdown -->

### P0-R1: Lambda SIGBUS Fix

| ID | Requirement | Affected Fixture |
|----|-------------|------------------|
| P0-R1.1 | `codegen/cranelift/mod.rs`: emit correct Cranelift function signature for lambda (anonymous) closures — current code omits closure environment parameter, causing calling-convention mismatch and SIGBUS at JIT call site | `language/lambda_expressions.py` |
| P0-R1.2 | Closure capture codegen: when lambda captures variables from enclosing scope, emit proper environment struct load before parameter access | `language/lambda_expressions.py` |

**Files**: `crates/mamba/src/codegen/cranelift/mod.rs`

### P0-R2: With-Statement SIGBUS Fix

| ID | Requirement | Affected Fixture |
|----|-------------|------------------|
| P0-R2.1 | `codegen/cranelift/mod.rs`: emit correct calling convention for `__enter__(self)` and `__exit__(self, exc_type, exc_val, exc_tb)` — current emission uses wrong parameter count causing SIGBUS | `language/context_managers.py` |
| P0-R2.2 | Exception propagation in `__exit__`: if `__exit__` returns truthy, suppress the exception; otherwise re-raise | `language/context_managers.py` |

**Files**: `crates/mamba/src/codegen/cranelift/mod.rs`

### P0-R3: @decorator SIGBUS Fix

| ID | Requirement | Affected Fixture |
|----|-------------|------------------|
| P0-R3.1 | `codegen/cranelift/mod.rs`: stacked decorator application must emit valid Cranelift IR — apply decorators bottom-up, each decorator call wraps the result of the previous | `decorator_full/decorator_full.py` |
| P0-R3.2 | Decorator with arguments: `@deco(arg)` must first call `deco(arg)` then apply the returned wrapper to the function | `decorator_full/decorator_full.py` |

**Files**: `crates/mamba/src/codegen/cranelift/mod.rs`

### P0-R4: Stdlib Functions Return None Fix

| ID | Requirement | Affected Fixtures |
|----|-------------|-------------------|
| P0-R4.1 | `runtime/class.rs` (`mb_call_method`): module Dict callable dispatch must handle module-level function objects — current code falls through to None for non-class callables | `stdlib/collections`, `stdlib/datetime`, `stdlib/hashlib`, `stdlib/itertools`, `stdlib/io`, `stdlib/pathlib`, `stdlib/random`, `stdlib/re`, `stdlib/struct`, `stdlib/math`, `stdlib/sys` |
| P0-R4.2 | `lower/hir_to_mir.rs`: CallExtern for module-level functions must store the return value to a register and propagate it — currently the return slot is discarded, causing None downstream | same as P0-R4.1 |

**Files**: `crates/mamba/src/runtime/class.rs`, `crates/mamba/src/lower/hir_to_mir.rs`

### P0-R5: Comprehension Scope Isolation Fix

| ID | Requirement | Affected Fixture |
|----|-------------|------------------|
| P0-R5.1 | `resolve/pass.rs`: list/dict/set comprehension must create an isolated scope for iteration variables — iteration variable bindings must not leak into the enclosing function scope | `language/comprehension_scope.py` |
| P0-R5.2 | Generator expressions must also use isolated scope (same mechanism) | `language/comprehension_scope.py` |

**Files**: `crates/mamba/src/resolve/pass.rs`

### P0-R6: Walrus Operator := Scope Fix (PEP 572)

| ID | Requirement | Affected Fixture |
|----|-------------|------------------|
| P0-R6.1 | `resolve/pass.rs`: when `:=` (NamedExpr) appears inside a comprehension, bind the target name in the enclosing **function** scope — walk up past comprehension and class scopes | `language/comprehension_scope.py` |
| P0-R6.2 | The walrus-bound variable must be visible after the comprehension expression completes in the enclosing scope | `language/comprehension_scope.py` |

**Files**: `crates/mamba/src/resolve/pass.rs`

### Constraints

- All existing passing conformance fixtures must continue to pass (no regressions)
- ExceptionGroup/except* remains xfailed (#755)
- Async generators remain xfailed (#800)
- Asyncio event loop remains xfailed (#801)
- Each retained xfail must carry `# mamba-xfail: <reason> (see #<issue>)` annotation
## Scenarios

<!-- type: scenarios lang: markdown -->

### Scenario: lambda closure compiles and executes

- **GIVEN** `adder = lambda x: lambda y: x + y; result = adder(3)(4)`
- **WHEN** compiled through Cranelift JIT backend
- **THEN** compilation succeeds without SIGBUS; `result == 7`

### Scenario: lambda with captured variable

- **GIVEN** `x = 10; f = lambda: x * 2; result = f()`
- **WHEN** executed
- **THEN** `result == 20` (closure captures `x` from enclosing scope)

### Scenario: with-statement context manager

- **GIVEN**
  ```python
  class CM:
      def __enter__(self): return self
      def __exit__(self, *args): return False
  with CM() as c:
      result = c
  ```
- **WHEN** compiled and executed
- **THEN** no SIGBUS; `__enter__` returns `self`; `__exit__` called on block exit

### Scenario: with-statement exception suppression

- **GIVEN**
  ```python
  class Suppress:
      def __enter__(self): return self
      def __exit__(self, *args): return True
  try:
      with Suppress():
          raise ValueError("suppressed")
      result = "ok"
  except ValueError:
      result = "leaked"
  ```
- **WHEN** executed
- **THEN** `result == "ok"` (`__exit__` returns True, exception suppressed)

### Scenario: stacked decorators

- **GIVEN**
  ```python
  def deco_a(f):
      def wrapper(*args): return f(*args) + " A"
      return wrapper
  def deco_b(f):
      def wrapper(*args): return f(*args) + " B"
      return wrapper
  @deco_a
  @deco_b
  def greet(): return "hello"
  result = greet()
  ```
- **WHEN** compiled and executed
- **THEN** no SIGBUS; `result == "hello B A"` (bottom-up application)

### Scenario: decorator with arguments

- **GIVEN**
  ```python
  def repeat(n):
      def decorator(f):
          def wrapper(*args): return f(*args) * n
          return wrapper
      return decorator
  @repeat(3)
  def say(): return "ha"
  result = say()
  ```
- **WHEN** compiled and executed
- **THEN** no SIGBUS; `result == "hahaha"`

### Scenario: stdlib module function returns value

- **GIVEN** `import itertools; result = list(itertools.chain([1, 2], [3, 4]))`
- **WHEN** executed
- **THEN** `result == [1, 2, 3, 4]` (not None)

### Scenario: stdlib collections functions return values

- **GIVEN** `from collections import OrderedDict; d = OrderedDict(a=1, b=2); result = list(d.keys())`
- **WHEN** executed
- **THEN** `result == ['a', 'b']` (not None)

### Scenario: comprehension scope isolation

- **GIVEN**
  ```python
  x = "outer"
  result = [x for x in range(3)]
  after = x
  ```
- **WHEN** executed
- **THEN** `result == [0, 1, 2]`; `after == "outer"` (comprehension `x` does not leak)

### Scenario: dict/set comprehension scope isolation

- **GIVEN**
  ```python
  k = "original"
  d = {k: k for k in range(3)}
  after = k
  ```
- **WHEN** executed
- **THEN** `d == {0: 0, 1: 1, 2: 2}`; `after == "original"`

### Scenario: walrus operator assigns to enclosing scope

- **GIVEN**
  ```python
  results = [y := x * 2 for x in range(3)]
  print(y)
  ```
- **WHEN** executed
- **THEN** prints `4` (last assigned value of `y` in enclosing scope); `results == [0, 2, 4]`

### Scenario: walrus in nested comprehension

- **GIVEN**
  ```python
  result = [y := x for x in range(5) if (z := x) > 2]
  ```
- **WHEN** executed
- **THEN** `result == [3, 4]`; `y == 4`; `z == 4` (both walrus targets in enclosing scope)

### Scenario: existing passing tests unaffected

- **GIVEN** full conformance suite
- **WHEN** `cargo test -p mamba --test conformance_tests` after all P0 fixes
- **THEN** all previously passing fixtures still pass; no regressions

### Scenario: retained xfails still skip

- **GIVEN** fixtures for ExceptionGroup/except* (#755), async generators (#800), asyncio (#801)
- **WHEN** conformance suite runs
- **THEN** these remain xfailed with `# mamba-xfail` markers intact
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

### Test Strategy

All tests run through the existing conformance harness (`conformance_tests.rs`): fixture `.py` → parse → type-check → HIR → MIR → Cranelift JIT → capture stdout → compare against `.expected` golden file. A fixture passes when its `# mamba-xfail` marker is removed and stdout matches the golden.

### TC-1: Lambda SIGBUS Fix (P0-R1)

| ID | Covers | Fixture |
|----|--------|---------|
| TC-1.1 | P0-R1.1 | `language/lambda_expressions.py` |
| TC-1.2 | P0-R1.2 | `language/lambda_expressions.py` |

**TC-1.1: Simple lambda compiles without SIGBUS**

- **GIVEN** fixture contains `square = lambda x: x * x; print(square(5))`
- **WHEN** compiled through Cranelift JIT backend and executed
- **THEN** no SIGBUS signal; stdout prints `25`; matches golden file

**TC-1.2: Nested lambda with closure capture**

- **GIVEN** fixture contains `adder = lambda x: lambda y: x + y; print(adder(3)(4))`
- **WHEN** compiled and executed
- **THEN** no SIGBUS; stdout prints `7`; closure environment parameter emitted correctly in Cranelift IR

**TC-1.3: Lambda capturing enclosing variable**

- **GIVEN** fixture contains `x = 10; f = lambda: x * 2; print(f())`
- **WHEN** compiled and executed
- **THEN** stdout prints `20`; environment struct load emitted before parameter access

### TC-2: With-Statement SIGBUS Fix (P0-R2)

| ID | Covers | Fixture |
|----|--------|---------|
| TC-2.1 | P0-R2.1 | `language/context_managers.py` |
| TC-2.2 | P0-R2.2 | `language/context_managers.py` |

**TC-2.1: Basic context manager protocol**

- **GIVEN** fixture defines a class with `__enter__(self)` returning `self` and `__exit__(self, *args)` returning `False`
- **WHEN** `with CM() as c:` is compiled and executed
- **THEN** no SIGBUS; `__enter__` called with correct 1-param convention; `__exit__` called with correct 4-param convention; body executes normally

**TC-2.2: Exception suppression via __exit__ returning True**

- **GIVEN** fixture defines a context manager where `__exit__` returns `True`, and body raises `ValueError`
- **WHEN** executed
- **THEN** exception is suppressed; execution continues past the `with` block; stdout confirms suppression (not the except branch)

**TC-2.3: Exception propagation when __exit__ returns False**

- **GIVEN** fixture defines a context manager where `__exit__` returns `False`, and body raises `ValueError`
- **WHEN** executed
- **THEN** exception propagates to enclosing `try/except`; `except ValueError` branch taken

### TC-3: Stacked Decorator SIGBUS Fix (P0-R3)

| ID | Covers | Fixture |
|----|--------|---------|
| TC-3.1 | P0-R3.1 | `decorator_full/decorator_full.py` |
| TC-3.2 | P0-R3.2 | `decorator_full/decorator_full.py` |

**TC-3.1: Stacked decorators apply bottom-up**

- **GIVEN** fixture defines `@deco_a @deco_b def greet(): return "hello"` where each decorator appends a suffix
- **WHEN** compiled and executed
- **THEN** no SIGBUS; `greet()` returns `"hello B A"` (bottom-up: `deco_b` wraps first, `deco_a` wraps result)

**TC-3.2: Decorator with arguments (two-stage application)**

- **GIVEN** fixture defines `@repeat(3) def say(): return "ha"` where `repeat(n)` returns a decorator that repeats the result
- **WHEN** compiled and executed
- **THEN** no SIGBUS; `say()` returns `"hahaha"`; `repeat(3)` called first to produce the decorator, then decorator applied to `say`

**TC-3.3: Identity decorator (no SIGBUS regression)**

- **GIVEN** fixture defines `@identity def add(a, b): return a + b`
- **WHEN** compiled and executed
- **THEN** no SIGBUS; `add(3, 4)` returns `7`; decorator returns function unchanged

### TC-4: Stdlib Functions Return None Fix (P0-R4)

| ID | Covers | Fixtures |
|----|--------|----------|
| TC-4.1 | P0-R4.1 | `stdlib/itertools/itertools_ops.py`, `stdlib/collections/collections_ops.py` |
| TC-4.2 | P0-R4.2 | All 11 stdlib fixtures below |

**TC-4.1: Module Dict callable dispatch (itertools.chain)**

- **GIVEN** fixture contains `import itertools; result = list(itertools.chain([1, 2], [3, 4])); print(result)`
- **WHEN** executed
- **THEN** stdout prints `[1, 2, 3, 4]` (not `None`); `mb_call_method` dispatches module-level function objects correctly

**TC-4.2: CallExtern return propagation (collections.OrderedDict)**

- **GIVEN** fixture contains `from collections import OrderedDict; d = OrderedDict(a=1, b=2); print(list(d.keys()))`
- **WHEN** executed
- **THEN** stdout prints `['a', 'b']` (not `None`); return value stored to register in MIR

**TC-4.3 – TC-4.11: Per-module stdlib return values**

Each of the following fixtures must produce non-None return values:

| TC | Fixture | Key assertion |
|----|---------|---------------|
| TC-4.3 | `stdlib/datetime/datetime_ops.py` | datetime constructor and method returns propagated |
| TC-4.4 | `stdlib/hashlib/hashlib_ops.py` | `hashlib.md5()` and `.hexdigest()` return values |
| TC-4.5 | `stdlib/io/io_ops.py` | `io.StringIO()` and `.getvalue()` return values |
| TC-4.6 | `stdlib/math/math_ops.py` | `math.sqrt()`, `math.floor()` return values |
| TC-4.7 | `stdlib/pathlib/pathlib_ops.py` | `Path()` constructor and method returns |
| TC-4.8 | `stdlib/random/random_ops.py` | `random.randint()` returns integer (not None) |
| TC-4.9 | `stdlib/re/pattern_matching.py` | `re.match()`, `re.search()` return match objects |
| TC-4.10 | `stdlib/struct/struct_ops.py` | `struct.pack()`, `struct.unpack()` return values |
| TC-4.11 | `stdlib/sys/sys_ops.py` | `sys.version`, `sys.platform` return strings |

- **GIVEN** each stdlib fixture calls module-level functions
- **WHEN** executed through JIT
- **THEN** stdout matches golden file; no `None` where a value is expected

### TC-5: Comprehension Scope Isolation (P0-R5)

| ID | Covers | Fixture |
|----|--------|---------|
| TC-5.1 | P0-R5.1 | `language/comprehension_scope.py` |
| TC-5.2 | P0-R5.2 | `language/comprehension_scope.py` |

**TC-5.1: List comprehension iteration variable does not leak**

- **GIVEN** fixture contains `x = "outer"; result = [x for x in range(3)]; print(x)`
- **WHEN** executed
- **THEN** stdout prints `outer` (not `2`); comprehension `x` is scoped to the comprehension body

**TC-5.2: Dict comprehension iteration variable does not leak**

- **GIVEN** fixture contains `k = "original"; d = {k: k for k in range(3)}; print(k)`
- **WHEN** executed
- **THEN** stdout prints `original` (not `2`); dict comprehension `k` isolated

**TC-5.3: Set comprehension scope isolation**

- **GIVEN** fixture contains `v = "keep"; s = {v for v in range(5)}; print(v)`
- **WHEN** executed
- **THEN** stdout prints `keep`; set comprehension `v` isolated

**TC-5.4: Generator expression scope isolation**

- **GIVEN** fixture contains `n = "outer"; total = sum(n for n in range(4)); print(n)`
- **WHEN** executed
- **THEN** stdout prints `outer`; generator expression `n` isolated

### TC-6: Walrus Operator := Scope Fix (P0-R6)

| ID | Covers | Fixture |
|----|--------|---------|
| TC-6.1 | P0-R6.1 | `language/comprehension_scope.py` |
| TC-6.2 | P0-R6.2 | `language/comprehension_scope.py` |

**TC-6.1: Walrus inside list comprehension binds to enclosing function scope**

- **GIVEN** fixture contains `results = [y := x * 2 for x in range(3)]; print(y)`
- **WHEN** executed
- **THEN** stdout prints `4` (last `:=` assignment); `y` bound in enclosing scope, not comprehension scope

**TC-6.2: Walrus target visible after comprehension**

- **GIVEN** fixture contains `[z := i for i in range(5)]; print(z)`
- **WHEN** executed
- **THEN** stdout prints `4`; `z` accessible in enclosing scope after comprehension completes

**TC-6.3: Walrus in comprehension filter clause**

- **GIVEN** fixture contains `result = [y := x for x in range(5) if (z := x) > 2]; print(y, z)`
- **WHEN** executed
- **THEN** stdout prints `4 4`; both `y` and `z` walrus targets in enclosing scope

### TC-7: Regression Gate

| ID | Covers | Scope |
|----|--------|-------|
| TC-7.1 | All | Full crate test suite |
| TC-7.2 | Constraints | Retained xfails |

**TC-7.1: No regressions in existing passing tests**

- **GIVEN** all P0 fixes applied to `codegen/cranelift/mod.rs`, `runtime/class.rs`, `lower/hir_to_mir.rs`, `resolve/pass.rs`
- **WHEN** `cargo test -p mamba` runs the full test suite
- **THEN** all previously passing tests still pass; exit code 0

**TC-7.2: Retained xfails remain skipped**

- **GIVEN** fixtures for ExceptionGroup/except* (#755), async generators (#800), asyncio event loop (#801) carry `# mamba-xfail: <reason> (see #<issue>)` markers
- **WHEN** conformance suite runs
- **THEN** these fixtures are skipped by the harness (printed as `[xfail]`); no attempt to compile or execute

### Verification Commands

```bash
# Full conformance suite
cargo test -p mamba --test conformance_tests

# P0-R1: lambda SIGBUS
cargo test -p mamba --test conformance_tests language::lambda

# P0-R2: with-statement SIGBUS
cargo test -p mamba --test conformance_tests language::context_managers

# P0-R3: decorator SIGBUS
cargo test -p mamba --test conformance_tests decorator_full

# P0-R4: stdlib return propagation
cargo test -p mamba --test conformance_tests stdlib

# P0-R5 + P0-R6: scope fixes
cargo test -p mamba --test conformance_tests language::comprehension_scope

# Full regression gate
cargo test -p mamba
```

### Expected Xfail Elimination Summary

| Requirement | Fixtures Unblocked | Before | After |
|-------------|-------------------|--------|-------|
| P0-R1 | `language/lambda_expressions.py` | xfail (SIGBUS) | pass |
| P0-R2 | `language/context_managers.py` | xfail (SIGBUS) | pass |
| P0-R3 | `decorator_full/decorator_full.py` | xfail (SIGBUS) | pass |
| P0-R4 | 11 stdlib fixtures | xfail (None returns) | pass |
| P0-R5 | `language/comprehension_scope.py` | xfail (scope leak) | pass |
| P0-R6 | `language/comprehension_scope.py` (walrus) | xfail (wrong scope) | pass |
| — | ExceptionGroup (#755), async gen (#800), asyncio (#801) | xfail | xfail (retained) |
# P0-R1: lambda SIGBUS
cargo test -p mamba --test conformance_tests language::lambda

# P0-R2: with-statement SIGBUS
cargo test -p mamba --test conformance_tests language::context_managers

# P0-R3: decorator SIGBUS
cargo test -p mamba --test conformance_tests decorator_full

# P0-R4: stdlib return propagation
cargo test -p mamba --test conformance_tests stdlib

# P0-R5 + P0-R6: scope fixes
cargo test -p mamba --test conformance_tests language::comprehension_scope
```

### Regression Gate

```bash
cargo test -p mamba
```

All existing tests must pass. No regressions permitted.
## Changes

<!-- type: changes lang: yaml -->

```yaml
files:
  # P0-R1/R2/R3: Cranelift codegen — SIGBUS fixes for lambda, with-statement, decorator
  - path: crates/mamba/src/codegen/cranelift/mod.rs
    action: MODIFY
    desc: |
      P0-R1: Fix lambda/closure codegen — emit correct function signature with closure
      environment parameter for anonymous functions. Handle captured variable loads.
      P0-R2: Fix with-statement codegen — emit correct calling convention for
      __enter__(self) and __exit__(self, exc_type, exc_val, exc_tb). Wire exception
      suppression based on __exit__ return value.
      P0-R3: Fix stacked decorator codegen — apply decorators bottom-up, each decorator
      call wraps previous result. Handle @decorator(args) two-stage application.
    requires: [P0-R1.1, P0-R1.2, P0-R2.1, P0-R2.2, P0-R3.1, P0-R3.2]

  # P0-R4: Stdlib functions return None — module dispatch fix
  - path: crates/mamba/src/runtime/class.rs
    action: MODIFY
    desc: |
      Fix mb_call_method module Dict callable dispatch: when callee is a module-level
      function object (not a class method), dispatch through the function call path
      instead of falling through to None.
    requires: [P0-R4.1]

  - path: crates/mamba/src/lower/hir_to_mir.rs
    action: MODIFY
    desc: |
      Fix CallExtern result propagation for module-level function calls: store the
      JIT return value to a register and propagate it instead of discarding the
      return slot.
    requires: [P0-R4.2]

  # P0-R5/R6: Scope resolution fixes
  - path: crates/mamba/src/resolve/pass.rs
    action: MODIFY
    desc: |
      P0-R5: Create isolated scope for list/dict/set/generator comprehensions —
      iteration variable bindings must not leak into enclosing function scope.
      P0-R6: When := (NamedExpr) appears inside a comprehension, walk up past
      comprehension and class scopes to bind the target in the enclosing function
      scope (PEP 572).
    requires: [P0-R5.1, P0-R5.2, P0-R6.1, P0-R6.2]

  # Xfail marker updates
  - path: crates/mamba/tests/fixtures/conformance/language/lambda_expressions.py
    action: MODIFY
    desc: Remove mamba-xfail marker after P0-R1 fix.
  - path: crates/mamba/tests/fixtures/conformance/language/context_managers.py
    action: MODIFY
    desc: Remove mamba-xfail marker after P0-R2 fix.
  - path: crates/mamba/tests/fixtures/conformance/decorator_full/decorator_full.py
    action: MODIFY
    desc: Remove mamba-xfail marker after P0-R3 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/itertools/itertools_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after P0-R4 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/io/io_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after P0-R4 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/pathlib/pathlib_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after P0-R4 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/random/random_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after P0-R4 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/re/pattern_matching.py
    action: MODIFY
    desc: Remove mamba-xfail marker after P0-R4 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/struct/struct_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after P0-R4 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/collections/collections_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after P0-R4 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/datetime/datetime_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after P0-R4 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/hashlib/hashlib_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after P0-R4 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/math/math_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after P0-R4 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/sys/sys_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after P0-R4 fix.
  - path: crates/mamba/tests/fixtures/conformance/language/comprehension_scope.py
    action: MODIFY
    desc: Remove mamba-xfail markers for scope leak and walrus after P0-R5/R6 fixes.
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