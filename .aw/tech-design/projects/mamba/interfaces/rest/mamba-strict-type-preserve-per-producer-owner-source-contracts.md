---
id: mamba-strict-type-preserve-per-producer-owner-source-contracts
summary: Per-producer raw-or-boxed Int ownership metadata for later compiler and runtime consumers.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-strict-type-owner-source-contract-logic
entry: producer
nodes:
  producer: { kind: start, label: "MIR producer writes raw-or-boxed Int" }
  abi: { kind: process, label: "read canonical #1448 physical return ABI" }
  site: { kind: process, label: "record producer kind and MIR block/instruction site" }
  local: { kind: decision, label: "is this a local producer rather than a call boundary" }
  static_ownerless: { kind: process, label: "classify raw constants and immortal values as ownerless" }
  fresh: { kind: process, label: "classify allocating numeric results as fresh owner" }
  passthrough: { kind: process, label: "classify unbox projection from argument as pass-through owner" }
  explicit_out: { kind: process, label: "classify runtime explicit owner-out result" }
  ingress: { kind: process, label: "record parameter as #1451 argument-ingress boundary" }
  return_channel: { kind: process, label: "record internal or dynamic call as #1452 return-channel boundary" }
  extern: { kind: process, label: "map six raw-or-boxed runtime symbols to exact ownership contracts" }
  inventory: { kind: process, label: "reject missing owner-source actions in exhaustive inventory" }
  done: { kind: terminal, label: "metadata is ready; later slices consume it without bit inference" }
edges:
  - { from: producer, to: abi }
  - { from: abi, to: site }
  - { from: site, to: local }
  - { from: local, to: ingress, label: "parameter" }
  - { from: local, to: return_channel, label: "internal or dynamic call" }
  - { from: local, to: static_ownerless, label: "constant or immortal local" }
  - { from: local, to: fresh, label: "fresh numeric local" }
  - { from: local, to: passthrough, label: "unbox projection" }
  - { from: local, to: explicit_out, label: "explicit runtime owner-out" }
  - { from: static_ownerless, to: extern }
  - { from: fresh, to: extern }
  - { from: passthrough, to: extern }
  - { from: explicit_out, to: extern }
  - { from: ingress, to: inventory }
  - { from: return_channel, to: inventory }
  - { from: extern, to: inventory }
  - { from: inventory, to: done }
---
flowchart TD
    producer([raw-or-boxed Int MIR producer]) --> abi[read canonical physical ABI]
    abi --> site[record producer kind and MIR site]
    site --> local{local producer}
    local -- parameter --> ingress[defer to #1451 argument ingress]
    local -- call result --> return_channel[defer to #1452 return channel]
    local -- constant or immortal --> static_ownerless[ownerless action]
    local -- allocating numeric result --> fresh[fresh-result owner action]
    local -- unbox projection --> passthrough[argument pass-through action]
    local -- explicit owner out --> explicit_out[explicit owner-out action]
    static_ownerless --> extern[map six runtime symbols]
    fresh --> extern
    passthrough --> extern
    explicit_out --> extern
    ingress --> inventory[exhaustive owner-source inventory]
    return_channel --> inventory
    extern --> inventory
    inventory --> done([no bit or semantic-TypeId inference])
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
