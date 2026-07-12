---
id: mamba-strict-type-preserve-per-producer-owner-source-contracts
summary: Per-producer raw-or-boxed Int ownership metadata for later compiler and runtime consumers.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-strict-type-per-producer-owner-source-logic
entry: start
nodes:
  start: { kind: start, label: "MIR producer has a raw-or-boxed Int result" }
  physical: { kind: process, label: "retain existing physical ABI analysis" }
  owner_kind: { kind: decision, label: "select exact owner source independently" }
  ownerless: { kind: process, label: "record ownerless or immortal companion action" }
  fresh: { kind: process, label: "record fresh-result companion action" }
  passthrough: { kind: process, label: "record argument pass-through companion action" }
  explicit_out: { kind: process, label: "record explicit owner-out companion action" }
  boundary: { kind: process, label: "record named #1451 or #1452 boundary action" }
  extern: { kind: process, label: "attach runtime extern Int projection metadata" }
  inventory: { kind: process, label: "require one action for every producer family" }
  verify: { kind: process, label: "run metadata and runtime-symbol unit tests" }
  done: { kind: terminal, label: "MIR owner-source contract ready for later consumers" }
edges:
  - { from: start, to: physical }
  - { from: physical, to: owner_kind }
  - { from: owner_kind, to: ownerless, label: "ownerless or immortal" }
  - { from: owner_kind, to: fresh, label: "fresh runtime result" }
  - { from: owner_kind, to: passthrough, label: "argument pass-through" }
  - { from: owner_kind, to: explicit_out, label: "explicit owner-out" }
  - { from: owner_kind, to: boundary, label: "call boundary" }
  - { from: ownerless, to: extern }
  - { from: fresh, to: extern }
  - { from: passthrough, to: extern }
  - { from: explicit_out, to: extern }
  - { from: boundary, to: extern }
  - { from: extern, to: inventory }
  - { from: inventory, to: verify }
  - { from: verify, to: done }
---
flowchart TD
    start([raw-or-boxed Int producer]) --> physical[retain physical ABI analysis]
    physical --> owner_kind{exact owner source}
    owner_kind -- ownerless or immortal --> ownerless[record ownerless companion action]
    owner_kind -- fresh runtime result --> fresh[record fresh-result companion action]
    owner_kind -- argument pass-through --> passthrough[record pass-through companion action]
    owner_kind -- explicit owner-out --> explicit_out[record owner-out companion action]
    owner_kind -- call boundary --> boundary[record #1451 or #1452 boundary action]
    ownerless --> extern[attach runtime extern metadata]
    fresh --> extern
    passthrough --> extern
    explicit_out --> extern
    boundary --> extern
    extern --> inventory[require exhaustive producer action]
    inventory --> verify[run metadata and runtime-symbol tests]
    verify --> done([MIR contract ready])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: mamba-strict-type-per-producer-owner-source-verification
requirements:
  deferred_boundaries:
    id: R4
    text: "Parameters and call results are recorded as the named #1451 and #1452 boundaries instead of receiving local ownership transfer behavior."
    kind: functional
    risk: medium
    verify: mir::return_abi::tests::parameter_and_call_owner_sources_remain_deferred_boundaries
  physical_independence:
    id: R2
    text: "Owner-source metadata remains distinct from raw-only, boxed-only, and mixed physical ABI classification."
    kind: functional
    risk: high
    verify: mir::return_abi::tests::owner_source_is_independent_of_physical_abi
  producer_inventory:
    id: R1
    text: "Every raw-or-boxed Int MIR producer has one explicit owner-source action, and adding an unclassified producer fails the inventory gate."
    kind: regression
    risk: high
    verify: mir::return_abi::tests::all_raw_or_boxed_producers_have_explicit_owner_source
  producer_location:
    id: R5
    text: "Producer metadata records the originating MIR instruction location and source category for downstream transaction and codegen consumers."
    kind: functional
    risk: medium
    verify: mir::return_abi::tests::producer_metadata_retains_mir_location_and_source
  runtime_extern_contracts:
    id: R3
    text: "The six raw-or-boxed runtime symbols expose exact fresh-result or argument-pass-through companion contracts without payload or semantic-TypeId inference."
    kind: regression
    risk: high
    verify: mir::return_abi::tests::raw_or_boxed_runtime_extern_contracts_are_exhaustive
---
flowchart TD
    r1[R1 producer inventory] --> mir_return_abi_tests_all_raw_or_boxed_producers_have_explicit_owner_source[mir::return_abi::tests::all_raw_or_boxed_producers_have_explicit_owner_source]
    r2[R2 physical independence] --> mir_return_abi_tests_owner_source_is_independent_of_physical_abi[mir::return_abi::tests::owner_source_is_independent_of_physical_abi]
    r3[R3 runtime extern contracts] --> mir_return_abi_tests_raw_or_boxed_runtime_extern_contracts_are_exhaustive[mir::return_abi::tests::raw_or_boxed_runtime_extern_contracts_are_exhaustive]
    r4[R4 deferred boundaries] --> mir_return_abi_tests_parameter_and_call_owner_sources_remain_deferred_boundaries[mir::return_abi::tests::parameter_and_call_owner_sources_remain_deferred_boundaries]
    r5[R5 producer location] --> mir_return_abi_tests_producer_metadata_retains_mir_location_and_source[mir::return_abi::tests::producer_metadata_retains_mir_location_and_source]
```
