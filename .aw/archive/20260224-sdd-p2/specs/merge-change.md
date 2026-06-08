---
id: merge-change
type: spec
title: "Update merge spec — document codebase_paths/knowledge_refs enrichment"
version: 1
spec_type: utility
created_at: 2026-02-23T16:50:14.317765+00:00
updated_at: 2026-02-23T16:50:14.317765+00:00
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
  - timestamp: 2026-02-23T16:50:14.317765+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Update merge spec — document codebase_paths/knowledge_refs enrichment

## Overview

Updates the merge phase spec to document the undocumented context enrichment that adds codebase_paths and knowledge_refs to merge begin/resume prompts. These are extracted from spec frontmatter and injected into the merge prompt for agent context. Addresses #477.

## Requirements

### R1 - Document codebase_paths enrichment

```yaml
id: R1
priority: medium
status: draft
```

Spec must document that merge begin/resume prompts are enriched with codebase_paths extracted from spec YAML frontmatter, giving the merge agent direct file references.

### R2 - Document knowledge_refs enrichment

```yaml
id: R2
priority: medium
status: draft
```

Spec must document that merge begin/resume prompts are enriched with knowledge_refs extracted from spec YAML frontmatter, giving the merge agent knowledge base references.

## Acceptance Criteria

### Scenario: Merge prompt enrichment

- **GIVEN** Specs have codebase_paths and knowledge_refs in frontmatter
- **WHEN** begin_merge or resume_merge executes
- **THEN** Merge prompt includes extracted codebase_paths and knowledge_refs for agent context

</spec>
