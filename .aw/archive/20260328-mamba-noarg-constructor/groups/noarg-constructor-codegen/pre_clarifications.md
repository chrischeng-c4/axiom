---
change: mamba-noarg-constructor
group: noarg-constructor-codegen
date: 2026-03-28
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Is the fix already applied?
- **Answer**: Partially — commit bc5921e9 included some P0 runtime fixes. Need to verify if list()/tuple()/set() zero-arg constructors are fixed or still broken. The codegen verifier error is in cranelift IR emission for zero-argument builtin calls.

