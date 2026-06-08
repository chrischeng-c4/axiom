---
id: mamba-runtime-bugs-spec
main_spec_ref: cclab-mamba/logic/mamba-runtime-bugs-spec.md
merge_strategy: new
filled_sections: [overview, requirements, scenarios, test_plan, changes]
fill_sections: [overview, requirements, scenarios, test_plan, changes]
---

# Mamba Runtime Bugs Spec

## Overview

Fix 5 independent CPython 3.12 behavioral conformance bugs in cclab-mamba (#1037). Each bug is in a different compiler/runtime layer:

| Bug | Layer | Root Cause | CPython Behavior |
|-----|-------|-----------|------------------|
| Semicolon separator | Parser (stmt.rs) | Semicolon token lexed but not consumed between statements | `;` is a valid statement separator |
| ZeroDivisionError | Runtime (builtins.rs) | `mb_floordiv()` returns `none()` on zero divisor | Raises `ZeroDivisionError` |
| Decorator return | Lowering (hir_to_mir.rs) | Decorated function result not propagated as call return | Decorated function returns wrapped result |
| Nested f-string | Parser (expr.rs) | `parse_fstring_parts()` does not recursively handle inner f-strings | `f"{f'{x}'}"` evaluates inner f-string correctly |
| json.dumps None | Stdlib (json_mod.rs) | `dispatch_dumps()` drops return value from `mb_json_dumps()` | Returns JSON string |
## Requirements

### R1: Semicolon as statement separator

The parser must accept `;` as a statement separator between simple statements on the same line. After parsing a statement, if the next token is `Token::Semicolon`, consume it and continue parsing the next statement. Multiple semicolons and trailing semicolons must be tolerated.

**File**: `crates/mamba/src/parser/stmt.rs`
**Constraint**: Only simple statements (not compound: `if`/`for`/`while`/`def`/`class`) may appear after `;`
**Priority**: high

### R2: ZeroDivisionError on floor division by zero

`mb_floordiv()` must raise `ZeroDivisionError` when the right operand is zero, for both inline integers and floats. Currently returns `MbValue::none()` silently.

**File**: `crates/mamba/src/runtime/builtins.rs` (lines ~1497-1510)
**Constraint**: Must match CPython — `ZeroDivisionError: integer division or modulo by zero`
**Priority**: high

### R3: Decorator preserves function return value

Calling a decorated function must return the result of calling the wrapper (decorated) function, not `None`. The lowering pass (`hir_to_mir.rs`) stores the decorator-applied function globally but does not propagate its return value when the decorated function is called.

**File**: `crates/mamba/src/lower/hir_to_mir.rs` (lines ~1782-1826)
**Constraint**: `@deco\ndef f(): return 1` — calling `f()` must return whatever `deco(f)()` returns
**Priority**: high

### R4: Nested f-string evaluation

`parse_fstring_parts()` must recursively handle nested f-strings. When an outer f-string contains `{f'...'}` as an expression, the inner f-string must be fully parsed and its formatted result used as the outer expression value.

**File**: `crates/mamba/src/parser/expr.rs` (lines ~442-527)
**Constraint**: `f"{f'{x}'}"` must equal the string value of `x`, not empty or `None`
**Priority**: high

### R5: json.dumps returns serialized string

`dispatch_dumps()` must return the `MbValue` produced by `mb_json_dumps()`. Currently the return value is dropped on at least one code path, causing `json.dumps(obj)` to evaluate to `None`.

**File**: `crates/mamba/src/runtime/stdlib/json_mod.rs` (lines ~24-69)
**Constraint**: `json.dumps({"a": 1})` must return `'{"a": 1}'`
**Priority**: high
## Scenarios

### Scenario: Semicolon separates two assignments
**GIVEN** source `a = 1; b = 2`
**WHEN** parsed and executed
**THEN** `a == 1` and `b == 2`

### Scenario: Semicolon separates print and assignment
**GIVEN** source `print(1); x = 2; print(x)`
**WHEN** executed
**THEN** stdout is `1\n2\n`

### Scenario: Trailing semicolon is tolerated
**GIVEN** source `x = 1;`
**WHEN** parsed
**THEN** no parse error, `x == 1`

### Scenario: Floor division by zero raises ZeroDivisionError
**GIVEN** source `x = 10 // 0`
**WHEN** executed
**THEN** raises `ZeroDivisionError` with message containing `division or modulo by zero`

### Scenario: Floor division by zero float raises ZeroDivisionError
**GIVEN** source `x = 10.0 // 0.0`
**WHEN** executed
**THEN** raises `ZeroDivisionError`

### Scenario: Normal floor division still works
**GIVEN** source `x = 7 // 2`
**WHEN** executed
**THEN** `x == 3`

### Scenario: Decorated function returns wrapper result
**GIVEN** source:
```python
def double(f):
    def wrapper(*args):
        return f(*args) * 2
    return wrapper

@double
def add(a, b):
    return a + b

result = add(3, 4)
```
**WHEN** executed
**THEN** `result == 14` (not `None`)

### Scenario: Stacked decorators preserve return
**GIVEN** two decorators stacked on a function
**WHEN** the function is called
**THEN** the outermost decorator's wrapper return value is returned

### Scenario: Nested f-string evaluates inner expression
**GIVEN** source `x = 42; s = f"{f'{x}'}"`
**WHEN** executed
**THEN** `s == "42"`

### Scenario: Nested f-string with format spec
**GIVEN** source `x = 3.14; s = f"{f'{x:.1f}'}"`
**WHEN** executed
**THEN** `s == "3.1"`

### Scenario: json.dumps returns string
**GIVEN** source:
```python
import json
result = json.dumps({"key": "value"})
```
**WHEN** executed
**THEN** `result == '{"key": "value"}'` (a string, not `None`)

### Scenario: json.dumps with indent kwarg
**GIVEN** source `json.dumps([1, 2], indent=2)`
**WHEN** executed
**THEN** returns indented JSON string, not `None`
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

### Strategy

| Layer | Mechanism | Location |
|-------|-----------|----------|
| Fixture conformance | `.py` fixture + `.expected` golden file, run via `conformance_tests.rs` harness | `tests/fixtures/conformance/` |
| Rust integration | `#[test]` in `runtime_bugs_conformance_tests.rs` using `jit_capture()` | `crates/mamba/tests/` |
| Regression guard | Each fix has a non-regression case verifying existing behavior unchanged | inline |

All tests run through the full JIT pipeline: parse → type-check → HIR → MIR → Cranelift JIT → capture stdout → assert.

### T1: Semicolon statement separator (R1)

**Fixture**: `tests/fixtures/conformance/language/semicolon_separator.py`

#### TC-1.1: Two assignments separated by semicolon
**GIVEN** source `a = 1; b = 2; print(a); print(b)`
**WHEN** parsed and executed through JIT pipeline
**THEN** stdout is `1\n2\n`

#### TC-1.2: Print and assignment separated by semicolons
**GIVEN** source `print(1); x = 2; print(x)`
**WHEN** executed
**THEN** stdout is `1\n2\n`

#### TC-1.3: Trailing semicolon tolerated
**GIVEN** source `x = 1; print(x);`
**WHEN** parsed and executed
**THEN** no parse error, stdout is `1\n`

#### TC-1.4: Multiple consecutive semicolons (empty statements)
**GIVEN** source `a = 1;; b = 2; print(a); print(b)`
**WHEN** parsed and executed
**THEN** stdout is `1\n2\n` — empty statements between `;;` are silently skipped

#### TC-1.5: Semicolon before compound statement is parse error
**GIVEN** source `x = 1; if True:\n    pass`
**WHEN** parsed
**THEN** raises parse error — compound statements (`if`/`for`/`while`/`def`/`class`) are not allowed after `;`

#### TC-1.6: Three statements on one line
**GIVEN** source `a = 1; b = 2; c = a + b; print(c)`
**WHEN** executed
**THEN** stdout is `3\n`

### T2: ZeroDivisionError on floor division (R2)

**Fixture**: `tests/fixtures/conformance/arithmetic/floor_div_zero.py`

#### TC-2.1: Integer floor division by zero
**GIVEN** source:
```python
try:
    x = 10 // 0
except ZeroDivisionError as e:
    print("caught")
    print(e)
```
**WHEN** executed
**THEN** stdout contains `caught` and message contains `integer division or modulo by zero`

#### TC-2.2: Float floor division by zero
**GIVEN** source:
```python
try:
    x = 10.0 // 0.0
except ZeroDivisionError:
    print("caught float")
```
**WHEN** executed
**THEN** stdout is `caught float\n`

#### TC-2.3: Normal floor division unchanged (non-regression)
**GIVEN** source `print(7 // 2)`
**WHEN** executed
**THEN** stdout is `3\n`

#### TC-2.4: Negative floor division unchanged (non-regression)
**GIVEN** source `print(-7 // 2)`
**WHEN** executed
**THEN** stdout is `-4\n`

#### TC-2.5: Modulo by zero still raises (non-regression)
**GIVEN** source:
```python
try:
    x = 10 % 0
except ZeroDivisionError:
    print("mod caught")
```
**WHEN** executed
**THEN** stdout is `mod caught\n`

### T3: Decorator return value propagation (R3)

**Fixture**: `tests/fixtures/conformance/language/decorator_return.py`

#### TC-3.1: Simple decorator preserves return value
**GIVEN** source:
```python
def double(f):
    def wrapper(*args):
        return f(*args) * 2
    return wrapper

@double
def add(a, b):
    return a + b

print(add(3, 4))
```
**WHEN** executed
**THEN** stdout is `14\n` (not `None`)

#### TC-3.2: Identity decorator returns original value
**GIVEN** source:
```python
def identity(f):
    return f

@identity
def greet():
    return 42

print(greet())
```
**WHEN** executed
**THEN** stdout is `42\n`

#### TC-3.3: Stacked decorators preserve return chain
**GIVEN** source:
```python
def add_one(f):
    def wrapper(*args):
        return f(*args) + 1
    return wrapper

def double(f):
    def wrapper(*args):
        return f(*args) * 2
    return wrapper

@add_one
@double
def val():
    return 5

print(val())
```
**WHEN** executed
**THEN** stdout is `11\n` — `add_one(double(val))()` = `(5*2)+1 = 11`

#### TC-3.4: Decorator with no-return function (returns None)
**GIVEN** source:
```python
def log(f):
    def wrapper(*args):
        f(*args)
    return wrapper

@log
def say():
    print("hello")

result = say()
print(result)
```
**WHEN** executed
**THEN** stdout is `hello\nNone\n` — wrapper returns `None` because it has no return statement

### T4: Nested f-string evaluation (R4)

**Fixture**: `tests/fixtures/conformance/language/nested_fstring.py`

#### TC-4.1: Simple nested f-string with literal
**GIVEN** source `print(f"{f'{42}'}")`
**WHEN** executed
**THEN** stdout is `42\n`

#### TC-4.2: Nested f-string with variable
**GIVEN** source `x = 5; print(f"{f'{x}'}")`
**WHEN** executed
**THEN** stdout is `5\n`

#### TC-4.3: Nested f-string with format spec
**GIVEN** source `x = 3.14; print(f"{f'{x:.1f}'}")`
**WHEN** executed
**THEN** stdout is `3.1\n`

#### TC-4.4: Nested f-string with expression
**GIVEN** source `print(f"{f'{1 + 2}'}")`
**WHEN** executed
**THEN** stdout is `3\n`

#### TC-4.5: Three-level nested f-string
**GIVEN** source `print(f"a{f"b{f"c"}"}")` 
**WHEN** executed
**THEN** stdout is `abc\n`

#### TC-4.6: Outer f-string with non-nested content still works (non-regression)
**GIVEN** source `x = 10; print(f"val={x}")`
**WHEN** executed
**THEN** stdout is `val=10\n`

### T5: json.dumps return value (R5)

**Fixture**: `tests/fixtures/conformance/stdlib/json/json_dumps_return.py`

#### TC-5.1: json.dumps with dict
**GIVEN** source:
```python
import json
result = json.dumps({"a": 1})
print(result)
print(type(result).__name__)
```
**WHEN** executed
**THEN** stdout contains the JSON string `{"a": 1}` and type is `str`

#### TC-5.2: json.dumps with list
**GIVEN** source:
```python
import json
print(json.dumps([1, 2, 3]))
```
**WHEN** executed
**THEN** stdout is `[1, 2, 3]\n`

#### TC-5.3: json.dumps with string
**GIVEN** source:
```python
import json
print(json.dumps("hello"))
```
**WHEN** executed
**THEN** stdout is `"hello"\n`

#### TC-5.4: json.dumps with None
**GIVEN** source:
```python
import json
print(json.dumps(None))
```
**WHEN** executed
**THEN** stdout is `null\n`

#### TC-5.5: json.dumps with indent kwarg
**GIVEN** source:
```python
import json
result = json.dumps({"a": 1}, indent=2)
print(type(result).__name__)
```
**WHEN** executed
**THEN** stdout is `str\n` — return is a string, not `None`

#### TC-5.6: json.dumps result used in expression (non-regression)
**GIVEN** source:
```python
import json
s = json.dumps([1])
print(len(s))
```
**WHEN** executed
**THEN** stdout is `3\n` — `len("[1]") == 3`, confirms return value is usable

### Traceability

| Test | Requirement | Scenario | Fixture |
|------|-------------|----------|---------|
| TC-1.1..TC-1.6 | R1 | Semicolon separates two assignments, Print+assign, Trailing semicolon | `language/semicolon_separator.py` |
| TC-2.1..TC-2.5 | R2 | Floor division by zero (int/float), Normal floor div | `arithmetic/floor_div_zero.py` |
| TC-3.1..TC-3.4 | R3 | Decorated function returns wrapper result, Stacked decorators | `language/decorator_return.py` |
| TC-4.1..TC-4.6 | R4 | Nested f-string evaluates inner expression, Nested with format | `language/nested_fstring.py` |
| TC-5.1..TC-5.6 | R5 | json.dumps returns string, json.dumps with indent | `stdlib/json/json_dumps_return.py` |
## Changes

```yaml
files:
  # R1: Semicolon separator
  - path: crates/mamba/src/parser/stmt.rs
    action: MODIFY
    desc: After parsing a simple statement, check for Token::Semicolon. If found, consume it and loop to parse the next simple statement. Handle trailing semicolons and empty statements.

  # R2: ZeroDivisionError
  - path: crates/mamba/src/runtime/builtins.rs
    action: MODIFY
    desc: In mb_floordiv(), replace MbValue::none() returns on zero divisor with mb_raise("ZeroDivisionError", "integer division or modulo by zero") for both inline int and float paths.

  # R3: Decorator return value
  - path: crates/mamba/src/lower/hir_to_mir.rs
    action: MODIFY
    desc: In decorator application lowering (~lines 1782-1826), ensure the decorated function's call result vreg is propagated as the return value, not just stored globally.

  # R4: Nested f-string
  - path: crates/mamba/src/parser/expr.rs
    action: MODIFY
    desc: In parse_fstring_parts(), detect when inner expression starts with f" or f' and recursively invoke f-string parsing. Ensure brace depth tracking handles nested f-string delimiters.

  # R5: json.dumps return
  - path: crates/mamba/src/runtime/stdlib/json_mod.rs
    action: MODIFY
    desc: In dispatch_dumps(), ensure all code paths return the MbValue from mb_json_dumps(). Fix missing return on the final fallthrough path (~line 68).

  # Tests
  - path: crates/mamba/tests/conformance/semicolon.py
    action: CREATE
    desc: Conformance test fixture for semicolon statement separator.
  - path: crates/mamba/tests/conformance/floor_div_zero.py
    action: CREATE
    desc: Conformance test fixture for ZeroDivisionError on floor division.
  - path: crates/mamba/tests/conformance/decorator_return.py
    action: CREATE
    desc: Conformance test fixture for decorator return value preservation.
  - path: crates/mamba/tests/conformance/nested_fstring.py
    action: CREATE
    desc: Conformance test fixture for nested f-string evaluation.
  - path: crates/mamba/tests/conformance/json_dumps.py
    action: CREATE
    desc: Conformance test fixture for json.dumps return value.
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