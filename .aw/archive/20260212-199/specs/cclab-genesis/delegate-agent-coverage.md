---
id: delegate-agent-coverage
type: spec
title: "Delegate Agent Action/Artifact Coverage Fix"
version: 1
spec_type: utility
spec_group: cclab-genesis
main_spec_ref: delegate-agent
merge_strategy: patch
created_at: 2026-02-12T08:24:26.594556+00:00
updated_at: 2026-02-12T08:24:26.594556+00:00
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
  - timestamp: 2026-02-12T08:24:26.594556+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Delegate Agent Action/Artifact Coverage Fix

## Overview

Extend delegate-agent.md action enum to cover all delegatable actions (gap-create, merge, per-task impl) and fix verification table artifact names for spec actions (spec.md should be specs/{spec_id}.md).

## Requirements

### R1 - Extend action enum with gap-create actions

```yaml
id: R1
priority: high
status: draft
```

Add gap_codebase_spec, gap_codebase_knowledge, gap_spec_knowledge to the action enum.

### R2 - Extend action enum with merge/impl actions

```yaml
id: R2
priority: high
status: draft
```

Add implement_task, review_implementation, begin_merge, resume_merge, review_merge, fix_merge to the action enum.

### R3 - Fix spec verification artifact names

```yaml
id: R3
priority: high
status: draft
```

Update create_spec expected_artifact from spec.md to specs/{spec_group}/{spec_id}.md and review_spec from review_spec.md to review_spec.md (add note about dynamic naming). Also add verification rows for the new actions.

## Acceptance Criteria

### Scenario: Action enum covers gap-create actions

- **WHEN** Reading delegate-agent.md action enum
- **THEN** Contains gap_codebase_spec, gap_codebase_knowledge, gap_spec_knowledge

### Scenario: Action enum covers merge actions

- **WHEN** Reading delegate-agent.md action enum
- **THEN** Contains begin_merge, resume_merge, review_merge, fix_merge

### Scenario: Spec verification uses correct artifact path

- **WHEN** Reading verification table for create_spec
- **THEN** expected_artifact is specs/{spec_group}/{spec_id}.md

</spec>
