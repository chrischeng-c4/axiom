---
change: merge-lens-into-sdd
group: merge-lens-into-sdd
date: 2026-03-19
---

# Requirements

Move all `cclab-lens` source code into `cclab-sdd` as a submodule under `crates/cclab-sdd/src/lens/`, eliminating the cross-crate SpecIR serialization boundary so SpecIR becomes a plain internal module interface. Make `cclab-lens` a thin re-export wrapper for backward compatibility during the transition period. Merge Lens MCP tools into the SDD MCP server handler (single unified namespace). Keep `cclab lens` CLI subcommands functional via re-exports or delegation from `cclab-sdd-cli`. All future `crate:lens` issues will be implemented within `cclab-sdd`.

Acceptance criteria:
- All Lens functionality (semantic search, symbol resolution, refactoring, codegen) accessible from `cclab-sdd` crate
- SpecIR no longer requires cross-crate serialization
- `cclab lens` CLI commands still work (backward compat)
- MCP tools unified under one server handler (no duplicate registration)
