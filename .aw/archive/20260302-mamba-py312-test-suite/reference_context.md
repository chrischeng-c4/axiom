---
change_id: mamba-py312-test-suite
type: reference_context
created_at: 2026-03-02T15:24:36.887219+00:00
updated_at: 2026-03-02T15:24:36.887219+00:00
---

# Reference Context: mamba-py312-test-suite

## Specs

No formal specs exist for cclab-mamba or cclab-mamba-tests.

## Codebase

### Affected Modules

| Module | Path | Relevance |
|--------|------|----------|
| cclab-mamba-tests | `crates/cclab-mamba-tests` | **Primary** — all new fixtures go here |
| cclab-mamba | `crates/cclab-mamba` | Reference — tests exercise its `driver::compile_source` API |

### Architecture

**cclab-mamba** is a force-typed Python compiler with modules: lexer → parser → resolve → types → hir → mir → lower → codegen → runtime → driver.

**cclab-mamba-tests** uses `datatest-stable` for file-based test discovery:
- Single test binary: `cpython_compat` (custom harness)
- Discovers all `*.py` fixtures under `tests/fixtures/cpython/`
- Groups by CPython test module: test_grammar, test_syntax, test_fstring, test_dict, test_list, test_set, test_tuple, test_builtins
- `known_failures.toml` maps paths to `{reason, category}` (categories: parser, codegen, runtime)
- Inline `# XFAIL` / `# REASON:` directives override toml xfails
- 4 outcomes: pass, xfail (expected), xpass (unexpected pass → warning), real failure

### Current State (20 fixtures, 799 LOC)

**Passing (17):** basic_statements, functions, type_alias_simple, expressions, basic (fstring), control_flow, exception_group, dict_operations, list_operations, set_operations, tuple_operations, basic_builtins, star_unpack_assignment, multiline_fstring*, nested_fstrings*, debug_format* (*xpass)

**Known Failures (3):** type_alias_complex (PEP 695), generic_class_keywords (class kwargs), dict_unpacking (dict literal unpacking)

**Stale XFails (3):** multiline_fstring, nested_fstrings, debug_format — parser has since gained support, entries should be removed from known_failures.toml.

### Dependencies
- `cclab-mamba` — parser + compiler under test
- `datatest-stable` — file-based test discovery harness
- `toml` + `serde` — known_failures.toml deserialization

### Patterns & Conventions
- Each .py fixture is a standalone snippet; no import/stdlib/runtime dependencies
- Fixture names match CPython test module naming (test_grammar/, test_syntax/, etc.)
- One .py file per feature/subcategory
- known_failures.toml is the single source of truth for expected failures

## Gap Analysis

### GAP-1: Low Coverage (HIGH)
Only 20 fixtures across 8 categories. CPython 3.12 `test_grammar` alone covers ~50 subcategories. Missing: decorators, imports, walrus operator (:=), match/case, comprehensions, generators, async/await, closures, nested classes, lambda, global/nonlocal, yield.

### GAP-2: Stale XFail Entries (MEDIUM)
3 xpass tests (multiline_fstring, nested_fstrings, debug_format) should be removed from `known_failures.toml`.

### GAP-3: Missing Test Categories (HIGH)
No fixtures for: test_string, test_bytes, test_int, test_float, test_bool, test_none, test_comprehensions, test_generators, test_decorators, test_import, test_with, test_async, test_match, test_walrus, test_exceptions.

### GAP-4: Unmirrored Parser Fixtures (MEDIUM)
`cclab-mamba/tests/fixtures/parse/cpython/` has 17 parser-level fixtures (including stdlib/) not mirrored in cclab-mamba-tests. These could be promoted to full compat tests.

### GAP-5: No Mamba Spec (LOW)
The mamba crate has no formal spec documentation.
