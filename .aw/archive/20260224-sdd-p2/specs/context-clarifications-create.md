---
id: context-clarifications-create
type: spec
title: "Update create-context-clarifications spec to match implementation"
version: 1
spec_type: utility
created_at: 2026-02-23T16:49:47.650565+00:00
updated_at: 2026-02-23T16:49:47.650565+00:00
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
  - timestamp: 2026-02-23T16:49:47.650565+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Update create-context-clarifications spec to match implementation

## Overview

Updates the create-context-clarifications.md spec to document the issue fetch feature (sdd_write_artifact with artifact=issues_context), scope collection requirement, and 3 response fields (scoped_categories, issues_fetched, contradiction_count) that exist in implementation but are missing from the spec. Addresses #482.

## Requirements

### R1 - Document issue fetch feature

```yaml
id: R1
priority: medium
status: draft
```

Spec must document that when description references issues (#NNN) or label patterns, the clarify phase calls sdd_write_artifact(artifact='issues_context', action='fetch') to create issue_*.md files and build DAG in STATE.yaml.

### R2 - Document scope collection

```yaml
id: R2
priority: medium
status: draft
```

Spec must document the mandatory step asking about affected modules/scope, with options: specific crates, specific paths, unknown/unsure, whole project.

### R3 - Document response fields

```yaml
id: R3
priority: medium
status: draft
```

Spec must document the 3 response fields returned by sdd_write_artifact for context_clarifications: scoped_categories, issues_fetched, contradiction_count.

## Acceptance Criteria

### Scenario: Issue fetch during clarification

- **GIVEN** A change description referencing #472 #473
- **WHEN** Clarify phase executes
- **THEN** Issues are fetched via sdd_write_artifact(artifact='issues_context') and DAG is built in STATE.yaml

</spec>
