---
change: 1132-patrol
group: wire-mamba-modules
date: 2026-04-04
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Where should MAMBA_MODULES lookup happen?
- **Answer**: Resolver (name resolution phase). Imported symbols get resolved before codegen, consistent with stdlib import handling.

### Q2: General
- **Question**: Python-name to FFI-name mapping strategy?
- **Answer**: Registry aliases. RuntimeSymbol already has python_name field — use it for lookup during import resolution.

