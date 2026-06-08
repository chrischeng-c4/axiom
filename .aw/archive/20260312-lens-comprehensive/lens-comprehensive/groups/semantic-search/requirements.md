---
change: lens-comprehensive
group: semantic-search
date: 2026-03-12
---

# Requirements

Implement semantic search engine using the existing query/result types in types/semantic_search.rs. Search modes:
1. ByTypeSignature — find functions matching type patterns like (str, int) -> bool
2. CallHierarchy — callers/callees at configurable depth
3. Implementations — find all impl of a trait/interface/protocol
4. Usages — smart references excluding comments/strings
5. SimilarCode — structural similarity detection
6. DocumentationSearch — search docstrings/comments

Build inverted index from SymbolTable across project. Incremental update on file change via IncrementalAnalyzer. Add MCP tools: lens_search, lens_call_graph. Add RPC methods to handler.
