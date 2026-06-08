---
change_id: genesis-372
type: spec_context
created_at: 2026-02-14T16:49:45.746516+00:00
updated_at: 2026-02-14T16:49:45.746516+00:00
iteration: 2
complexity: high
stage: spec
scanned_groups:
  - cclab-aurora
  - cclab-cli
  - cclab-core
  - cclab-genesis
  - cclab-grid
  - cclab-grid-db
  - cclab-ion
  - cclab-meteor
  - cclab-nebula
  - cclab-nova
  - cclab-nucleus
  - cclab-orbit
  - cclab-photon
  - cclab-prism
  - cclab-probe
  - cclab-pulsar-array-core
  - cclab-quasar
  - cclab-server
  - cclab-shield
  - cclab-taipan
  - cclab-titan
  - nebula
---

# Spec Context

## Relevant Specs

- **create-spec** (group: cclab-genesis)
  - relevance: high
  - reason: Will be modified to include SpecIR YAML generation logic and tool updates.
  - key sections: OpenRPC: genesis_create_spec, Compositional Tag System
- **spec-ir-contract** (group: cclab-aurora)
  - relevance: high
  - reason: The core contract being migrated from Rust enums to YAML manifests.
  - key sections: R1 - SpecIR enum type, R5 - SpecBundle for multi-spec input
- **prism-codegen-unification** (group: cclab-prism)
  - relevance: high
  - reason: Prism must be updated to read the new YAML manifests instead of Rust-serialized IR.
  - key sections: R2 - Unify CodeGenerator trait with SpecIR input, R6 - Generator registry
- **genesis-implement-integration** (group: cclab-genesis)
  - relevance: high
  - reason: Describes how Genesis uses SpecIR for codegen, which must be updated for the new YAML format.
  - key sections: R1 - Detect codegen-eligible tasks, R2 - Structured codegen prompt
- **spec-validator** (group: cclab-aurora)
  - relevance: medium
  - reason: Will need to be updated to validate the new YAML manifest structure.
  - key sections: R1 - Type Validation, R2 - Reference Validation
- **aurora-codegen-system** (group: cclab-aurora)
  - relevance: medium
  - reason: Architecture being merged into Genesis and Prism.
  - key sections: R1 - Unified Internal Representation, R3 - Template-Based Generation

## Dependencies

- cclab-prism/prism-codegen-unification depends on cclab-aurora/spec-ir-contract
- cclab-genesis/genesis-implement-integration depends on cclab-prism/prism-codegen-unification
- cclab-genesis/create-spec drives the generation of SpecIR in the new workflow

## Gaps

- No spec defines the k8s-style YAML schema for SpecIR manifests
- No spec describes the integration of Aurora spec generation logic into Genesis crate
- No spec defines the storage and naming conventions for spec_ir/ files under change directories
