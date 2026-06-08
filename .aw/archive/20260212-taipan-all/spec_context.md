---
change_id: taipan-all
type: spec_context
created_at: 2026-02-12T10:29:05.709969+00:00
updated_at: 2026-02-12T10:29:05.709969+00:00
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
  - cclab-titan
  - genesis
  - nebula
  - cclab-taipan (change specs)
---

# Spec Context

## Relevant Specs

- **02-architecture-principles** (group: cclab-core)
  - relevance: high
  - reason: Core motivation for a native compiler like Taipan. (Score: 5/5)
  - key sections: Zero Python Byte Handling, Performance First
- **01-roadmap** (group: cclab-core)
  - relevance: medium
  - reason: Strategic alignment with 2xx series performance goals. (Score: 3/5)
  - key sections: 2xx Series: Performance Optimization
- **taipan-syntax** (group: cclab-taipan (change))
  - relevance: high
  - reason: Initial grammar for v0.1; currently limited to basic syntax. (Score: 5/5)
  - key sections: Requirements R1-R5, Taipan Parsing Logic diagram
- **taipan-ir** (group: cclab-taipan (change))
  - relevance: high
  - reason: Base for compiler's intermediate representation. (Score: 5/5)
  - key sections: SSA form requirements, Instruction Set Architecture Core
- **taipan-backend-cranelift** (group: cclab-taipan (change))
  - relevance: high
  - reason: Implementation of the primary code generation backend. (Score: 4/5)
  - key sections: Type Mapping, Instruction Translation, External Function Support
- **taipan-cli-integration** (group: cclab-taipan (change))
  - relevance: medium
  - reason: Integration into the unified CLI tool. (Score: 3/5)
  - key sections: Subcommand Definition, Taipan CLI Execution Flow diagram

## Dependencies

- cclab-core/02-architecture-principles -> taipan-all (Performance First)
- cclab-core/01-roadmap -> taipan-all (2xx series performance)
- taipan-syntax -> taipan-ir -> taipan-backend-cranelift
- taipan-backend-cranelift -> taipan-cli-integration

## Gaps

- No spec for Pattern Matching (Issues #235-239)
- Existing taipan-syntax is v0.1 only; lacks control flow and complex data structures (Issues #213-235)
- No spec for Advanced Types (Structs/Enums/Arrays) (Issues #240-249)
- FFI spec is rudimentary; lacks details on complex data exchange (Issues #255-271)
- Build and Config specs are missing (Issues #250-254)
