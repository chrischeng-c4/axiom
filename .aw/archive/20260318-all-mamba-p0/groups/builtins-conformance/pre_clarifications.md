---
change: all-mamba-p0
group: builtins-conformance
date: 2026-03-18
status: answered
---

# Pre-Clarifications

### Q1: Verification method
- **Question**: Should conformance tests run the same code on both Mamba and CPython and diff outputs, or use CPython's expected output as golden files?
- **Answer**: Diff all three runtimes — Mamba has two runtimes (JIT and AOT), so conformance tests should compare CPython 3.12, Mamba JIT, and Mamba AOT outputs.

### Q2: Failure handling
- **Question**: For builtins that currently diverge from CPython, should we fix them in this change or just document the failures in a known_failures list?
- **Answer**: Fix divergences — fix builtins that don't match CPython behavior in this change.

