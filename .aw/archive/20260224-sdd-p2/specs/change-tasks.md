---
id: change-tasks
type: spec
title: "Update tasks phase spec — sdd_write_artifact replaces sdd_generate_tasks"
version: 1
spec_type: utility
created_at: 2026-02-23T16:49:56.733687+00:00
updated_at: 2026-02-23T16:49:56.733687+00:00
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
  - timestamp: 2026-02-23T16:49:56.733687+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Update tasks phase spec — sdd_write_artifact replaces sdd_generate_tasks

## Overview

Updates the tasks phase spec to document that implementation uses sdd_write_artifact(artifact='tasks', action='generate') instead of the spec-defined sdd_generate_tasks tool. The generic artifact tool with action='generate' provides the same deterministic task generation. Addresses #475.

## Requirements

### R1 - Document sdd_write_artifact for task generation

```yaml
id: R1
priority: medium
status: draft
```

Spec must document that task generation uses sdd_write_artifact(artifact='tasks', action='generate') rather than a dedicated sdd_generate_tasks tool.

### R2 - Document generate action semantics

```yaml
id: R2
priority: medium
status: draft
```

Spec must document that the 'generate' action on 'tasks' artifact performs deterministic task generation from specs, equivalent to what sdd_generate_tasks would have done.

## Acceptance Criteria

### Scenario: Task generation via write_artifact

- **GIVEN** Proposal is approved with spec_plan
- **WHEN** Tasks phase executes
- **THEN** sdd_write_artifact(artifact='tasks', action='generate') creates tasks.md with deterministic task list

</spec>
