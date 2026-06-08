---
id: cranelift
main_spec_ref: "crates/mamba/codegen/cranelift"
merge_strategy: append
fill_sections: [overview, changes]
filled_sections: [overview, changes]
create_complete: true
---

# Cranelift

## Overview

The Cranelift AOT backend (`CraneliftBackend` in `codegen/cranelift/mod.rs`) has the same return-value propagation bug as the JIT backend when emitting calls to user-defined functions. `emit_internal_call` retrieves the call result via `builder.inst_results(call)[0]` and stores it directly into the destination VReg using the call-site TypeId — without checking whether the callee's actual return type matches the call-site's expected type.

When a callee is compiled with a primitive return type (`Ty::Int` → raw i64, `Ty::Float` → f64) but the call-site TypeId resolves to a non-primitive (`Ty::Dynamic` or `Ty::Any`), the raw value is stored without NaN-boxing. Subsequent dynamic-dispatch operations — such as `BinOp` emission through `mb_dispatch_binop` — receive raw i64/f64 integers instead of tagged MbValues and produce incorrect results (e.g., `fib(30) == 0` instead of 832040).

This mirrors the root cause fixed in `CraneliftJitBackend` (see cranelift-jit spec). The AOT backend requires the same fix: track each callee's declared return TypeId and, in `emit_internal_call`, conditionally emit a `mb_box_int` / `mb_box_bool` / `mb_box_float` CallExtern to NaN-box the result before writing it to the destination VReg when the callee return type is primitive but the call-site expects a non-primitive value.

`mb_box_int`, `mb_box_bool`, and `mb_box_float` are already declared in the runtime (`crates/mamba/src/runtime/builtins.rs`) and exposed as `MirExtern` entries via `runtime_externs()`. No new runtime functions are required — the AOT backend links these symbols at object-link time.
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
  - path: crates/mamba/src/codegen/cranelift/mod.rs
    action: MODIFY
    desc: "Add `internal_return_tys: HashMap<u32, TypeId>` field to CraneliftBackend; initialize to HashMap::new() in new()"
  - path: crates/mamba/src/codegen/cranelift/mod.rs
    action: MODIFY
    desc: "In declare_internal: after self.internal_funcs.insert(body.name.0, func_id), insert self.internal_return_tys.insert(body.name.0, body.return_ty) to track each callee's declared return TypeId"
  - path: crates/mamba/src/codegen/cranelift/mod.rs
    action: MODIFY
    desc: "In emit_internal_call: after builder.inst_results(call)[0], look up callee_return_ty = self.internal_return_tys.get(&sym_id); if callee's Ty resolves to Int or Bool and call-site Ty is non-primitive (Dynamic/Any), emit CallExtern mb_box_int(result) before def_var; if callee returns Float and call-site is non-primitive, emit CallExtern mb_box_float(result)"
  - path: crates/mamba/tests/codegen_tests.rs
    action: MODIFY
    desc: "Add test_aot_recursive_fib: compile recursive fib() without type annotations through the AOT backend, link and run, assert fib(10) == 55"
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
