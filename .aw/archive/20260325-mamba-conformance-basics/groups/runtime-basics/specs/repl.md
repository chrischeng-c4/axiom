---
id: repl
main_spec_ref: "crates/mamba/driver/repl"
merge_strategy: new
fill_sections: [overview, changes]
filled_sections: [overview, changes]
create_complete: true
---

# Repl

## Overview

The REPL currently echoes expression-statement results verbatim via `println!("{result}")` whenever `has_echo = true`, without inspecting the returned value. Before the cranelift-jit and builtins changes, `print()` returned raw `0`; after those fixes it returns `MbValue::none().to_bits()` (bit-pattern `0xFFFB_0000_0000_0000`, tag `TAG_NONE = 3`). Without a None-guard, this NaN sentinel is forwarded directly to stdout as extra output after any `print()` call — reproducing the reported spurious `0` behaviour.

Fix: in `eval`, decode the raw `i64` result via `MbValue::from_bits(result as u64)` and suppress the echo when `is_none()` returns `true`. This implements the R5 requirement ("print the result unless it is `None`") without changing the `eval_raw` return signature. The import `use crate::runtime::MbValue` is added to `driver/repl.rs`; `runtime::mod.rs` already re-exports `MbValue` via `pub use value::MbValue`.

A `test_repl_print_no_echo` unit test is added to verify that evaluating `print(42)\n` yields `has_echo = true` (the call is an expression statement) and a result for which `MbValue::from_bits(val as u64).is_none()` holds — confirming the None-guard fires correctly.
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
  - path: crates/mamba/src/driver/repl.rs
    action: MODIFY
    desc: "Add `use crate::runtime::MbValue;` import (runtime::mod.rs already re-exports MbValue via pub use value::MbValue)"
  - path: crates/mamba/src/driver/repl.rs
    action: MODIFY
    desc: "In eval: replace `if has_echo { println!(\"{result}\"); }` with `if has_echo { if !MbValue::from_bits(result as u64).is_none() { println!(\"{result}\"); } }` — suppress None results per R5"
  - path: crates/mamba/src/driver/repl.rs
    action: MODIFY
    desc: "Add test_repl_print_no_echo in #[cfg(test)] mod: call eval_raw(\"print(42)\\n\"); assert has_echo == true; assert MbValue::from_bits(val as u64).is_none() — verifies that print() result is TAG_NONE and would be suppressed by the None-guard in eval"
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
