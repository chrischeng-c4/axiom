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
    verify: mir::return_abi::tests::producer_owner_metadata_is_actionable_at_stable_sites
  producer_inventory:
    id: R1
    text: "Every value-producing MIR instruction has one explicit owner-source action, so a newly added producer cannot bypass the inventory gate."
    kind: regression
    risk: high
    verify: mir::producer_owner::tests::every_value_producing_mir_inst_has_an_owner_action
  producer_location:
    id: R5
    text: "Producer metadata records the originating MIR instruction location and source category for downstream transaction and codegen consumers."
    kind: functional
    risk: medium
    verify: mir::return_abi::tests::producer_owner_metadata_is_actionable_at_stable_sites
  runtime_extern_contracts:
    id: R3
    text: "The six raw-or-boxed runtime symbols expose exact fresh-result or argument-pass-through companion contracts without payload or semantic-TypeId inference."
    kind: regression
    risk: high
    verify: mir::producer_owner::tests::mixed_int_extern_contracts_preserve_exact_owner_source
  invalid_extern_contract:
    id: R6
    text: "A declared argument pass-through contract fails closed when its required source argument is absent."
    kind: regression
    risk: high
    verify: mir::producer_owner::tests::declared_argument_contract_fails_closed_when_argument_is_missing
---
flowchart TD
    r1[R1 producer inventory] --> mir_producer_owner_tests_every_value_producing_mir_inst_has_an_owner_action[mir::producer_owner::tests::every_value_producing_mir_inst_has_an_owner_action]
    r3[R3 runtime extern contracts] --> mir_producer_owner_tests_mixed_int_extern_contracts_preserve_exact_owner_source[mir::producer_owner::tests::mixed_int_extern_contracts_preserve_exact_owner_source]
    r4[R4 deferred boundaries] --> mir_return_abi_tests_producer_owner_metadata_is_actionable_at_stable_sites[mir::return_abi::tests::producer_owner_metadata_is_actionable_at_stable_sites]
    r5[R5 producer location] --> mir_return_abi_tests_producer_owner_metadata_is_actionable_at_stable_sites
    r6[R6 invalid extern contract] --> mir_producer_owner_tests_declared_argument_contract_fails_closed_when_argument_is_missing[mir::producer_owner::tests::declared_argument_contract_fails_closed_when_argument_is_missing]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/mamba/src/mir/mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-mir-producer-owner
    tracker: "#1459"
    reason: "The MIR facade declares and re-exports the owner-source metadata module."
  - path: projects/mamba/src/mir/return_abi.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-mir-producer-owner
    tracker: "#1459"
    reason: "Physical ABI analysis stores and exposes producer-site ownership metadata."
  - path: projects/mamba/src/mir/producer_owner.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-mir-producer-owner
    tracker: "#1459"
    reason: "The exhaustive MIR producer inventory and typed extern companion contract remain hand-written pending a MIR metadata generator."
```
