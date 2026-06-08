---
id: future-stdlib
main_spec_ref: crates/mamba/stdlib/future.md
merge_strategy: new
fill_sections: [overview, changes]
filled_sections: [overview, changes]
create_complete: true
---

# Future Stdlib

## Overview

<!-- type: overview lang: markdown -->

Native implementation of Python's `__future__` module for Mamba. This module provides future statement definitions — compile-time feature flags that enable language features from later Python versions.

In CPython, `__future__` exports `_Feature` objects and `CO_*` compiler flag constants. For Mamba, we implement a simplified version that exports the feature flag names as integer constants (matching CPython's `CO_FUTURE_*` values), since Mamba already enables all modern Python features by default.

The module follows the standard `register()` + `register_module()` pattern per `native-implementations.md`. It registers under the name `__future__` (double-underscore) and exports:
- Feature flags: `annotations`, `division`, `print_function`, `unicode_literals`, `with_statement`, `absolute_import`
- Compiler flag constants: `CO_NESTED`, `CO_GENERATOR_ALLOWED`, `CO_FUTURE_DIVISION`, `CO_FUTURE_ABSOLUTE_IMPORT`, `CO_FUTURE_WITH_STATEMENT`, `CO_FUTURE_PRINT_FUNCTION`, `CO_FUTURE_UNICODE_LITERALS`, `CO_FUTURE_ANNOTATIONS`

This is a constants-only module with no runtime behavior. It exists purely for CPython compatibility — libraries that `from __future__ import annotations` will resolve the import without error.
## Requirements
<!-- type: requirements lang: markdown -->

<!-- TODO -->

## Scenarios
<!-- type: scenarios lang: markdown -->

<!-- TODO -->

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

<!-- TODO -->

## Changes

<!-- type: changes lang: yaml -->

```yaml
files:
  - path: crates/mamba/src/runtime/stdlib/future_mod.rs
    action: create
    description: |
      New native __future__ module.
      - `pub fn register()`: registers "__future__" via `register_module("__future__", attrs)`
      - CO_* compiler flag constants as MbValue::from_int():
        CO_NESTED=0x0010, CO_GENERATOR_ALLOWED=0,
        CO_FUTURE_DIVISION=0x20000, CO_FUTURE_ABSOLUTE_IMPORT=0x40000,
        CO_FUTURE_WITH_STATEMENT=0x80000, CO_FUTURE_PRINT_FUNCTION=0x100000,
        CO_FUTURE_UNICODE_LITERALS=0x200000, CO_FUTURE_ANNOTATIONS=0x1000000
      - Feature flag constants:
        annotations=CO_FUTURE_ANNOTATIONS, division=CO_FUTURE_DIVISION,
        print_function=CO_FUTURE_PRINT_FUNCTION, unicode_literals=CO_FUTURE_UNICODE_LITERALS,
        with_statement=CO_FUTURE_WITH_STATEMENT, absolute_import=CO_FUTURE_ABSOLUTE_IMPORT
      - Unit tests: test_co_future_annotations_value, test_feature_flags_are_integers,
        test_register_module

  - path: crates/mamba/src/runtime/stdlib/mod.rs
    action: modify
    description: |
      - Add `pub mod future_mod;` declaration
      - Add `future_mod::register();` call in `register_stdlib()`
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
