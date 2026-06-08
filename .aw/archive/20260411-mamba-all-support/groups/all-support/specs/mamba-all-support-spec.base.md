---
id: mamba-all-support-spec
main_spec_ref: "crates/mamba/runtime/module.md"
merge_strategy: new
---

# Mamba All Support Spec

## Overview

Implement Python's `__all__` mechanism for controlling `from module import *` behavior.

In CPython, when a module defines `__all__` (a list of strings), `from module import *` only imports the names listed in `__all__`. If `__all__` is not defined, the star import exports all public names (those not starting with `_`).

Currently Mamba has no `__all__` support. Star imports (`from X import *`) are parsed but the MIR lowering treats `*` as a literal attribute name, which is incorrect. This change adds:

1. A new runtime function `mb_import_star(module_name) -> MbValue` that returns a dict of names to import, respecting `__all__` if present.
2. MIR lowering logic to detect `from X import *` and emit a call to `mb_import_star` followed by dynamic symbol binding.
3. Proper `__all__` storage in module attrs during module compilation.

The `__all__` variable must be preserved during module compilation (currently `__`-prefixed names are filtered out at line 587 of module.rs). The star-import runtime function checks for `__all__` in the module's attrs and filters accordingly.
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

| Test | Type | Validates |
|------|------|-----------|
| `test_import_star_with_all` | unit | R1, R2: module with `__all__` returns only listed names |
| `test_import_star_without_all` | unit | R2: module without `__all__` returns all public (non-underscore) names |
| `test_import_star_empty_all` | unit | R2: module with `__all__ = []` returns empty dict |
| `test_import_star_preserves_all_attr` | unit | R1: `__all__` is stored in module attrs |
| `test_import_star_registered_in_symbols` | unit | R3: `mb_import_star` appears in RT_SYMBOLS |
| `test_resolve_star_import_no_star_symbol` | unit | R5: resolver does not define `*` as a symbol |
## Changes

```yaml
files:
  - path: crates/mamba/src/runtime/module.rs
    action: modify
    description: |
      1. Add mb_import_star function that loads a module and returns a dict
         of exported names respecting __all__ if present.
      2. In compile_and_exec_module, preserve __all__ in module attrs
         (add it to the whitelist alongside __name__ and __doc__).
      3. Add unit tests for mb_import_star.
  - path: crates/mamba/src/runtime/symbols.rs
    action: modify
    description: |
      Register mb_import_star in RT_SYMBOLS with signature
      fn(MbValue) -> MbValue, [I64], I64.
  - path: crates/mamba/src/lower/hir_to_mir.rs
    action: modify
    description: |
      In the HirStmt::Import lowering, detect when names == [(star, None)]
      and emit CallExtern to mb_import_star instead of per-name getattr.
  - path: crates/mamba/src/resolve/pass.rs
    action: modify
    description: |
      Skip defining star as a symbol when processing from-import-star.
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