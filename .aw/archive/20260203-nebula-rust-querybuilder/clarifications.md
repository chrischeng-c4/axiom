---
change: nebula-rust-querybuilder
date: 2026-02-01
---

# Clarifications

## Q1: Validation Failures
- **Question**: Why are there validation errors in the specs directory?
- **Answer**: The current environment lacks 'run_shell_command' or 'write_file' tools, preventing the deletion of invalid spec files (querybuilder-types.md, querybuilder-pyo3.md) that do not follow the required naming convention. Correct versions ending in '-spec.md' have been created.
- **Rationale**: To document the technical constraint preventing a clean validation pass.

