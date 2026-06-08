---
id: taipan-295-297
type: proposal
version: 2
created_at: 2026-02-13T07:26:20.681463+00:00
updated_at: 2026-02-13T07:26:20.681463+00:00
iteration: 1
scope: minor
spec_plan:
  - id: taipan-jit-backend
    title: "Taipan JIT Backend and Symbol Wiring"
    depends: []
    context_refs:
      codebase: ["CraneliftBackend"]
      spec: ["taipan-backend-cranelift.md"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 0 }
      - { source: gap_spec_knowledge, gap_index: 0 }
    affected_code: ["crates/cclab-taipan/src/codegen/cranelift/mod.rs", "crates/cclab-taipan/src/driver/config.rs"]
  - id: taipan-runtime-ffi-mapping
    title: "Taipan Runtime FFI Mapping"
    depends: []
    context_refs:
      codebase: ["emit_inst", "TpValue"]
      knowledge: ["Generator Inference Pattern"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 1 }
      - { source: gap_codebase_knowledge, gap_index: 0 }
    affected_code: ["crates/cclab-taipan/src/codegen/cranelift/mod.rs", "crates/cclab-taipan/src/codegen/cranelift/marshal.rs"]
  - id: taipan-cli-run-execution
    title: "Taipan CLI Run Integration"
    depends: []
    context_refs:
      codebase: ["TaipanCli", "CompilerSession"]
      spec: ["taipan-cli-integration.md"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 0 }
      - { source: gap_spec_knowledge, gap_index: 2 }
    affected_code: ["crates/cclab-cli/src/taipan.rs", "crates/cclab-taipan/src/driver/mod.rs"]
history:
  - timestamp: 2026-02-13T07:26:20.681463+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
---

<proposal>

# Spec Navigation Map: taipan-295-297

## Scope Overview (Mindmap)

```mermaid
mindmap
  root((taipan-295-297))  
    Codegen
      JITModule Implementation
      Runtime Symbol Wiring
      FFI Call Generation
    Runtime
      TpValue Marshaling
      tp_* Function Mapping
    CLI
      run subcommand integration
      JIT session handling
```

## Spec Dependency Graph (Block Diagram)

```mermaid
block-beta
  columns 3

  taipan_jit_backend["taipan-jit-backend\n codebase: CraneliftBackend\n gaps: codebase_spec#0, spec_knowledge#0"]
  taipan_runtime_ffi_mapping["taipan-runtime-ffi-mapping\n codebase: emit_inst, TpValue\n gaps: codebase_spec#1, codebase_knowledge#0"]
  taipan_cli_run_execution["taipan-cli-run-execution\n codebase: TaipanCli, CompilerSession\n gaps: codebase_spec#0, spec_knowledge#2"]

```

## Spec Execution Order

1. **taipan-cli-run-execution** — Taipan CLI Run Integration
   - code: crates/cclab-cli/src/taipan.rs, crates/cclab-taipan/src/driver/mod.rs
2. **taipan-jit-backend** — Taipan JIT Backend and Symbol Wiring
   - code: crates/cclab-taipan/src/codegen/cranelift/mod.rs, crates/cclab-taipan/src/driver/config.rs
3. **taipan-runtime-ffi-mapping** — Taipan Runtime FFI Mapping
   - code: crates/cclab-taipan/src/codegen/cranelift/mod.rs, crates/cclab-taipan/src/codegen/cranelift/marshal.rs

</proposal>
