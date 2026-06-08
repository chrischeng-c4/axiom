---
id: genesis-372
type: proposal
version: 2
created_at: 2026-02-14T17:23:48.996332+00:00
updated_at: 2026-02-14T17:23:48.996332+00:00
iteration: 1
scope: major
spec_plan:
  - id: migration-architecture
    title: "Migration Architecture & Compatibility Matrix"
    depends: []
    context_refs:
      knowledge: ["40-mcp/index.md", "genesis-372-impact.md"]
    gap_repairs:
      - { source: gap_codebase_knowledge, gap_index: 2 }
    affected_code: ["crates/cclab-genesis/src/lib.rs", "crates/cclab-genesis/src/migration.rs"]
  - id: spec-ir-yaml-schema
    title: "SpecIR YAML Manifest Schema"
    depends: []
    context_refs:
      knowledge: ["spec-to-code/spec-model.md", "genesis-372-impact.md"]
    gap_repairs:
      - { source: gap_spec_knowledge, gap_index: 1 }
      - { source: gap_spec_knowledge, gap_index: 2 }
    affected_code: ["crates/cclab-genesis/src/spec_ir/mod.rs", "crates/cclab-genesis/src/spec_ir/types.rs", "crates/cclab-genesis/src/spec_ir/schema.rs", "crates/cclab-aurora/src/spec_ir/mod.rs", "crates/cclab-aurora/src/spec_ir/types.rs"]
  - id: genesis-spec-generation
    title: "Genesis Spec Generation Logic"
    depends: [spec-ir-yaml-schema, migration-architecture]
    context_refs:
      codebase: ["crates/cclab-genesis/src/mcp/tools/spec.rs"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 2 }
    affected_code: ["crates/cclab-genesis/src/mcp/tools/spec.rs", "crates/cclab-genesis/src/spec_ir/writer.rs"]
  - id: prism-yaml-codegen
    title: "Prism YAML-Based Code Generation"
    depends: [spec-ir-yaml-schema, migration-architecture]
    context_refs:
      codebase: ["crates/cclab-prism/src/gen/", "crates/cclab-aurora/src/generators/"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 1 }
    affected_code: ["crates/cclab-prism/src/gen/registry.rs", "crates/cclab-prism/src/lib.rs", "crates/cclab-prism/src/gen/traits.rs", "crates/cclab-aurora/src/generators/mod.rs"]
  - id: genesis-codegen-orchestration
    title: "Genesis Codegen Orchestration"
    depends: [genesis-spec-generation, prism-yaml-codegen]
    context_refs:
      codebase: ["crates/cclab-genesis/src/mcp/tools/run_change/implement.rs"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 4 }
    affected_code: ["crates/cclab-genesis/src/mcp/tools/run_change/implement.rs", "crates/cclab-genesis/src/mcp/tools/run_change/task_graph.rs"]
history:
  - timestamp: 2026-02-14T17:23:48.996332+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
---

<proposal>

# Spec Navigation Map: genesis-372

## Scope Overview (Mindmap)

```mermaid
mindmap
  root((genesis-372))  
    Spec IR
      YAML Schema
      Manifest Format
      Validation
    Migration Architecture
      Transition Strategy
      Compatibility Matrix
      Deprecation Plan
    Genesis Integration
      Spec Generation
      Codegen Orchestration
      Artifact Management
    Prism Codegen
      YAML Ingestion
      Code Generator Trait
      Aurora Migration
```

## Spec Dependency Graph (Block Diagram)

```mermaid
block-beta
  columns 3

  migration_architecture["migration-architecture\n gaps: codebase_knowledge#2"]
  spec_ir_yaml_schema["spec-ir-yaml-schema\n gaps: spec_knowledge#1, spec_knowledge#2"]
  genesis_spec_generation["genesis-spec-generation\n codebase: crates/cclab-genesis/src/mcp/tools/spec.rs\n gaps: codebase_spec#2"]
  prism_yaml_codegen["prism-yaml-codegen\n codebase: crates/cclab-prism/src/gen/, crates/cclab-aurora/src/generators/\n gaps: codebase_spec#1"]
  genesis_codegen_orchestration["genesis-codegen-orchestration\n codebase: crates/cclab-genesis/src/mcp/tools/run_change/implement.rs\n gaps: codebase_spec#4"]

  spec_ir_yaml_schema --> genesis_spec_generation
  migration_architecture --> genesis_spec_generation
  spec_ir_yaml_schema --> prism_yaml_codegen
  migration_architecture --> prism_yaml_codegen
  genesis_spec_generation --> genesis_codegen_orchestration
  prism_yaml_codegen --> genesis_codegen_orchestration
```

## Spec Execution Order

1. **migration-architecture** — Migration Architecture & Compatibility Matrix
   - code: crates/cclab-genesis/src/lib.rs, crates/cclab-genesis/src/migration.rs
2. **spec-ir-yaml-schema** — SpecIR YAML Manifest Schema
   - code: crates/cclab-genesis/src/spec_ir/mod.rs, crates/cclab-genesis/src/spec_ir/types.rs, crates/cclab-genesis/src/spec_ir/schema.rs, crates/cclab-aurora/src/spec_ir/mod.rs, crates/cclab-aurora/src/spec_ir/types.rs
3. **genesis-spec-generation** — Genesis Spec Generation Logic
   - depends: spec-ir-yaml-schema, migration-architecture
   - code: crates/cclab-genesis/src/mcp/tools/spec.rs, crates/cclab-genesis/src/spec_ir/writer.rs
4. **prism-yaml-codegen** — Prism YAML-Based Code Generation
   - depends: spec-ir-yaml-schema, migration-architecture
   - code: crates/cclab-prism/src/gen/registry.rs, crates/cclab-prism/src/lib.rs, crates/cclab-prism/src/gen/traits.rs, crates/cclab-aurora/src/generators/mod.rs
5. **genesis-codegen-orchestration** — Genesis Codegen Orchestration
   - depends: genesis-spec-generation, prism-yaml-codegen
   - code: crates/cclab-genesis/src/mcp/tools/run_change/implement.rs, crates/cclab-genesis/src/mcp/tools/run_change/task_graph.rs

</proposal>
