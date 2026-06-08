---
id: expand-cpython-compat-fixtures
type: spec
title: "Expand CPython 3.12 Compatibility Test Fixtures"
version: 1
spec_type: utility
created_at: 2026-03-02T15:29:28.922255+00:00
updated_at: 2026-03-02T15:29:28.922255+00:00
requirements:
  total: 11
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
    - R7
    - R8
    - R9
    - R10
    - R11
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-03-02T15:29:28.922255+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Expand CPython 3.12 Compatibility Test Fixtures

## Overview

Add comprehensive CPython 3.12 test fixtures to cclab-mamba-tests, covering missing language features (decorators, comprehensions, match/case, async, walrus, etc.) and clean up stale xfail entries.

## Requirements

### R1 - Clean up stale xfail entries

```yaml
id: R1
priority: high
status: draft
```

Remove 3 stale entries from known_failures.toml that now pass: test_fstring/multiline_fstring, test_fstring/nested_fstrings, test_fstring/debug_format.

### R2 - Add test_grammar fixtures

```yaml
id: R2
priority: high
status: draft
```

Add fixtures for: decorators.py, lambda_expressions.py, comprehensions.py, walrus_operator.py, global_nonlocal.py, yield_expressions.py. Each standalone .py snippet exercising the feature.

### R3 - Add test_match fixtures

```yaml
id: R3
priority: high
status: draft
```

Add match/case structural pattern matching fixtures (PEP 634): match_basic.py (literal/variable patterns), match_class.py (class patterns), match_mapping.py (dict patterns), match_star.py (sequence star patterns).

### R4 - Add test_comprehensions fixtures

```yaml
id: R4
priority: high
status: draft
```

Add fixtures: list_comp.py, dict_comp.py, set_comp.py, generator_expr.py covering list/dict/set comprehensions and generator expressions with filters.

### R5 - Add test_async fixtures

```yaml
id: R5
priority: medium
status: draft
```

Add fixtures: async_def.py (async functions), async_for.py (async iteration), async_with.py (async context managers), await_expr.py (await expressions).

### R6 - Add test_string fixtures

```yaml
id: R6
priority: medium
status: draft
```

Add fixtures: string_methods.py (split, join, format, etc.), string_slicing.py (slice operations), raw_strings.py (r-strings, byte strings).

### R7 - Add test_exceptions fixtures

```yaml
id: R7
priority: medium
status: draft
```

Add fixtures: try_except.py (basic exception handling), exception_chaining.py (raise from), exception_groups.py (except* PEP 654).

### R8 - Add test_with fixtures

```yaml
id: R8
priority: medium
status: draft
```

Add fixtures: context_manager.py (basic with), multiple_with.py (parenthesized with targets PEP 617).

### R9 - Add test_decorators fixtures

```yaml
id: R9
priority: medium
status: draft
```

Add fixtures: function_decorators.py, class_decorators.py, stacked_decorators.py.

### R10 - Add test_import fixtures

```yaml
id: R10
priority: low
status: draft
```

Add fixtures: import_basic.py (import, from import), import_relative.py (relative imports). Mark as XFAIL if parser doesn't support.

### R11 - Update known_failures.toml for new xfails

```yaml
id: R11
priority: high
status: draft
```

Add entries for any new fixtures that exercise unsupported features (e.g., match/case, async, certain import forms).

## Acceptance Criteria

### Scenario: All existing tests still pass

- **GIVEN** Current 20 fixtures
- **WHEN** cargo test -p mamba-tests
- **THEN** All 20 pass with 0 failures

### Scenario: Stale xfails removed

- **GIVEN** Updated known_failures.toml without 3 stale entries
- **WHEN** cargo test -p mamba-tests
- **THEN** No xpass warnings for multiline_fstring, nested_fstrings, debug_format

### Scenario: New fixtures discovered

- **GIVEN** New .py files added to tests/fixtures/cpython/
- **WHEN** cargo test -p mamba-tests -- --list
- **THEN** Test count increases from 20 to ~50+

### Scenario: Unsupported features marked as xfail

- **GIVEN** New fixtures with # XFAIL or known_failures.toml entries
- **WHEN** cargo test -p mamba-tests
- **THEN** Unsupported features show [xfail] not hard failure

</spec>
