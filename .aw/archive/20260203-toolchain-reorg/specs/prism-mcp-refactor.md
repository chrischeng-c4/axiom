---
id: prism-mcp-refactor
type: spec
title: "Prism MCP Refactor and Routing"
version: 1
spec_type: utility
created_at: 2026-01-28T08:23:09.387010+00:00
updated_at: 2026-01-28T08:23:09.387010+00:00
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
  - timestamp: 2026-01-28T08:23:09.387010+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Prism MCP Refactor and Routing

## Overview

This specification covers the refactoring of Prism MCP tools and their routing in cclab-server. It implements a split between 'Daemon' tools (requiring per-project analysis context) and 'Local' tools (standalone functional tools). It also implements the removal of sensitive environment configuration tools from LLM access. Key files include crates/cclab-server/src/mcp/router.rs, crates/cclab-prism/src/mcp/tools.rs, and crates/cclab-prism/src/server/handler.rs.

## Requirements

### R1 - Filter Exposed MCP Tools

```yaml
id: R1
priority: medium
status: draft
```

ArgusTools::list() in crates/cclab-prism/src/mcp/tools.rs must only include analysis, spec generation, and state machine tools. Sensitive environment tools (get_config, set_python_paths, configure_venv, detect_environment, list_modules) must be removed.

### R2 - Daemon Tool Routing

```yaml
id: R2
priority: medium
status: draft
```

UnifiedMcpRouter in crates/cclab-server/src/mcp/router.rs must route analysis tools (check, hover, etc.) to the per-project Prism daemon handler via PrismHandlerPool. Modify call_tool and list_tools accordingly.

### R3 - Local Tool Routing

```yaml
id: R3
priority: medium
status: draft
```

UnifiedMcpRouter in crates/cclab-server/src/mcp/router.rs must route standalone tools (spec generation, code to mermaid, state machines) to local handlers (e.g. in crates/cclab-prism/src/mcp/spec_handler.rs) within the server process.

### R4 - Update RequestHandler Handlers

```yaml
id: R4
priority: medium
status: draft
```

RequestHandler in crates/cclab-prism/src/server/handler.rs must be updated to only process the filtered set of inspection tools.

## Acceptance Criteria

### Scenario: List tools via UnifiedMcpRouter

- **WHEN** The client calls list_tools() on the router.
- **THEN** The returned list contains 9 analysis tools and 5 spec/state machine tools, but NO environment tools.

### Scenario: Call an analysis tool

- **WHEN** The client calls prism_check with a project_path.
- **THEN** The router resolves the project_path and forwards the request to the corresponding Prism daemon handler.

### Scenario: Call a spec generation tool

- **WHEN** The client calls prism_spec_to_mermaid.
- **THEN** The router calls cclab_prism::mcp::spec_handler directly without resolving a daemon handler.

</spec>
