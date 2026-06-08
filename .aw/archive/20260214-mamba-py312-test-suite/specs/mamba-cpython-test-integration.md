---
id: mamba-cpython-test-integration
type: spec
title: "CPython Test Integration"
version: 1
spec_type: integration
tags: [external]
merge_strategy: replace
created_at: 2026-02-13T10:36:51.134392+00:00
updated_at: 2026-02-13T10:36:51.134392+00:00
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: sequence
      title: "CPython Fixture Execution Flow"
history:
  - timestamp: 2026-02-13T10:36:51.134392+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# CPython Test Integration

## Overview

This specification defines the integration of CPython test cases into the Mamba test suite. It focuses on using syntax snippets from CPython's own test suite to verify the Mamba parser's compatibility with Python 3.12 syntax.

## Requirements

### R1 - Fixture Directory Structure

```yaml
id: R1
priority: medium
status: draft
```

CPython test snippets must be stored in 'crates/mamba/tests/fixtures/parse/cpython/' to separate them from internal Mamba tests.

### R2 - Directive-based Snippet Format

```yaml
id: R2
priority: high
status: draft
```

Each CPython fixture file must start with the '# RUN: parse' directive to ensure it is processed by the parse-only runner in the test harness.

### R3 - Syntax-focused Extraction

```yaml
id: R3
priority: high
status: draft
```

Snippets must be extracted from CPython 'test_grammar.py', 'test_syntax.py', and py312-specific test files, focusing on syntax constructs and omitting CPython-specific test harness code.

## Acceptance Criteria

### Scenario: Run CPython Syntax Snippet

- **WHEN** A file 'tests/fixtures/parse/cpython/fstring_pep701.py' with '# RUN: parse' is added.
- **THEN** The fixture_tests.rs harness should auto-discover the file and pass it to the Mamba parser, which must successfully parse it.

### Scenario: Handle Syntax Error in Snippet

- **WHEN** A CPython snippet containing unsupported syntax is run.
- **THEN** The test runner should report a failure for that specific fixture.

### Scenario: Subdirectory Discovery

- **WHEN** A snippet is placed at 'tests/fixtures/parse/cpython/pep695/generic_fn.py'.
- **THEN** The harness should discover and run snippets located in nested subdirectories under 'cpython/'.

## Diagrams

### CPython Fixture Execution Flow

```mermaid
sequenceDiagram
    participant Harness as fixture_tests.rs
    participant Filesystem as OS Filesystem
    participant Parser as Mamba Parser
    actor User as Test Runner
    Harness->>+Filesystem: Discover tests in fixtures/parse/cpython/
    Filesystem->>Harness: List of .py files
    Harness->>Filesystem: Read file content
    Filesystem->>Harness: Source with # RUN: parse
    Harness->>Parser: parse(src)
    Parser->>Harness: AST Module
    Harness->>-User: Report PASS
```

</spec>
