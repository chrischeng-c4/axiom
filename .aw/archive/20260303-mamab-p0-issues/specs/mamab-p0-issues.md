---
id: mamab-p0-issues
type: spec
title: "Implement All P0 Mamba Parser Fixture Issues"
version: 1
spec_type: utility
created_at: 2026-03-03T14:53:00.749955+00:00
updated_at: 2026-03-03T14:53:00.749955+00:00
requirements:
  total: 7
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
    - R7
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-03-03T14:53:00.749955+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Implement All P0 Mamba Parser Fixture Issues

## Overview

Add comprehensive CPython 3.12 parse-only fixtures and targeted edge-case test files to the Mamba compiler test suite. This covers 30 P0 issues: importing CPython stdlib test files (#510-#519), creating targeted syntax edge-case fixtures (#550-#554, #559-#561, #566-#576), and enhancing the test harness with EXPECT-ERROR support (#629).

## Requirements

### R1 - Enhance fixture harness with EXPECT-ERROR support

```yaml
id: R1
priority: high
status: draft
```

Enhance fixture_tests.rs to support EXPECT-ERROR directive in run_parse mode, enabling negative parse tests (#629)

### R2 - Add CPython 3.12 core language fixtures

```yaml
id: R2
priority: high
status: draft
```

Add CPython 3.12 core language test files as parse-only fixtures: test_grammar.py, test_augassign.py, test_binop.py, test_call.py, test_extcall.py, test_named_expressions.py (#510-#512)

### R3 - Add CPython 3.12 pattern matching and type system fixtures

```yaml
id: R3
priority: high
status: draft
```

Add CPython 3.12 pattern matching, unpacking, and type system fixtures: test_patma.py, test_unpack.py, test_unpack_ex.py, test_type_aliases.py, test_type_annotations.py, test_type_comments.py, test_type_params.py, test_pep646_syntax.py (#513-#515)

### R4 - Add CPython 3.12 generator, async, comprehension, exception fixtures

```yaml
id: R4
priority: high
status: draft
```

Add CPython 3.12 generator, async, comprehension, and exception fixtures (#516-#519)

### R5 - Add targeted syntax edge-case fixtures

```yaml
id: R5
priority: high
status: draft
```

Add targeted edge-case fixtures: walrus operator, f-string nesting, match statement, starred expressions, async syntax, type parameters (PEP 695), exception groups (PEP 654), complex string literals (#550-#554, #559-#561)

### R6 - Add negative parse and PEP-specific fixtures

```yaml
id: R6
priority: high
status: draft
```

Add negative parse test suite with syntax errors that must be rejected, PEP 701 f-string tests, parenthesized context managers (PEP 617), soft keywords tests (#566-#569)

### R7 - Add comprehensive syntax pattern fixtures

```yaml
id: R7
priority: high
status: draft
```

Add operator precedence, deeply nested expressions, import patterns, multi-line continuations, class definitions, function signatures, and try/except combination fixtures (#570-#576)

## Acceptance Criteria

### Scenario: all_parse_fixtures_pass

- **GIVEN** All new .py fixture files are in tests/fixtures/parse/
- **WHEN** cargo test -p mamba fixture_tests runs
- **THEN** All parse-only fixtures pass without errors

### Scenario: negative_parse_tests_reject

- **GIVEN** EXPECT-ERROR harness enhancement is implemented and negative fixtures exist
- **WHEN** cargo test -p mamba fixture_tests runs
- **THEN** Negative test fixtures correctly fail to parse with expected error messages

### Scenario: fixture_format_convention

- **GIVEN** Any new fixture file
- **WHEN** File is inspected
- **THEN** File starts with # RUN: parse, contains only pure Python syntax

</spec>
