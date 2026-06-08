---
change: mamba-conformance-xfail
group: conformance-xfail-reduction
date: 2026-03-26
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should we prioritize breadth or depth?
- **Answer**: Depth — kwargs first. Fix the 5 kwargs verifier errors first, then move to output mismatches.

### Q2: General
- **Question**: For output mismatches — fix runtime or update golden files?
- **Answer**: Fix runtime to match CPython 3.12 output exactly. Conformance means matching CPython.

