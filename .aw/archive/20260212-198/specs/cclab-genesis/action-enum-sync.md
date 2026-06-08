---
id: action-enum-sync
type: spec
title: "Action Enum Synchronization"
version: 1
spec_type: utility
spec_group: cclab-genesis
main_spec_ref: run-change
merge_strategy: patch
created_at: 2026-02-12T08:21:48.777460+00:00
updated_at: 2026-02-12T08:21:48.777460+00:00
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
  - timestamp: 2026-02-12T08:21:48.777460+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Action Enum Synchronization

## Overview

Synchronize the run-change/README.md OpenRPC action enum with actual actions used in implement-change.md and merge-change.md sub-specs. Add 5 missing actions and remove 1 orphan action.

## Requirements

### R1 - Add missing implementation actions

```yaml
id: R1
priority: high
status: draft
```

Add review_task, revise_task, task_terminal_failure, all_tasks_done to the action enum in run-change/README.md OpenRPC definition.

### R2 - Add missing merge action

```yaml
id: R2
priority: high
status: draft
```

Add merge_complete to the action enum in run-change/README.md OpenRPC definition.

### R3 - Remove orphan complete action

```yaml
id: R3
priority: medium
status: draft
```

Remove 'complete' from the action enum — no sub-spec defines its behavior. It was likely a placeholder replaced by merge_complete.

## Acceptance Criteria

### Scenario: Action enum includes all implementation actions

- **WHEN** Reading run-change/README.md OpenRPC action enum
- **THEN** Contains review_task, revise_task, task_terminal_failure, all_tasks_done

### Scenario: Action enum includes merge_complete

- **WHEN** Reading run-change/README.md OpenRPC action enum
- **THEN** Contains merge_complete

### Scenario: Orphan complete removed

- **WHEN** Reading run-change/README.md OpenRPC action enum
- **THEN** Does not contain 'complete'

</spec>
