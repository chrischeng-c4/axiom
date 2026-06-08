---
id: no-arg-constructor-codegen-fix
main_spec_ref: "crates/mamba/lower/hir-to-mir"
merge_strategy: new
fill_sections: [overview, requirements, scenarios, changes]
filled_sections: [overview, requirements, scenarios, changes]
create_complete: true
---

# No Arg Constructor Codegen Fix

## Overview

`builtin_extern_map()` routes `list` → `mb_list_from_iterable`, `tuple` → `mb_tuple_from_iterable`, `set` → `mb_set_from_iterable`, `dict` → `mb_dict_from_pairs`. These runtime functions expect exactly 1 parameter (the iterable). When Python code calls `list()`, `tuple()`, or `set()` with zero arguments, the HIR-to-MIR lowering emits `CallExtern { name: "mb_list_from_iterable", args: [] }` — a zero-arg call against a one-param extern. Cranelift's verifier rejects the IR because the argument count mismatches the declared function signature.

The zero-param constructor variants `mb_list_new`, `mb_tuple_new`, `mb_set_new`, `mb_dict_new` already exist in the runtime (registered with `params=[], return=I64`) and are used directly by comprehension lowering (R1). Fix: add an arity guard in the builtin call lowering path — when `args.is_empty()` and the extern name is a `_from_iterable`/`_from_pairs` variant, redirect to the corresponding `_new` variant.

Issue: #1109
## Requirements

| ID | Title | Priority | Acceptance Criteria |
|----|-------|----------|---------------------|
| R1 | Zero-arg builtin constructor arity guard | P0 | When `list()`, `tuple()`, `set()`, or `dict()` is called with zero arguments, the lowering emits `CallExtern` to the `_new` variant (`mb_list_new`, `mb_tuple_new`, `mb_set_new`, `mb_dict_new`) instead of the `_from_iterable`/`_from_pairs` variant |
| R2 | Non-zero-arg path unchanged | P0 | When `list(x)`, `tuple(x)`, `set(x)`, `dict(pairs)` is called with 1+ arguments, the lowering continues to emit `CallExtern` to `mb_list_from_iterable`, `mb_tuple_from_iterable`, `mb_set_from_iterable`, `mb_dict_from_pairs` as before |
| R3 | Cranelift verifier passes | P0 | Zero-arg constructor calls no longer produce IR signature mismatches; `cargo test -p mamba` passes without verifier errors for `list()`, `tuple()`, `set()` |
| R4 | Runtime correctness | P0 | `list()` returns `[]`, `tuple()` returns `()`, `set()` returns `set()`, `dict()` returns `{}` — matching CPython 3.12 semantics |

### Constraints

- Fix is localized to `hir_to_mir.rs` builtin call dispatch — no runtime or symbol changes needed
- `mb_list_new`/`mb_tuple_new`/`mb_set_new`/`mb_dict_new` are already registered in `symbols.rs` with `params=[], return=I64`
- Comprehension lowering (existing R1 in main spec) already calls `_new` variants directly — the pattern is proven
## Scenarios

### S1: list() with zero args produces empty list (R1, R3, R4)

**GIVEN** `x = list(); print(x); print(type(x).__name__)`
**WHEN** executed through Mamba JIT
**THEN** output is `[]` and `list` — `mb_list_new` is called with 0 args, Cranelift verifier passes

### S2: tuple() with zero args produces empty tuple (R1, R3, R4)

**GIVEN** `x = tuple(); print(x); print(len(x))`
**WHEN** executed through Mamba JIT
**THEN** output is `()` and `0` — `mb_tuple_new` is called with 0 args

### S3: set() with zero args produces empty set (R1, R3, R4)

**GIVEN** `x = set(); print(len(x)); print(type(x).__name__)`
**WHEN** executed through Mamba JIT
**THEN** output is `0` and `set` — `mb_set_new` is called with 0 args

### S4: list(iterable) still routes to mb_list_from_iterable (R2)

**GIVEN** `x = list(range(3)); print(x)`
**WHEN** executed through Mamba JIT
**THEN** output is `[0, 1, 2]` — `mb_list_from_iterable` is called with 1 arg as before

### S5: tuple(iterable) still routes to mb_tuple_from_iterable (R2)

**GIVEN** `x = tuple([1, 2, 3]); print(x)`
**WHEN** executed
**THEN** output is `(1, 2, 3)` — `mb_tuple_from_iterable` receives the iterable arg

### S6: set(iterable) still routes to mb_set_from_iterable (R2)

**GIVEN** `x = set([1, 2, 2, 3]); print(sorted(x))`
**WHEN** executed
**THEN** output is `[1, 2, 3]` — deduplication via `mb_set_from_iterable`

### S7: dict() with zero args produces empty dict (R1, R4)

**GIVEN** `d = dict(); print(d); print(len(d))`
**WHEN** executed
**THEN** output is `{}` and `0` — `mb_dict_new` is called with 0 args
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
  - path: crates/mamba/src/lower/hir_to_mir.rs
    action: MODIFY
    desc: |
      In the builtin call dispatch path (after `if let Some(extern_name) = self.builtin_syms.get(&func_sym.0).cloned()`),
      add a zero-arg arity guard before the generic CallExtern: when `boxed_args.is_empty()` and
      `extern_name` matches a constructor-from-iterable variant, redirect to the zero-param _new variant.
      Mapping:
        mb_list_from_iterable  → mb_list_new
        mb_tuple_from_iterable → mb_tuple_new
        mb_set_from_iterable   → mb_set_new
        mb_dict_from_pairs     → mb_dict_new
      Emit `CallExtern { dest: Some(dest), name: <new_variant>, args: vec![], ty: *ty }` and return dest early.
      Add unit tests: test_zero_arg_{list,tuple,set,dict}_constructor_emits_mb_{type}_new
      and test_one_arg_{list,tuple,set}_constructor_emits_mb_{type}_from_iterable.
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
