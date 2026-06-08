---
change: mamba-py312-test-suite
date: 2026-03-02
---

# Context Clarifications

## Q1: General
- **Question**: What scope of Python 3.12 test coverage do you want?
- **Answer**: Expand CPython compat fixtures — add more CPython 3.12 stdlib test fixtures to tests/fixtures/parse/cpython/ and update the xfail manifest (known_failures.toml).
- **Rationale**: 

## Q2: General
- **Question**: Which crates/paths will this change affect?
- **Answer**: cclab-mamba-tests only — only add/modify test fixtures and the test harness crate. No changes to the compiler crate itself.
- **Rationale**: 

