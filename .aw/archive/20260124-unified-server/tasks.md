---
id: unified-server
change_id: unified-server
type: tasks
version: 1
created_at: 2026-01-24T08:59:22.277301+00:00
updated_at: 2026-01-24T08:59:22.277301+00:00
proposal_ref: unified-server
summary:
  total: 10
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 10
layers:
  data:
    task_count: 1
    estimated_files: 0
  logic:
    task_count: 2
    estimated_files: 0
  integration:
    task_count: 6
    estimated_files: 1
  testing:
    task_count: 1
    estimated_files: 1
history:
  - timestamp: 2026-01-24T08:59:22.277301+00:00
    agent: "mcp"
    tool: "create_tasks"
    action: "created"
  - timestamp: 2026-01-24T08:59:28.842949+00:00
    agent: "gemini-3-flash-preview"
    tool: "create_tasks"
    action: "created"
    duration_secs: 106.30
  - timestamp: 2026-01-24T09:00:04.104134+00:00
    agent: "gpt-5.2-codex"
    tool: "review_tasks"
    action: "reviewed"
    duration_secs: 35.26---

<tasks>

# Implementation Tasks

## Overview

This document outlines 10 implementation tasks for change `unified-server`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Data Layer | 1 | 🔲 Pending |
| Logic Layer | 2 | 🔲 Pending |
| Integration Layer | 6 | 🔲 Pending |
| Testing Layer | 1 | 🔲 Pending |

## 1. Data Layer

### Task 1.1: Add LSP port to config

```yaml
id: 1.1
action: MODIFY
status: pending
file: crates/cclab-prism/src/core/config.rs
spec_ref: unified-server-architecture:R5
```

Add lsp_port to ArgusSettings and default to 5007.

## 2. Logic Layer

### Task 2.1: Refactor RequestHandler for memory overrides

```yaml
id: 2.1
action: MODIFY
status: pending
file: crates/cclab-prism/src/server/handler.rs
spec_ref: unified-server-architecture:R1
```

Add support for in-memory document overrides to RequestHandler to allow LSP clients to provide unsaved content.

### Task 2.2: Modularize ArgusServer

```yaml
id: 2.2
action: MODIFY
status: pending
file: crates/cclab-prism/src/lsp/server.rs
spec_ref: unified-server-architecture:R1
depends_on: [2.1]
```

Refactor ArgusServer to accept an external RequestHandler and allow shared state. Ensure it can be instantiated without taking ownership of stdin/stdout directly if needed for TCP.

## 3. Integration Layer

### Task 3.1: Update PrismHandlerPool to host engines

```yaml
id: 3.1
action: MODIFY
status: pending
file: crates/cclab-server/src/prism_pool.rs
spec_ref: unified-server-architecture:R2
depends_on: [2.1]
```

Change the pool to store Arc<RequestHandler> instead of DaemonClient. Update get_handler to initialize RequestHandler directly.

### Task 3.2: Implement UnifiedLspRouter

```yaml
id: 3.2
action: CREATE
status: pending
file: crates/cclab-server/src/lsp/mod.rs
spec_ref: unified-server-architecture:R3
depends_on: [3.1, 2.2]
```

Create a new module to handle multi-project LSP routing. It should listen on TCP and delegate to per-project ArgusServer instances.

### Task 3.3: Expose LSP module in cclab-server lib

```yaml
id: 3.3
action: MODIFY
status: pending
file: crates/cclab-server/src/lib.rs
spec_ref: unified-server-architecture:R3
depends_on: [3.2]
```

Update lib.rs to include and expose the new lsp module.

### Task 3.4: Update UnifiedMcpRouter for local calls

```yaml
id: 3.4
action: MODIFY
status: pending
file: crates/cclab-server/src/mcp/router.rs
spec_ref: unified-server-architecture:R4
depends_on: [3.1]
```

Update tool calls to invoke RequestHandler methods directly. Remove dependency on ArgusDaemon running as a separate process.

### Task 3.5: Integrate LSP listener in http_server

```yaml
id: 3.5
action: MODIFY
status: pending
file: crates/cclab-server/src/http_server.rs
spec_ref: unified-server-architecture:R3
depends_on: [3.2, 3.3, 3.4]
```

Start the LSP TCP listener alongside the HTTP server in start_server.

### Task 3.6: Update CLI for unified server

```yaml
id: 3.6
action: MODIFY
status: pending
file: crates/cclab-server/src/cli.rs
spec_ref: unified-server-architecture:R5
depends_on: [3.5, 1.1]
```

Add --lsp-port option to 'server start' and 'server run' commands. Ensure 'argus server' command is deprecated or integrated.

## 4. Testing Layer

### Task 4.1: Integration testing for unified server

```yaml
id: 4.1
action: CREATE
status: pending
file: crates/cclab-server/tests/unified_server.rs
spec_ref: unified-server-architecture:acceptance-criteria
depends_on: [3.6]
```

Create integration tests to verify that MCP tools and LSP requests both work correctly when served from the unified cclab-server.

</tasks>
