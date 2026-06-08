---
id: builtins
main_spec_ref: "crates/mamba/runtime/builtins"
merge_strategy: append
fill_sections: [overview, changes]
filled_sections: [overview, changes]
create_complete: true
---

# Builtins

## Overview

`mb_print` and `mb_print_args` are declared `void`-returning in both Rust and the JIT symbol table (`[I64], Void`). When Cranelift JIT calls a `Void`-returning extern, the result register is undefined — on x86-64 this decodes as 0, which NaN-boxing interprets as `TAG_INT(0)`. The top-level expression evaluator then outputs this spurious `0` after every `print()` call.

Fix: change `mb_print` and `mb_print_args` to return `MbValue::none()` (TAG_NONE), and update their `RuntimeSymbol` entries in `symbols.rs` from `MirType::Void` to `MirType::I64`. Statement-level callers in `lower/hir_to_mir.rs` discard the return value, so the change is backward compatible.
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

```yaml
files:
  - path: crates/mamba/src/runtime/builtins.rs
    action: MODIFY
    desc: "Change mb_print signature from void to `-> MbValue`; return MbValue::none() at end of function body"
  - path: crates/mamba/src/runtime/builtins.rs
    action: MODIFY
    desc: "Change mb_print_args signature to `-> MbValue`; replace implicit void return and early `return;` with `return MbValue::none();`; tail-call mb_print returns MbValue::none()"
  - path: crates/mamba/src/runtime/symbols.rs
    action: MODIFY
    desc: "Update rt_sym! for mb_print: cast to `fn(super::MbValue) -> super::MbValue`, change return_type from Void to I64"
  - path: crates/mamba/src/runtime/symbols.rs
    action: MODIFY
    desc: "Update rt_sym! for mb_print_args: cast to `fn(super::MbValue) -> super::MbValue`, change return_type from Void to I64"
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
