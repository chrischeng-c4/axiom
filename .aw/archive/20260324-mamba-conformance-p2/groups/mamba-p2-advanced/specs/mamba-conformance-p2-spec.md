---
id: mamba-conformance-p2-spec
main_spec_ref: "crates/mamba/testing/mamba-py312-conformance.md"
merge_strategy: new
filled_sections: [overview, requirements, scenarios, logic, test-plan, changes]
fill_sections: [overview, requirements, scenarios, logic, test-plan, changes]
create_complete: true
---

# Mamba Conformance P2 Spec

## Overview

Fix 3 P2 advanced conformance gaps that prevent Mamba from matching CPython 3.12 behavior on advanced language features:

| ID | Feature | Root Cause | Affected Layers |
|----|---------|-----------|----------------|
| P2-R1 | Nested f-strings (PEP 701) | `parse_fstring_parts` in `parser/expr.rs` treats nested `f"..."` inside `{...}` as a plain string literal — the inner `f` prefix is not recognized, so the nested f-string is never recursively parsed. The lexer (`lex_fstr_inner`) correctly captures the full token, but the parser's brace-content extraction discards the f-prefix context. | lexer/token.rs, parser/expr.rs |
| P2-R2 | `metaclass=` keyword in class definition | AST and HIR correctly store `metaclass` field, but `hir_to_mir.rs` emits `mb_class_define_multi` without forwarding the metaclass to the runtime. `mb_class_register` ignores metaclass — no `__class_getitem__`, no custom `__new__`/`__init__` interception via metaclass `__call__`. | lower/hir_to_mir.rs, codegen/cranelift/mod.rs, runtime/class.rs |
| P2-R3 | Descriptor protocol codegen | Runtime has `is_descriptor`, `invoke_descriptor_get/set/delete`, but codegen never emits descriptor-aware attribute access. `mb_getattr`/`mb_setattr` calls from JIT-compiled code bypass the data-descriptor priority check for user-defined descriptors. User classes with `__get__`/`__set__`/`__delete__` are never invoked from compiled code paths. | codegen/cranelift/mod.rs, runtime/class.rs, runtime/symbols.rs |

All 3 features are independent — no ordering dependency. Acceptance: new conformance fixtures pass with `cargo test -p mamba --test conformance_tests`.
## Requirements

### P2-R1: Nested F-String Parsing (PEP 701)

```yaml
id: P2-R1
priority: high
affects: parser/expr.rs, lexer/token.rs
```

When `parse_fstring_parts` encounters `{...}` content that starts with `f"` or `f'`, it must recursively parse the inner f-string instead of treating it as a plain string expression. The inner f-string yields its own `Expr::FString(parts)` node. Nesting depth is unbounded (matching CPython 3.12 PEP 701).

Sub-requirements:

| ID | Requirement |
|----|-------------|
| P2-R1.1 | `parse_fstring_expr` must detect `f"..."` / `f'...'` prefix in the extracted expression text and recursively invoke `parse_fstring_parts` on the inner content |
| P2-R1.2 | Same-quote reuse inside f-string expressions (e.g., `f"{'hello'}"`) must parse correctly — the lexer already handles this via `lex_fstr_inner` stack |
| P2-R1.3 | Multi-line f-string expressions with line breaks inside `{...}` must work (PEP 701 relaxation) |
| P2-R1.4 | Backslashes inside f-string expressions (e.g., `f"{chr(10)}"`, `f"{'\t'}"`) must be allowed |
| P2-R1.5 | Format spec with nested f-string (e.g., `f"{value:{width}.{precision}}"`) must split correctly — the `:` after `{width}` is inside a nested `{}`, not a top-level format separator |

### P2-R2: metaclass= Keyword Forwarding to Runtime

```yaml
id: P2-R2
priority: high
affects: lower/hir_to_mir.rs, codegen/cranelift/mod.rs, runtime/class.rs
```

When `class Foo(Base, metaclass=Meta):` is defined, the metaclass name must be forwarded from HIR through MIR to the runtime class registration. The runtime must invoke the metaclass `__call__` (or `__new__` + `__init__`) instead of the default `type.__call__` when creating instances.

