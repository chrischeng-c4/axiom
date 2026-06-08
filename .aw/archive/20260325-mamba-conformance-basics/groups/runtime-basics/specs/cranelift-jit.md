---
id: cranelift-jit
main_spec_ref: "crates/mamba/codegen/cranelift-jit"
merge_strategy: append
fill_sections: [overview, changes]
filled_sections: [overview, requirements, scenarios, changes]
create_complete: true
---

# Cranelift Jit

## Overview

This change fixes the recursive function return-value propagation bug in `CraneliftJitBackend`. When a JIT-compiled user function calls another user function (including recursive self-calls), `emit_internal_call` retrieves the result via `builder.inst_results(call)[0]` and stores it directly into the destination VReg using the call-site's TypeId. The callee, however, is compiled with its own `body.return_ty`. When these types disagree — specifically when the callee returns a raw primitive (`Ty::Int` → raw i64) but the call-site TypeId resolves to a non-primitive (`Ty::Dynamic` or `Ty::Any`) — the raw value is stored without NaN-boxing. Subsequent dynamic-dispatch operations (`mb_dispatch_binop` for `BinOp` with `Ty::Dynamic`) then receive raw i64 integers instead of NaN-boxed MbValues, producing 0 or incorrect results instead of the correct value.

For example, `fib(30)` compiled without type annotations uses `Ty::Dynamic` throughout. The recursive calls emit `MirInst::Call` (fib is a user-defined function). `emit_internal_call` gets the callee's raw i64 result (e.g., 55) and stores it as-is. The subsequent `BinOp::Add` with `Ty::Dynamic` dispatches through `mb_dispatch_binop`, which expects NaN-boxed MbValues; receiving raw integers, it returns 0. The result propagates as 0 through all recursion levels, giving `fib(30) = 0` instead of 832040.

Fix: extend `CraneliftJitBackend` with `internal_return_tys: HashMap<u32, TypeId>`, populated during `declare_internal` (Phase 2). In `emit_internal_call`, after obtaining the call result, look up the callee's stored return TypeId. If the callee returns a primitive (`Ty::Int`, `Ty::Bool`, `Ty::Float`) and the call-site type is non-primitive, emit an `mb_box_int` / `mb_box_bool` / `mb_box_float` extern call to NaN-box the result before storing it in the destination VReg.
## Requirements

### R6 – Void Extern Return Produces `MbValue::none()`

```yaml
id: R6
priority: high
```

When `emit_extern_call` is invoked with a non-`None` `dest` VReg for an extern function declared with `return_type: MirType::Void`, the destination register must be set to `MbValue::none().to_bits() as i64` (TAG_NONE NaN-boxed sentinel). Writing raw integer `0` is incorrect and must be eliminated from both the `Some(ext)` void branch and the `None`-ext fallback branch.

### R7 – Primitive Internal Return is NaN-boxed

```yaml
id: R7
priority: high
```

When `emit_internal_call` captures the return value of an internal function whose TypeContext return type resolves to `Ty::Int` or `Ty::Bool`, the raw i64 scalar must be NaN-boxed (using inline Cranelift IR) before it is stored in the destination VReg. The NaN-boxing operation for `Int` is: `bor(band(raw, PAYLOAD_MASK), INT_NAN_PREFIX)` where `PAYLOAD_MASK = 0x0000_FFFF_FFFF_FFFF` and `INT_NAN_PREFIX = 0xFFF9_0000_0000_0000`. This ensures all values in VRegs are valid `MbValue` bit-patterns at all use sites including extern call arguments.
## Scenarios

### Scenario: Void Extern Result Captured as None

- **GIVEN** A JIT-compiled function that calls a void extern (`mb_print`) and assigns its result to a VReg.
- **WHEN** The compiler emits a `CallExtern` with `dest: Some(vreg)` against a `Void`-returning extern.
- **THEN** The VReg is initialized to `MbValue::none().to_bits()` (TAG_NONE), not `0`. Downstream reads of the VReg treat the value as Python `None`.

### Scenario: Recursive fib Return Value Propagates as NaN-boxed Int

- **GIVEN** A JIT-compiled `fib(n: int) -> int` function using recursive calls.
- **WHEN** `fib(30)` is executed and its result is forwarded to `mb_print(MbValue)`.
- **THEN** `emit_internal_call` NaN-boxes the raw i64 `832040` into `MbValue::from_int(832040)` before storing in dest. `mb_print` receives a valid `MbValue` and prints `832040` with no spurious output.

### Scenario: Existing Typed Return Calls Unaffected

- **GIVEN** An extern call to `mb_add(I64, I64) -> I64` or `mb_dispatch_binop(I64, I64, I64) -> I64`.
- **WHEN** The extern has a non-void return type.
- **THEN** The existing non-void path in `emit_extern_call` (via `unmarshal_return`) continues to operate correctly. No boxing is applied to these paths.
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
  - path: crates/mamba/src/codegen/cranelift/jit.rs
    action: MODIFY
    desc: "Add `internal_return_tys: HashMap<u32, TypeId>` field to CraneliftJitBackend; initialize to HashMap::new() in new() and new_with_externals()"
  - path: crates/mamba/src/codegen/cranelift/jit.rs
    action: MODIFY
    desc: "In declare_internal: after self.internal_funcs.insert(body.name.0, func_id), insert self.internal_return_tys.insert(body.name.0, body.return_ty)"
  - path: crates/mamba/src/codegen/cranelift/jit.rs
    action: MODIFY
    desc: "In emit_internal_call: after builder.inst_results(call)[0], look up callee_return_ty = self.internal_return_tys.get(&sym_id); if callee_return_ty resolves to Ty::Int or Ty::Bool and call-site type tcx.get(*ty) is not a primitive (i.e., not Int/Bool/Float), emit CallExtern mb_box_int(result) to NaN-box before def_var; if callee_return_ty is Ty::Float and call-site type is non-primitive, emit mb_box_float(result)"
  - path: crates/mamba/tests/jit_tests.rs
    action: MODIFY
    desc: "Add test_jit_recursive_fib: compile and JIT-run recursive fib(n) without type annotations, assert jit_run result for fib(30) == 832040"
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