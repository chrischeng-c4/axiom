---
id: orbit-p0-safety
type: proposal
version: 1
created_at: 2026-01-31T10:50:00.303191+00:00
updated_at: 2026-01-31T10:50:00.303191+00:00
author: mcp
status: proposed
iteration: 1
summary: "Implement P0 Safety requirements for cclab-orbit core Rust code, focusing on panic elimination and structured error handling."
history:
  - timestamp: 2026-01-31T10:50:00.303191+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-31T10:52:02.376475+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-01-31T10:52:26.002167+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 20
  new_files: 0
affected_specs:
  - id: core-safety-standards
    path: specs/core-safety-standards.md
    depends: []
  - id: structured-error-handling
    path: specs/structured-error-handling.md
    depends: [core-safety-standards]
  - id: shutdown-management
    path: specs/shutdown-management.md
    depends: [structured-error-handling]---

<proposal>

# Change: orbit-p0-safety

## Summary

Implement P0 Safety requirements for cclab-orbit core Rust code, focusing on panic elimination and structured error handling.

## Why

cclab-orbit currently contains 140+ potential panic points (.unwrap()) and inconsistent error handling. Strengthening the Rust core safety is essential for production reliability, preventing race conditions, and ensuring proper resource management.

## What Changes

- Eliminate all manual unwrap() and expect() calls in favor of structured error handling (140+ instances).
- Confirm and enforce zero 'unsafe' blocks in crates/cclab-orbit/src/ with #[forbid(unsafe_code)].
- Add static assertions for Send and Sync on all public structs and enums to guarantee thread safety.
- Refactor manual io::Error usage to a centralized PyLoopError enum using the thiserror crate.
- Implement a graceful shutdown mechanism with configurable timeout for PyLoop and TimerWheel.

## Impact

- **Scope**: minor
- **Affected Files**: ~20
- **New Files**: ~0
- Affected specs:
  - `core-safety-standards` (no dependencies)
  - `structured-error-handling` → depends on: `core-safety-standards`
  - `shutdown-management` → depends on: `structured-error-handling`
- Affected code: `crates/cclab-orbit/src/lib.rs`, `crates/cclab-orbit/src/error.rs`, `crates/cclab-orbit/src/loop_impl.rs`, `crates/cclab-orbit/src/task.rs`, `crates/cclab-orbit/src/network.rs`, `crates/cclab-orbit/src/subprocess.rs`, `crates/cclab-orbit/src/dns.rs`, `crates/cclab-orbit/src/tls.rs`, `crates/cclab-orbit/src/timer_wheel.rs`, `crates/cclab-orbit/src/unix_socket.rs`, `crates/cclab-orbit/src/udp.rs`
- **Breaking Changes**: Minimal. Internal Rust error types will change, but the public Python API for shutdown may have slight changes in behavior.

</proposal>
