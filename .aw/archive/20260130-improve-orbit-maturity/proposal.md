---
id: improve-orbit-maturity
type: proposal
version: 1
created_at: 2026-01-28T07:21:14.780667+00:00
updated_at: 2026-01-28T07:21:14.780667+00:00
author: mcp
status: proposed
iteration: 1
summary: "Upgrade cclab-orbit to 95% maturity by adding UDP support, Named Pipes, and Zero-Copy APIs."
history:
  - timestamp: 2026-01-28T07:21:14.780667+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-28T07:21:27.056608+00:00
    agent: "gemini-3-flash-preview"
    tool: "create_proposal"
    action: "created"
    duration_secs: 321.21
  - timestamp: 2026-01-28T07:22:19.315649+00:00
    agent: "gemini-3-flash-preview"
    tool: "review_proposal"
    action: "reviewed"
    duration_secs: 52.25
  - timestamp: 2026-01-28T07:27:44.179020+00:00
    agent: "gemini-3-flash-preview"
    tool: "revise_proposal"
    action: "revised"
    duration_secs: 324.86
  - timestamp: 2026-01-28T07:29:05.544671+00:00
    agent: "gemini-3-flash-preview"
    tool: "review_proposal"
    action: "reviewed"
    duration_secs: 81.36
impact:
  scope: major
  affected_files: 10
  new_files: 0
affected_specs:
  - id: orbit-udp-support
    path: specs/orbit-udp-support.md
    depends: []
  - id: orbit-named-pipes
    path: specs/orbit-named-pipes.md
    depends: []
  - id: orbit-zero-copy-apis
    path: specs/orbit-zero-copy-apis.md
    depends: []
  - id: orbit-documentation
    path: specs/orbit-documentation.md
    depends: []---

<proposal>

# Change: improve-orbit-maturity

## Summary

Upgrade cclab-orbit to 95% maturity by adding UDP support, Named Pipes, and Zero-Copy APIs.

## Why

cclab-orbit is currently at 75% maturity, lacking critical networking features (UDP, Named Pipes) and performance optimizations (Zero-Copy) required for a full uvloop alternative. This upgrade will bring it to 95% maturity, making it suitable for production high-performance Python applications.

## What Changes

- Add UDP support (UdpTransport, create_datagram_endpoint) to cclab-orbit.
- Add Named Pipes support for Windows and Unix.
- Implement Zero-Copy APIs (sendfile) for high-performance networking.
- Enhance testing with Windows IOCP verification and high-concurrency stress tests.
- Add technical documentation for bridge internals and performance tuning.

## Impact

- **Scope**: major
- **Affected Files**: ~10
- **New Files**: ~0
- Affected specs:
  - `orbit-udp-support` (no dependencies)
  - `orbit-named-pipes` (no dependencies)
  - `orbit-zero-copy-apis` (no dependencies)
  - `orbit-documentation` (no dependencies)
- Affected code: `crates/cclab-orbit/src/network.rs`, `crates/cclab-orbit/src/loop_impl.rs`, `crates/cclab-orbit/src/lib.rs`, `crates/cclab-orbit/src/unix_socket.rs`

</proposal>

<review iteration="1" reviewer="codex" status="needs_revision">
## Summary
The proposal is well-structured and addresses a significant technical debt/maturity gap in `cclab-orbit`. The goals are clear and the impact analysis in the proposal itself is accurate regarding the affected code paths.

## Issues
1. **Consistency with Tasks**: There is a major disconnect between the `proposal.md` and `tasks.md`.
    - `proposal.md` correctly identifies affected code in `crates/cclab-orbit/src/`, but `tasks.md` targets non-existent paths like `src/logic/`.
    - `proposal.md` states `new_files: 0`, but `tasks.md` specifies `CREATE` actions for 8 new files.
2. **Spec Quality**: `orbit-documentation.md` is insufficient. It lacks concrete details on what needs to be documented and has very weak acceptance criteria ("files are clearly visible").
3. **Implementation Pattern**: The task to create `orbit-documentation.rs` as a logic layer file is inappropriate for documentation. Documentation should be in Markdown files or Rust doc comments.
4. **Task Dependencies**: The dependencies in `tasks.md` seem arbitrary (e.g., UDP support depending on Named Pipes).

## Verdict
NEEDS_REVISION

## Next Steps
1. Align `tasks.md` with the actual codebase structure (`crates/cclab-orbit/src/`).
2. Update `proposal.md` to reflect that new files will indeed be created if that is the intent, or update tasks to `MODIFY` existing files.
3. Improve the depth and detail of all specs, especially `orbit-documentation.md`.
4. Ensure documentation tasks target documentation artifacts (Markdown files) rather than source code files.
</review>
