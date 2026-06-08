---
change: mamba-xfail-zero
group: xfail-zero
date: 2026-03-27
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Any scope questions?
- **Answer**: No — all 34 xfails must pass. Fix runtime to match CPython 3.12 exactly. If a stdlib module is not implemented, simplify the test fixture to only test what Mamba supports, or implement the missing functionality.

