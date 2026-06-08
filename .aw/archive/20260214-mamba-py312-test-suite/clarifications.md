---
change: mamba-py312-test-suite
date: 2026-02-13
---

# Clarifications

## Q1: Test Scope
- **Question**: Which CPython test files to use for verifying py312 syntax?
- **Answer**: Curated subset: extract syntax snippets from test_grammar.py, test_syntax.py, and targeted files for py312 features (fstrings, match, type_params). Focus on syntax constructs, not test harness code.
- **Rationale**: Full Lib/test/ suite is too broad. Grammar+syntax-only misses py312-specific features. A curated subset covers all syntax categories efficiently.

## Q2: Integration Approach
- **Question**: How to integrate CPython test cases into Mamba test suite?
- **Answer**: Parse-only fixtures using existing datatest-stable harness with # RUN: parse directive. Place under tests/fixtures/parse/cpython/ subdirectory.
- **Rationale**: Reuses the fixture_tests.rs harness just built. Parse-only is the right level since the goal is syntax coverage, not semantic correctness. Separate cpython/ subdirectory keeps these organized.

## Q3: Git Workflow
- **Question**: Which git workflow?
- **Answer**: in_place on current feat/mamba branch
- **Rationale**: Continuation of the mamba compiler work already on this branch.

