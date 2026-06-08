---
number: 747
title: "Test coverage: Name resolution — target 95–98% line coverage"
state: open
labels: [enhancement, P2, crate:mamba]
group: "codegen-hir-mir-coverage"
---

# #747 — Test coverage: Name resolution — target 95–98% line coverage

## Target
Line coverage: **95–98%**

## Scope
- `src/resolver/` or name resolution pass — scope analysis, symbol binding

## Approach
1. Test all scope types (module, function, class, comprehension)
2. Cover import resolution, global/nonlocal declarations
3. Error paths: undefined names, conflicting definitions
