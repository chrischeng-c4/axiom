---
id: mcp-session-binding
type: spec
title: "MCP Session-based Project Binding"
version: 1
spec_type: algorithm
tags: [logic]
created_at: 2026-02-24T02:58:27.722823+00:00
updated_at: 2026-02-24T02:58:27.722823+00:00
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
      title: "Project Path Resolution"
history:
  - timestamp: 2026-02-24T02:58:27.722823+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# MCP Session-based Project Binding

## Overview

Bind project_path to MCP sessions via X-Cclab-Project HTTP header. On initialize, the server reads the header and associates it with the Mcp-Session-Id. Subsequent tool calls use the session-bound project_path, with fallback to the explicit project_path argument for backwards compatibility.

## Requirements

### R1 - Read X-Cclab-Project header on initialize

```yaml
id: R1
priority: medium
status: draft
```

When the server receives an initialize JSON-RPC request, extract the X-Cclab-Project header from the HTTP request. If present, store the value as session.project_path keyed by the Mcp-Session-Id.

### R2 - Session-bound project_path resolution

```yaml
id: R2
priority: medium
status: draft
```

On tool calls, resolve project_path with priority: (1) session.project_path if bound, (2) args.project_path if provided, (3) error. Log a warning if both exist and differ.

### R3 - Backwards compatibility

```yaml
id: R3
priority: medium
status: draft
```

When no X-Cclab-Project header is sent, behavior is unchanged — project_path remains a required tool argument.

## Acceptance Criteria

### Scenario: Client with header

- **GIVEN** MCP client sends X-Cclab-Project: /path/to/project on initialize
- **WHEN** Client calls sdd_run_change without project_path arg
- **THEN** Server uses session-bound /path/to/project

### Scenario: Client without header

- **GIVEN** MCP client sends no X-Cclab-Project header
- **WHEN** Client calls sdd_run_change with project_path=/some/path
- **THEN** Server uses /some/path from args (current behavior)

### Scenario: Mismatch warning

- **GIVEN** Session bound to /project-a
- **WHEN** Tool call passes project_path=/project-b
- **THEN** Server logs warning, uses session.project_path

## Diagrams

### Project Path Resolution

```mermaid
flowchart TB
    start[Tool Call Received]
    check_session[Session has project_path?]
    use_session[Use session.project_path]
    check_args[Args has project_path?]
    use_args[Use args.project_path]
    error[Error: no project_path]
    start --> check_session
    check_session -->|yes| use_session
    check_session -->|no| check_args
    check_args -->|yes| use_args
    check_args -->|no| error
```

</spec>
