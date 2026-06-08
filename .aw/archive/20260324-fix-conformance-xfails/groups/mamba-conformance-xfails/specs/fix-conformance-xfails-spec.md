---
id: fix-conformance-xfails-spec
main_spec_ref: "crates/mamba/testing/mamba-py312-conformance.md"
merge_strategy: merge
filled_sections: [overview, requirements, scenarios, test-plan, changes]
fill_sections: [overview]
create_complete: true
---

# Fix Conformance Xfails Spec

## Overview

## Overview

Fix all 31 active conformance xfails in the cclab-mamba crate so every affected fixture produces output identical to CPython 3.12. Xfails span four root-cause categories:

| Category | Fixtures | Root Cause |
|----------|----------|------------|
| Codegen IR bugs | 4 | Calling-convention mismatches for classmethod, descriptor `__get__`, getattr/setattr/delattr, super() in `codegen/cranelift/mod.rs` |
| Runtime bugs | 14 | (a) Module function CallExtern result not propagated in `hir_to_mir.rs`; (b) bytes/bytearray methods incomplete; (c) exception chaining `__cause__`/`__context__` missing; (d) stdlib output divergence |
| Parser gaps | 2 | Lexer lacks re-entrant f-string state for nested f-strings (PEP 701); metaclass= keyword not recognized |
| Compiler/scope bugs | 5 | Walrus `:=` assigns to comprehension scope instead of enclosing scope (`resolve/pass.rs`); integer literal patterns emit wrong values in match lowering; type-checker rejects valid multi-arg stdlib forms |

