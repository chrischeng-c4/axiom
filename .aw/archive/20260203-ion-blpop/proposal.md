---
id: ion-blpop
type: proposal
version: 1
created_at: 2026-01-31T10:54:45.330148+00:00
updated_at: 2026-01-31T10:54:45.330148+00:00
author: mcp
status: proposed
iteration: 1
summary: "Implement BLPOP/BRPOP blocking list operations for efficient task queue consumption."
history:
  - timestamp: 2026-01-31T10:54:45.330148+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-31T10:56:39.195147+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-01-31T10:56:47.646371+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 4
  new_files: 0
affected_specs:
  - id: blocking-lists
    path: specs/blocking-lists.md
    depends: []---

<proposal>

# Change: ion-blpop

## Summary

Implement BLPOP/BRPOP blocking list operations for efficient task queue consumption.

## Why

Blocking list operations (BLPOP/BRPOP) allow workers to wait for tasks without busy-polling, significantly reducing latency and CPU overhead in task queue implementations.

## What Changes

- Add LPUSH, RPUSH, LPOP, RPOP, BLPOP, BRPOP commands to binary protocol.
- Implement WaiterManager in server using tokio::sync::Notify to manage suspended clients.
- Transition server request processing to async to handle blocking operations without consuming threads.
- Provide async client methods for all new list operations.

## Impact

- **Scope**: minor
- **Affected Files**: ~4
- **New Files**: ~0
- Affected specs:
  - `blocking-lists` (no dependencies)
- Affected code: `crates/cclab-ion-server/src/protocol.rs`, `crates/cclab-ion-server/src/server.rs`, `crates/cclab-ion-client/src/protocol.rs`, `crates/cclab-ion-client/src/client.rs`
- **Breaking Changes**: Protocol update to support blocking commands; server request processing becomes async.

</proposal>
