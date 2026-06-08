---
change: mamba-string-reverse-slice
group: string-reverse-slice
date: 2026-03-28
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Is the fix already applied?
- **Answer**: Partially — commit bc5921e9 fixed the runtime logic in string_ops.rs. The remaining work is: (1) verify the fix works for all edge cases (s[::-1], s[4:1:-1], etc.), (2) convert xfail conformance test to passing test, (3) update specs.

