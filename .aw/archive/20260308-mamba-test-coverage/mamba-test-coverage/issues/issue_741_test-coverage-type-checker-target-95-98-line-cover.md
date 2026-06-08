---
number: 741
title: "Test coverage: Type checker — target 95–98% line coverage"
state: open
labels: [enhancement, P0, crate:mamba]
group: "typechecker-coverage"
---

# #741 — Test coverage: Type checker — target 95–98% line coverage

## Target
Line coverage: **95–98%**

## Scope
- `src/typechecker/` — type inference, generics, protocols, Any, Optional

## Approach
1. Measure per-file coverage
2. Add tests for all type inference paths, error diagnostics, edge cases
3. Cover generic instantiation, protocol conformance, union types
