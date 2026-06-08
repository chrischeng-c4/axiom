---
id: fetch-issues-action-rename
type: spec
title: "Fetch Issues Action Name Update"
version: 1
spec_type: utility
spec_group: cclab-genesis
main_spec_ref: fetch-issues
merge_strategy: patch
created_at: 2026-02-12T08:27:16.439879+00:00
updated_at: 2026-02-12T08:27:16.439879+00:00
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
  - timestamp: 2026-02-12T08:27:16.439879+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Fetch Issues Action Name Update

## Overview

Rename outdated action names in fetch-issues.md: create_spec_context → explore_spec, create_knowledge_context → explore_knowledge. Aligns with run-change/README.md canonical naming.

## Requirements

### R1 - Rename create_spec_context to explore_spec

```yaml
id: R1
priority: high
status: draft
```

Replace all occurrences of create_spec_context with explore_spec in fetch-issues.md.

### R2 - Rename create_knowledge_context to explore_knowledge

```yaml
id: R2
priority: high
status: draft
```

Replace all occurrences of create_knowledge_context with explore_knowledge in fetch-issues.md.

## Acceptance Criteria

### Scenario: No outdated action names remain

- **WHEN** Searching fetch-issues.md for create_spec_context or create_knowledge_context
- **THEN** Zero matches found

### Scenario: New action names present

- **WHEN** Searching fetch-issues.md for explore_spec or explore_knowledge
- **THEN** Matches found at previously-referenced locations

</spec>
