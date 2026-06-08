---
id: prism-rust-symbols
type: proposal
version: 1
created_at: 2026-02-06T07:49:43.410841+00:00
updated_at: 2026-02-06T07:49:43.410841+00:00
author: mcp
status: proposed
iteration: 1
summary: "Implement Rust symbol extraction in Prism to enable cross-language semantic analysis for Rust codebases."
history:
  - timestamp: 2026-02-06T07:49:43.410841+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-02-06T07:51:48.763246+00:00
    agent: "unknown"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-02-06T07:52:19.510704+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 1
  new_files: 0
affected_specs:
  - id: rust-symbol-analysis
    path: specs/rust-symbol-analysis.md
    depends: []---

<proposal>

# Change: prism-rust-symbols

## Summary

Implement Rust symbol extraction in Prism to enable cross-language semantic analysis for Rust codebases.

## Why

Currently, Prism supports Python symbol extraction but lacks Rust support. Adding this enables critical features like "Go to Definition" and "Find References" for Rust, bringing it to parity with Python and allowing unified analysis of mixed-language projects (e.g., Rust extensions for Python). This is essential for providing a complete multi-language analysis experience.

## What Changes

- Add `build_rust` method to `SymbolTableBuilder` in `crates/cclab-prism/src/semantic/symbols.rs`
- Implement AST visitor pattern for Rust language nodes (functions, structs, traits, impls)
- Add logic to extract Rust documentation comments (///) and attach to symbols
- Implement basic type signature parsing for Rust symbols to support hover information

## Impact

- **Scope**: minor
- **Affected Files**: ~1
- **New Files**: ~0
- Affected specs:
  - `rust-symbol-analysis` (no dependencies)
- Affected code: `crates/cclab-prism/src/semantic/symbols.rs`

</proposal>
