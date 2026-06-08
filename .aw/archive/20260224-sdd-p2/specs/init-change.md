---
id: init-change
type: spec
title: "Update init-change spec — git_workflow replaces branch_hint"
version: 1
spec_type: utility
created_at: 2026-02-23T16:50:06.007357+00:00
updated_at: 2026-02-23T16:50:06.007357+00:00
requirements:
  total: 2
  ids:
    - R1
    - R2
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-23T16:50:06.007357+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Update init-change spec — git_workflow replaces branch_hint

## Overview

Updates the init-change spec to document that implementation uses git_workflow parameter with enum values 'new_branch'|'in_place' instead of the spec-defined branch_hint with 'main'|'feature'. Addresses #476.

## Requirements

### R1 - Document git_workflow parameter

```yaml
id: R1
priority: medium
status: draft
```

Spec must document git_workflow parameter replacing branch_hint, with enum values: 'new_branch' (creates cclab/{change-id} branch) and 'in_place' (stays on current branch).

### R2 - Remove branch_hint references

```yaml
id: R2
priority: medium
status: draft
```

Spec must replace all branch_hint references with git_workflow and update enum values from 'main'|'feature' to 'new_branch'|'in_place'.

## Acceptance Criteria

### Scenario: New branch workflow

- **GIVEN** User chooses new_branch git_workflow
- **WHEN** init_change executes
- **THEN** A new branch cclab/{change-id} is created and checked out

### Scenario: In-place workflow

- **GIVEN** User chooses in_place git_workflow
- **WHEN** init_change executes
- **THEN** No branch change occurs, work continues on current branch

</spec>
