---
id: type-checker
main_spec_ref: "crates/mamba/types/type-checker"
merge_strategy: append
fill_sections: [overview, changes]
filled_sections: [overview, changes]
create_complete: true
---

# Type Checker

## Overview

The type checker's `check_binop` function in `types/check_expr.rs` incorrectly rejects the `+` operator when both operands are `Ty::Str`. The `BinOp::Add` arm handles numeric tower promotion (int+float) and then guards with `is_numeric()` before allowing the operation — but `Ty::Str` is not numeric, so `"hello" + "world"` raises "arithmetic requires numeric types" instead of resolving to `Ty::Str`.

This is the type-checking half of the string concatenation bug (#Q3 in pre-clarifications). The codegen side already dispatches `str + str` through `mb_dispatch_binop` → `mb_str_concat` at runtime; the type checker's spurious error is what prevents the expression from reaching codegen.

Fix: in `check_binop`, add an early `Str + Str → Str` branch in the `BinOp::Add` arm, placed before the `is_numeric()` guard. If both operands are `Ty::Str`, return `Ty::Str` immediately. If only one operand is `Ty::Str` (e.g. `str + int`), fall through to the existing type-mismatch error. No changes to codegen, runtime symbols, or other compiler phases are required.
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
    desc: "In check_binop, BinOp::Add arm: insert early branch — if both lt and rt are Ty::Str, return self.tcx.str() immediately, before the numeric_promotion and is_numeric() guards. Mixed str+non-str falls through to the existing operand-mismatch error."
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
