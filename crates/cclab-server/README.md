# cclab-server

Unified HTTP Server for cclab

## Overview

cclab-server provides a unified HTTP server that combines:

- **SDD MCP Tools** (21 tools): Spec-driven development workflow
- **Lens MCP Tools** (26 tools): Code analysis and diagram generation
- **Dashboard UI**: Project listing and management
- **Plan Viewer UI**: Interactive change review interface

## Architecture

```
cc server start --port 3456
       │
       ▼
┌──────────────────────────────────────────────┐
│         Unified HTTP Server (Axum)           │
├──────────────────────────────────────────────┤
│  /              Dashboard (all projects)     │
│  /view/*        Plan Viewer UI               │
│  /mcp           Combined MCP (47 tools)      │
│  /health        Health check                 │
└──────────────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│     Registry (~/.cclab/registry.json)        │
└──────────────────────────────────────────────┘
```

## MCP Tools (47 total)

### SDD Tools (21)
Workflow management, file operations, knowledge base.

### Lens Tools (26)
| Category | Count | Tools |
|----------|-------|-------|
| Analysis | 9 | check, type_at, symbols, diagnostics, hover, definition, references, index_status, invalidate |
| Spec Generation | 3 | generate_from_spec, spec_to_mermaid, code_to_mermaid |
| State Machine | 2 | validate_state_machine, generate_state_machine |
| Mermaid Diagrams | 8 | flowchart, sequence, class, state, erd, mindmap, requirement, journey |
| API Specs | 4 | openapi, asyncapi, openrpc, serverless_workflow |

## Usage

### Start Server

```bash
# Start in foreground
cc server start --port 3456

# Start as daemon
cc server start --daemon

# Start with auto-open browser
cc server start --open
```

### Project Management

```bash
# Register project
cc server register /path/to/project

# List projects
cc server list

# Unregister project
cc server unregister myproject
```

### View Changes

```bash
# Open change in viewer
cc server view myproject change-1

# Open dashboard
cc server dashboard
```

### Shutdown

```bash
cc server shutdown
```

## Configuration

### Registry

Projects are stored in `~/.cclab/registry.json`:

```json
{
  "projects": {
    "myproject": {
      "path": "/path/to/myproject",
      "registered_at": "2026-01-28T10:00:00Z"
    }
  }
}
```

### MCP Client Configuration

For Claude Code:
```json
{
  "mcpServers": {
    "cclab": {
      "url": "http://localhost:3456/mcp"
    }
  }
}
```

## Stage-Specific Tool Filtering

Different workflow stages can use filtered tool sets:

```bash
# Implement stage (4 tools)
cc server start --tools implement

# Review stage (3 tools)
cc server start --tools review

# All tools (default)
cc server start
```

## Related Crates

- **sdd**: Workflow tools
- **cclab-lens**: Analysis and diagram tools
- **cclab-cli**: CLI commands