Three xfails remain intentionally xfailed as genuinely unimplemented features: ExceptionGroup/except* (#755), async generators (#800), asyncio event loop (#801).

All fixes must preserve existing passing tests. Acceptance criterion: `cclab mamba test --conformance` with all 31 targeted fixtures passing.
## Requirements

## Requirements

### R1: Codegen IR — Class System Calling-Convention Fixes

| ID | Requirement | Affected Fixture |
|----|-------------|------------------|
| R1.1 | `classmethod` codegen: emit correct function signature with `cls` as first parameter; current emission uses wrong parameter count causing call-convention mismatch | `builtins/functional.py` |
| R1.2 | Descriptor `__get__` codegen: emit signature with `(self, obj, objtype)` parameter order; current code emits wrong count | `class_system/descriptors.py` |
| R1.3 | `getattr`/`setattr`/`delattr` codegen: generate valid Cranelift IR; current emission produces instructions that fail the Cranelift verifier | `builtins/object_protocol.py` |
| R1.4 | `super()` codegen: eliminate duplicate function-definition emission; current code defines the same symbol twice causing linker conflict | `class_system/super_call.py` |
| R1.5 | Stacked decorators with global state: fix SIGBUS crash in JIT codegen; root cause same calling-convention area as R1.1 | `decorator_full/decorator_full.py` |

**Files**: `codegen/cranelift/mod.rs`

### R2: Runtime — Module Function CallExtern Return Propagation

| ID | Requirement | Affected Fixtures |
|----|-------------|-------------------|
| R2.1 | `hir_to_mir.rs` CallExtern for module-level functions: the return value from the JIT call must be stored to a register and propagated; currently the return slot is discarded, causing `None` to be used downstream | `stdlib/itertools`, `stdlib/io`, `stdlib/pathlib`, `stdlib/random`, `stdlib/re`, `stdlib/struct` |

**Files**: `lower/hir_to_mir.rs`

### R3: Runtime — bytes/bytearray Method Implementations

| ID | Requirement | Priority |
|----|-------------|----------|
| R3.1 | `bytes.replace(old, new[, count])` — implement with correct semantics matching CPython | P1 |
| R3.2 | `bytes.strip([chars])`/`bytes.lstrip`/`bytes.rstrip` — implement ASCII strip | P1 |
| R3.3 | `bytes.startswith(prefix)` and `bytes.endswith(suffix)` — implement with tuple-of-prefixes support | P1 |
| R3.4 | Same three methods for `bytearray` | P1 |

**Files**: `runtime/bytes.rs` (or equivalent bytes runtime)

### R4: Runtime — Exception Chaining

| ID | Requirement | Note |
|----|-------------|------|
| R4.1 | `raise X from Y` populates `X.__cause__ = Y` and `X.__suppress_context__ = True` | Active xfail |
| R4.2 | Implicit chaining in `except` handler: `X.__context__` is set to the active exception | Active xfail |
| R4.3 | ExceptionGroup/except* — remains `# mamba-xfail` (see #755) | Intentional skip |

**Files**: `runtime/exception.rs`, `lower/hir_to_mir.rs` (raise-from lowering)

### R5: Parser — Nested F-Strings (PEP 701)

| ID | Requirement |
|----|-------------|
| R5.1 | Lexer (`lexer/mod.rs`): support re-entrant f-string tokenization. When a `{` is encountered inside an `FStr` token, the lexer must recursively lex inner expressions, allowing `f"{f'{x}'}"` to produce correctly nested tokens |
| R5.2 | Parser (`parser/expr_compound.rs`): `parse_fstring_parts` handles nested `FStr` tokens within interpolation — produces `FStringPart::Expr(Expr::FString(...))` for nested cases |

**Files**: `lexer/mod.rs`, `parser/expr_compound.rs`

### R6: Parser — metaclass Keyword in Class Declaration

| ID | Requirement |
|----|-------------|
| R6.1 | `parser/stmt_compound.rs`: recognize `metaclass=<expr>` as a keyword argument in the class declaration base-list; store in `ClassDef.metaclass: Option<Expr>` |
| R6.2 | AST/HIR propagation: pass metaclass through `ast_to_hir.rs` into HIR ClassDef; no codegen required (metaclass application can be a stub returning the class unchanged for now) |

**Files**: `parser/stmt_compound.rs`, `ast.rs`, `hir/mod.rs`, `lower/ast_to_hir.rs`

### R7: Compiler — Walrus Operator Scope (PEP 572)

| ID | Requirement |
|----|-------------|
| R7.1 | `resolve/pass.rs`: when `:=` appears inside a comprehension, bind the target name in the enclosing **function** scope (skipping comprehension and class scopes), not in the comprehension's own scope |
| R7.2 | Verify the binding is visible after the comprehension expression completes |

**Files**: `resolve/pass.rs`

### R8: Compiler — Integer Literal Patterns in Match

| ID | Requirement |
|----|-------------|
| R8.1 | `lower/hir_to_mir.rs` pattern-matching lowering (R3 in hir-to-mir spec): integer literal patterns must emit the correct integer value for comparison; current code emits wrong constant |

**Files**: `lower/hir_to_mir.rs`

### R9: Type Checker — Multi-Argument Stdlib Forms

| ID | Requirement | Affected Fixture |
|----|-------------|------------------|
| R9.1 | Accept `next(iterator, default)` 2-argument form | `builtins/iteration.py` |
| R9.2 | Accept `generator.throw(exc_type, value, traceback)` 3-argument form | `language/generators.py` |
| R9.3 | Accept type annotations used in `json.loads`/`json.dumps`, `functools.lru_cache`, `csv.DictWriter` | `stdlib/json`, `stdlib/functools`, `stdlib/csv` |

**Files**: type-checker annotation validator (exact file TBD during implementation)

### R10: Runtime — Stdlib Output Divergence

| ID | Requirement | Affected Fixture |
|----|-------------|------------------|
| R10.1 | `datetime` module output matches CPython 3.12 format | `stdlib/datetime` |
| R10.2 | `hashlib` digest output matches CPython 3.12 | `stdlib/hashlib` |
| R10.3 | `math` function outputs match CPython 3.12 (edge cases: inf, nan, precision) | `stdlib/math` |
| R10.4 | `sys` module attribute values match CPython 3.12 | `stdlib/sys` |
| R10.5 | `os` module output matches CPython 3.12 | `stdlib/os` |
| R10.6 | `collections` module output matches CPython 3.12 | `stdlib/collections` |

**Files**: corresponding runtime wrapper modules

### Constraints

- ExceptionGroup/except* remains xfailed (see #755)
- async generators remain xfailed (see #800)
- asyncio event loop remains xfailed (see #801)
- All 40+ existing passing conformance fixtures must continue to pass
- Each retained xfail must carry `# mamba-xfail: <reason> (see #<issue>)` annotation
## Scenarios

## Scenarios

### Scenario: classmethod signature fix

- **GIVEN** `class Foo: @classmethod\n  def create(cls): return cls()`
- **WHEN** compiled through Cranelift backend
- **THEN** compilation succeeds; `Foo.create()` returns a `Foo` instance

### Scenario: descriptor __get__ fix

- **GIVEN** a descriptor class `class D: def __get__(self, obj, objtype=None): return 42` attached as a class attribute
- **WHEN** `instance.attr` triggers the descriptor protocol
- **THEN** `__get__` is called with `(self, instance, Foo)` and returns `42` without IR verification failure

### Scenario: getattr/setattr/delattr IR fix

- **GIVEN** `getattr(obj, 'x')`, `setattr(obj, 'x', 1)`, `delattr(obj, 'x')`
- **WHEN** compiled and executed
- **THEN** Cranelift IR verifier passes; runtime reads, writes, and deletes the attribute correctly

### Scenario: super() deduplication fix

- **GIVEN** `class Child(Parent): def __init__(self): super().__init__()`
- **WHEN** compiled
- **THEN** no duplicate symbol definitions in IR; `Child()` invokes `Parent.__init__` correctly

### Scenario: module function return propagation

- **GIVEN** `import itertools; result = list(itertools.chain([1, 2], [3, 4]))`
- **WHEN** executed
- **THEN** `result == [1, 2, 3, 4]` (chain object iterable, not `None`)

### Scenario: bytes replace method

- **GIVEN** `b"hello world".replace(b"world", b"mamba")`
- **WHEN** executed
- **THEN** returns `b"hello mamba"`

### Scenario: bytes strip/startswith/endswith

- **GIVEN** `b"  hi  ".strip()`, `b"hello".startswith(b"he")`, `b"hello".endswith(b"lo")`
- **WHEN** executed
- **THEN** returns `b"hi"`, `True`, `True` respectively

### Scenario: exception chaining __cause__

- **GIVEN**
  ```python
  try:
      int("x")
  except ValueError as orig:
      raise RuntimeError("wrapped") from orig
  ```
- **WHEN** `RuntimeError.__cause__` is accessed in the except handler
- **THEN** `__cause__ is orig` and `__suppress_context__ is True`

### Scenario: exception implicit context

- **GIVEN**
  ```python
  try:
      int("x")
  except ValueError:
      raise RuntimeError("inner")
  ```
- **WHEN** `RuntimeError.__context__` is accessed
- **THEN** `__context__` is the `ValueError` instance

### Scenario: nested f-string lexing

- **GIVEN** `f"outer {f'inner {x}'} end"`
- **WHEN** lexed and parsed
- **THEN** produces `FString([Literal("outer "), Expr(FString([Literal("inner "), Expr(x)])), Literal(" end")])`

### Scenario: metaclass keyword parsing

- **GIVEN** `class Meta(type): pass\nclass Foo(object, metaclass=Meta): pass`
- **WHEN** parsed
- **THEN** `ClassDef(name="Foo", bases=[object], metaclass=Meta)` with no parse error

### Scenario: walrus operator assigns to enclosing scope

- **GIVEN**
  ```python
  results = [y := x * 2 for x in range(3)]
  print(y)
  ```
- **WHEN** executed
- **THEN** prints `4` (last assigned value of `y` in enclosing scope); `results == [0, 2, 4]`

### Scenario: integer literal pattern match

- **GIVEN**
  ```python
  val = 1
  match val:
      case 0: print("zero")
      case 1: print("one")
      case _: print("other")
  ```
- **WHEN** executed
- **THEN** prints `"one"` (correct integer comparison)

### Scenario: next() with default

- **GIVEN** `next(iter([]), 42)`
- **WHEN** type-checked and executed
- **THEN** type checker accepts the 2-arg form; returns `42`

### Scenario: generator.throw() 3-arg form

- **GIVEN**
  ```python
  def gen():
      try:
          yield
      except ValueError:
          yield "caught"
  g = gen(); next(g)
  result = g.throw(ValueError, "msg", None)
  ```
- **WHEN** type-checked and executed
- **THEN** type checker accepts 3-arg `throw`; `result == "caught"`

### Scenario: retained xfails still skip

- **GIVEN** fixtures `stdlib/asyncio/asyncio_ops.py`, `language/exceptions.py` (ExceptionGroup section)
- **WHEN** conformance suite runs
- **THEN** these remain xfailed with their `# mamba-xfail` markers intact
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

## Test Plan

### Conformance Suite

Run after each category fix to verify xfail count decreases:

```bash
cargo test -p mamba --test conformance_tests
```

### Expected Xfail Elimination by Requirement

| Requirement | Fixtures Unblocked | Expected Outcome |
|-------------|-------------------|------------------|
| R1 (Codegen IR) | `functional.py`, `descriptors.py`, `object_protocol.py`, `super_call.py`, `decorator_full.py` | 5 xfails → pass |
| R2 (CallExtern return) | `itertools_ops.py`, `io_ops.py`, `pathlib_ops.py`, `random_ops.py`, `pattern_matching.py` (re), `struct_ops.py` | 6 xfails → pass |
| R3 (bytes methods) | `bytes_ops.py` | 1 xfail → pass |
| R4 (exception chaining) | `exceptions.py` (chaining section) | 1 xfail → pass |
| R5 (nested f-strings) | `fstring_advanced.py` | 1 xfail → pass |
| R6 (metaclass keyword) | `inheritance.py` | 1 xfail → pass |
| R7 (walrus scope) | `comprehension_scope.py` | 1 xfail → pass |
| R8 (match int patterns) | `pattern_matching.py` (language) | 1 xfail → pass |
| R9 (type checker) | `iteration.py`, `generators.py` (throw section), `json_encode_decode.py`, `functools_ops.py`, `csv_ops.py` | 5 xfails → pass |
| R10 (stdlib divergence) | `datetime_ops.py`, `hashlib_ops.py`, `math_ops.py`, `sys_ops.py`, `os_ops.py`, `collections_ops.py` | 6 xfails → pass |

**Total targeted**: 28 xfails eliminated (3 intentional remain: asyncio #801, ExceptionGroup #755, async generators #800)

### Regression Gate

```bash
cargo test -p mamba
```

All existing tests must pass. No regressions permitted.

### Per-Category Verification

```bash
# R1: codegen IR fixes
cargo test -p mamba --test conformance_tests class_system
cargo test -p mamba --test conformance_tests builtins::functional
cargo test -p mamba --test conformance_tests builtins::object_protocol
cargo test -p mamba --test conformance_tests decorator_full

# R2: module return propagation
cargo test -p mamba --test conformance_tests stdlib

# R7-R8: compiler/scope
cargo test -p mamba --test conformance_tests language::comprehension_scope
cargo test -p mamba --test conformance_tests language::pattern_matching
```
## Changes

## Changes

```yaml
files:
  # R1: Codegen IR — calling-convention and duplicate-symbol fixes
  - path: crates/mamba/src/codegen/cranelift/mod.rs
    action: MODIFY
    desc: |
      Fix classmethod signature emission (R1.1): emit correct param count with cls as first param.
      Fix descriptor __get__ signature (R1.2): emit (self, obj, objtype) parameter order.
      Fix getattr/setattr/delattr IR (R1.3): generate valid Cranelift instructions that pass verifier.
      Fix super() deduplication (R1.4): guard against re-emitting symbol already defined in module.
      Fix stacked-decorator SIGBUS (R1.5): same calling-convention area as R1.1.
    requires: [R1.1, R1.2, R1.3, R1.4, R1.5]

  # R2: Runtime — module function CallExtern return propagation
  - path: crates/mamba/src/lower/hir_to_mir.rs
    action: MODIFY
    desc: |
      Fix CallExtern result propagation for module-level function calls: store return value to
      a register and propagate it, instead of discarding. Fixes all 6 stdlib return-None xfails.
      Also fix integer literal pattern lowering (R8): emit correct constant value for integer
      patterns in match/case dispatch.
    requires: [R2.1, R8.1]

  # R3: Runtime — bytes/bytearray method implementations
  - path: crates/mamba/src/runtime/bytes.rs
    action: MODIFY
    desc: |
      Implement mb_bytes_replace(bytes, old, new, count), mb_bytes_strip(bytes, chars),
      mb_bytes_lstrip, mb_bytes_rstrip, mb_bytes_startswith(bytes, prefix),
      mb_bytes_endswith(bytes, suffix) with tuple-of-prefixes support.
      Apply same implementations for bytearray variants.
    requires: [R3.1, R3.2, R3.3, R3.4]

  # R4: Runtime — exception chaining
  - path: crates/mamba/src/runtime/exception.rs
    action: MODIFY
    desc: |
      Populate __cause__ and __suppress_context__ when raise-from executes.
      Populate __context__ for implicit chaining inside except handlers.
    requires: [R4.1, R4.2]

  - path: crates/mamba/src/lower/hir_to_mir.rs
    action: MODIFY
    desc: Emit mb_exception_set_cause / mb_exception_set_context calls in raise-from lowering.
    requires: [R4.1, R4.2]

  # R5: Parser — nested f-strings (PEP 701)
  - path: crates/mamba/src/lexer/mod.rs
    action: MODIFY
    desc: |
      Add re-entrant f-string tokenization: when { is encountered inside an FStr, recursively
      lex inner expressions tracking brace depth per nesting level. Allows f"{f'{x}'}" to
      produce correctly nested FStr tokens.
    requires: [R5.1]

  - path: crates/mamba/src/parser/expr_compound.rs
    action: MODIFY
    desc: |
      Update parse_fstring_parts to handle nested FStr tokens within interpolation segments,
      producing FStringPart::Expr(Expr::FString(...)) for nested cases.
    requires: [R5.2]

  # R6: Parser — metaclass keyword
  - path: crates/mamba/src/parser/stmt_compound.rs
    action: MODIFY
    desc: |
      Recognize metaclass=<expr> as a keyword argument in the class declaration base-list.
      Store in ClassDef.metaclass: Option<Expr>.
    requires: [R6.1]

  - path: crates/mamba/src/ast.rs
    action: MODIFY
    desc: Add metaclass: Option<Box<Expr>> field to ClassDef AST node.
    requires: [R6.1]

  - path: crates/mamba/src/hir/mod.rs
    action: MODIFY
    desc: Add metaclass: Option<HirExpr> to HIR ClassDef node.
    requires: [R6.2]

  - path: crates/mamba/src/lower/ast_to_hir.rs
    action: MODIFY
    desc: Propagate ClassDef.metaclass through lowering into HIR (stub: apply metaclass = identity for now).
    requires: [R6.2]

  # R7: Compiler — walrus scope (PEP 572)
  - path: crates/mamba/src/resolve/pass.rs
    action: MODIFY
    desc: |
      When := appears inside a comprehension, bind the target in the enclosing function scope
      (walk up past comprehension and class scopes). Verify binding is visible after the
      comprehension expression.
    requires: [R7.1, R7.2]

  # R9: Type checker — multi-arg stdlib forms
  - path: crates/mamba/src/typeck/mod.rs
    action: MODIFY
    desc: |
      Accept next(iterator, default) 2-arg form (R9.1).
      Accept generator.throw(exc_type, value, traceback) 3-arg form (R9.2).
      Accept type annotations in json/functools/csv stdlib wrappers (R9.3).
    requires: [R9.1, R9.2, R9.3]

  # R10: Runtime — stdlib output divergence fixes
  - path: crates/mamba/src/runtime/datetime.rs
    action: MODIFY
    desc: Fix datetime string formatting to match CPython 3.12 output.
    requires: [R10.1]

  - path: crates/mamba/src/runtime/hashlib.rs
    action: MODIFY
    desc: Fix hashlib digest hex encoding to match CPython 3.12.
    requires: [R10.2]

  - path: crates/mamba/src/runtime/math.rs
    action: MODIFY
    desc: Fix math function edge-case outputs (inf, nan, precision) to match CPython 3.12.
    requires: [R10.3]

  - path: crates/mamba/src/runtime/sys_module.rs
    action: MODIFY
    desc: Fix sys module attribute values to match CPython 3.12.
    requires: [R10.4]

  - path: crates/mamba/src/runtime/os_module.rs
    action: MODIFY
    desc: Fix os module output to match CPython 3.12.
    requires: [R10.5]

  - path: crates/mamba/src/runtime/collections.rs
    action: MODIFY
    desc: Fix collections module output (OrderedDict repr, defaultdict, etc.) to match CPython 3.12.
    requires: [R10.6]

  # Xfail marker removal — one fixture per fix category
  - path: crates/mamba/tests/fixtures/conformance/builtins/functional.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R1.1 fix.
  - path: crates/mamba/tests/fixtures/conformance/class_system/descriptors.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R1.2 fix.
  - path: crates/mamba/tests/fixtures/conformance/builtins/object_protocol.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R1.3 fix.
  - path: crates/mamba/tests/fixtures/conformance/class_system/super_call.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R1.4 fix.
  - path: crates/mamba/tests/fixtures/conformance/decorator_full/decorator_full.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R1.5 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/itertools/itertools_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R2.1 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/io/io_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R2.1 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/pathlib/pathlib_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R2.1 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/random/random_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R2.1 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/re/pattern_matching.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R2.1 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/struct/struct_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R2.1 fix.
  - path: crates/mamba/tests/fixtures/conformance/data_structures/bytes_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R3 fix.
  - path: crates/mamba/tests/fixtures/conformance/language/exceptions.py
    action: MODIFY
    desc: Remove __cause__/__context__ xfail marker after R4 fix; retain ExceptionGroup xfail (#755).
  - path: crates/mamba/tests/fixtures/conformance/language/fstring_advanced.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R5 fix.
  - path: crates/mamba/tests/fixtures/conformance/class_system/inheritance.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R6 fix.
  - path: crates/mamba/tests/fixtures/conformance/language/comprehension_scope.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R7 fix.
  - path: crates/mamba/tests/fixtures/conformance/language/pattern_matching.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R8 fix.
  - path: crates/mamba/tests/fixtures/conformance/builtins/iteration.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R9.1 fix.
  - path: crates/mamba/tests/fixtures/conformance/language/generators.py
    action: MODIFY
    desc: Remove throw 3-arg xfail after R9.2 fix; retain async generator xfail (#800).
  - path: crates/mamba/tests/fixtures/conformance/stdlib/json/json_encode_decode.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R9.3 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/functools/functools_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R9.3 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/csv/csv_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R9.3 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/datetime/datetime_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R10.1 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/hashlib/hashlib_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R10.2 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/math/math_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R10.3 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/sys/sys_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R10.4 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/os/os_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R10.5 fix.
  - path: crates/mamba/tests/fixtures/conformance/stdlib/collections/collections_ops.py
    action: MODIFY
    desc: Remove mamba-xfail marker after R10.6 fix.
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
