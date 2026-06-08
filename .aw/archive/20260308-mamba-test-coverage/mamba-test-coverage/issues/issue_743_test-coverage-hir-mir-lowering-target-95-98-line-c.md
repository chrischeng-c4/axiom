---
number: 743
title: "Test coverage: HIR/MIR/Lowering — target 95–98% line coverage"
state: open
labels: [enhancement, P1, crate:mamba]
group: "codegen-hir-mir-coverage"
---

# #743 — Test coverage: HIR/MIR/Lowering — target 95–98% line coverage

## Target
Line coverage: **95–98%**

## Scope
- `src/hir/`, `src/mir/`, `src/lowering/` — AST→HIR→MIR pipeline

## Approach
1. Test each lowering transformation independently
2. Cover all AST node types and their MIR representations
3. Error paths: invalid transformations, unsupported constructs
