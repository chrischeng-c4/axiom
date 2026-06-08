---
change_id: mamba-py312-test-suite
type: codebase_context
created_at: 2026-02-13T10:33:04.421237+00:00
updated_at: 2026-02-13T10:33:04.421237+00:00
iteration: 1
complexity: high
stage: codebase
prism_tools_used:
  - prism_symbols
  - search_file_content
---

# Codebase Context

## Analyzed Files

- **crates/mamba/src/parser/mod.rs** — Parser entry point and state management.
  - symbols: `Parser`, `parse_module`, `parse`
- **crates/mamba/src/parser/stmt.rs** — Statement parsing, including Py 3.12 type parameters.
  - symbols: `parse_stmt`, `parse_optional_type_params`
- **crates/mamba/src/parser/stmt_compound.rs** — Compound statements (def, class) using type parameters.
  - symbols: `parse_fn_def`, `parse_class_def`, `parse_type_alias`
- **crates/mamba/src/parser/type_expr.rs** — Type expression parsing (unions, generics).
  - symbols: `parse_type_expr`, `parse_type_atom`
- **crates/mamba/tests/fixture_tests.rs** — Test harness for directive-based .py fixtures.
  - symbols: `run_fixture`, `run_parse`, `parse_directives`

## Prism Results

- **search_file_content** (query: `fn parse_optional_type_params`)
  - Found implementation in stmt.rs; it parses [T, U] style type parameters.
- **prism_symbols** (query: `crates/mamba/src/parser/mod.rs`)
  - Listed symbols for Parser struct and main entry points.

## Dependency Graph

- cclab-mamba/src/parser/mod.rs -> cclab-mamba/src/parser/stmt.rs
- cclab-mamba/src/parser/stmt.rs -> cclab-mamba/src/parser/stmt_compound.rs
- cclab-mamba/src/parser/stmt_compound.rs -> cclab-mamba/src/parser/type_expr.rs
- cclab-mamba/tests/fixture_tests.rs -> cclab-mamba/src/parser/mod.rs
