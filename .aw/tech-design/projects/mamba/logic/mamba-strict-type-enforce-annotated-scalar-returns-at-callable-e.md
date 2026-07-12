---
id: mamba-strict-type-enforce-annotated-scalar-returns-at-callable-e
summary: Enforce retained scalar return annotations at synchronous user-function egress without changing the raw or boxed ABI.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-strict-type-scalar-return-egress
entry: annotated_return
nodes:
  annotated_return: { kind: start, label: "synchronous user return" }
  lower_return: { kind: process, label: "lower explicit, bare, or implicit return" }
  resolve_contract: { kind: process, label: "resolve retained scalar contract and source spelling" }
  strict_scalar: { kind: decision, label: "definite scalar contract?" }
  existing_abi: { kind: process, label: "use existing return lowering and ABI" }
  validate_adapt: { kind: process, label: "validate and adapt before ABI crossing" }
  type_error: { kind: terminal, label: "set catchable TypeError and return error sentinel" }
  normalized_abi: { kind: process, label: "preserve or adapt raw, boxed, or F64 ABI" }
  caller: { kind: terminal, label: "direct and dynamic caller observes one result contract" }
edges:
  - { from: annotated_return, to: lower_return }
  - { from: lower_return, to: resolve_contract }
  - { from: resolve_contract, to: strict_scalar }
  - { from: strict_scalar, to: existing_abi, label: "unannotated, Any, or unsupported" }
  - { from: strict_scalar, to: validate_adapt, label: "int, bool, float, str, bytes, or None" }
  - { from: validate_adapt, to: type_error, label: "mismatch" }
  - { from: validate_adapt, to: normalized_abi, label: "accepted" }
  - { from: normalized_abi, to: existing_abi }
  - { from: existing_abi, to: caller }
---
flowchart TD
    annotated_return([synchronous user return]) --> lower_return[lower explicit, bare, or implicit return]
    lower_return --> resolve_contract[resolve retained scalar contract and source spelling]
    resolve_contract --> strict_scalar{definite scalar contract?}
    strict_scalar -- unannotated, Any, or unsupported --> existing_abi[use existing return lowering and ABI]
    strict_scalar -- int, bool, float, str, bytes, or None --> validate_adapt[validate and adapt before ABI crossing]
    validate_adapt -- mismatch --> type_error([set catchable TypeError and return error sentinel])
    validate_adapt -- accepted --> normalized_abi[preserve or adapt raw, boxed, or F64 ABI]
    normalized_abi --> existing_abi
    existing_abi --> caller([direct and dynamic caller observes one result contract])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: mamba-strict-type-scalar-return-egress-verification
requirements:
  direct_any_rejection:
    id: R1
    text: "A synchronous function declared as returning int rejects an Any-origin str at callee egress after executing its body, with a catchable TypeError before a direct caller can consume the result."
    kind: regression
    risk: high
    verify: return_annotation/func_int_return_any_str_direct.py
  dynamic_any_rejection:
    id: R2
    text: "The same retained callee contract rejects an Any-origin str when the function value is invoked through an Any-erased dynamic call, proving validation is not dispatcher-only."
    kind: regression
    risk: high
    verify: return_annotation/func_int_return_any_str_dynamic.py
  fail_open_boundaries:
    id: R5
    text: "Unannotated, explicit Any, and unsupported container, generic, union, or forward-reference return annotations retain the existing fail-open behavior."
    kind: regression
    risk: medium
    verify: codegen::cranelift::tests::strict_scalar_return_contract_keeps_unsupported_annotations_open
  fixture_accounting:
    id: R6
    text: "The two executable return_annotation fixtures are discovered and strict-type accounting advances from 7418 to 7420 without changing unconstrained or unresolved denominators."
    kind: integration
    risk: medium
    verify: strict_type_accounting_gate_704::strict_type_fixture_accounting_matches_manifest
  return_form_contracts:
    id: R4
    text: "Explicit, bare, and implicit returns enforce non-None scalar contracts while bare and implicit returns satisfy a None contract without caller continuation after rejection."
    kind: functional
    risk: high
    verify: codegen::cranelift::tests::strict_scalar_return_contract_covers_return_forms
  scalar_abi_compatibility:
    id: R3
    text: "Accepted int, bool, float, str, bytes, None, and resolved PEP 695 scalar aliases preserve the existing raw, boxed, or F64 return ABI; bool-to-int and int-or-bool-to-float follow the ingress numeric compatibility rule."
    kind: functional
    risk: high
    verify: codegen::cranelift::tests::strict_scalar_return_contract_preserves_compatible_abi
---
flowchart TD
    r1[R1 direct any rejection] --> return_annotation_func_int_return_any_str_direct_py[return_annotation/func_int_return_any_str_direct.py]
    r2[R2 dynamic any rejection] --> return_annotation_func_int_return_any_str_dynamic_py[return_annotation/func_int_return_any_str_dynamic.py]
    r3[R3 scalar abi compatibility] --> codegen_cranelift_tests_strict_scalar_return_contract_preserves_compatible_abi[codegen::cranelift::tests::strict_scalar_return_contract_preserves_compatible_abi]
    r4[R4 return form contracts] --> codegen_cranelift_tests_strict_scalar_return_contract_covers_return_forms[codegen::cranelift::tests::strict_scalar_return_contract_covers_return_forms]
    r5[R5 fail open boundaries] --> codegen_cranelift_tests_strict_scalar_return_contract_keeps_unsupported_annotations_open[codegen::cranelift::tests::strict_scalar_return_contract_keeps_unsupported_annotations_open]
    r6[R6 fixture accounting] --> strict_type_accounting_gate_704_strict_type_fixture_accounting_matches_manifest[strict_type_accounting_gate_704::strict_type_fixture_accounting_matches_manifest]
```
