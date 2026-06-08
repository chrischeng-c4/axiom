---
change: lens-comprehensive
group: refactoring-engine
date: 2026-03-12
---

# Requirements

Implement refactoring execution engines using the existing request/result types in types/refactoring.rs. Operations in priority order:
1. Rename Symbol — across files, respecting scope (use ScopeAnalyzer + SymbolTable)
2. Extract Function — selection → new function with inferred params/return type
3. Extract Variable — expression → named variable
4. Inline Symbol — replace references with definition body
5. Move Definition — between modules, update imports
6. Change Signature — add/remove/reorder params

Support Python, TypeScript, Rust. Add RPC methods to server/handler.rs and MCP tools to mcp/tools.rs. Each operation produces Vec<TextEdit> applied across files.
