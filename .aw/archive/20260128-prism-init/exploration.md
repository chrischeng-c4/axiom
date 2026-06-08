---
id: prism-init
type: exploration
created_at: 2026-01-27T15:06:36.768863+00:00
needs_clarification: false
---

# Codebase Exploration

### Architecture Overview
The cclab-server uses a `Registry` (~/.cclab/registry.json) to track registered projects and a `PrismHandlerPool` to manage `RequestHandler` instances for each project. `RequestHandler` is responsible for code analysis and indexing.

### Relevant Files
- `crates/cclab-server/src/registry.rs`: Manages project registration and server process info.
- `crates/cclab-server/src/prism_pool.rs`: Manages the lifecycle of Prism handlers.
- `crates/cclab-server/src/cli.rs`: Handles server lifecycle commands (start, shutdown, run).
- `crates/cclab-server/src/http_server.rs`: The main HTTP/MCP server entry point.

### Impact Analysis
- **Registry**: Needs to become more persistent. Currently, the file is deleted on shutdown or stale restart, losing all projects.
- **PrismHandlerPool**: Needs a method to initialize multiple projects in bulk.
- **Server Startup**: Needs to trigger initialization of all projects found in the registry.

### Technical Considerations
- Initialization (especially indexing) can be heavy, so it must be done in a background task to avoid blocking the HTTP server from starting.
- `Registry::load()` needs to be robust against missing server info while projects are present.

### Recommendations
- Modify `cli::start_server_process` to merge existing projects from a previous registry if available.
- Implement a background task in `http_server::start_server` that iterates over `registry.projects` and calls `prism_pool.get_handler`.
- Add a manual `check` call on the project root to trigger the indexer.
