---
change: mamba-all-support
group: all-support
date: 2026-04-10
written_by: artifact_cli
review_verdict: APPROVED
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| module | crates/mamba/runtime | high | R1, R2, R3, R4, R5 |
| symbols | crates/mamba/runtime | high | R1, R2, R3 |
| hir-to-mir | crates/mamba/lower | high | R4 |
| statements | crates/mamba/parser | medium | R1 |
| name-resolution | crates/mamba/resolve | medium | R5 |

## Spec Plan

| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| mamba-all-support-spec | modify | crates/mamba/runtime/module.md | overview, changes, logic, test-plan |

# Reviews

## Review: reviewer (Iteration 1)

**Change ID**: mamba-all-support

**Verdict**: APPROVED

### Summary

Reference context covers all affected areas for __all__ support: runtime/module.md (mb_import_star + __all__ preservation), runtime/symbols.md (symbol registration), lower/hir-to-mir.md (star-import lowering), parser/statements.md (from-import parsing), resolve/name-resolution.md (star-import resolve pass). Spec plan correctly targets the existing module.md which already has the full change spec written.

### Issues

No issues found.
