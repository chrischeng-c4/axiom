---
id: mamba-test-harness-refinement
type: spec
title: "Test Harness Refinement"
version: 1
spec_type: algorithm
tags: [logic]
merge_strategy: replace
created_at: 2026-02-13T10:37:27.279714+00:00
updated_at: 2026-02-13T10:37:27.279714+00:00
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
    - type: flowchart
      title: "Harness Dispatch and Execution Flow"
history:
  - timestamp: 2026-02-13T10:37:27.279714+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Test Harness Refinement

## Overview

This specification defines the refinements to the Mamba test harness (fixture_tests.rs) to support robust testing of Python 3.12 syntax. It includes the logic for multi-stage runner dispatch and enhanced diagnostic reporting for fixture failures.

## Requirements

### R1 - Directive Dispatch Logic

```yaml
id: R1
priority: high
status: draft
```

The harness must correctly parse the '# RUN:' directive and dispatch the fixture to the appropriate runner (parse, typecheck, or jit).

### R2 - Enhanced Error Reporting

```yaml
id: R2
priority: medium
status: draft
```

Failure messages must include the fixture file path and detailed error context from the Mamba diagnostic engine.

### R3 - Recursive Fixture Discovery

```yaml
id: R3
priority: medium
status: draft
```

The harness must support Recursive discovery of .py files within the 'tests/fixtures/' directory tree.

## Acceptance Criteria

### Scenario: Dispatch to Parse Runner

- **WHEN** A fixture with '# RUN: parse' is encountered.
- **THEN** The 'run_parse' function should be called.

### Scenario: Report Detailed Failure

- **WHEN** A fixture fails to parse.
- **THEN** The error message should contain the file path and the specific syntax error.

### Scenario: Recursive Discovery

- **WHEN** A new .py file is added to a deeply nested directory under 'tests/fixtures/'.
- **THEN** The harness should automatically find and run the file.

## Diagrams

### Harness Dispatch and Execution Flow

```mermaid
flowchart TB
    Start((Start run_fixture(path)))
    ReadFile[Read File to String]
    ParseDirectives[Extract # RUN: / # EXPECT:]
    DispatchRunner{Switch on RUN mode} 
    RunParse[[Execute parser::parse()]]
    RunTypeCheck[[Execute TypeChecker]]
    RunJit[[Execute Cranelift JIT]]
    CheckResult{Matches Expected?} 
    ReportPass([Return Ok(())])
    ReportFail([Panic with detailed info])
    End((End))
    Start --> ReadFile
    ReadFile --> ParseDirectives
    ParseDirectives --> DispatchRunner
    DispatchRunner -->|parse| RunParse
    DispatchRunner -->|typecheck| RunTypeCheck
    DispatchRunner -->|jit| RunJit
    RunParse --> CheckResult
    RunTypeCheck --> CheckResult
    RunJit --> CheckResult
    CheckResult -->|Success| ReportPass
    CheckResult -->|Failure| ReportFail
    ReportPass --> End
    ReportFail --> End
```

</spec>
