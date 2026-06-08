---
id: quasar-pyo3-bindings
type: proposal
version: 1
created_at: 2026-02-01T10:29:10.259687+00:00
updated_at: 2026-02-01T10:29:10.259687+00:00
author: mcp
status: proposed
iteration: 1
summary: "Reorganize cclab-quasar PyO3 bindings and add comprehensive tests"
history:
  - timestamp: 2026-02-01T10:29:10.259687+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-02-01T10:30:44.010916+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-02-01T10:30:54.827941+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 12
  new_files: 6
affected_specs:
  - id: quasar-pyo3-bindings-spec
    path: specs/quasar-pyo3-bindings-spec.md
    depends: []---

<proposal>

# Change: quasar-pyo3-bindings

## Summary

Reorganize cclab-quasar PyO3 bindings and add comprehensive tests

## Why

Consolidating PyO3 bindings improves codebase maintainability and provides a cleaner separation between core Rust logic and Python integration. Comprehensive tests are needed to ensure the robustness of the bridge, and fixing existing failures is critical for production readiness.

## What Changes

- Create src/pyo3_bindings/ module to consolidate all PyO3 integration code
- Extract conversion logic between Rust and Python types into a dedicated conversions module
- Move PythonHandler and PyWebSocket to the new module structure
- Implement comprehensive integration tests in crates/cclab-quasar/tests/
- Fix 4 failing PyO3 tests in python_handler.rs by adding proper Python initialization

## Impact

- **Scope**: minor
- **Affected Files**: ~12
- **New Files**: ~6
- Affected specs:
  - `quasar-pyo3-bindings-spec` (no dependencies)
- Affected code: `crates/cclab-quasar/src/lib.rs`, `crates/cclab-quasar/src/python_handler.rs`, `crates/cclab-quasar/src/websocket.rs`, `crates/cclab-quasar/src/pyo3_bindings/`, `crates/cclab-quasar/tests/`
- **Breaking Changes**: Internal module reorganization; PythonHandler and PyWebSocket moved to pyo3_bindings module. Re-exports in lib.rs will maintain top-level API compatibility but module-path imports will break.

</proposal>
