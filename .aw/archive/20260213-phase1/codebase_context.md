---
change_id: phase1
type: codebase_context
created_at: 2026-02-12T17:56:30.550720+00:00
updated_at: 2026-02-12T17:56:30.550720+00:00
iteration: 1
complexity: high
stage: codebase
prism_tools_used:
  - manual inspection
---

# Codebase Context

## Analyzed Files

- **crates/cclab-taipan/src/hir/mod.rs** — NEW — HIR data structures (#275)
  - symbols: `HirModule`, `HirFn`, `HirExpr`, `HirStmt`
- **crates/cclab-taipan/src/resolve/scope.rs** — EXISTING — SymbolTable and Scope (needs traversal pass #276)
  - symbols: `SymbolTable`, `Scope`, `SymbolId`, `ScopeId`
- **crates/cclab-taipan/src/lower/** — NEW — AST→HIR (#277) and HIR→MIR (#278) lowering passes
  - symbols: `ast_to_hir`, `hir_to_mir`
- **crates/cclab-taipan/src/runtime/** — NEW — Runtime object model (#279), refcounting (#280), builtins (#281)
  - symbols: `TpValue`, `TpObject`, `tp_retain`, `tp_release`
- **crates/cclab-taipan/src/driver/mod.rs** — EXISTING — Compilation driver (needs end-to-end pipeline #282)
  - symbols: `compile_file`
- **crates/cclab-taipan/src/parser/ast.rs** — EXISTING — AST types consumed by lowering
  - symbols: `Stmt`, `Expr`, `Module`
- **crates/cclab-taipan/src/types/check.rs** — EXISTING — Type checker producing typed AST
  - symbols: `TypeChecker`, `check_module`
- **crates/cclab-taipan/src/mir/mod.rs** — EXISTING — MIR types consumed by codegen
  - symbols: `MirBody`, `MirInst`, `MirModule`
- **crates/cclab-taipan/src/codegen/cranelift/mod.rs** — EXISTING — Cranelift backend consuming MIR
  - symbols: `CraneliftBackend`, `codegen`
- **crates/cclab-taipan/src/lib.rs** — MODIFY — Add pub mod lower, pub mod runtime

## Prism Results

- **manual inspection** (query: `cclab-taipan module structure`)
  - Crate has 14 modules: error, source, lexer, parser, resolve, types, hir (empty), mir, codegen, diagnostic, driver, config, build, ffi. Phase 1 adds: lower (new), runtime (new), populates hir, extends resolve and driver.

## Dependency Graph

- parser/ast.rs → resolve/pass.rs → lower/ast_to_hir.rs → lower/hir_to_mir.rs → codegen/cranelift/mod.rs
- runtime/value.rs → runtime/rc.rs → runtime/builtins.rs
- driver/mod.rs orchestrates: lexer → parser → resolve → typecheck → lower → codegen → link
