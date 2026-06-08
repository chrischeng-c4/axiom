---
change: mamba-runtime-bugs
group: mamba-runtime-fixes
date: 2026-03-24
status: answered
---

# Pre-Clarifications

### Q1: approach
- **Answer**: 5 independent fixes: parser for semicolons, codegen for ZeroDivisionError check, HIR/MIR for decorator return, parser for nested f-strings, runtime for json module dispatch.

