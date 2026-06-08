---
id: orbit-p0-safety
type: exploration
created_at: 2026-01-31T10:49:26.601501+00:00
needs_clarification: false
---

# Codebase Exploration

# Codebase Analysis: orbit-p0-safety

## Architecture Overview
`cclab-orbit` provides a Rust-backed asyncio event loop for Python. The core is located in `crates/cclab-orbit/src/`. Key files include `lib.rs` (crate root), `loop_impl.rs` (event loop), `error.rs` (error types), and `timer_wheel.rs` (timer management).

## Safety Audit Findings
- **Unsafe Code**: Confirmed zero 'unsafe' blocks in `crates/cclab-orbit/src/`.
- **Panic Hazards**: Found 147 warnings using `prism_check`, mostly `.unwrap()` and `.expect()` calls in `unix_socket.rs`, `subprocess.rs`, `lib.rs`, and `loop_impl.rs`.
- **Thread Safety**: Core types like `PyLoop` and `Task` use thread-safe primitives, but lack compile-time verification.

## Implementation Strategy
- **File Structure**: All changes MUST be made in existing files within `crates/cclab-orbit/src/`. No new files in `src/logic/` should be created.
- **Core Safety**: Add `#![forbid(unsafe_code)]` to `lib.rs` and replace panics with `Result` types.
- **Error Handling**: Refactor `error.rs` using `thiserror` and update I/O modules (`network.rs`, `dns.rs`, `subprocess.rs`).
- **Shutdown**: Implement `shutdown_with_timeout` in `loop_impl.rs` and coordinate with `timer_wheel.rs`.

## Impact Analysis
- Affected Files: ~20 files in `crates/cclab-orbit/src/`.
- Affected Specs: `core-safety-standards`, `structured-error-handling`, `shutdown-management`.
- Breaking Changes: Minimal internal API changes.

## Spec Recommendations
- `core-safety-standards` (algorithm): Zero unsafe and panic elimination.
- `structured-error-handling` (algorithm): Consolidated error enums.
- `shutdown-management` (algorithm): Graceful shutdown lifecycle.

