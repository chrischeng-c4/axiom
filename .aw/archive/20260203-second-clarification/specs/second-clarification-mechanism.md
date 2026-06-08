---
id: second-clarification-mechanism
type: spec
title: "Second Clarification Mechanism"
version: 1
spec_type: utility
created_at: 2026-01-27T03:20:08.134765+00:00
updated_at: 2026-01-27T03:20:08.134765+00:00
requirements:
  total: 5
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-01-27T03:20:08.134765+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Second Clarification Mechanism

## Overview

This spec defines the technical mechanism for a second clarification phase in the Genesis workflow. It introduces a new state, a tool for appending clarifications to existing files, and logic to handle the transition from exploration to this new clarification phase.

## Requirements

### R1 - New State Phase

```yaml
id: R1
priority: high
status: draft
```

A new StatePhase::NeedsSecondClarification must be added to track the state when exploration requires more info.

### R2 - Conditional Transition from Exploration

```yaml
id: R2
priority: high
status: draft
```

The exploration tool must detect when more info is needed and update the state to NeedsSecondClarification.

### R3 - Append Clarifications Tool

```yaml
id: R3
priority: high
status: draft
```

A new MCP tool 'genesis_append_clarifications' must be implemented to append Q&A to clarifications.md.

### R4 - Clarification Phase Markers

```yaml
id: R4
priority: medium
status: draft
```

Appended clarifications must include a phase marker (e.g., 'Phase: Post-Exploration') to distinguish from initial clarifications.

### R5 - Status Display Update

```yaml
id: R5
priority: medium
status: draft
```

The CLI status command must correctly display the new phase with a yellow color and a '❓' icon.

## Acceptance Criteria

### Scenario: Exploration requires clarification

- **GIVEN** A change in 'exploring' phase
- **WHEN** The exploration tool is called with needs_clarification: true.
- **THEN** The STATE.yaml phase is updated to 'needs_second_clarification' and exploration.md is written with needs_clarification: true.

### Scenario: User provides follow-up decisions

- **GIVEN** A change in 'needs_second_clarification' phase with existing clarifications.md
- **WHEN** The append_clarifications tool is called with new questions.
- **THEN** The clarifications.md file is updated with the new Q&A under a 'Post-Exploration' header, and the phase is updated to 'exploring' (to allow re-exploration with new info).

</spec>
