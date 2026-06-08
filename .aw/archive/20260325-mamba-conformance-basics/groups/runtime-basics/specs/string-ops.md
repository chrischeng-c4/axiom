---
id: string-ops
main_spec_ref: "crates/mamba/runtime/string-ops"
merge_strategy: append
fill_sections: [overview, changes]
filled_sections: [overview, changes]
create_complete: true
---

# String Ops

## Overview

`mb_str_concat(a: MbValue, b: MbValue) -> MbValue` is already implemented in `runtime/string_ops.rs` (line 28) and registered in `symbols.rs` as `[I64, I64] -> I64`. However, `str + str` raises "arithmetic requires numeric types" at compile time due to two bugs in the type-checker and lowering layers.

**Bug 1 - Type checker** (`types/check_expr.rs`, `check_binop`, `BinOp::Add` arm):
- `numeric_promotion(lt, rt)` returns None for Str+Str.
- `types_compatible(lt, rt)` passes for Str+Str (same types).
- Falls through to `is_numeric()` guard: `Ty::Str` is not numeric, emits "arithmetic requires numeric types" and aborts compilation.
- Fix: insert a `Ty::Str + Ty::Str -> Ty::Str` short-circuit **before** the `is_numeric()` guard.

**Bug 2 - HIR-to-MIR lowering** (`lower/hir_to_mir.rs`, `HirExpr::BinOp`):
- `needs_runtime = true` for Str+Str (Ty::Str is not in Int | Float | Bool).
- The generic `binop_to_runtime(HirBinOp::Add)` returns `"mb_add"`, which is correct for mixed numeric but wrong for strings.
- Fix: before the `binop_to_runtime` path, detect `op == Add && lt == Ty::Str && rt == Ty::Str` and emit `CallExtern { name: "mb_str_concat", args: [l, r] }` directly.

No changes to `string_ops.rs` or `symbols.rs` are needed -- the runtime function is already implemented and registered.
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
  - path: crates/mamba/src/types/check_expr.rs
    action: MODIFY
    desc: "In check_binop, BinOp::Add arm: add Ty::Str + Ty::Str -> Ty::Str case before the is_numeric() guard; when both lt and rt are Ty::Str, return self.tcx.str() immediately without requiring numeric promotion"
  - path: crates/mamba/src/lower/hir_to_mir.rs
    action: MODIFY
    desc: "In HirExpr::BinOp lowering, before the needs_runtime/binop_to_runtime dispatch: detect op==Add && lt==Ty::Str && rt==Ty::Str, box both operands, emit CallExtern { name: 'mb_str_concat', args: [boxed_l, boxed_r], ty: self.tcx.str() } and return dest early -- bypassing the generic mb_add dispatch"
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
