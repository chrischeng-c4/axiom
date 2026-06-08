---
change: quasar-pyo3-bindings
date: 2026-02-01
---

# Clarifications

## Q1: Scope
- **Question**: Which PyO3 exports should be moved to src/pyo3_bindings/?
- **Answer**: All Python-facing types - Include PyWebSocket + any future Python bindings (Request, Response wrappers, etc.)
- **Rationale**: Comprehensive reorganization following cclab-shield and cclab-titan patterns, preparing for future Python bindings

## Q2: Test Organization
- **Question**: How should tests be organized?
- **Answer**: Separate tests/ directory - Integration tests in crates/cclab-quasar/tests/
- **Rationale**: Cleaner separation of unit and integration tests, better for PyO3 tests that require Python initialization

## Q3: Fix Existing Tests
- **Question**: Should we fix the 4 failing PyO3 tests in python_handler.rs as part of this change?
- **Answer**: Yes, fix them - Add pyo3::prepare_freethreaded_python() initialization
- **Rationale**: Addresses production readiness blocker, consolidates all PyO3-related fixes in one change

## Q4: Git Workflow
- **Question**: Preferred git workflow for this change?
- **Answer**: In place - Stay on current cclab-quasar branch
- **Rationale**: Simple workflow, changes are localized to cclab-quasar crate

