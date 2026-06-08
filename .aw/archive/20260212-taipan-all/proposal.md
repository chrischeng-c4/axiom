---
id: taipan-all
type: proposal
version: 2
created_at: 2026-02-12T10:58:39.077529+00:00
updated_at: 2026-02-12T10:58:39.077529+00:00
iteration: 1
scope: major
spec_plan:
  - id: taipan-syntax
    title: "Taipan Lexer and Parser v1"
    depends: []
    context_refs:
      codebase: ["crates/cclab-taipan/src/lexer", "crates/cclab-taipan/src/parser"]
      spec: ["cclab-taipan/taipan-syntax"]
      knowledge: ["changelogs/orbit-testing-safety.md"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 2 }
    affected_code: ["crates/cclab-taipan/src/lexer", "crates/cclab-taipan/src/parser", "crates/cclab-taipan/src/source"]
  - id: taipan-patterns
    title: "Pattern Matching Support"
    depends: [taipan-syntax]
    context_refs:
      codebase: ["crates/cclab-taipan/src/parser/ast.rs"]
      spec: ["cclab-taipan/taipan-syntax"]
      knowledge: ["changelogs/orbit-architecture.md"]
    affected_code: ["crates/cclab-taipan/src/parser", "crates/cclab-taipan/src/hir"]
  - id: taipan-types
    title: "Type System and Inference"
    depends: [taipan-patterns]
    context_refs:
      codebase: ["crates/cclab-taipan/src/types"]
      spec: ["cclab-taipan/taipan-syntax"]
      knowledge: ["05-titan/architecture-guide.md"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 3 }
    affected_code: ["crates/cclab-taipan/src/types", "crates/cclab-taipan/src/resolve"]
  - id: taipan-config
    title: "Compiler Configuration"
    depends: []
    context_refs:
      codebase: ["crates/cclab-taipan/src/driver"]
      knowledge: ["40-mcp/dynamic-config.md"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 6 }
    affected_code: ["crates/cclab-taipan/src/config", "crates/cclab-taipan/src/driver"]
  - id: taipan-build
    title: "Build System Integration"
    depends: [taipan-config]
    context_refs:
      codebase: ["crates/cclab-taipan/src/driver"]
      spec: ["cclab-taipan/taipan-config"]
      knowledge: ["changelogs/orbit-architecture.md"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 6 }
    affected_code: ["crates/cclab-taipan/src/build", "crates/cclab-taipan/src/driver"]
  - id: taipan-ffi
    title: "FFI and C Interoperability"
    depends: [taipan-types]
    context_refs:
      codebase: ["crates/cclab-taipan/src/codegen"]
      spec: ["cclab-taipan/taipan-types"]
      knowledge: ["orbit/bridge-internals.md"]
    gap_repairs:
      - { source: gap_spec_knowledge, gap_index: 0 }
    affected_code: ["crates/cclab-taipan/src/ffi", "crates/cclab-taipan/src/codegen"]
history:
  - timestamp: 2026-02-12T10:58:39.077529+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
---

<proposal>

# Spec Navigation Map: taipan-all

## Scope Overview (Mindmap)

```mermaid
mindmap
  root((taipan-all))  
    Compiler Frontend
      Lexical Analysis
      Recursive Descent Parsing
      AST Generation
      Pattern Matching
    Type System
      Type Inference
      Generics
      Built-in Types
      Type Checking
    Build Tooling
      Configuration (TOML)
      Dependency Management
      Cargo Integration
      Caching
    Interoperability
      C Header Parsing
      FFI Safety
      Type Mapping
      Code Generation
```

## Spec Dependency Graph (Block Diagram)

```mermaid
block-beta
  columns 3

  taipan_syntax["taipan-syntax\n codebase: crates/cclab-taipan/src/lexer, crates/cclab-taipan/src/parser\n gaps: codebase_spec#2"]
  taipan_patterns["taipan-patterns\n codebase: crates/cclab-taipan/src/parser/ast.rs"]
  taipan_types["taipan-types\n codebase: crates/cclab-taipan/src/types\n gaps: codebase_spec#3"]
  taipan_config["taipan-config\n codebase: crates/cclab-taipan/src/driver\n gaps: codebase_spec#6"]
  taipan_build["taipan-build\n codebase: crates/cclab-taipan/src/driver\n gaps: codebase_spec#6"]
  taipan_ffi["taipan-ffi\n codebase: crates/cclab-taipan/src/codegen\n gaps: spec_knowledge#0"]

  taipan_syntax --> taipan_patterns
  taipan_patterns --> taipan_types
  taipan_config --> taipan_build
  taipan_types --> taipan_ffi
```

## Spec Execution Order

1. **taipan-config** — Compiler Configuration
   - code: crates/cclab-taipan/src/config, crates/cclab-taipan/src/driver
2. **taipan-build** — Build System Integration
   - depends: taipan-config
   - code: crates/cclab-taipan/src/build, crates/cclab-taipan/src/driver
3. **taipan-syntax** — Taipan Lexer and Parser v1
   - code: crates/cclab-taipan/src/lexer, crates/cclab-taipan/src/parser, crates/cclab-taipan/src/source
4. **taipan-patterns** — Pattern Matching Support
   - depends: taipan-syntax
   - code: crates/cclab-taipan/src/parser, crates/cclab-taipan/src/hir
5. **taipan-types** — Type System and Inference
   - depends: taipan-patterns
   - code: crates/cclab-taipan/src/types, crates/cclab-taipan/src/resolve
6. **taipan-ffi** — FFI and C Interoperability
   - depends: taipan-types
   - code: crates/cclab-taipan/src/ffi, crates/cclab-taipan/src/codegen

</proposal>
