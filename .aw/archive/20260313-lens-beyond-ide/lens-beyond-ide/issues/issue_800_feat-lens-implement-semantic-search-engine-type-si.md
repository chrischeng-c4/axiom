---
number: 800
title: "feat(lens): implement semantic search engine — type signatures, call hierarchy, usages"
state: closed
labels: [enhancement, P1, crate:lens]
group: "rpc-mcp-wiring"
---

# #800 — feat(lens): implement semantic search engine — type signatures, call hierarchy, usages

## Context

Semantic search query types and result structures are fully defined in `types/semantic_search.rs` but the search algorithm is not wired up. With MCP integration, this enables AI-assisted code exploration.

## Scope

### Search modes
1. **ByTypeSignature** — find functions matching `(str, int) -> bool`
2. **CallHierarchy** — callers/callees at any depth
3. **Implementations** — find all impl of a trait/interface
4. **Usages** — smart references (exclude comments/strings)
5. **SimilarCode** — structural similarity detection
6. **DocumentationSearch** — search docstrings/comments

### Index
- Build inverted index from SymbolTable across project
- Incremental update on file change (leverage existing `IncrementalAnalyzer`)

## Existing code to build on
- `types/semantic_search.rs` — Query/Result types defined
- `semantic/symbols/` — Per-language symbol extraction
- `types/incremental.rs` — DependencyGraph + ChangeTracker
- `server/handler.rs` — Already has `ensure_analyzed()` pattern

## New MCP tools
- `lens_search` — unified semantic search
- `lens_call_graph` — call hierarchy visualization

## Files to create/modify
- CREATE `crates/cclab-lens/src/search/mod.rs` — SearchEngine
- CREATE `crates/cclab-lens/src/search/index.rs` — Inverted index
- CREATE `crates/cclab-lens/src/search/query.rs` — Query execution
- MODIFY `crates/cclab-lens/src/mcp/tools.rs` — Add search tools
- MODIFY `crates/cclab-lens/src/server/handler.rs` — Add RPC methods
