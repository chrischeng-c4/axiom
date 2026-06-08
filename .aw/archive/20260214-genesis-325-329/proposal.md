---
id: genesis-325-329
type: proposal
version: 2
created_at: 2026-02-14T10:07:22.037730+00:00
updated_at: 2026-02-14T10:07:22.037730+00:00
iteration: 1
scope: major
spec_plan:
  - id: spec-ir-contract
    title: "SpecIR Contract Definition"
    depends: []
    context_refs:
      codebase: ["crates/cclab-aurora/src/schema/mod.rs"]
      spec: ["aurora-codegen-system"]
      knowledge: ["spec-to-code/spec-model.md"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 1 }
      - { source: gap_spec_knowledge, gap_index: 8 }
    affected_code: ["crates/cclab-aurora/src/spec_ir/", "crates/cclab-aurora/src/lib.rs"]
  - id: prism-codegen-unification
    title: "Prism Codegen Unification & Migration"
    depends: [spec-ir-contract]
    context_refs:
      codebase: ["crates/cclab-aurora/src/generators/", "crates/cclab-prism/src/gen/mod.rs"]
      spec: ["generator-fastapi", "generator-express", "generator-axum"]
      knowledge: ["spec-to-code/code-generator-contract.md"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 2 }
      - { source: gap_codebase_knowledge, gap_index: 3 }
      - { source: gap_codebase_knowledge, gap_index: 4 }
    affected_code: ["crates/cclab-prism/src/gen/", "crates/cclab-prism/src/mcp/"]
  - id: genesis-implement-integration
    title: "Genesis Implement Phase Integration"
    depends: [prism-codegen-unification]
    context_refs:
      codebase: ["crates/cclab-genesis/src/mcp/tools/run_change/implement.rs"]
      spec: ["implement-change"]
      knowledge: ["40-mcp/http-server.md"]
    gap_repairs:
      - { source: gap_spec_knowledge, gap_index: 7 }
    affected_code: ["crates/cclab-genesis/src/mcp/tools/run_change/implement.rs"]
history:
  - timestamp: 2026-02-14T10:07:22.037730+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
---

<proposal>

# Spec Navigation Map: genesis-325-329

## Scope Overview (Mindmap)

```mermaid
mindmap
  root((genesis-325-329))  
    Aurora
      SpecIR Definition
      Generator Cleanup
    Prism
      Generator Migration
      Codegen Unification
      MCP Tool Exposure
    Genesis
      Implement Phase Integration
```

## Spec Dependency Graph (Block Diagram)

```mermaid
block-beta
  columns 3

  spec_ir_contract["spec-ir-contract\n codebase: crates/cclab-aurora/src/schema/mod.rs\n gaps: codebase_spec#1, spec_knowledge#8"]
  prism_codegen_unification["prism-codegen-unification\n codebase: crates/cclab-aurora/src/generators/, crates/cclab-prism/src/gen/mod.rs\n gaps: codebase_spec#2, codebase_knowledge#3, codebase_knowledge#4"]
  genesis_implement_integration["genesis-implement-integration\n codebase: crates/cclab-genesis/src/mcp/tools/run_change/implement.rs\n gaps: spec_knowledge#7"]

  spec_ir_contract --> prism_codegen_unification
  prism_codegen_unification --> genesis_implement_integration
```

## Spec Execution Order

1. **spec-ir-contract** — SpecIR Contract Definition
   - code: crates/cclab-aurora/src/spec_ir/, crates/cclab-aurora/src/lib.rs
2. **prism-codegen-unification** — Prism Codegen Unification & Migration
   - depends: spec-ir-contract
   - code: crates/cclab-prism/src/gen/, crates/cclab-prism/src/mcp/
3. **genesis-implement-integration** — Genesis Implement Phase Integration
   - depends: prism-codegen-unification
   - code: crates/cclab-genesis/src/mcp/tools/run_change/implement.rs

</proposal>