Sub-requirements:

| ID | Requirement |
|----|-------------|
| P2-R2.1 | `hir_to_mir.rs`: when `HirClass.metaclass` is `Some(name)`, emit an additional `CallExtern` to `mb_class_set_metaclass(class_name, metaclass_name)` after `mb_class_define_multi` |
| P2-R2.2 | `runtime/class.rs`: implement `mb_class_set_metaclass` — store metaclass association in CLASS_REGISTRY so instance creation routes through the metaclass |
| P2-R2.3 | `runtime/class.rs`: when `mb_class_instantiate` is called for a class with a metaclass, invoke the metaclass's `__call__` method instead of direct `__init__` — this enables custom instance creation (e.g., singletons, ABCMeta enforcement) |
| P2-R2.4 | `runtime/symbols.rs`: register `mb_class_set_metaclass` symbol for JIT linking |
| P2-R2.5 | ABCMeta integration: when metaclass is `ABCMeta`, enforce abstract method checks (already partially implemented via `mb_register_abstract` / `mb_check_abstract`) |

### P2-R3: Descriptor Protocol Codegen Integration

```yaml
id: P2-R3
priority: high
affects: codegen/cranelift/mod.rs, runtime/class.rs, runtime/symbols.rs
```

The runtime already implements `is_descriptor`, `is_data_descriptor`, `invoke_descriptor_get/set/delete` for the attribute access path through `mb_getattr`/`mb_setattr`. However, JIT-compiled code must emit calls that go through the descriptor-aware path for user-defined attribute access on class instances.

Sub-requirements:

| ID | Requirement |
|----|-------------|
| P2-R3.1 | Codegen must emit `mb_getattr(instance, attr_name)` for all attribute reads on user-class instances — this already dispatches through the descriptor protocol in `runtime/class.rs` |
| P2-R3.2 | Codegen must emit `mb_setattr(instance, attr_name, value)` for all attribute writes on user-class instances — this already routes through data-descriptor `__set__` in `runtime/class.rs` |
| P2-R3.3 | User classes defining `__get__(self, obj, objtype)` must be invokable as non-data descriptors when accessed as class attributes on instances |
| P2-R3.4 | User classes defining `__set__(self, obj, value)` must be treated as data descriptors — data descriptor priority over instance `__dict__` must work for user-defined descriptors, not just built-in `@property` |
| P2-R3.5 | User classes defining `__delete__(self, obj)` must be invokable when `del instance.attr` is executed on a data-descriptor attribute |
| P2-R3.6 | `try_get_dunder` in `class.rs` must support dispatching `__get__`/`__set__`/`__delete__` on user-defined descriptor classes (currently works for built-in property only) |

### Constraints

