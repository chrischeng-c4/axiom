---
change: mamba-test-coverage
group: typechecker-coverage
date: 2026-03-08
---

# Requirements

Improve type checker coverage from 70.3% to 95-98% (gap: ~265 lines). Worst files: types/protocol.rs (50%), types/generic.rs (53.5%), types/check_expr.rs (63%), types/check.rs (69%), types/context.rs (71.7%). Add tests for all type inference paths, error diagnostics, generic instantiation, protocol conformance.
