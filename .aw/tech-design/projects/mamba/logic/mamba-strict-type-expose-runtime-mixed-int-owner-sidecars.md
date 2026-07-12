---
id: mamba-strict-type-expose-runtime-mixed-int-owner-sidecars
summary: Runtime-declared owner sidecars for raw-or-boxed Int results.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
(fill)
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: mamba-strict-type-runtime-mixed-int-owner-sidecar-verification
requirements:
  registry_contract:
    id: R1
    text: "Every registered RawOrBoxedInt helper has its declared companion source, while non-Int and dynamic helpers have no mixed-Int sidecar."
    kind: regression
    risk: high
    verify: runtime::symbols::tests::runtime_mixed_int_companion_contracts_are_explicit
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
---
flowchart TD
    r1[R1 registry contract] --> symbols[runtime::symbols tests]
    r2[R2 fresh transfer] --> bigint[runtime::bigint_ops tests]
    r3[R3 pass through] --> boxing[runtime::builtins::boxing tests]
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
