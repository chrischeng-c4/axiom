---
id: prism-pyo3-codegen
type: proposal
version: 1
created_at: 2026-01-31T15:08:18.651460+00:00
updated_at: 2026-01-31T15:08:18.651460+00:00
author: mcp
status: proposed
iteration: 1
summary: "Add proc-macro-driven PyO3 binding generator and CLI entry `cclab prism gen pyo3` that emits Python wrappers under python/cclab/<crate>"
history:
  - timestamp: 2026-01-31T15:08:18.651460+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-31T15:08:22.999628+00:00
    agent: "codex:deep"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-01-31T15:08:34.349240+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 12
  new_files: 2---

<proposal>

# Change: prism-pyo3-codegen

## Summary

Add proc-macro-driven PyO3 binding generator and CLI entry `cclab prism gen pyo3` that emits Python wrappers under python/cclab/<crate>

## Why

Current PyO3 codegen assumes crates already carry PyO3 annotations, which forces manual binding upkeep and couples every crate to PyO3. Introducing a small proc-macro marker (`#[pyexport]`) lets pure Rust APIs opt into Python exposure while Prism generates a consistent PyO3 wrapper and Python package stubs, reducing duplication and drift between Rust APIs and Python bindings.

## What Changes

- Add a new proc-macro crate providing `#[pyexport]` (with optional rename/module args) to mark Rust items for export without importing PyO3 in the source crate
- Extend Prism’s Python generator to parse `#[pyexport]` from Rust sources, build a PyO3 binding IR, and emit a thin PyO3 wrapper module plus `__init__.py`/`__init__.pyi` into `python/cclab/<crate>`
- Add a new CLI entry `cclab prism gen pyo3` with options for crate path, output directory, module name, and dry-run while keeping existing commands intact
- Wire new generator exports in `cclab-prism` and add targeted tests for attribute extraction and wrapper generation

## Impact

- **Scope**: minor
- **Affected Files**: ~12
- **New Files**: ~2
- Affected code: `crates/cclab-cli/src/main.rs`, `crates/cclab-prism/src/gen/python/pyo3.rs`, `crates/cclab-prism/src/gen/python/mod.rs`, `crates/cclab-prism/src/lib.rs`, `crates/cclab-prism/tests/*`, `crates/cclab-pyo3-export/Cargo.toml`, `crates/cclab-pyo3-export/src/lib.rs`, `Cargo.toml`
- **Breaking Changes**: None; new CLI subcommand and generator are additive.

</proposal>
