---
id: state-machine-tools
type: spec
title: "State Machine MCP Tools"
version: 1
spec_type: utility
created_at: 2026-01-28T08:22:36.210025+00:00
updated_at: 2026-01-28T08:22:36.210025+00:00
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
  - timestamp: 2026-01-28T08:22:36.210025+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# State Machine MCP Tools

## Overview

This specification covers the addition of state machine MCP tools to Prism. It exposes the existing state machine validation and Mermaid+ generation capabilities through the MCP protocol. Implementation will involve adding handlers to crates/cclab-prism/src/mcp/spec_handler.rs and exposing them in crates/cclab-prism/src/mcp/tools.rs.

## Requirements

### R1 - State Machine Validation Tool

```yaml
id: R1
priority: medium
status: draft
```

Expose a tool to validate a state machine definition (JSON) semantically using StateMachineValidator.

### R2 - State Machine Generation Tool

```yaml
id: R2
priority: medium
status: draft
```

Expose a tool to generate Mermaid+ output from a state machine definition using MermaidPlusGenerator.

### R3 - Support Complex Machines

```yaml
id: R3
priority: medium
status: draft
```

The tools must support nested states, parallel states, guards, and actions.

### R4 - Rich Validation Feedback

```yaml
id: R4
priority: medium
status: draft
```

Validation results must include clear error messages and usage locations.

## Acceptance Criteria

### Scenario: Validate a simple machine

- **WHEN** The client calls prism_validate_state_machine with a valid toggle machine JSON.
- **THEN** The tool returns a validation report indicating the machine is valid.

### Scenario: Validate an invalid machine

- **WHEN** The client calls prism_validate_state_machine with a machine missing its initial state.
- **THEN** The tool returns a validation report with errors (e.g., MISSING_INITIAL_STATE).

### Scenario: Generate Mermaid+ for a machine

- **WHEN** The client calls prism_generate_state_machine with a valid machine JSON.
- **THEN** The tool returns a string containing YAML frontmatter and a Mermaid stateDiagram-v2.

</spec>
