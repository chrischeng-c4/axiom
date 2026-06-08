---
id: mamba-stdlib-main-spec
main_spec_ref: "crates/mamba/stdlib/main-module.md"
merge_strategy: new
filled_sections: [overview, changes, test-plan]
fill_sections: [overview, changes, test-plan]
create_complete: true
---

# Mamba Stdlib Main Spec

## Overview

Native `__main__` module for Mamba stdlib.

In CPython, `__main__` is the module in which top-level script code executes. The `if __name__ == "__main__"` idiom checks this module attribute. Mamba needs an equivalent native module that registers `__main__` with standard module attributes so that top-level scripts behave consistently with CPython.

This module follows the same constants-only pattern as `future_mod.rs`: a `register()` function that builds a `HashMap<String, MbValue>` of attributes and calls `register_module("__main__", attrs)`.

### Attributes

| Attribute | Value | Type |
|-----------|-------|------|
| `__name__` | `"__main__"` | `MbValue::from_string` |
| `__doc__` | `None` | `MbValue::none()` |
| `__loader__` | `None` | `MbValue::none()` |
| `__spec__` | `None` | `MbValue::none()` |
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

### Unit Tests (main_mod.rs)

| Test | Given | When | Then |
|------|-------|------|------|
| test_main_name_value | MbValue::from_string("__main__") created | as_str() called | Returns "__main__" |
| test_main_doc_is_none | MbValue::none() created | is_none() checked | Returns true |
| test_main_loader_is_none | MbValue::none() created | is_none() checked | Returns true |
| test_main_spec_is_none | MbValue::none() created | is_none() checked | Returns true |
| test_register_module | register() called | No panic | Module registered successfully |

### Integration

- Verify `cargo test -p mamba --lib` passes with no regressions
- Verify `__main__` module is accessible after register_stdlib()
## Changes

```yaml
files:
  - path: crates/mamba/src/runtime/stdlib/main_mod.rs
    action: create
    description: |
      New __main__ module implementation.
      - pub fn register() that creates HashMap with __name__, __doc__, __loader__, __spec__
      - Calls register_module("__main__", attrs)
      - Unit tests for attribute values and register() call

  - path: crates/mamba/src/runtime/stdlib/mod.rs
    action: modify
    description: |
      Wire main_mod into stdlib:
      - Add `pub mod main_mod;` declaration
      - Add `main_mod::register();` call in register_stdlib()
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
