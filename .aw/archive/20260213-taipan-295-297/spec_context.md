---
change_id: taipan-295-297
type: spec_context
created_at: 2026-02-13T07:20:11.126487+00:00
updated_at: 2026-02-13T07:20:11.126487+00:00
iteration: 1
complexity: high
stage: spec
scanned_groups:
  - cclab-taipan
  - taipan-283-294
---

# Spec Context

## Relevant Specs

- **taipan-backend-cranelift** (group: cclab-taipan)
  - relevance: high
  - reason: Core backend spec that needs to be extended with JIT support and real FFI calls.
  - key sections: R4 - Object File Emission, R5 - External Function Support
- **taipan-ir** (group: cclab-taipan)
  - relevance: high
  - reason: Defines the instructions (MirInst) that currently have placeholders in codegen.
  - key sections: R3 - Instruction Set Architecture (ISA) Core
- **taipan-core-types** (group: taipan-283-294)
  - relevance: medium
  - reason: Defines the runtime structures (List, Dict, etc.) that the FFI calls will manipulate.
  - key sections: R1-R4 Implementation Requirements
- **taipan-cli-integration** (group: cclab-taipan)
  - relevance: high
  - reason: Defines how 'cclab taipan run' should behave; needs update for JIT.
  - key sections: Scenario: Execute Run Command

## Dependencies

- taipan-backend-cranelift -> taipan-ir
- taipan-cli-integration -> taipan-backend-cranelift
- taipan-core-types -> taipan-ir

## Gaps

- taipan-backend-cranelift: Missing JITModule implementation details and runtime symbol wiring.
- taipan-backend-cranelift: Codegen placeholders for GetAttr/SetAttr/GetItem/SetItem/MakeList/MakeDict/MakeTuple/Raise need specification of FFI signatures.
- taipan-cli-integration: 'run' subcommand needs to transition from 'spawn process' to 'in-memory JIT execution'.
- taipan-ir: Explicit mapping of complex object operations to runtime FFI calls is not fully specified in the backend algorithm.
