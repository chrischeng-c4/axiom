---
id: unified-server-architecture
type: spec
title: "Unified Server Architecture"
version: 1
created_at: 2026-01-24T08:52:31.382748+00:00
updated_at: 2026-01-24T08:52:31.382748+00:00
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
history:
  - timestamp: 2026-01-24T08:52:31.382748+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Unified Server Architecture

## Overview

This specification defines the architectural changes required to merge the Argus analysis engine (Prism) into cclab-server. It covers the refactoring of RequestHandler to serve as the core engine, the integration of LSP support into the unified server, and the transition from external daemon communication to direct in-process calls.

## Requirements

### R1 - Core Analysis Engine Overrides

```yaml
id: R1
priority: medium
status: draft
```

RequestHandler must be updated to support in-memory document overrides to handle unsaved changes from LSP clients.

### R2 - In-process Engine Hosting

```yaml
id: R2
priority: medium
status: draft
```

cclab-server must host Arc<RequestHandler> instances in PrismHandlerPool and manage their lifecycle directly.

### R3 - Multi-project LSP Support

```yaml
id: R3
priority: medium
status: draft
```

A UnifiedLspRouter (or equivalent) must handle incoming LSP connections over TCP and route requests to the correct project engine based on rootUri.

### R4 - Unified MCP Integration

```yaml
id: R4
priority: medium
status: draft
```

The MCP tool implementation in cclab-server must be updated to call RequestHandler methods directly instead of using DaemonClient.

### R5 - Configurable LSP Port

```yaml
id: R5
priority: medium
status: draft
```

The server must support a configurable LSP port, defaulting to 5007, and expose it via CLI.

## Acceptance Criteria

### Scenario: MCP Tool Execution via Local Engine

- **GIVEN** cclab-server is running with a registered project.
- **WHEN** An MCP tool call (e.g., cclab_prism_check) is received for the project.
- **THEN** The tool should return results from the local RequestHandler without spawning an external process.

### Scenario: LSP Initialization and Diagnostics

- **GIVEN** cclab-server is listening on the LSP port.
- **WHEN** An LSP client connects and sends an initialize request with a valid project rootUri.
- **THEN** The server should return an InitializeResult and start publishing diagnostics for the project.

### Scenario: Multi-project Routing

- **GIVEN** An LSP client is connected and has two projects open.
- **WHEN** A textDocument/hover request is sent for file-b in the second project.
- **THEN** The server should correctly route the request to the RequestHandler corresponding to file-b's project.

</spec>
