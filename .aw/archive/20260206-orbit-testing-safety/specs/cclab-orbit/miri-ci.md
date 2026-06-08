---
id: miri-ci
type: spec
title: "Miri CI Integration"
version: 1
spec_type: utility
spec_group: cclab-orbit
created_at: 2026-02-05T16:13:59.825089+00:00
updated_at: 2026-02-05T16:13:59.825089+00:00
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-05T16:13:59.825089+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Miri CI Integration

## Overview

Add Miri testing to CI pipeline to detect undefined behavior, particularly around atomic operations and memory ordering. Since the crate uses #![forbid(unsafe_code)], Miri primarily validates correctness of atomic Ordering choices (Release/Acquire patterns).

## Requirements

### R1 - CI workflow configuration

```yaml
id: R1
priority: high
status: draft
```

Add Miri test job to GitHub Actions workflow using nightly toolchain with miri component

### R2 - Miri-compatible test subset

```yaml
id: R2
priority: high
status: draft
```

Identify and run tests that don't require PyO3/Python runtime under Miri (pure-Rust unit tests)

### R3 - Atomic ordering validation

```yaml
id: R3
priority: medium
status: draft
```

Miri validates AtomicBool operations in PythonWaker, Handle, and TimerWheel use correct ordering

## Acceptance Criteria

### Scenario: Miri runs pure-Rust tests

- **WHEN** cargo miri test is executed
- **THEN** Pure-Rust tests pass without UB detection

### Scenario: Atomic ordering correctness

- **GIVEN** Tests using PythonWaker with concurrent access
- **WHEN** Miri runs with -Zmiri-symbolic-alignment-check
- **THEN** No ordering violations detected

### Scenario: CI workflow integration

- **WHEN** PR is opened
- **THEN** Miri test job runs and reports results

</spec>
