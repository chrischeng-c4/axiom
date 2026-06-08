---
id: mamba-p1-lang-features-spec
main_spec_ref: ~
---

# Mamba P1 Lang Features Spec

## Overview
<!-- type: overview lang: markdown -->

Implement five P1 language features for the Mamba Python compiler to reach Py3.12 parity:

1. **Decorator arguments and chaining (#847)** — Full PEP 614: any expression in decorator position, stacked decorators applied bottom-up, @property/@staticmethod/@classmethod special-cased in type checker.
2. **Star expressions / extended unpacking (#845)** — `*a, b = iterable` with nested destructuring, `f(*args, **kwargs)` splat in calls, runtime::unpack_star helper.
3. **Global and nonlocal statements (#846)** — `global x` / `nonlocal y` scope modifiers integrated with name resolution and closure capture (mutable upgrade).
4. **List/tuple slicing with step (#835)** — `a[::2]`, `a[::-1]` for lists, tuples, and strings (Unicode codepoints). `a[::0]` raises ValueError.
5. **String escape sequences, raw strings, byte strings (#848)** — Full Unicode escapes (\\uXXXX, \\UXXXXXXXX, \\xHH, \\ooo, \\N{name}), raw strings (r'...'), bytes type (b'...') with full CPython bytes API.

All features follow CPython 3.12 semantics. Changes span: lexer, parser/AST, name resolution, HIR, MIR, codegen (Cranelift JIT), runtime, and type checker.
## Requirements
<!-- type: overview lang: markdown -->

### R1: Decorator Full Support (PEP 614)
- Parser accepts any expression in decorator position (name, attr access, call, subscript, chained)
- Multiple `@` lines per function/class, applied bottom-up (innermost first)
- Type checker: @property/@staticmethod/@classmethod have hardcoded semantics; other decorators fall back to Any
- Codegen: emit function def, then apply decorator chain in reverse order

### R2: Star Expressions / Extended Unpacking (PEP 3132)
- Assignment target `*name` captures remaining elements as list
- Nested destructuring: `a, (*b, c) = ...` — full Py3.12 behavior
- Splat in function calls: `f(*args, **kwargs)`
- Starred variable typed as `list[T]` where T inferred from RHS
- Runtime helper: `unpack_star(iter, n_before, n_after)`

### R3: Global and Nonlocal Statements
- `global x` marks x as module-scope in enclosing function
- `nonlocal y` marks y as referring to nearest enclosing scope's y
- `global x` where x not yet assigned: silently allowed (runtime NameError if accessed before assignment)
- `nonlocal` upgrades implicit read-only capture to mutable capture
- Name resolution pass updated to classify variables with global/nonlocal annotations
- Type checker respects scope annotations for cross-scope inference

### R4: Slice Step
- 3-argument slice `a[start:stop:step]` in parser/AST/HIR/MIR/codegen
- List, tuple, string slicing with step
- String slicing on Unicode codepoints (not bytes)
- `a[::0]` raises Python ValueError
- Negative step reverses iteration direction

### R5: String Escape Sequences and Bytes Type
- Lexer: string prefixes r, b, rb/br recognized
- Full escape sequences: \\n, \\t, \\\\, \\', \\\", \\a, \\b, \\f, \\r, \\v, \\0, \\xHH, \\uXXXX, \\UXXXXXXXX, \\ooo, \\N{name}
- Raw strings: no escape processing
- Bytes type: ObjKind::Bytes wrapping Vec<u8>, NaN-boxed heap object
- Full CPython bytes API: len, index, slice, concat, find, replace, decode, startswith, endswith, split, join, hex, etc.
- \\N{name} requires Unicode name database lookup
## Scenarios
<!-- type: overview lang: markdown -->

### S1: Decorator with arguments
```python
@app.route("/api", methods=["GET"])
def handler(): pass
```
Expected: parser produces Decorator expr = CallExpr(AttrExpr("app", "route"), args). Codegen applies decorator call result to handler.

### S2: Stacked decorators
```python
@login_required
@cache(maxsize=128)
def view(): pass
```
Expected: view = login_required(cache(maxsize=128)(view)) — bottom-up application.

### S3: Star unpacking basic
```python
a, *b, c = [1, 2, 3, 4, 5]
assert a == 1 and b == [2, 3, 4] and c == 5
```

### S4: Star unpacking nested
```python
a, (*b, c) = [1, [2, 3, 4]]
assert a == 1 and b == [2, 3] and c == 4
```

### S5: Global statement
```python
x = 10
def f():
    global x
    x = 20
f()
assert x == 20
```

### S6: Nonlocal statement
```python
def outer():
    x = 1
    def inner():
        nonlocal x
        x = 2
    inner()
    return x
assert outer() == 2
```

### S7: Slice with step
```python
assert [0,1,2,3,4][::2] == [0,2,4]
assert [0,1,2,3,4][::-1] == [4,3,2,1,0]
assert "hello"[::2] == "hlo"
```

### S8: Slice zero step error
```python
try:
    [1,2,3][::0]
except ValueError:
    pass  # expected
```

### S9: Raw strings
```python
assert r"\n" == "\\n"
assert len(r"\n") == 2
```

### S10: Bytes type
```python
b = b"hello"
assert len(b) == 5
assert b[0] == 104
assert b.decode("utf-8") == "hello"
```

### S11: Unicode named escapes
```python
assert "\N{SNOWMAN}" == "\u2603"
```
## Diagrams
<!-- type: overview lang: markdown -->

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- score-td-placeholder -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- score-td-placeholder -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- score-td-placeholder -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- score-td-placeholder -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- score-td-placeholder -->

## API Spec
<!-- type: overview lang: markdown -->

### REST API
<!-- type: rest-api lang: yaml -->
<!-- score-td-placeholder -->

### RPC API
<!-- type: rpc-api lang: yaml -->
<!-- score-td-placeholder -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- score-td-placeholder -->

### CLI
<!-- type: cli lang: yaml -->
<!-- score-td-placeholder -->

### Schema
<!-- type: schema lang: yaml -->
<!-- score-td-placeholder -->

### Config
<!-- type: config lang: yaml -->
<!-- score-td-placeholder -->

## Test Plan
<!-- type: overview lang: markdown -->

<!-- TODO -->

## Changes
<!-- type: changes lang: yaml -->
```yaml
files:
  # Lexer
  - path: crates/mamba/src/lexer/mod.rs
    action: MODIFY
    desc: Add string prefix recognition (r, b, rb/br), escape sequence processing, \N{name} lookup

  # Parser
  - path: crates/mamba/src/parser/ast.rs
    action: MODIFY
    desc: Add Starred expr variant, extend Slice with step field, add bytes literal node
  - path: crates/mamba/src/parser/expr.rs
    action: MODIFY
    desc: Parse star expressions in assignment targets and function call args
  - path: crates/mamba/src/parser/stmt.rs
    action: MODIFY
    desc: Parse global/nonlocal statements, decorator any-expression (PEP 614)
  - path: crates/mamba/src/parser/stmt_compound.rs
    action: MODIFY
    desc: Full PEP 614 decorator expression parsing

  # Name Resolution
  - path: crates/mamba/src/resolve/pass.rs
    action: MODIFY
    desc: Handle global/nonlocal declarations in scope classification
  - path: crates/mamba/src/resolve/scope.rs
    action: MODIFY
    desc: Add Global/Nonlocal variable kinds to scope chain

  # HIR / Lowering
  - path: crates/mamba/src/hir/mod.rs
    action: MODIFY
    desc: Add HIR nodes for star unpacking, 3-arg slice, global/nonlocal
  - path: crates/mamba/src/lower/ast_to_hir.rs
    action: MODIFY
    desc: Desugar decorators, star unpacking, global/nonlocal, bytes literals
  - path: crates/mamba/src/lower/hir_to_mir.rs
    action: MODIFY
    desc: Lower star unpacking to unpack_star calls, 3-arg slices, scope modifiers

  # MIR
  - path: crates/mamba/src/mir/mod.rs
    action: MODIFY
    desc: Add MIR instructions for unpack_star, slice_step, global/nonlocal load/store

  # Codegen
  - path: crates/mamba/src/codegen/cranelift/jit.rs
    action: MODIFY
    desc: JIT emit for new MIR instructions (unpack, slice step, scope modifiers, bytes)
  - path: crates/mamba/src/codegen/cranelift/mod.rs
    action: MODIFY
    desc: Symbol wiring for new runtime helpers

  # Runtime
  - path: crates/mamba/src/runtime/builtins.rs
    action: MODIFY
    desc: Add unpack_star, slice_step dispatch, bytes() constructor
  - path: crates/mamba/src/runtime/list_ops.rs
    action: MODIFY
    desc: List slice with step implementation
  - path: crates/mamba/src/runtime/tuple_ops.rs
    action: MODIFY
    desc: Tuple slice with step implementation
  - path: crates/mamba/src/runtime/string_ops.rs
    action: MODIFY
    desc: String slice with step on Unicode codepoints
  - path: crates/mamba/src/runtime/bytes_ops.rs
    action: CREATE
    desc: Full bytes type operations (find, replace, decode, startswith, split, join, hex, etc.)
  - path: crates/mamba/src/runtime/rc.rs
    action: MODIFY
    desc: Add ObjKind::Bytes and ObjData::Bytes(Vec<u8>)
  - path: crates/mamba/src/runtime/closure.rs
    action: MODIFY
    desc: Mutable capture upgrade for nonlocal variables
  - path: crates/mamba/src/runtime/symbols.rs
    action: MODIFY
    desc: Register new runtime symbols (mb_unpack_star, mb_slice_step, mb_bytes_*)

  # Type Checker
  - path: crates/mamba/src/types/check_stmt.rs
    action: MODIFY
    desc: Handle global/nonlocal in type inference, decorator special cases
  - path: crates/mamba/src/types/check_expr.rs
    action: MODIFY
    desc: Star expression type inference, bytes literal type

  # Tests
  - path: crates/mamba/tests/fixtures/conformance/decorator_full.py
    action: CREATE
    desc: Conformance tests for PEP 614 decorators
  - path: crates/mamba/tests/fixtures/conformance/star_unpacking.py
    action: CREATE
    desc: Conformance tests for extended unpacking
  - path: crates/mamba/tests/fixtures/conformance/scope_modifiers.py
    action: CREATE
    desc: Conformance tests for global/nonlocal
  - path: crates/mamba/tests/fixtures/conformance/slice_step.py
    action: CREATE
    desc: Conformance tests for 3-arg slice
  - path: crates/mamba/tests/fixtures/conformance/string_escapes.py
    action: CREATE
    desc: Conformance tests for escape sequences, raw strings, bytes
```
## Wireframe
<!-- type: wireframe lang: yaml -->

```yaml
# score-td-placeholder
```

## Component
<!-- type: component lang: yaml -->

```yaml
# score-td-placeholder
```

## Design Token
<!-- type: design-token lang: yaml -->

```yaml
# score-td-placeholder
```

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->
