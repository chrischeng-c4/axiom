---
id: prism-init
type: proposal
version: 1
created_at: 2026-01-27T16:49:52.211997+00:00
updated_at: 2026-01-27T16:49:52.211997+00:00
author: mcp
status: proposed
iteration: 1
summary: "Automatic initialization of Prism handlers for registered projects at server startup"
history:
  - timestamp: 2026-01-27T16:49:52.211997+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 4
  new_files: 0
affected_specs:
  - id: prism-init-spec
    path: specs/prism-init-spec.md
    depends: []
---

<proposal>

# Change: prism-init

## Summary

Automatic initialization of Prism handlers for registered projects at server startup

## Why

Improve responsiveness by pre-indexing registered projects at server startup instead of lazily on first access, which causes significant delays for the first request.

## What Changes

- Modify Registry persistence logic in crates/cclab-server/src/registry.rs to preserve registered projects across server restarts.
- Add initialize_all_projects method to PrismHandlerPool in crates/cclab-server/src/prism_pool.rs background initialization.
- Implement an auto-initialization background task in http_server::start_server in crates/cclab-server/src/http_server.rs that triggers indexing.
- Update server CLI in crates/cclab-server/src/cli.rs to support registry persistence and merging existing projects.

## Impact

- **Scope**: minor
- **Affected Files**: ~4
- **New Files**: ~0
- Affected specs:
  - `prism-init-spec` (no dependencies)

</proposal>
