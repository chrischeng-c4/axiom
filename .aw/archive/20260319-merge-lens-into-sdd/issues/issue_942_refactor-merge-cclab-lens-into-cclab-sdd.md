---
number: 942
title: "refactor: merge cclab-lens into cclab-sdd"
state: open
labels: [enhancement, P1, crate:sdd, crate:lens]
group: "merge-lens-into-sdd"
---

# #942 — refactor: merge cclab-lens into cclab-sdd

## Summary

Merge `cclab-lens` (code analysis) into `cclab-sdd`. In AI-driven development, code analysis is an internal capability of the spec-driven workflow, not a standalone IDE tool. Keeping them separate creates unnecessary boundaries (SpecIR serialization, cross-crate dependency, duplicate CLI registration).

## Rationale

- **Lens has no standalone use case**: Modern development is AI-assisted. Code analysis (semantic search, refactoring, symbol resolution) serves the SDD workflow, not human IDE users.
- **SpecIR boundary is artificial**: SDD produces SpecIR, Lens consumes it for codegen. Merging makes SpecIR an internal module interface instead of a cross-crate contract.
- **Competitors (Pyright, Ruff, JetBrains) are IDE tools**: Lens competing in that space is misaligned with cclab's AI-first direction.
- **Simplifies architecture**: One crate, one CLI registration, one MCP tool namespace.

## Scope

### Move into cclab-sdd
- `crates/cclab-lens/src/` → `crates/cclab-sdd/src/lens/` (or inline where appropriate)
- Lens MCP tools → SDD MCP tool namespace
- Lens CLI subcommands → remain under `cclab lens` (backward compat)

### Keep separate (for now)
- `cclab-lens` crate can become a thin re-export wrapper during transition
- Existing `crate:lens` issues remain valid, just implemented within cclab-sdd

## Acceptance Criteria

- [ ] All Lens functionality accessible from cclab-sdd crate
- [ ] SpecIR no longer needs cross-crate serialization
- [ ] `cclab lens` CLI commands still work
- [ ] MCP tools unified under one server handler
