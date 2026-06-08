---
change: mamba-py312-conformance
group: mamba-py312-conformance
date: 2026-03-23
status: answered
---

# Pre-Clarifications

### Q1: conformance-runner-architecture
- **Answer**: (b) Golden files. Continue with existing infrastructure — tests/regen_golden.py generates .expected files from CPython 3.12. Python fixtures produce stdout, golden files capture CPython 3.12 expected output, Rust harness compares Mamba output against golden files. Deterministic, no CPython needed at test time.

### Q2: divergence-handling
- **Answer**: (a) Fix immediately. All divergences must be fixed as part of this change. Behavioral consistency with CPython 3.12 is non-negotiable.

### Q3: test-fixture-location
- **Answer**: (a) Continue in crates/mamba/tests/fixtures/conformance/ with conformance_tests.rs harness using datatest_stable and regen_golden.py.

### Q4: cpython-version-pinning
- **Answer**: Use latest CPython 3.12.x available on the system. No specific patch pinning.

### Q5: stdlib-scope-for-remaining-66-modules
- **Answer**: Cover all stdlib modules that Mamba currently implements, not just the top 16. Test conformance for every module that has a Mamba implementation.

