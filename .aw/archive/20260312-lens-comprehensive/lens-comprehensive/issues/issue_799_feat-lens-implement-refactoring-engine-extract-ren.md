---
number: 799
title: "feat(lens): implement refactoring engine — Extract, Rename, Move, Inline"
state: open
labels: [enhancement, P1, crate:lens]
group: "refactoring-engine"
---

# #799 — feat(lens): implement refactoring engine — Extract, Rename, Move, Inline

## Context

Refactoring request/result types are fully defined in `types/refactoring.rs` but the execution engines are not implemented. This is a major differentiator vs Ruff/Pyright (lint-only).

## Scope

### Operations (priority order)
1. **Rename Symbol** — across files, respecting scope
2. **Extract Function** — selection → new function with params/return
3. **Extract Variable** — expression → named variable
4. **Inline Symbol** — replace references with definition body
5. **Move Definition** — between modules, update imports
6. **Change Signature** — add/remove/reorder params

### Languages
- Python (primary)
- TypeScript
- Rust

## Architecture

```
RefactoringRequest → RefactoringEngine::apply()
  → ScopeAnalyzer (find all references)
  → TextEdit[] (compute edits)
  → Apply edits across files
```

## Existing code to build on
- `types/refactoring.rs` — All request/result structs defined
- `semantic/symbols/` — Symbol extraction per language
- `semantic/scope.rs` — Scope analysis

## Files to create/modify
- CREATE `crates/cclab-lens/src/refactoring/mod.rs` — Engine trait + registry
- CREATE `crates/cclab-lens/src/refactoring/rename.rs`
- CREATE `crates/cclab-lens/src/refactoring/extract.rs`
- CREATE `crates/cclab-lens/src/refactoring/inline.rs`
- CREATE `crates/cclab-lens/src/refactoring/move_def.rs`
- MODIFY `crates/cclab-lens/src/server/handler.rs` — Add RPC methods
- MODIFY `crates/cclab-lens/src/mcp/tools.rs` — Add MCP tools
