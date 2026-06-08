---
id: test-id-no-spec
type: spec
title: "Test Spec"
version: 1
spec_type: utility
created_at: 2026-02-01T07:09:03.526723+00:00
updated_at: 2026-02-01T07:09:03.526723+00:00
requirements:
  total: 1
  ids:
    - R1
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-01T07:09:03.526723+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Test Spec

## Overview

This is a test specification to verify if the spec creation tool allows IDs that do not match the required pattern ending in -spec. This is important for understanding how to resolve the validation issues in the current change directory.

## Requirements

### R1 - Test Requirement

```yaml
id: R1
priority: medium
status: draft
```

This is a test requirement to ensure the spec creation works.

## Acceptance Criteria

### Scenario: Test Scenario

- **WHEN** The tool is called with an invalid ID pattern.
- **THEN** The system should successfully create the spec file.

</spec>
