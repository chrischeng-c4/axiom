---
id: prism-pdg-mcp-tools
type: spec
title: "Prism PDG MCP Tools"
version: 1
spec_type: utility
created_at: 2026-01-31T02:58:51.273693+00:00
updated_at: 2026-01-31T02:58:51.273693+00:00
requirements:
  total: 6
  ids:
    - R101
    - R102
    - R103
    - R104
    - R105
    - R106
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "MCP Tool Request Flow"
history:
  - timestamp: 2026-01-31T02:58:51.273693+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Prism PDG MCP Tools

## Overview

This specification defines the Model Context Protocol (MCP) tools exposed by Prism to enable LLMs to interact with the Program Dependence Graph (PDG). It covers tool definitions, input schemas, and expected output formats for PDG visualization, slicing, impact analysis, and taint tracking.

## Requirements

### R101 - prism_pdg Tool

```yaml
id: R101
priority: medium
status: draft
```

Expose 'prism_pdg' tool in 'crates/cclab-prism/src/mcp/tools.rs' to retrieve the PDG for a given function or file.

### R102 - prism_slice Tool

```yaml
id: R102
priority: medium
status: draft
```

Expose 'prism_slice' tool in 'crates/cclab-prism/src/mcp/tools.rs' to compute forward or backward slices from a specific statement.

### R103 - prism_impact Tool

```yaml
id: R103
priority: medium
status: draft
```

Expose 'prism_impact' tool in 'crates/cclab-prism/src/mcp/tools.rs' to analyze the impact of changes in specific lines.

### R104 - prism_taint Tool

```yaml
id: R104
priority: medium
status: draft
```

Expose 'prism_taint' tool in 'crates/cclab-prism/src/mcp/tools.rs' to trace untrusted data from sources to sensitive sinks.

### R105 - Daemon Routing

```yaml
id: R105
priority: medium
status: draft
```

Ensure all PDG tools are routed via the Argus Daemon in 'crates/cclab-prism/src/server/daemon.rs' for efficient caching and cross-file analysis.

### R106 - PDG Serialization

```yaml
id: R106
priority: medium
status: draft
```

Implement JSON serialization for PDG nodes and edges to be consumed by MCP clients.

## Acceptance Criteria

### Scenario: Requesting a Backward Slice via MCP

- **GIVEN** A specific file and line number in a Python project.
- **WHEN** The 'prism_slice' tool is called with 'direction=backward'.
- **THEN** The tool returns a list of related code ranges and descriptions representing the backward slice.

### Scenario: Analyzing Change Impact via MCP

- **GIVEN** A list of modified files/lines.
- **WHEN** The 'prism_impact' tool is called with the list of changes.
- **THEN** The tool returns all downstream functions and modules that might be affected by these changes.

## Diagrams

### MCP Tool Request Flow

```mermaid
flowchart LR
    LLM[LLM / User]
    ArgusMCP[Argus MCP Server]
    ArgusDaemon[Argus Daemon (Rust)]
    PDGAnalyzer[PDG Analyzer]
    LLM -->|prism_pdg(...)| ArgusMCP
    ArgusMCP -->|Route Request| ArgusDaemon
    ArgusDaemon -->|Analyze PDG| PDGAnalyzer
    PDGAnalyzer -->|JSON PDG| ArgusDaemon
    ArgusDaemon -->|JSON Response| ArgusMCP
    ArgusMCP -->|Final Output| LLM
```

</spec>
