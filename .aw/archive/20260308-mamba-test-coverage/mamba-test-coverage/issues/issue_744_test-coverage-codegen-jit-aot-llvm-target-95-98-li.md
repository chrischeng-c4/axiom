---
number: 744
title: "Test coverage: Codegen (JIT/AOT/LLVM) — target 95–98% line coverage"
state: open
labels: [enhancement, P1, crate:mamba]
group: "codegen-hir-mir-coverage"
---

# #744 — Test coverage: Codegen (JIT/AOT/LLVM) — target 95–98% line coverage

## Target
Line coverage: **95–98%**

## Scope
- `src/codegen/`, `src/jit/`, `src/aot/` — code generation and compilation

## Approach
1. Unit test each codegen pattern (arithmetic, control flow, function calls, closures)
2. Test JIT compilation and execution of representative programs
3. Cover error paths: invalid IR, unsupported operations
4. AOT compilation tests for all supported targets
