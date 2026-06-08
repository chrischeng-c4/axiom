---
id: phase1
type: proposal
version: 2
created_at: 2026-02-12T18:10:11.133743+00:00
updated_at: 2026-02-12T18:10:11.133743+00:00
iteration: 1
scope: minor
spec_plan:
  - id: hir-data
    title: "HIR Data Structures"
    depends: []
    context_refs:
      codebase: ["hir/mod.rs (empty)", "parser/ast.rs"]
      spec: ["02-architecture-principles"]
      knowledge: ["NaN-boxing pattern"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 1 }
    affected_code: ["crates/cclab-taipan/src/hir/mod.rs", "crates/cclab-taipan/src/hir/expr.rs", "crates/cclab-taipan/src/hir/stmt.rs"]
  - id: resolve-pass
    title: "Name Resolution Pass"
    depends: []
    context_refs:
      codebase: ["resolve/scope.rs — SymbolTable, Scope, SymbolId"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 4 }
    affected_code: ["crates/cclab-taipan/src/resolve/pass.rs", "crates/cclab-taipan/src/resolve/mod.rs"]
  - id: ast-to-hir
    title: "AST to HIR Lowering"
    depends: [hir-data, resolve-pass]
    context_refs:
      codebase: ["parser/ast.rs", "types/check.rs"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 2 }
    affected_code: ["crates/cclab-taipan/src/lower/mod.rs", "crates/cclab-taipan/src/lower/ast_to_hir.rs"]
  - id: runtime-value
    title: "Runtime Object Model (NaN-boxing) and Refcounting"
    depends: []
    context_refs:
      knowledge: ["NaN-boxing pattern", "RC with cycle collector"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 3 }
      - { source: gap_codebase_knowledge, gap_index: 1 }
      - { source: gap_spec_knowledge, gap_index: 1 }
      - { source: gap_spec_knowledge, gap_index: 2 }
    affected_code: ["crates/cclab-taipan/src/runtime/mod.rs", "crates/cclab-taipan/src/runtime/value.rs", "crates/cclab-taipan/src/runtime/rc.rs"]
  - id: hir-to-mir
    title: "HIR to MIR Lowering (SSA)"
    depends: [hir-data, runtime-value]
    context_refs:
      codebase: ["mir/mod.rs — MirBody, MirInst, MirModule"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 2 }
    affected_code: ["crates/cclab-taipan/src/lower/hir_to_mir.rs"]
  - id: builtins
    title: "Built-in Function Implementations"
    depends: [runtime-value]
    context_refs:
      codebase: ["types/builtins.rs — 40+ builtin type stubs"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 3 }
    affected_code: ["crates/cclab-taipan/src/runtime/builtins.rs"]
  - id: driver-pipeline
    title: "End-to-End Driver CLI"
    depends: [ast-to-hir, hir-to-mir, builtins]
    context_refs:
      codebase: ["driver/mod.rs", "codegen/cranelift/mod.rs"]
      knowledge: ["setjmp/longjmp exception handling"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 5 }
      - { source: gap_spec_knowledge, gap_index: 3 }
    affected_code: ["crates/cclab-taipan/src/driver/mod.rs", "crates/cclab-taipan/src/lib.rs"]
history:
  - timestamp: 2026-02-12T18:10:11.133743+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
---

<proposal>

# Spec Navigation Map: phase1

## Scope Overview (Mindmap)

```mermaid
mindmap
  root((phase1))  
    Intermediate Representations
      HIR data structures
      AST→HIR lowering
      HIR→MIR lowering (SSA)
    Name Resolution
      Scope tree traversal
      Symbol binding
      Variable resolution
    Runtime
      NaN-boxed TpValue
      Reference counting
      Cycle collector
      Built-in functions (print, len, range, etc.)
    Driver Pipeline
      End-to-end compilation
      Source → executable
      Exception handling (setjmp/longjmp)
```

## Spec Dependency Graph (Block Diagram)

```mermaid
block-beta
  columns 3

  hir_data["hir-data\n codebase: hir/mod.rs (empty), parser/ast.rs\n gaps: codebase_spec#1"]
  resolve_pass["resolve-pass\n codebase: resolve/scope.rs — SymbolTable, Scope, SymbolId\n gaps: codebase_spec#4"]
  ast_to_hir["ast-to-hir\n codebase: parser/ast.rs, types/check.rs\n gaps: codebase_spec#2"]
  runtime_value["runtime-value\n gaps: codebase_spec#3, codebase_knowledge#1, spec_knowledge#1, spec_knowledge#2"]
  hir_to_mir["hir-to-mir\n codebase: mir/mod.rs — MirBody, MirInst, MirModule\n gaps: codebase_spec#2"]
  builtins["builtins\n codebase: types/builtins.rs — 40+ builtin type stubs\n gaps: codebase_spec#3"]
  driver_pipeline["driver-pipeline\n codebase: driver/mod.rs, codegen/cranelift/mod.rs\n gaps: codebase_spec#5, spec_knowledge#3"]

  hir_data --> ast_to_hir
  resolve_pass --> ast_to_hir
  hir_data --> hir_to_mir
  runtime_value --> hir_to_mir
  runtime_value --> builtins
  ast_to_hir --> driver_pipeline
  hir_to_mir --> driver_pipeline
  builtins --> driver_pipeline
```

## Spec Execution Order

1. **hir-data** — HIR Data Structures
   - code: crates/cclab-taipan/src/hir/mod.rs, crates/cclab-taipan/src/hir/expr.rs, crates/cclab-taipan/src/hir/stmt.rs
2. **resolve-pass** — Name Resolution Pass
   - code: crates/cclab-taipan/src/resolve/pass.rs, crates/cclab-taipan/src/resolve/mod.rs
3. **ast-to-hir** — AST to HIR Lowering
   - depends: hir-data, resolve-pass
   - code: crates/cclab-taipan/src/lower/mod.rs, crates/cclab-taipan/src/lower/ast_to_hir.rs
4. **runtime-value** — Runtime Object Model (NaN-boxing) and Refcounting
   - code: crates/cclab-taipan/src/runtime/mod.rs, crates/cclab-taipan/src/runtime/value.rs, crates/cclab-taipan/src/runtime/rc.rs
5. **builtins** — Built-in Function Implementations
   - depends: runtime-value
   - code: crates/cclab-taipan/src/runtime/builtins.rs
6. **hir-to-mir** — HIR to MIR Lowering (SSA)
   - depends: hir-data, runtime-value
   - code: crates/cclab-taipan/src/lower/hir_to_mir.rs
7. **driver-pipeline** — End-to-End Driver CLI
   - depends: ast-to-hir, hir-to-mir, builtins
   - code: crates/cclab-taipan/src/driver/mod.rs, crates/cclab-taipan/src/lib.rs

</proposal>
