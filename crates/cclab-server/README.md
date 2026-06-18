# cclab-server

## Brief

Cclab Server is the local HTTP dashboard and plan-viewer service for registered
cclab projects.

It owns the `cc server ...` lifecycle commands, the persisted
`~/.cclab/registry.json` project registry, the Axum dashboard and plan-viewer
routes, and the per-project Lens handler pool. The legacy MCP transport/tool
surface has been removed from the active server contract; Agentic Workflow and
SDD behavior is now routed through CLI commands instead.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Server CLI And Project Registry | - | implemented | failing | smoke | not_ready | configured full gate currently fails in registry test due home registry write permission |
| Dashboard Plan Viewer And Lens Pool | - | implemented | planned | smoke | not_ready | Lens pool integration passes; live HTTP route smoke still needs a direct gate |

### Server CLI And Project Registry

ID: server-cli-and-project-registry
Type: Service
Surfaces: CLI: `cc server start`, `cc server ensure`, `cc server register`, `cc server unregister`, `cc server list`, `cc server view`, `cc server shutdown`; Registry: `~/.cclab/registry.json`
EC Dimensions: behavior: `cargo test -p cclab-server`
Root WI: -
Status: blocked
Required Verification: smoke
Promise:
Cclab Server provides local CLI lifecycle commands and a persistent project registry for starting or ensuring the server, registering projects, listing status, opening plan-viewer URLs, and shutting down the daemonized server process.
Gate Inventory: `cargo test -p cclab-server`; crates/cclab-server/src/cli.rs; crates/cclab-server/src/registry.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Server CLI lifecycle and registry persistence contract | epic | - | implemented | failing | smoke | `cargo test -p cclab-server`; crates/cclab-server/src/cli.rs; crates/cclab-server/src/registry.rs |

### Dashboard Plan Viewer And Lens Pool

ID: dashboard-plan-viewer-and-lens-pool
Type: Service
Surfaces: HTTP UI/API: `/`, `/api/dashboard`, `/view/{project}`, `/view/{project}/{change}`, `/health`; Rust API: `start_server`, `UnifiedAppState`, `LensHandlerPool`
EC Dimensions: behavior: `cargo test -p cclab-server --test unified_server`
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cclab Server hosts a local Axum dashboard and plan-viewer server for registered projects, with health checks, project/change listing APIs, file viewers, and Lens handler pooling that keeps per-project analysis state isolated and supports unsaved document overrides.
Gate Inventory: `cargo test -p cclab-server --test unified_server`; crates/cclab-server/src/http_server.rs; crates/cclab-server/src/lens_pool.rs; crates/cclab-server/tests/unified_server.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Lens handler pool and document override contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-server --test unified_server`; crates/cclab-server/src/http_server.rs; crates/cclab-server/src/lens_pool.rs; crates/cclab-server/tests/unified_server.rs |
| Dashboard and plan-viewer HTTP route smoke | epic | - | implemented | planned | smoke | crates/cclab-server/src/http_server.rs |

## Overview

cclab-server combines:

- **Server CLI**: start, ensure, register, unregister, list, view, and shutdown.
- **Project registry**: persisted server and project state in `~/.cclab`.
- **Dashboard UI/API**: project and change listing for registered projects.
- **Plan Viewer UI**: change and project file views.
- **Lens pool**: per-project analysis handlers with unsaved document overrides.

## Architecture

```
cc server start --port 3456
       │
       ▼
┌──────────────────────────────────────────────┐
│         Unified HTTP Server (Axum)           │
├──────────────────────────────────────────────┤
│  /              Dashboard (all projects)     │
│  /api/dashboard Dashboard JSON API           │
│  /view/*        Plan Viewer UI               │
│  /health        Health check                 │
└──────────────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│     Registry (~/.cclab/registry.json)        │
└──────────────────────────────────────────────┘
```

## Usage

### Start Server

```bash
cc server start --port 3456

# Start as daemon
cc server start --daemon

# Start and update supported client configuration files
cc server start --update-clients

# Ensure a daemon is running
cc server ensure --port 3456
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

## Retired MCP Surface

The old server-hosted SDD/Lens MCP tool surface is no longer part of
cclab-server. Agentic Workflow operations should use the active CLI surfaces
instead of an HTTP MCP endpoint.

## Related Crates

- **agentic-workflow**: workflow CLI and UI support used by the server.
- **cclab-cli**: top-level CLI command registration.
