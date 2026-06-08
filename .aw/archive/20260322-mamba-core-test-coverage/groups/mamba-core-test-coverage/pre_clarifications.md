---
change: mamba-core-test-coverage
group: mamba-core-test-coverage
date: 2026-03-22
status: answered
---

# Pre-Clarifications

### Q1: test-location-strategy
- **Answer**: Inline #[cfg(test)] modules co-located with source. For cross-module tests, use crates/mamba/tests/ integration tests. Both are needed.

### Q2: stdlib-test-mechanism
- **Answer**: Rust #[test] in crates/mamba/src/runtime/stdlib/{module}_mod.rs as inline #[cfg(test)] modules. Test each mb_* function directly with MbValue inputs/outputs.

### Q3: scope-boundary
- **Answer**: This change covers runtime, lower, resolve, and stdlib top-10. Parser/types/codegen/lexer are lower priority — defer to follow-up. Focus on zero-coverage modules first.

### Q4: coverage-ci-gate
- **Answer**: Out of scope. Coverage tooling (cargo-llvm-cov) setup is separate. This change just adds test files.

### Q5: xfail-resolution
- **Answer**: All XFAIL already resolved in previous changes (0 remaining). No action needed here.

