---
change: mamba-test-coverage
group: runtime-core-coverage
date: 2026-03-08
status: answered
---

# Pre-Clarifications

### Q1: bytes-ops-priority
- **Answer**: runtime/bytes_ops.rs (131 lines, 0% coverage) is a real implementation. The bytes type is used for binary data handling. It needs full test coverage — all operations (concat, slice, find, replace, decode, etc.) should be tested.

### Q2: runtime-test-location
- **Answer**: Prefer inline #[cfg(test)] modules in each source file for unit tests of internal functions. Use tests/runtime_tests.rs for integration tests that exercise multiple runtime components together. This matches the existing pattern and keeps tests close to the code they verify.

