---
id: init-mcp-json
type: spec
title: "Generate .mcp.json with Project Header in cclab init"
version: 1
spec_type: utility
created_at: 2026-02-24T02:58:52.541177+00:00
updated_at: 2026-02-24T02:58:52.541177+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-24T02:58:52.541177+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Generate .mcp.json with Project Header in cclab init

## Overview

Update cclab init to generate a .mcp.json file with X-Cclab-Project header pointing to the project root. Add .mcp.json to .gitignore since it contains local-specific paths. Track .mcp.json.example as a template for onboarding.

## Requirements

### R1 - Generate .mcp.json with header

```yaml
id: R1
priority: medium
status: draft
```

During cclab init (fresh install), generate .mcp.json at project root with the cclab-mcp server entry including headers.X-Cclab-Project set to the current working directory.

### R2 - Add .mcp.json to .gitignore

```yaml
id: R2
priority: medium
status: draft
```

Ensure .mcp.json is in .gitignore since it contains local-specific paths and ports.

### R3 - Track .mcp.json.example

```yaml
id: R3
priority: medium
status: draft
```

Generate .mcp.json.example alongside .mcp.json as a tracked template without local-specific values, using placeholder paths.

### R4 - Update mode preservation

```yaml
id: R4
priority: medium
status: draft
```

During cclab init update mode, do not overwrite existing .mcp.json if it exists. Only create if missing.

## Acceptance Criteria

### Scenario: Fresh install generates .mcp.json

- **GIVEN** Project has no .mcp.json
- **WHEN** User runs cclab init
- **THEN** .mcp.json created with X-Cclab-Project header, .mcp.json added to .gitignore

### Scenario: Update preserves existing

- **GIVEN** Project has existing .mcp.json with custom settings
- **WHEN** User runs cclab init (update mode)
- **THEN** Existing .mcp.json is not overwritten

### Scenario: Example file tracked

- **GIVEN** Fresh install
- **WHEN** cclab init completes
- **THEN** .mcp.json.example exists with placeholder values, suitable for git tracking

</spec>
