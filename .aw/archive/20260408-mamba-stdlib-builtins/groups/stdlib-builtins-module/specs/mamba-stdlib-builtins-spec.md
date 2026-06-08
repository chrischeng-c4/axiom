---
id: mamba-stdlib-builtins-spec
main_spec_ref: crates/mamba/stdlib/builtins.md
merge_strategy: new
fill_sections: [overview, changes, test-plan]
filled_sections: [overview, changes, test-plan]
create_complete: true
---

# Mamba Stdlib Builtins Spec

## Overview

<!-- type: overview lang: markdown -->

Native stdlib `builtins` module for Mamba (#997). In CPython, `import builtins` gives access to the built-in namespace containing all built-in functions and constants. Mamba already implements all built-in functions as `mb_*` runtime functions in `builtins.rs`, `class.rs`, `iter.rs`, and `file_io.rs`. This module creates an importable `builtins` module that exposes these existing implementations as module attributes.

Follows the `future_mod.rs` / `main_mod.rs` registration pattern: a `register()` function builds a `HashMap<String, MbValue>` and calls `register_module("builtins", attrs)`. Functions are exposed as `MbValue::from_func()` via `unsafe extern "C" fn` dispatch wrappers registered in `NATIVE_FUNC_ADDRS`. Constants (`True`, `False`, `None`) are exposed as `MbValue::from_bool()` / `MbValue::none()`.

Source: `runtime/stdlib/builtins_mod.rs`
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

## Test Plan

<!-- type: test-plan lang: markdown -->

### Acceptance Criteria

1. `builtins_mod.rs` exists at `crates/mamba/src/runtime/stdlib/builtins_mod.rs` and compiles without warnings
2. `register()` calls `register_module("builtins", attrs)` with all required attributes
3. Functions exposed via `MbValue::from_func()` with `unsafe extern "C" fn dispatch_*` wrappers using `(args_ptr, nargs)` ABI
4. Each dispatch wrapper registered in `NATIVE_FUNC_ADDRS` so `mb_call_spread` uses native ABI
5. Constants: `True` -> `MbValue::from_bool(true)`, `False` -> `MbValue::from_bool(false)`, `None` -> `MbValue::none()`
6. All Python builtins from requirements are mapped
7. `mod.rs` declares `pub mod builtins_mod` and `register_stdlib()` calls `builtins_mod::register()`
8. `cargo test -p mamba --lib` passes with no regressions

### Unit Tests

- `test_register_module` - register() does not panic
- `test_true_false_none_constants` - True/False/None have correct MbValue representations
- `test_dispatch_len` - dispatch_len returns correct length for a list
- `test_dispatch_abs` - dispatch_abs returns absolute value of negative int
- `test_dispatch_int` - dispatch_int converts float to int
- `test_dispatch_bool` - dispatch_bool converts int to bool
- `test_dispatch_chr` - dispatch_chr converts int to character string
- `test_dispatch_ord` - dispatch_ord converts character to int
## Changes

<!-- type: changes lang: yaml -->

changes:
  - path: crates/mamba/src/runtime/stdlib/builtins_mod.rs
    action: CREATE
    description: |
      New builtins module. Implements register() function that builds
      HashMap<String, MbValue> with all built-in functions and constants.
      Functions are wrapped in unsafe extern "C" fn dispatch_* wrappers
      using the (args_ptr, nargs) ABI and registered via NATIVE_FUNC_ADDRS.
      Constants True, False, None are MbValue::from_bool / MbValue::none.
      Calls register_module("builtins", attrs).
  - path: crates/mamba/src/runtime/stdlib/mod.rs
    action: MODIFY
    targets:
      - type: function
        name: register_stdlib
        change: Add builtins_mod::register() call
    description: |
      Add `pub mod builtins_mod;` declaration and call
      `builtins_mod::register()` in register_stdlib().
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
