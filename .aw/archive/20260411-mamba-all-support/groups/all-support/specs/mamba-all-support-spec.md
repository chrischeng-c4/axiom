---
id: mamba-all-support-spec
main_spec_ref: crates/mamba/runtime/module.md
merge_strategy: extend
fill_sections: [overview, changes, logic, test-plan]
filled_sections: [overview, changes, logic, test-plan]
create_complete: true
---

# Mamba All Support Spec

## Overview

<!-- type: overview lang: markdown -->
Implement Python's `__all__` mechanism for controlling `from module import *` behavior in Mamba.

When a module defines `__all__` as a list/tuple of string literals at top level, `from X import *` must bind only those names. When `__all__` is absent, fall back to current behavior (all public names — no leading underscore). If a name in `__all__` does not exist in the module namespace, raise `AttributeError` at import time.

This requires three coordinated changes: (1) preserve `__all__` in module attrs during compilation (currently filtered out), (2) add a new runtime function `mb_import_star` that respects `__all__` when collecting star-import names, (3) update the HIR-to-MIR lowering to emit a call to `mb_import_star` for `from X import *` statements.
## Requirements

| ID | Requirement | Priority |
|----|------------|----------|
| R1 | Preserve `__all__` in module attrs: during module compilation, `__all__` must not be filtered out by the `starts_with("__")` check in `compile_and_exec_module()` | P0 |
| R2 | New runtime function `mb_import_star(module_name: MbValue) -> MbValue`: imports the module, reads `__all__` from attrs if present, and returns a dict mapping name->value for each name in `__all__`. If `__all__` is absent, returns all public attrs (not starting with `_`). | P0 |
| R3 | Register `mb_import_star` in the runtime symbol table (`symbols.rs`) so it is available to the JIT backend | P0 |
| R4 | MIR lowering: when `from X import *` is detected (names contains a single `("*", None)` entry), emit `CallExtern` to `mb_import_star` instead of per-name `mb_module_getattr` calls, then iterate the returned dict to store each name into the global namespace | P0 |
| R5 | Resolve pass: when `from X import *` is encountered, skip defining `*` as a symbol (star imports bind names dynamically at runtime, not statically at resolve time) | P1 |
## Scenarios

### Scenario 1: Module with `__all__` defined

**Given** a module `mymod.py` containing:
```python
__all__ = ["foo", "bar"]
foo = 1
bar = 2
_private = 3
baz = 4
```

**When** another file executes `from mymod import *`

**Then** only `foo` and `bar` are imported into the caller's namespace. `_private` and `baz` are not available.

### Scenario 2: Module without `__all__`

**Given** a module `othermod.py` containing:
```python
foo = 1
bar = 2
_private = 3
```

**When** another file executes `from othermod import *`

**Then** `foo` and `bar` are imported (public names). `_private` is not imported (underscore-prefixed).

### Scenario 3: Native module star import

**Given** a native module registered via `mb_module_register` with attrs including `__all__`

**When** `from native_mod import *` is executed

**Then** only names listed in `__all__` are imported.

### Scenario 4: Empty `__all__`

**Given** a module with `__all__ = []`

**When** `from module import *` is executed

**Then** no names are imported.
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

| Test | Type | Validates |
|------|------|-----------|
| `test_import_star_with_all` | unit | R1, R2: mb_import_star with module defining __all__ returns only listed names |
| `test_import_star_without_all` | unit | R2: mb_import_star without __all__ returns all public (non-underscore) names |
| `test_import_star_empty_all` | unit | R2: mb_import_star with __all__ = [] returns empty dict |
| `test_import_star_preserves_all_attr` | unit | R1: __all__ is present in module attrs after compile_and_exec_module |
| `test_import_star_registered_in_symbols` | unit | R3: mb_import_star appears in RT_SYMBOLS |
| `test_resolve_star_import_no_star_symbol` | unit | R5: resolve pass does not define * as a symbol for from X import * |

### Scenario: Module with __all__
- **Given** a module `mymod` with `__all__ = ["foo", "bar"]`, `foo = 1`, `bar = 2`, `baz = 3`
- **When** `from mymod import *` is executed
- **Then** `foo` and `bar` are bound in caller scope; `baz` is not

### Scenario: Module without __all__
- **Given** a module `othermod` with `foo = 1`, `_priv = 2`, `bar = 3`
- **When** `from othermod import *` is executed
- **Then** `foo` and `bar` are bound; `_priv` is not

### Scenario: __all__ with missing name (R3/AC3)
- **Given** a module `badmod` with `__all__ = ["missing"]`
- **When** `from badmod import *` is executed
- **Then** `AttributeError: module 'badmod' has no attribute 'missing'` is raised

### Scenario: Explicit import not affected (R5/AC4)
- **Given** a module `selectmod` with `__all__ = ["a"]` and `b = 99`
- **When** `from selectmod import b` is executed
- **Then** `b` is bound (explicit import ignores __all__)
## Changes

<!-- type: changes lang: yaml -->
changes:
  - path: crates/mamba/src/runtime/module.rs
    action: MODIFY
    description: Add mb_import_star function and preserve __all__ in module attrs
    targets:
      - type: function
        name: compile_and_exec_module
        change: add __all__ to the dunder whitelist so it is preserved in module attrs
      - type: function
        name: mb_import_star
        change: new public extern fn — loads module by name, reads __all__ from attrs if present, returns dict of name->value for star-import
  - path: crates/mamba/src/runtime/symbols.rs
    action: MODIFY
    description: Register mb_import_star in RT_SYMBOLS
    targets:
      - type: function
        name: register_symbols
        change: add rt_sym! entry for mb_import_star with signature fn(MbValue) -> MbValue, [I64], I64
  - path: crates/mamba/src/lower/hir_to_mir.rs
    action: MODIFY
    description: Detect from X import * and emit CallExtern to mb_import_star
    targets:
      - type: function
        name: lower_import_stmt
        change: when names == [("*", None)], emit CallExtern mb_import_star instead of per-name mb_module_getattr calls; iterate returned dict to store each name into globals
  - path: crates/mamba/src/resolve/pass.rs
    action: MODIFY
    description: Skip defining * as a symbol for star-imports
    targets:
      - type: function
        name: resolve_from_import
        change: when names contains ("*", None), skip symbol definition for * — star imports bind names dynamically at runtime
do_not_touch: [mb_import_from, mb_module_register, mb_module_import]
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

<!-- type: logic lang: mermaid -->
flowchart TD
    A[from X import *] --> B{is star-import?}
    B -- yes --> C[CallExtern mb_import_star with module name]
    B -- no --> D[per-name mb_module_getattr existing path]
    C --> E[mb_import_star loads module X]
    E --> F{__all__ in module.attrs?}
    F -- yes --> G[iterate __all__ list]
    G --> H{name exists in module.attrs?}
    H -- yes --> I[add name->value to result dict]
    H -- no --> J[raise AttributeError: module X has no attribute name]
    F -- no --> K[iterate all module.attrs]
    K --> L{name starts with _?}
    L -- no --> I
    L -- yes --> M[skip]
    I --> N[StoreGlobal name value in caller scope]
    N --> O[next name]
    O --> H
    O --> L

# Reviews
