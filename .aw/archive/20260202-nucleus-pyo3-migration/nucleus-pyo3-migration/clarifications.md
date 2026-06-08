---
change: nucleus-pyo3-migration
date: 2026-02-01
---

# Clarifications

## Q1: Shared Conversion Logic
- **Question**: How should we handle the shared conversion.rs (1020 lines) that contains BSON/Python conversion logic used by multiple crates?
- **Answer**: Keep in cclab-core
- **Rationale**: Move shared conversion logic to cclab-core so each crate can import what it needs. This maintains DRY principle and centralizes the GIL-free conversion patterns.

## Q2: Nucleus Deprecation
- **Question**: Should we deprecate cclab-nucleus entirely after migration, or keep it as a thin re-export layer?
- **Answer**: Deprecate entirely
- **Rationale**: Remove nucleus completely. Python imports directly from each crate (_titan, _nebula, _ion, etc.). Cleaner architecture with no intermediate layer.

## Q3: Python Import Migration
- **Question**: What's the migration strategy for existing Python code that imports from nucleus?
- **Answer**: Use prism auto-generation with backup
- **Rationale**: Python imports should be auto-generated via `cclab prism gen-stub`. Backup existing code first, try auto-generation, and fix prism if needed. This validates the tooling.

## Q4: Git Workflow
- **Question**: Which git workflow do you prefer for this change?
- **Answer**: New branch: genesis/nucleus-pyo3-migration
- **Rationale**: Isolated branch for this significant refactoring to keep main stable during migration.

