---
id: orbit-pipes-zerocopy
type: proposal
version: 1
created_at: 2026-02-05T08:52:01.235678+00:00
updated_at: 2026-02-05T08:52:01.235678+00:00
author: mcp
status: proposed
iteration: 1
summary: "Add cross-platform pipes, zero-copy I/O, protocol lifecycle, and comprehensive testing to cclab-orbit"
history:
  - timestamp: 2026-02-05T08:52:01.235678+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 15
  new_files: 12
affected_specs:
  - id: unix-pipes
    path: specs/unix-pipes.md
    depends: []
  - id: windows-pipes
    path: specs/windows-pipes.md
    depends: []
  - id: pipe-abstraction
    path: specs/pipe-abstraction.md
    depends: [unix-pipes, windows-pipes]
  - id: protocol-lifecycle
    path: specs/protocol-lifecycle.md
    depends: []
  - id: buffer-pool
    path: specs/buffer-pool.md
    depends: []
  - id: zero-copy-io
    path: specs/zero-copy-io.md
    depends: [buffer-pool]
  - id: integration-tests
    path: specs/integration-tests.md
    depends: [pipe-abstraction, protocol-lifecycle]
  - id: benchmarks
    path: specs/benchmarks.md
    depends: [zero-copy-io]
  - id: stress-tests
    path: specs/stress-tests.md
    depends: [integration-tests]
---

<proposal>

# Change: orbit-pipes-zerocopy

## Summary

Add cross-platform pipes, zero-copy I/O, protocol lifecycle, and comprehensive testing to cclab-orbit

## Why

The cclab-orbit event loop needs extended IPC capabilities (pipes) for subprocess communication, zero-copy I/O for high-performance data transfer, proper protocol lifecycle management for resource cleanup, and comprehensive testing infrastructure to ensure reliability. These features address GitHub issues #62-67 and #69-71, completing the core functionality needed for production use.

## What Changes

- Add Unix FIFO pipe support (mkfifo, open, read/write) with async I/O
- Add Windows named pipe support (CreateNamedPipe, ConnectNamedPipe) with async I/O
- Create cross-platform PipeTransport abstraction unifying Unix and Windows APIs
- Implement protocol lifecycle management with async init, graceful shutdown, and timeout handling
- Add buffer pool for reusable I/O buffers with configurable sizes and limits
- Implement zero-copy send/recv APIs using splice/sendfile on Linux, TransmitFile on Windows
- Create integration test framework for event loop functionality
- Add performance benchmarks using criterion for regression tracking
- Implement stress tests for high-concurrency scenarios

## Impact

- **Scope**: minor
- **Affected Files**: ~15
- **New Files**: ~12
- Affected specs:
  - `unix-pipes` (no dependencies)
  - `windows-pipes` (no dependencies)
  - `pipe-abstraction` → depends on: `unix-pipes`, `windows-pipes`
  - `protocol-lifecycle` (no dependencies)
  - `buffer-pool` (no dependencies)
  - `zero-copy-io` → depends on: `buffer-pool`
  - `integration-tests` → depends on: `pipe-abstraction`, `protocol-lifecycle`
  - `benchmarks` → depends on: `zero-copy-io`
  - `stress-tests` → depends on: `integration-tests`
- Affected code: `crates/cclab-orbit/src/pipe/mod.rs`, `crates/cclab-orbit/src/pipe/unix.rs`, `crates/cclab-orbit/src/pipe/windows.rs`, `crates/cclab-orbit/src/protocol.rs`, `crates/cclab-orbit/src/buffer_pool.rs`, `crates/cclab-orbit/src/zero_copy.rs`, `crates/cclab-orbit/tests/integration/`, `crates/cclab-orbit/benches/`

</proposal>
