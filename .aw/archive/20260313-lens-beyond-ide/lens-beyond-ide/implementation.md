---
id: implementation
type: change_implementation
change_id: lens-beyond-ide
---

# Implementation

## Summary

Beyond-IDE upgrade across 5 areas: (1) RPC/MCP wiring — refactor+search+call_graph wired into handler.rs, protocol.rs, mcp/tools.rs, daemon.rs. (2) Schema validation — SchemaRegistry with programmatic K8s (10 resource types) and GitLab CI schemas, wired to K8002/GL002 rules. (3) Tree-sitter upgrade 0.24→0.25 — resolves 123/124 HCL grammar test failures, all grammar crates upgraded. (4) HTML/CSS expansion — 2 symbol builders (html.rs, css.rs) + 10 new lint rules (HTM002-011, CSS006-010). (5) Go support — tree-sitter-go grammar, GoChecker with 8 rules (GO001-008), Go symbol builder. Clean build, 560 tests pass (+123 fixed).

## Diff

```diff
Modified (10 files):
- Cargo.toml: tree-sitter 0.24→0.25, grammar crate upgrades, +tree-sitter-go
- Cargo.lock: dependency updates
- crates/cclab-sdd/Cargo.toml: tree-sitter 0.25
- crates/cclab-warp-bundler/Cargo.toml: tree-sitter 0.25
- crates/cclab-warp-transform/Cargo.toml: tree-sitter 0.25
- src/syntax/parser.rs: +Language::Go variant, Go parser init
- src/lint/mod.rs: +go, html_rules, css_rules modules, GoChecker registered
- src/semantic/symbols/mod.rs: +html, css, go builders registered
- src/lsp/server.rs: +Go/Html/Css dispatch for symbol building
- src/types/incremental.rs: +Language::Go match arm

- src/server/handler.rs: +refactoring+search_engine fields, handle_refactor/search/call_graph
- src/server/protocol.rs: +RefactorParams, SearchParams, CallGraphParams, parse helpers
- src/server/mod.rs: export new protocol types
- src/server/daemon.rs: +DaemonClient refactor/search/call_graph methods
- src/mcp/tools.rs: +lens_refactor/search/call_graph tool schemas
- src/mcp/server.rs: +MCP routing for 3 new tools

- src/lint/kubernetes.rs: wire schema validation
- src/lint/kubernetes_rules.rs: +check_schema_validation
- src/lint/gitlab_ci.rs: wire schema validation
- src/lint/gitlab_ci_rules.rs: +check_schema_validation
- src/lint/html.rs: +HTM002-005,010 rules, pub(crate) helpers
- src/lint/css.rs: wire css_rules
- src/lib.rs: +pub mod schemas

New (8 files):
- src/schemas/mod.rs (124) — SchemaRegistry
- src/schemas/k8s.rs (491) — K8s schemas for 10 resource types
- src/schemas/gitlab.rs (124) — GitLab CI schema
- src/lint/go.rs (420) — GoChecker with GO001-008
- src/lint/html_rules.rs (208) — HTM006-009,011
- src/lint/css_rules.rs (291) — CSS006-010
- src/semantic/symbols/html.rs (263) — HTML symbol builder
- src/semantic/symbols/css.rs (252) — CSS symbol builder
- src/semantic/symbols/go.rs (592) — Go symbol builder
```

## Review: lens-beyond-ide-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: lens-beyond-ide

**Summary**: All 12 requirements implemented. Clean build, 560 tests pass (+123 fixed by tree-sitter upgrade), 0 regressions. 1 pre-existing test failure (storage path resolution) unrelated to changes.

### Checklist

- [PASS] R1 - RPC Refactor Method
  - handle_refactor in handler.rs with dry_run support
- [PASS] R2 - RPC Search Methods
  - handle_search + handle_call_graph in handler.rs
- [PASS] R3 - MCP Tools Registration
  - lens_refactor, lens_search, lens_call_graph in tools.rs + server.rs
- [PASS] R4 - Search Index Eager Build
  - SearchEngine initialized in RequestHandler::new()
- [PASS] R5 - SchemaRegistry
  - schemas/mod.rs + k8s.rs + gitlab.rs with programmatic schemas
- [PASS] R6 - Schema-Backed Lint Rules
  - K8002 and GL002 wired to SchemaRegistry
- [PASS] R7 - Tree-sitter Upgrade
  - 0.24→0.25, all grammar crates upgraded, 123 test failures fixed
- [PASS] R8 - HTML Symbol Builder
  - symbols/html.rs with IDs, classes, forms, meta tags
- [PASS] R9 - CSS Symbol Builder
  - symbols/css.rs with selectors, custom properties, keyframes
- [PASS] R10 - HTML Lint Expansion
  - 10 new rules HTM002-011
- [PASS] R11 - CSS Lint Expansion
  - 5 new rules CSS006-010 (total 10)
- [PASS] R12 - Go Language Support
  - tree-sitter-go, GoChecker GO001-008, Go symbol builder

