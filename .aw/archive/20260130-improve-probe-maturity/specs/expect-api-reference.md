---
id: expect-api-reference
type: spec
title: "Expect Assertion API Reference"
version: 1
spec_type: utility
created_at: 2026-01-28T16:57:56.014322+00:00
updated_at: 2026-01-28T16:57:56.014322+00:00
requirements:
  total: 6
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-01-28T16:57:56.014322+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Expect Assertion API Reference

## Overview

Provide a comprehensive API reference for the expect() assertion matchers in cclab-probe. This includes documentation for equality, boolean, option, numeric, string, collection, and JSON matchers. It specifically covers advanced features like JSON Path lookup and Option type assertions.

## Requirements

### R1 - Equality Matchers

```yaml
id: R1
priority: medium
status: draft
```

Document all equality matchers (to_equal, to_not_equal).

### R2 - Numeric Matchers

```yaml
id: R2
priority: medium
status: draft
```

Document all numeric matchers (greater_than, less_than, etc.).

### R3 - String Matchers

```yaml
id: R3
priority: medium
status: draft
```

Document all string matchers (contains, matches, etc.).

### R4 - Collection Matchers

```yaml
id: R4
priority: medium
status: draft
```

Document all collection matchers (contains_item, have_length).

### R5 - Option Matchers

```yaml
id: R5
priority: medium
status: draft
```

Document matchers for Option types (to_be_some, to_be_none).

### R6 - JSON Path Support

```yaml
id: R6
priority: medium
status: draft
```

Document dot-notation path lookup for JSON objects (to_have_path_value).

## Acceptance Criteria

### Scenario: Verify Documentation Completeness

- **GIVEN** The documentation is generated.
- **WHEN** A check for implemented matchers is performed.
- **THEN** All matchers implemented in src/assertions.rs should be documented.

### Scenario: Verify Example Accuracy

- **GIVEN** A user reads the documentation.
- **WHEN** Documentation examples are reviewed.
- **THEN** The examples should be correct and testable.

### Scenario: JSON Path Assertion

- **GIVEN** A test uses to_have_path_value with a nested path.
- **WHEN** A nested JSON path like 'a.b.c' is asserted.
- **THEN** The assertion should correctly resolve the path and verify the value.

</spec>
