---
id: lens-pdg-mcp-tools
type: spec
title: "Lens PDG MCP Tools"
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
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Standardization TDs support brownfield takeover, semantic coverage, traceability, and production readiness gates."
---

<spec>

# Lens PDG MCP Tools

## Overview
<!-- type: doc lang: markdown -->

This specification defines the Model Context Protocol (MCP) tools exposed by Lens to enable LLMs to interact with the Program Dependence Graph (PDG). It covers tool definitions, input schemas, and expected output formats for PDG visualization, slicing, impact analysis, and taint tracking.

## Requirements
<!-- type: doc lang: markdown -->

### R101 - lens_pdg Tool

```yaml
id: R101
priority: medium
status: draft
```

Expose 'lens_pdg' tool in 'crates/cclab-lens/src/mcp/tools.rs' to retrieve the PDG for a given function or file.

### R102 - lens_slice Tool

```yaml
id: R102
priority: medium
status: draft
```

Expose 'lens_slice' tool in 'crates/cclab-lens/src/mcp/tools.rs' to compute forward or backward slices from a specific statement.

### R103 - lens_impact Tool

```yaml
id: R103
priority: medium
status: draft
```

Expose 'lens_impact' tool in 'crates/cclab-lens/src/mcp/tools.rs' to analyze the impact of changes in specific lines.

### R104 - lens_taint Tool

```yaml
id: R104
priority: medium
status: draft
```

Expose 'lens_taint' tool in 'crates/cclab-lens/src/mcp/tools.rs' to trace untrusted data from sources to sensitive sinks.

### R105 - Daemon Routing

```yaml
id: R105
priority: medium
status: draft
```

Ensure all PDG tools are routed via the Argus Daemon in 'crates/cclab-lens/src/server/daemon.rs' for efficient caching and cross-file analysis.

### R106 - PDG Serialization

```yaml
id: R106
priority: medium
status: draft
```

Implement JSON serialization for PDG nodes and edges to be consumed by MCP clients.

## Acceptance Criteria
<!-- type: doc lang: markdown -->

### Scenario: Requesting a Backward Slice via MCP

- **GIVEN** A specific file and line number in a Python project.
- **WHEN** The 'lens_slice' tool is called with 'direction=backward'.
- **THEN** The tool returns a list of related code ranges and descriptions representing the backward slice.

### Scenario: Analyzing Change Impact via MCP

- **GIVEN** A list of modified files/lines.
- **WHEN** The 'lens_impact' tool is called with the list of changes.
- **THEN** The tool returns all downstream functions and modules that might be affected by these changes.

## Diagrams
<!-- type: doc lang: markdown -->

### MCP Tool Request Flow

```mermaid
flowchart LR
    LLM[LLM / User]
    ArgusMCP[Argus MCP Server]
    ArgusDaemon[Argus Daemon (Rust)]
    PDGAnalyzer[PDG Analyzer]
    LLM -->|lens_pdg(...)| ArgusMCP
    ArgusMCP -->|Route Request| ArgusDaemon
    ArgusDaemon -->|Analyze PDG| PDGAnalyzer
    PDGAnalyzer -->|JSON PDG| ArgusDaemon
    ArgusDaemon -->|JSON Response| ArgusMCP
    ArgusMCP -->|Final Output| LLM
```

</spec>
