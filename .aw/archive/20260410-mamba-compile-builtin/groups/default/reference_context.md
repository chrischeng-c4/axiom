---
change: mamba-compile-builtin
group: default
date: 2026-04-10
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| builtins | crates/mamba/runtime | high | R5, R6 |
| value-and-rc | crates/mamba/runtime | high | R3, R4 |
| source-and-diagnostics | crates/mamba/source | high | R1, R2, R3, R4 |
| exception-chaining-spec | crates/mamba/runtime | high | R2, R4 |
| symbols | crates/mamba/runtime | medium | — |

## Spec Plan

| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| mamba-compile-builtin-runtime | modify | crates/mamba/runtime/builtins.md | overview, requirements, changes |
| mamba-compile-builtin-value-rc | modify | crates/mamba/runtime/value-and-rc.md | requirements, changes |

