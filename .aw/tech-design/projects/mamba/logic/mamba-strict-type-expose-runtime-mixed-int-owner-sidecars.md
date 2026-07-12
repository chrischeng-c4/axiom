---
id: mamba-strict-type-expose-runtime-mixed-int-owner-sidecars
summary: Runtime-declared owner sidecars for raw-or-boxed Int results.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-strict-type-runtime-mixed-int-owner-sidecars
entry: runtime_symbol
nodes:
  runtime_symbol: { kind: start, label: "runtime symbol lookup" }
  contract: { kind: decision, label: "mixed-Int companion contract" }
  fresh: { kind: process, label: "publish fresh BigInt owner or None" }
  pass_through: { kind: process, label: "publish declared argument owner" }
  typed: { kind: process, label: "require typed-Int projection before owner-out" }
  deferred: { kind: terminal, label: "leave consumer deferred" }
edges:
  - { from: runtime_symbol, to: contract }
  - { from: contract, to: fresh, label: "FreshResultOrNone" }
  - { from: contract, to: pass_through, label: "ArgumentPassThroughOrNone" }
  - { from: contract, to: typed, label: "TypedIntProjection" }
  - { from: contract, to: deferred, label: "other" }
---
flowchart TD
    runtime_symbol([runtime symbol lookup]) --> contract{mixed-Int contract}
    contract -- fresh --> fresh[publish fresh BigInt owner or None]
    contract -- pass through --> pass_through[publish declared argument owner]
    contract -- typed Int proof --> typed[resolve typed projection only]
    contract -- other --> deferred([leave consumer deferred])
```

`RuntimeSymbol` owns an optional scoped companion declaration in addition to `ReturnAbi`. `mb_pow_int` and `mb_bigint_{add,sub,mul}` declare a mixed `FreshResultOrNone` sidecar; `mb_unbox_{int,inline_int}_if_boxed` declares mixed `ArgumentPassThroughOrNone { argument_index: 0 }`. Non-Int and dynamic `Unknown` symbols have no mixed sidecar, so #1452 remains the only owner of dynamic return transport.

The owner-out adapter keeps result bits distinct from ownership: helper-created BigInts transfer exactly one fresh owner, while raw, inline, float, and handle results transfer none. Smart-unbox selects only its declared argument companion as `Borrowed`, never by inspecting result bits. The C/JIT ABI remains `i64`; #1462 consumes the sidecar after evaluation.

Polymorphic `mb_floordiv`, `mb_mod`, bitwise, and shift helpers keep their public `Unknown` ABI. They register a separate `TypedIntProjection(FreshResultOrNone)` that is available only through the typed-projection lookup; ordinary and dynamic callers cannot resolve it. This makes the static Int proof an explicit owner-out boundary rather than globally reclassifying polymorphic runtime results.
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: mamba-strict-type-runtime-mixed-int-owner-sidecar-verification
requirements:
  fresh_transfer:
    id: R2
    text: "Overflowing power and BigInt arithmetic publish exactly their one fresh BigInt owner; raw results publish none."
    kind: functional
    risk: high
    verify: runtime::bigint_ops::tests::mixed_int_owner_out_transfers_fresh_bigint_once
  pass_through:
    id: R3
    text: "Smart unbox selects its declared argument companion even when the result bits are raw, without type or tag inference."
    kind: regression
    risk: high
    verify: runtime::builtins::boxing::tests::mixed_int_unbox_owner_out_uses_declared_argument
  registry_contract:
    id: R1
    text: "Every registered RawOrBoxedInt helper has its declared companion source, while non-Int and dynamic helpers have no mixed-Int sidecar."
    kind: regression
    risk: high
    verify: runtime::symbols::tests::runtime_mixed_int_companion_contracts_are_explicit
  typed_projection:
    id: R4
    text: "Polymorphic floor/mod/bitwise/shift helpers remain Unknown generally but publish fresh-or-none ownership only through an explicit typed-Int projection."
    kind: regression
    risk: high
    verify: runtime::symbols::tests::runtime_mixed_int_companion_contracts_are_explicit
---
flowchart TD
    r1[R1 registry contract] --> runtime_symbols_tests_runtime_mixed_int_companion_contracts_are_explicit[runtime::symbols::tests::runtime_mixed_int_companion_contracts_are_explicit]
    r4[R4 typed projection] --> runtime_symbols_tests_runtime_mixed_int_companion_contracts_are_explicit
    r2[R2 fresh transfer] --> runtime_bigint_ops_tests_mixed_int_owner_out_transfers_fresh_bigint_once[runtime::bigint_ops::tests::mixed_int_owner_out_transfers_fresh_bigint_once]
    r3[R3 pass through] --> runtime_builtins_boxing_tests_mixed_int_unbox_owner_out_uses_declared_argument[runtime::builtins::boxing::tests::mixed_int_unbox_owner_out_uses_declared_argument]
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/mamba/src/runtime/symbols.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-runtime-mixed-int-owner-sidecars
    tracker: "#1461"
    reason: "The runtime symbol registry is the authoritative declaration point for mixed-Int result ownership."
  - path: projects/mamba/src/runtime/bigint_ops.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-runtime-mixed-int-owner-sidecars
    tracker: "#1461"
    reason: "Overflow arithmetic must expose fresh BigInt ownership separately from raw result bits."
  - path: projects/mamba/src/runtime/builtins/boxing.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-runtime-mixed-int-owner-sidecars
    tracker: "#1461"
    reason: "Smart-unbox ownership must select an explicit argument sidecar rather than infer from result bits."
```
