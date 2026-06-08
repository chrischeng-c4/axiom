---
id: clarification-tools
type: spec
title: "Clarification Tools"
version: 1
spec_type: utility
created_at: 2026-02-01T10:20:27.060510+00:00
updated_at: 2026-02-01T10:20:27.060510+00:00
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
  - timestamp: 2026-02-01T10:20:27.060510+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Clarification Tools

## Overview

This spec defines the new post_comment tool used by AnalystAgent to request clarifications from users on external platforms.

## Requirements

### R1 - Post Comment Tool

```yaml
id: R1
priority: high
status: draft
```

A post_comment tool must be available for GitHub, GitLab, and Jira integrations.

### R2 - Checkbox Support

```yaml
id: R2
priority: high
status: draft
```

The tool must accept a question and an optional list of checkbox options.

### R3 - Trigger Pause Status

```yaml
id: R3
priority: high
status: draft
```

The tool must return a status indicating that user input is required, triggering a session pause.

## Acceptance Criteria

### Scenario: Call github_post_comment with options

- **GIVEN** AnalystAgent with GitHub integration
- **WHEN** github_post_comment is called with question and options
- **THEN** The tool posts a markdown comment with checkboxes and returns 'user_input_required' status

</spec>
