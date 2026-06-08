---
change: pyo3-pyi-stub
date: 2026-01-30
---

# Clarifications

## Q1: Scope
- **Question**: Should this generate entire Python package or just .pyi stubs?
- **Answer**: Just .pyi stubs. Full Python codegen will be a separate issue.
- **Rationale**: Issue #25 is specifically about .pyi generation. The larger refactoring to auto-generate entire python/cclab/ package will be tracked separately.

## Q2: Source
- **Question**: GitHub issue reference
- **Answer**: https://github.com/chrischeng-c4/cclab/issues/25
- **Rationale**: Full specification including type mappings, PyO3 attributes, CLI interface, and test fixtures are defined in the issue.

