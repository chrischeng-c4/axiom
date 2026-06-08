---
change: mamba-conformance-p0
group: mamba-py312-conformance
date: 2026-03-25
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Where should conformance tests live?
- **Answer**: crates/mamba/tests/conformance/ — co-located with existing mamba tests. Each .py fixture runs on both CPython 3.12 and Mamba.

### Q2: General
- **Question**: How deep should conformance coverage go for p0 scope?
- **Answer**: Full CPython parity — all 33 list methods, all 47 str methods, all generator edge cases. Comprehensive coverage.