- All existing passing conformance tests must continue to pass (no regressions)
- Async generators remain unsupported (#800)
- Asyncio event loop remains unsupported (#801)
- Each feature fix must include a new conformance fixture with `.py` + `.expected` golden file
## Scenarios

### Scenario: Nested f-string with inner expression

- **GIVEN** `s = f"result: {f"inner {1 + 2}"}"`
- **WHEN** parsed and executed through Cranelift JIT
- **THEN** `s == "result: inner 3"` — inner f-string is recursively parsed and evaluated

### Scenario: Deeply nested f-strings (3 levels)

- **GIVEN** `s = f"outer {f"middle {f"deep"}"}"`
- **WHEN** executed
- **THEN** `s == "outer middle deep"` — all 3 nesting levels resolve correctly

### Scenario: Same-quote reuse in f-string expression (PEP 701)

- **GIVEN** `s = f"{'hello'}"`
- **WHEN** parsed and executed
- **THEN** `s == "hello"` — double quotes inside `{...}` expression do not terminate the outer f-string

### Scenario: F-string with format spec containing nested braces

- **GIVEN** `width = 10; s = f"{'hello':>{width}}"`
- **WHEN** executed
- **THEN** `s == "     hello"` — the `:` after `width` is inside `{}` and not treated as a top-level format separator

### Scenario: Backslash in f-string expression (PEP 701)

- **GIVEN** `s = f"tab: {'\t'}"`
- **WHEN** parsed and executed
- **THEN** `s` contains a literal tab character — backslash escapes are allowed in f-string expressions per PEP 701

### Scenario: Lambda in f-string expression

- **GIVEN** `s = f"{(lambda x: x + 1)(5)}"`
- **WHEN** executed
- **THEN** `s == "6"` — lambda parens prevent `:` confusion with format spec

### Scenario: metaclass= keyword basic instantiation

- **GIVEN**
  ```python
  class Meta(type):
      def __call__(cls, *args, **kwargs):
          instance = super().__call__(*args, **kwargs)
          instance._meta_created = True
          return instance
  class Foo(metaclass=Meta):
      pass
  obj = Foo()
  ```
- **WHEN** executed
- **THEN** `obj._meta_created == True` — Meta.__call__ intercepts instance creation

### Scenario: ABCMeta prevents instantiation of abstract class

- **GIVEN**
  ```python
  from abc import ABC, abstractmethod
  class Shape(ABC):
      @abstractmethod
      def area(self): pass
  try:
      s = Shape()
      print("ERROR: should not reach")
  except TypeError as e:
      print(f"caught: {e}")
  ```
- **WHEN** executed
- **THEN** prints `caught: Can't instantiate abstract class Shape with abstract method area` — ABCMeta enforcement works through metaclass pipeline

### Scenario: metaclass __new__ customizes class creation

- **GIVEN**
  ```python
  class UpperMeta(type):
      def __new__(mcs, name, bases, namespace):
          namespace = {k.upper(): v for k, v in namespace.items() if not k.startswith('_')}
          return super().__new__(mcs, name, bases, namespace)
  class Conf(metaclass=UpperMeta):
      host = "localhost"
      port = 8080
  ```
- **WHEN** executed
- **THEN** `Conf.HOST == "localhost"` and `Conf.PORT == 8080` — metaclass __new__ transforms the class namespace

### Scenario: User-defined descriptor __get__

- **GIVEN**
  ```python
  class Verbose:
      def __get__(self, obj, objtype=None):
          print(f"accessing on {objtype.__name__}")
          return 42
  class MyClass:
      attr = Verbose()
  obj = MyClass()
  result = obj.attr
  ```
- **WHEN** executed
- **THEN** prints `accessing on MyClass`; `result == 42` — non-data descriptor `__get__` is invoked on instance attribute access

### Scenario: Data descriptor __set__ priority over instance dict

- **GIVEN**
  ```python
  class Validated:
      def __get__(self, obj, objtype=None):
          return obj.__dict__.get('_val', 0)
      def __set__(self, obj, value):
          if value < 0:
              raise ValueError("must be >= 0")
          obj.__dict__['_val'] = value
  class Item:
      price = Validated()
  item = Item()
  item.price = 10
  result = item.price
  ```
- **WHEN** executed
- **THEN** `result == 10` — data descriptor `__set__` stores to `_val`, `__get__` retrieves from `_val`; descriptor takes priority over instance `__dict__` direct write

### Scenario: Data descriptor rejects invalid value

- **GIVEN** `item = Item(); item.price = -5` (using Validated descriptor from above)
- **WHEN** executed
- **THEN** raises `ValueError: must be >= 0` — data descriptor `__set__` validates and rejects

### Scenario: Descriptor __delete__

- **GIVEN**
  ```python
  class Deletable:
      def __get__(self, obj, objtype=None): return obj.__dict__.get('_v')
      def __set__(self, obj, value): obj.__dict__['_v'] = value
      def __delete__(self, obj): del obj.__dict__['_v']
  class Holder:
      val = Deletable()
  h = Holder()
  h.val = 99
  del h.val
  result = h.val
  ```
- **WHEN** executed
- **THEN** `result is None` — `__delete__` removes the backing store; subsequent `__get__` returns None

### Scenario: Existing passing tests unaffected

- **GIVEN** full conformance suite with all P2 fixes applied
- **WHEN** `cargo test -p mamba --test conformance_tests`
- **THEN** all previously passing tests still pass; no regressions
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

### Test Strategy

All tests run through the existing conformance harness (`conformance_tests.rs`): fixture `.py` is parsed, type-checked, lowered to HIR/MIR, compiled via Cranelift JIT, stdout captured, and compared against `.expected` golden file. New fixtures are added for each P2 feature.

### TC-1: Nested F-String Parsing (P2-R1)

| ID | Covers | Fixture |
|----|--------|---------|
| TC-1.1 | P2-R1.1, P2-R1.2 | `language/fstring_nested.py` |
| TC-1.2 | P2-R1.3 | `language/fstring_nested.py` |
| TC-1.3 | P2-R1.4 | `language/fstring_nested.py` |
| TC-1.4 | P2-R1.5 | `language/fstring_nested.py` |

**TC-1.1: Basic nested f-string**

- **GIVEN** fixture contains `s = f"result: {f"inner {1 + 2}"}"; print(s)`
- **WHEN** compiled and executed
- **THEN** stdout prints `result: inner 3`; inner f-string is recursively parsed

**TC-1.2: 3-level nested f-string**

- **GIVEN** fixture contains `s = f"a {f"b {f"c"}"}"` ; print(s)`
- **WHEN** compiled and executed
- **THEN** stdout prints `a b c`

**TC-1.3: Same-quote reuse (PEP 701)**

- **GIVEN** fixture contains `s = f"{'hello'}"; print(s)`
- **WHEN** compiled and executed
- **THEN** stdout prints `hello`; same quotes inside `{...}` do not break parsing

**TC-1.4: Format spec with nested braces**

- **GIVEN** fixture contains `width = 10; s = f"{'hi':>{width}}"; print(repr(s))`
- **WHEN** compiled and executed
- **THEN** stdout prints `'        hi'` (right-aligned in 10 chars)

### TC-2: Metaclass Keyword (P2-R2)

| ID | Covers | Fixture |
|----|--------|---------|
| TC-2.1 | P2-R2.1, P2-R2.2, P2-R2.3 | `language/metaclass.py` |
| TC-2.2 | P2-R2.5 | `language/metaclass.py` |

**TC-2.1: Metaclass __call__ intercepts instantiation**

- **GIVEN** fixture defines `Meta(type)` with `__call__` that sets `_meta_created = True`, and `class Foo(metaclass=Meta)`
- **WHEN** `Foo()` is created and `obj._meta_created` is printed
- **THEN** stdout prints `True`

**TC-2.2: Metaclass __new__ transforms namespace**

- **GIVEN** fixture defines `UpperMeta(type)` with `__new__` that uppercases attribute names
- **WHEN** class `Conf(metaclass=UpperMeta)` is created with `host = "localhost"`
- **THEN** `Conf.HOST == "localhost"` is printed as `True`

**TC-2.3: ABCMeta prevents abstract class instantiation**

- **GIVEN** fixture defines abstract class `Shape(ABC)` with `@abstractmethod area`
- **WHEN** `Shape()` is attempted
- **THEN** TypeError is raised and caught; stdout confirms the error message

### TC-3: Descriptor Protocol Codegen (P2-R3)

| ID | Covers | Fixture |
|----|--------|---------|
| TC-3.1 | P2-R3.1, P2-R3.3 | `language/descriptors.py` |
| TC-3.2 | P2-R3.2, P2-R3.4 | `language/descriptors.py` |
| TC-3.3 | P2-R3.5 | `language/descriptors.py` |

**TC-3.1: Non-data descriptor __get__ invoked on attribute read**

- **GIVEN** fixture defines `Verbose` descriptor class with `__get__` returning 42
- **WHEN** `obj.attr` is accessed on an instance of a class using `Verbose()` as class attribute
- **THEN** stdout prints the __get__ invocation message and the value 42

**TC-3.2: Data descriptor __set__ enforces validation**

- **GIVEN** fixture defines `Validated` data descriptor with `__get__` and `__set__` (rejects negative values)
- **WHEN** `item.price = 10; print(item.price)` then `item.price = -5`
- **THEN** stdout prints `10`, then raises `ValueError`

**TC-3.3: Data descriptor __delete__ clears backing store**

- **GIVEN** fixture defines `Deletable` descriptor with `__get__`, `__set__`, `__delete__`
- **WHEN** `h.val = 99; del h.val; print(h.val)`
- **THEN** stdout prints `None` — `__delete__` removed the backing value

### TC-4: Regression Gate

| ID | Covers | Scope |
|----|--------|-------|
| TC-4.1 | All | Full crate test suite |

**TC-4.1: No regressions in existing passing tests**

- **GIVEN** all P2 fixes applied
- **WHEN** `cargo test -p mamba` runs the full test suite
- **THEN** all previously passing tests still pass; exit code 0

### Verification Commands

```bash
# Full conformance suite
cargo test -p mamba --test conformance_tests

# P2-R1: nested f-strings
cargo test -p mamba --test conformance_tests language::fstring_nested

# P2-R2: metaclass
cargo test -p mamba --test conformance_tests language::metaclass

# P2-R3: descriptors
cargo test -p mamba --test conformance_tests language::descriptors

# Full regression gate
cargo test -p mamba
```
## Changes

```yaml
files:
  # P2-R1: Nested f-string parsing (PEP 701)
  - path: crates/mamba/src/parser/expr.rs
    action: MODIFY
    desc: |
      P2-R1: In parse_fstring_expr, detect f-prefix on extracted expression text
      and recursively invoke parse_fstring_parts for nested f-strings. Update
      parse_fstring_parts brace-depth tracking to correctly handle nested
      f-string boundaries where inner f-strings reuse the same quote character.
    requires: [P2-R1.1, P2-R1.2, P2-R1.5]

  - path: crates/mamba/src/lexer/token.rs
    action: MODIFY
    desc: |
      P2-R1: Verify lex_fstr_inner correctly handles multi-line expressions
      and backslash escapes in f-string expression bodies (PEP 701).
      Minor fixes if edge cases found during nested f-string testing.
    requires: [P2-R1.3, P2-R1.4]

  # P2-R2: metaclass= keyword forwarding
  - path: crates/mamba/src/lower/hir_to_mir.rs
    action: MODIFY
    desc: |
      P2-R2: When HirClass.metaclass is Some(name), emit an additional
      CallExtern to mb_class_set_metaclass(class_name, metaclass_name)
      after the existing mb_class_define_multi call.
    requires: [P2-R2.1]

  - path: crates/mamba/src/runtime/class.rs
    action: MODIFY
    desc: |
      P2-R2: Implement mb_class_set_metaclass — store metaclass association
      in MbClass within CLASS_REGISTRY. Modify mb_class_instantiate to check
      for metaclass and route through metaclass.__call__ instead of default
      type.__call__ when present.
      P2-R3: Verify try_get_dunder dispatches __get__/__set__/__delete__
      for user-defined descriptor classes (not just built-in property).
    requires: [P2-R2.2, P2-R2.3, P2-R3.6]

  - path: crates/mamba/src/runtime/symbols.rs
    action: MODIFY
    desc: |
      P2-R2: Register mb_class_set_metaclass symbol for JIT linking.
    requires: [P2-R2.4]

  - path: crates/mamba/src/codegen/cranelift/mod.rs
    action: MODIFY
    desc: |
      P2-R2: Emit mb_class_set_metaclass call after class definition when
      metaclass is present in the MIR instruction.
      P2-R3: Ensure all instance attribute access on user classes goes
      through mb_getattr/mb_setattr which already dispatch through the
      descriptor protocol — verify no direct field access bypass exists
      in the codegen path for user-class instances.
    requires: [P2-R2.1, P2-R3.1, P2-R3.2]

  # New conformance test fixtures
  - path: crates/mamba/tests/fixtures/conformance/language/fstring_nested.py
    action: CREATE
    desc: |
      New conformance fixture for nested f-strings (PEP 701): basic nesting,
      3-level nesting, same-quote reuse, format spec with nested braces,
      backslash in expression, lambda in f-string.
    requires: [P2-R1.1, P2-R1.2, P2-R1.3, P2-R1.4, P2-R1.5]

  - path: crates/mamba/tests/fixtures/conformance/language/fstring_nested.expected
    action: CREATE
    desc: Golden output for fstring_nested.py fixture.

  - path: crates/mamba/tests/fixtures/conformance/language/metaclass.py
    action: CREATE
    desc: |
      New conformance fixture for metaclass= keyword: Meta.__call__ intercept,
      Meta.__new__ namespace transform, ABCMeta abstract enforcement.
    requires: [P2-R2.1, P2-R2.2, P2-R2.3, P2-R2.5]

  - path: crates/mamba/tests/fixtures/conformance/language/metaclass.expected
    action: CREATE
    desc: Golden output for metaclass.py fixture.

  - path: crates/mamba/tests/fixtures/conformance/language/descriptors.py
    action: CREATE
    desc: |
      New conformance fixture for descriptor protocol: non-data descriptor
      __get__, data descriptor __set__ with validation, data descriptor
      priority over instance __dict__, __delete__ support.
    requires: [P2-R3.1, P2-R3.2, P2-R3.3, P2-R3.4, P2-R3.5]

  - path: crates/mamba/tests/fixtures/conformance/language/descriptors.expected
    action: CREATE
    desc: Golden output for descriptors.py fixture.
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


## Logic

```mermaid
flowchart TB
    subgraph P2_R1["P2-R1: Nested F-String Parsing"]
        FStr([FStr token from lexer])
        ParseParts[parse_fstring_parts]
        BraceOpen{"{ detected?"}
        ExtractExpr[Extract expression text]
        CheckFPrefix{"Starts with f\" or f'?"}
        RecurseParse[Recursive parse_fstring_parts on inner content]
        NormalParse[parse_fstring_expr — standard expression]
        BuildFString[Build Expr::FString with parts]
        FStr --> ParseParts
        ParseParts --> BraceOpen
        BraceOpen -->|Yes| ExtractExpr
        BraceOpen -->|No| BuildFString
        ExtractExpr --> CheckFPrefix
        CheckFPrefix -->|Yes| RecurseParse
        CheckFPrefix -->|No| NormalParse
        RecurseParse --> BuildFString
        NormalParse --> BuildFString
    end

    subgraph P2_R2["P2-R2: Metaclass Forwarding"]
        HirClass([HirClass with metaclass field])
        EmitDefine[Emit mb_class_define_multi]
        CheckMeta{"metaclass is Some?"}
        EmitSetMeta[Emit mb_class_set_metaclass]
        RegisterMeta[Store metaclass in CLASS_REGISTRY]
        Instantiate{"Instantiate class?"}
        HasMeta{"Has metaclass?"}
        MetaCall[Call metaclass.__call__]
        DefaultCall[Default type.__call__ — __new__ + __init__]
        HirClass --> EmitDefine
        EmitDefine --> CheckMeta
        CheckMeta -->|Yes| EmitSetMeta
        CheckMeta -->|No| Instantiate
        EmitSetMeta --> RegisterMeta
        RegisterMeta --> Instantiate
        Instantiate --> HasMeta
        HasMeta -->|Yes| MetaCall
        HasMeta -->|No| DefaultCall
    end

    subgraph P2_R3["P2-R3: Descriptor Protocol Codegen"]
        AttrAccess([Attribute access on instance])
        EmitGetattr[Codegen emits mb_getattr]
        CheckDataDesc{"Data descriptor in class MRO?"}
        InvokeGet[invoke_descriptor_get — __get__]
        CheckInstDict{"In instance __dict__?"}
        CheckNonDataDesc{"Non-data descriptor in MRO?"}
        InvokeGetND[invoke_descriptor_get — __get__]
        ReturnClassAttr[Return class attribute directly]
        RaiseAttrErr[Raise AttributeError]
        ReturnInstAttr[Return instance __dict__ value]
        AttrAccess --> EmitGetattr
        EmitGetattr --> CheckDataDesc
        CheckDataDesc -->|Yes| InvokeGet
        CheckDataDesc -->|No| CheckInstDict
        CheckInstDict -->|Found| ReturnInstAttr
        CheckInstDict -->|Not found| CheckNonDataDesc
        CheckNonDataDesc -->|Yes| InvokeGetND
        CheckNonDataDesc -->|No, class attr exists| ReturnClassAttr
        CheckNonDataDesc -->|No attr at all| RaiseAttrErr
    end
```

# Reviews
