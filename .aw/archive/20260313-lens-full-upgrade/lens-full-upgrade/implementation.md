---
id: implementation
type: change_implementation
change_id: lens-full-upgrade
---

# Implementation

## Summary

Full cclab-lens upgrade: 10 feature groups across 29 files (12 modified, 17 new). Groups: (1) Disk cache already done, (2) Lint densification TS/JS/Rust/CSS 5→15+ rules each, (3) Auto-fix framework, (4) Go type inference with interface satisfaction + generics, (5) New language support: TOML/SQL/Proto/GraphQL checkers + symbol extractors, (6) Cross-file import graph with circular dep detection, (7) Formatter integration wrapping rustfmt/prettier/gofmt/black. Total: +1196 -98 lines in modified files, ~4200 lines in new files. cargo check passes (1 pre-existing warning). 720/721 tests pass (1 pre-existing macOS test failure).

## Diff

```diff
Modified files (12): lib.rs, lint/mod.rs, lint/css.rs (+384), lint/javascript.rs (+244), lint/rust_checker.rs (+310), lint/typescript.rs (+292), lsp/server.rs, semantic/mod.rs, semantic/symbols/mod.rs, server/handler.rs, syntax/parser.rs, types/incremental.rs

New files (17):
- format/detect.rs (51L) - Binary detection via `which`
- format/mod.rs (395L) - FormatterRegistry: rustfmt/prettier/gofmt/black/terraform/pg_format
- graph/mod.rs (302L) - ImportGraph: build, find_circular_dependencies, find_unused_files
- graph/resolve.rs (318L) - Per-language import extraction (Python/JS/TS/Rust/Go)
- lint/autofix.rs (232L) - apply_fix, apply_all_fixes, position_to_offset
- lint/graphql.rs (293L) - GraphqlChecker: GQ001-GQ007
- lint/proto.rs (326L) - ProtoChecker: PB001-PB007
- lint/sql.rs (321L) - SqlChecker: SQ001-SQ005, PG001, MY001 + detect_sql_injection
- lint/toml_checker.rs (375L) - TomlChecker: TM001-TM005
- semantic/symbols/graphql_sym.rs (278L) - GraphQL symbol extraction
- semantic/symbols/proto_sym.rs (233L) - Proto symbol extraction
- semantic/symbols/sql_sym.rs (250L) - SQL symbol extraction
- semantic/symbols/toml_sym.rs (139L) - TOML symbol extraction
- semantic/types/go.rs (498L) - GoType enum, GoTypeInference
- semantic/types/go_advanced.rs (428L) - check_interface_satisfaction, validate_type_assertions
- semantic/types/go_tests.rs (252L) - Go type inference tests
- semantic/types/mod.rs (15L) - Module re-exports

Key integration points:
- Language enum: +4 variants (Toml, Sql, Proto, GraphQL) in parser.rs
- CheckerRegistry: +4 checkers registered
- SymbolTableBuilder: +4 build_* methods wired in LSP server + daemon handler
- Line-based fallback: expanded to include all new languages
- incremental.rs: new variants in dependency extraction match
```

## Review: lens-full-upgrade-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: lens-full-upgrade

**Summary**: All 7 groups implemented with core logic complete. 29 files (12 modified, 17 new), ~5400 lines added. cargo check passes (1 pre-existing warning). 720/721 tests pass (1 pre-existing macOS canonicalize test). MCP tool registration and LSP code-action/formatting handlers are follow-up work — core analysis, lint, and inference logic is fully functional.

### Checklist

- [PASS] R1 Disk Cache
  - Already implemented from prior plan. DiskCache, PersistedEntry, CacheManifest, load/store/flush all working.
- [PASS] R2 TypeScript Lint 5→15+
  - TS006-TS015 added (10 new rules). 489 lines total.
- [PASS] R3 JavaScript Lint 5→15+
  - JS006-JS015 added (10 new rules). 434 lines total.
- [PASS] R4 Rust Lint 5→15+
  - RS006-RS015 added (10 new rules). 488 lines total.
- [PASS] R5 CSS Lint 5→15+
  - CSS006-CSS020 added (10+ new rules). 479 lines total.
- [PASS] R6 Auto-Fix Framework
  - autofix.rs with apply_fix, apply_all_fixes, position_to_offset. 232 lines. LSP code action provider deferred.
- [PASS] R7 Go Type Inference Deep
  - GoType enum, GoTypeInference, interface satisfaction, generics, channels, method sets. go.rs (498L) + go_advanced.rs (428L) + go_tests.rs (252L).
- [PASS] R8 TOML Checker
  - TomlChecker TM001-TM005 + toml_sym.rs symbol extraction. Registered in CheckerRegistry.
- [PASS] R9 SQL Support
  - SqlChecker SQ001-SQ005, PG001, MY001 + detect_sql_injection + sql_sym.rs. Line-based.
- [PASS] R10 Proto/gRPC Support
  - ProtoChecker PB001-PB007 + proto_sym.rs. Line-based, proto3 focus.
- [PASS] R11 GraphQL Support
  - GraphqlChecker GQ001-GQ007 + graphql_sym.rs. Line-based.
- [PASS] R12 Cross-File Import Graph
  - ImportGraph with build, find_circular_dependencies, find_unused_files, detect_entry_points. graph/mod.rs (302L) + resolve.rs (318L).
- [PASS] R13 Formatter Integration
  - FormatterRegistry wrapping rustfmt/prettier/gofmt/black/terraform/pg_format. format/mod.rs (395L) + detect.rs (51L). LSP formatting handler deferred.

### Issues

- **[LOW]** MCP tools (lens_fix, lens_import_graph, lens_format) not yet registered in MCP server
  - *Recommendation*: Follow-up change to wire MCP tool handlers
- **[LOW]** LSP code action provider for auto-fix and textDocument/formatting handler not yet added
  - *Recommendation*: Follow-up change to add LSP protocol handlers
- **[LOW]** New languages use line-based parsing instead of tree-sitter grammars
  - *Recommendation*: Acceptable for initial release; tree-sitter grammars can be added later for richer AST analysis
