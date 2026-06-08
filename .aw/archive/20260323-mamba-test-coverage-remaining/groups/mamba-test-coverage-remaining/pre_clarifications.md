---
change: mamba-test-coverage-remaining
group: mamba-test-coverage-remaining
date: 2026-03-23
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: Both: inline #[cfg(test)] unit tests at the bottom of each source file for per-function branch coverage, AND integration tests in crates/mamba/tests/ for cross-module scenarios. Same pattern as round 1.

### Q2: General
- **Answer**: Use real loopback socket (127.0.0.1) for socket tests. No mocks. Tests must be self-contained — bind to port 0 (OS-assigned) to avoid port conflicts.

### Q3: General
- **Answer**: Test all branches: thread lifecycle, synchronization primitives, and thread-local storage. For race-condition branches, use deterministic synchronization (barriers, channels) to force specific orderings. No branches excluded — test everything.

### Q4: General
- **Answer**: Read the source to determine which case (a/b/c). All types are Rust-side definitions. Add inline tests for every type conversion, size assertion, and alignment check. No C library required for testing the type definitions themselves.

### Q5: General
- **Answer**: Use both: unit tests with string inputs (compile_str-style helpers if they exist, otherwise create minimal ones) for branch coverage, and integration tests with real .py source files for end-to-end paths.

### Q6: General
- **Answer**: Read existing codegen test patterns first. Use whatever fixtures exist (MIR graphs or synthetic IR). If existing tests use full MIR, follow that pattern. The goal is branch coverage of the Rust code, not Cranelift IR correctness.

### Q7: General
- **Answer**: Read existing lowering test patterns. Use whichever approach is already established (manual AST construction or parse-from-string helpers). Follow the existing pattern for consistency.

### Q8: General
- **Answer**: Read the source file and cross-reference with existing parser tests to identify exactly which constructs lack coverage. Then write targeted test inputs for each uncovered branch. The spec must enumerate every uncovered construct explicitly.

