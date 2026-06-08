---
id: main-spec-integration
type: spec
title: "Main Spec Integration"
version: 1
spec_type: utility
target_crate: cclab-genesis
created_at: 2026-02-02T10:29:44.605758+00:00
updated_at: 2026-02-02T10:29:44.605758+00:00
requirements:
  total: 6
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
design_elements:
  has_mermaid: true
  has_json_schema: true
  has_pseudo_code: false
  has_api_spec: true
  has_semantic_diagrams: false
  api_spec_type: openrpc-1.3
  diagrams:
    - type: flowchart
      title: "Main Spec Integration Flow"
history:
  - timestamp: 2026-02-02T10:29:44.605758+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
  - timestamp: 2026-02-02T10:31:45.689014+00:00
    agent: "gemini:pro"
    tool: "revise_spec"
    action: "revised"
  - timestamp: 2026-02-02T10:32:03.204554+00:00
    agent: "codex:max"
    tool: "review_spec"
    action: "reviewed"---

<spec>

# Main Spec Integration

## Overview

This spec defines the integration of main spec awareness into the Genesis planning workflow. It introduces new MCP tools to list and read specs from the `cclab/specs/` directory, and updates the `SpecFrontmatter` to support linking new specs to existing main specs. This enables the planning agent to understand the existing system state ("source of truth") before proposing changes, reducing duplication and ensuring consistency.

## Requirements

### R1 - List Main Specs Tool

```yaml
id: R1
priority: high
status: draft
```

The system must provide an MCP tool `list_main_specs` to list files in `cclab/specs/` directory.

### R2 - Read Main Spec Tool

```yaml
id: R2
priority: high
status: draft
```

The system must provide an MCP tool `read_main_spec` to read the content of a spec file from `cclab/specs/`.

### R3 - Spec Group Field

```yaml
id: R3
priority: medium
status: draft
```

The `SpecFrontmatter` must support a `spec_group` field to categorize specs (e.g., "auth", "payment").

### R4 - Main Spec Reference Field

```yaml
id: R4
priority: medium
status: draft
```

The `SpecFrontmatter` must support a `main_spec_ref` field to link a change spec to an existing main spec.

### R5 - Merge Strategy Field

```yaml
id: R5
priority: medium
status: draft
```

The `SpecFrontmatter` must support a `merge_strategy` field to define how the spec should be merged back to the main spec (e.g., "replace", "append", "patch").

### R6 - Update Planning Prompts

```yaml
id: R6
priority: high
status: draft
```

The planning prompts must be updated to instruct agents to use the new tools to check existing specs.

## Acceptance Criteria

### Scenario: List main specs

- **WHEN** The `list_main_specs` tool is called
- **THEN** A list of spec files in `cclab/specs/` is returned

### Scenario: Read main spec

- **WHEN** The `read_main_spec` tool is called with a valid file path
- **THEN** The content of the spec file is returned

### Scenario: Link to main spec

- **WHEN** A new spec is created with `main_spec_ref` set to "auth-spec"
- **THEN** The spec file contains `main_spec_ref: "auth-spec"` in frontmatter

## Diagrams

### Main Spec Integration Flow

```mermaid
flowchart TB
    Planner[Planning Agent]
    MainSpecs[(Main Specs (cclab/specs))]
    Proposal[Proposal (cclab/changes)]
    Planner -->|Lists specs| MainSpecs
    Planner -->|Reads spec| MainSpecs
    Planner -->|Creates proposal| Proposal
```

## Data Model

```json
{
  "properties": {
    "main_spec_ref": {
      "description": "ID or path of the existing main spec this spec modifies",
      "type": "string"
    },
    "merge_strategy": {
      "description": "Strategy for merging this spec into the main spec",
      "enum": [
        "replace",
        "append",
        "patch"
      ],
      "type": "string"
    },
    "spec_group": {
      "description": "Category or group the spec belongs to (e.g. 'auth')",
      "type": "string"
    }
  },
  "required": [],
  "type": "object"
}
```

## API Specification (OpenRPC 1.3)

```yaml
info:
  title: Main Spec Integration Tools
  version: 1.0.0
methods:
- name: list_main_specs
  params: []
  result:
    items:
      type: string
    type: array
  summary: List files in cclab/specs/
- name: read_main_spec
  params:
  - name: path
    required: true
    schema:
      type: string
  result:
    type: string
  summary: Read content of a spec file
openrpc: 1.3.2
```

</spec>
