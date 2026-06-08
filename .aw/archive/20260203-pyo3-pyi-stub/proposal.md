---
id: pyo3-pyi-stub
type: proposal
version: 1
created_at: 2026-01-30T03:39:25.188566+00:00
updated_at: 2026-01-30T03:39:25.188566+00:00
author: mcp
status: proposed
iteration: 1
summary: "Add PyO3 to Python stub (.pyi) generator in Prism"
history:
  - timestamp: 2026-01-30T03:39:25.188566+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-30T03:39:46.322191+00:00
    agent: "unknown"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-30T03:40:02.256395+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
  - timestamp: 2026-01-30T03:44:49.870407+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-01-30T03:45:06.100690+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 6
  new_files: 2
affected_specs:
  - id: cclab-prism-pyo3-stub
    path: specs/cclab-prism-pyo3-stub.md
    depends: []---

<proposal>

# Change: pyo3-pyi-stub

## Summary

Add PyO3 to Python stub (.pyi) generator in Prism

## Why

Manually maintaining .pyi files for PyO3 extensions is error-prone. Automating this ensures Python type safety and better IDE support for Rust-powered Python libraries.

## What Changes

- Rename Argus to Prism in CLI for unified branding
- Implement PyO3 stub generator in crates/cclab-prism/src/gen/python/pyo3.rs
- Add tree-sitter based Rust parsing to extract #[pyclass], #[pyfunction], and #[pymethods] entities
- Implement Rust-to-Python type mapping and docstring extraction
- Add cclab prism gen-stub CLI command
- Expose prism_generate_pyo3_stub MCP tool

## Impact

- **Scope**: minor
- **Affected Files**: ~6
- **New Files**: ~2
- Affected specs:
  - `cclab-prism-pyo3-stub` (no dependencies)
- Affected code: `crates/cclab-cli/src/main.rs`, `crates/cclab-prism/src/lib.rs`, `crates/cclab-prism/src/gen/python/mod.rs`, `crates/cclab-prism/src/gen/python/pyo3.rs`, `crates/cclab-prism/src/mcp/tools.rs`, `crates/cclab-prism/src/mcp/spec_handler.rs`

</proposal>
