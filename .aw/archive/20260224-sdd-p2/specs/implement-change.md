---
id: implement-change
type: spec
title: "Update implement spec — document codegen action and agent/mainthread boundary"
version: 1
spec_type: utility
created_at: 2026-02-23T16:50:27.319389+00:00
updated_at: 2026-02-23T16:50:27.319389+00:00
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
  - timestamp: 2026-02-23T16:50:27.319389+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Update implement spec — document codegen action and agent/mainthread boundary

## Overview

Updates the implement phase spec to document: (1) the implement_task_with_codegen action that exists in implementation but is missing from spec's action enum (#478), and (2) the correct agent/mainthread responsibility boundary where agents call sdd_run_change(advance_to=...) directly rather than mainthread (#481).

## Requirements

### R1 - Document implement_task_with_codegen action

```yaml
id: R1
priority: medium
status: draft
```

Spec must add implement_task_with_codegen to the action enum. This action allows task implementation with code generation capabilities.

### R2 - Document agent-calls-advance pattern

```yaml
id: R2
priority: medium
status: draft
```

Spec must document that in implement and merge phases, the agent (not mainthread) calls sdd_run_change(advance_to=...) to advance the state machine. This is the inverse of what spec currently says.

### R3 - Update executor field documentation

```yaml
id: R3
priority: medium
status: draft
```

Spec must document that executor field in run_change response indicates who should execute the prompt (agent or mainthread).

## Acceptance Criteria

### Scenario: Agent advances state after implementation

- **GIVEN** Agent completes implement_task action
- **WHEN** Agent finishes writing code
- **THEN** Agent calls sdd_run_change(advance_to='task_implemented') directly

### Scenario: Codegen implementation

- **GIVEN** Task requires code generation
- **WHEN** implement_task_with_codegen action is used
- **THEN** Agent implements task with code generation capabilities enabled

</spec>
