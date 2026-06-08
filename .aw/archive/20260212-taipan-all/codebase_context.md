---
change_id: taipan-all
type: codebase_context
created_at: 2026-02-12T10:38:45.141307+00:00
updated_at: 2026-02-12T10:38:45.141307+00:00
iteration: 1
complexity: high
stage: codebase
prism_tools_used:
  - prism_symbols
  - prism_references
  - prism_impact
---

# Codebase Context

## Analyzed Files

- **crates/cclab-taipan/src/lib.rs** — crate root
  - symbols: `error`, `source`, `lexer`, `parser`, `resolve`, `types`, `hir`, `mir`, `codegen`, `diagnostic`, `driver`
- **crates/cclab-taipan/src/driver/mod.rs** — compiler orchestrator
  - symbols: `CompilerSession`, `load_file`, `check`, `build`, `render_error`
- **crates/cclab-taipan/src/lexer/mod.rs** — lexing entry point
  - symbols: `lex`, `lex_raw`, `token_span`
- **crates/cclab-taipan/src/lexer/token.rs** — token definitions
  - symbols: `TokenKind`, `Token`
- **crates/cclab-taipan/src/parser/mod.rs** — recursive descent parser
  - symbols: `Parser`, `parse`, `parse_module`
- **crates/cclab-taipan/src/parser/ast.rs** — AST structure
  - symbols: `Module`, `Stmt`, `Expr`, `BinOp`, `UnaryOp`, `Pattern`, `TypeExpr`
- **crates/cclab-taipan/src/types/check.rs** — type checking implementation
  - symbols: `TypeChecker`, `check_module`, `check_stmt`, `check_expr`
- **crates/cclab-taipan/src/types/context.rs** — type interning and registry
  - symbols: `TypeContext`, `intern`, `get`
- **crates/cclab-taipan/src/mir/mod.rs** — MIR definitions
  - symbols: `MirBody`, `BasicBlock`, `MirInst`, `Terminator`
- **crates/cclab-taipan/src/codegen/mod.rs** — codegen interface
  - symbols: `CodegenBackend`, `CodegenOutput`
- **crates/cclab-taipan/src/codegen/cranelift/mod.rs** — Cranelift backend
  - symbols: `CraneliftBackend`, `compile_function`, `codegen`

## Prism Results

- **prism_symbols** (query: `prism_symbols(crates/cclab-taipan/src/lexer/mod.rs)`)
  - Identified core lexing functions: lex, lex_raw.
- **prism_symbols** (query: `prism_symbols(crates/cclab-taipan/src/parser/mod.rs)`)
  - Identified Parser struct and parse entry point.
- **prism_symbols** (query: `prism_symbols(crates/cclab-taipan/src/driver/mod.rs)`)
  - Identified CompilerSession which drives the entire pipeline.
- **search_file_content** (query: `search_file_content("Parser::new")`)
  - Found usage in parser/mod.rs and multiple integration points in prism and genesis crates.

## Dependency Graph

- driver -> parser, types, source, error, diagnostic
- parser -> lexer, error
- lexer -> token, indent
- types -> hir, ast, error
- codegen -> mir, types, error
- codegen/cranelift -> codegen, mir, types
