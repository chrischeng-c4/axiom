---
change_id: taipan-283-294
type: spec_context
created_at: 2026-02-13T04:12:20.464922+00:00
updated_at: 2026-02-13T04:12:20.464922+00:00
iteration: 1
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
  - cclab-taipan (parallel change)
---

# Spec Context

## Relevant Specs

- **taipan-syntax** (group: cclab-taipan (change))
  - relevance: high
  - reason: Defines base grammar; must be extended for all 12 features (e.g., try/except, class, for, async).
  - key sections: R1-R5, Acceptance Criteria, Taipan Parsing Logic diagram
- **taipan-ir** (group: cclab-taipan (change))
  - relevance: high
  - reason: Defines IR used for control flow lowering; needs new opcodes for exception blocks, iterators, and state machines.
  - key sections: SSA structure, ISA Core opcodes
- **taipan-backend-cranelift** (group: cclab-taipan (change))
  - relevance: high
  - reason: Defines how IR is emitted to machine code; must support new IR instructions for P1+P2 features.
  - key sections: R1-R5, Cranelift Backend Pipeline diagram
- **gil-waker-polling** (group: cclab-orbit)
  - relevance: medium
  - reason: Established pattern for async/await parity with CPython/asyncio as requested in #293 and Clarification Q4.
  - key sections: R1-R2 (Waker-driven polling, GIL release), Waker-driven Polling Flow diagram
- **structured-error-handling** (group: cclab-core)
  - relevance: medium
  - reason: Reference for implementing Python-compatible exception handling (#283).
  - key sections: Error hierarchy, Contextual information

## Dependencies

- taipan-syntax -> taipan-ir -> taipan-backend-cranelift

## Gaps

- No existing specs for any of the 12 P1+P2 features (#283-294) in the main specs directory.
- The base Taipan specs are currently located in an unmerged parallel change 'cclab-taipan'.
- Lack of a 'taipan-runtime' spec for managing complex data structures (list, dict) and async state.
