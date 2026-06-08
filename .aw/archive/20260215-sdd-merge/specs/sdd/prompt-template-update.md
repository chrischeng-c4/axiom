---
id: prompt-template-update
type: spec
title: "Prompt Template Updates"
version: 1
spec_type: utility
spec_group: sdd
created_at: 2026-02-15T03:49:16.817205+00:00
updated_at: 2026-02-15T03:49:16.817205+00:00
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Template Update Flow"
depends:
  - crate-unification
changes:
  - file: crates/cclab-sdd/src/orchestrator/prompts.rs
    action: MODIFY
  - file: docs/PROMPT_TEMPLATE_INTEGRATION.md
    action: MODIFY
history:
  - timestamp: 2026-02-15T03:49:16.817205+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Prompt Template Updates

## Overview

This spec covers the renaming of all MCP tool references in prompt templates and documentation. It ensures that the AI agents invoke the new `sdd_generate_*` tools instead of the deprecated `aurora_generate_*` tools.

## Requirements

### R1 - Rename in Code

```yaml
id: R1
priority: medium
status: draft
```

Replace all occurrences of `aurora_generate_*` with `sdd_generate_*` in the prompt template strings within `prompts.rs`.

### R2 - Update Documentation

```yaml
id: R2
priority: medium
status: draft
```

Update the prompt integration documentation to reference the new tool names.

### R3 - Cleanup Legacy References

```yaml
id: R3
priority: medium
status: draft
```

Scan the codebase for any remaining legacy tool references and fix them.

## Acceptance Criteria

### Scenario: Render Prompt

- **WHEN** rendering the `create_spec` prompt template
- **THEN** the output string contains `sdd_generate_spec` instead of `aurora_generate_spec`

### Scenario: Check Docs

- **WHEN** reading `PROMPT_TEMPLATE_INTEGRATION.md`
- **THEN** the examples show `sdd_generate_*` tool usage

## Diagrams

### Template Update Flow

```mermaid
flowchart TB
    Templates[Old Templates (aurora_*)]
    UpdatedTemplates[New Templates (sdd_*)]
    Docs[Documentation]
    UpdatedDocs[Updated Documentation]
    Templates -->|rename tools| UpdatedTemplates
    Docs -->|update refs| UpdatedDocs
```

</spec>
