---
change_id: sdd-merge
type: spec_context
created_at: 2026-02-15T03:24:03.199301+00:00
updated_at: 2026-02-15T03:24:03.199301+00:00
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
  - cclab-mamba
  - cclab-meteor
  - cclab-nebula
  - cclab-nova
  - cclab-nucleus
  - cclab-orbit
  - cclab-photon
  - cclab-prism
  - cclab-probe
  - cclab-pulsar
  - cclab-pulsar-array-core
  - cclab-quasar
  - cclab-server
  - cclab-shield
  - cclab-titan
  - cclab-vortex
  - nebula
---

# Spec Context

## Relevant Specs

- **migration-architecture** (group: cclab-genesis)
  - relevance: high
  - reason: Defines the transition from Aurora relay to YAML IR, which is the technical foundation for this merge.
  - key sections: Overview, Requirements (R1-R4), Migration Logic Flow
- **spec-ir-contract** (group: cclab-aurora)
  - relevance: high
  - reason: Defines the SpecIR contract which is the bridge between the two crates being merged.
  - key sections: Overview, Requirements (R1, R5)
- **aurora-codegen-system** (group: cclab-aurora)
  - relevance: high
  - reason: Defines the generation engine being merged into genesis/sdd.
  - key sections: Overview, Requirements (R1-R4)
- **merge-change** (group: cclab-genesis)
  - relevance: high
  - reason: Relevant as it defines the merge workflow, although sdd-merge is a meta-merge of crates.
  - key sections: Overview, Phase Routing, Merge Logic
- **genesis-codegen-orchestration** (group: cclab-genesis)
  - relevance: high
  - reason: Defines how the implementation phase uses the generation engine.
  - key sections: Overview, Implementation Orchestration Flow
- **comparison** (group: cclab-cli)
  - relevance: medium
  - reason: Details the CLI commands that will need renaming/updating.
  - key sections: Complete Tool/Command Mapping, Architecture Benefits
- **00-architecture** (group: cclab-server)
  - relevance: medium
  - reason: Defines the API server which depends on both crates and will need dependency updates.
  - key sections: Overview
- **prism-codegen-unification** (group: cclab-prism)
  - relevance: high
  - reason: Directly addresses migrating Aurora generators to Prism, which is a key part of merging them into sdd.
  - key sections: Overview, R1, R2, R4
- **prism-yaml-codegen** (group: cclab-prism)
  - relevance: high
  - reason: Defines how the system consumes YAML IR, which is the goal of the unification.
  - key sections: Overview, R1-R3, Codegen Flow

## Dependencies

- cclab-aurora/spec-ir-contract -> cclab-genesis/genesis-codegen-orchestration
- cclab-genesis/migration-architecture -> cclab-genesis/genesis-codegen-orchestration
- cclab-prism/prism-codegen-unification -> cclab-aurora/spec-ir-contract
- cclab-prism/prism-yaml-codegen -> cclab-aurora/spec-ir-contract

## Gaps

- Lack of unified architecture spec for the new cclab-sdd crate structure.
- No spec for the sdd_generate_* tool renaming and its impact on existing prompts.
- Gap in documenting the final unified SpecIR-based implementation flow within the new crate.
