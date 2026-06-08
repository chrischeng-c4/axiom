---
change_id: mamba-py312-test-suite
type: gap_codebase_spec
created_at: 2026-02-13T10:33:26.436136+00:00
updated_at: 2026-02-13T10:33:26.436136+00:00
---

# Gap Analysis: Codebase vs Spec

## Code with NO matching Spec

- **crates/mamba/src/parser/stmt.rs** (Severity: High)
  - Implementation of `parse_optional_type_params` (Py 3.12 syntax) exists but is not covered by any specification.
- **crates/mamba/src/parser/type_expr.rs** (Severity: Medium)
  - Support for Type Unions (`|`) and Optional (`?`) types exists in code but lacks a formal specification.
- **crates/mamba/tests/fixture_tests.rs** (Severity: High)
  - The directive-based test harness (`# RUN:`, `# EXPECT:`) exists but its architecture and supported modes are not spec'd.

## Specs with NO matching Implementation

- **mamba-jit-backend** R4 (Severity: Medium)
  - Memory management for executable code. While the spec exists, the implementation in `jit.rs` might be basic and lacks explicit session-based cleanup tests.

## Missing Specs for Change Goal

- **Py 3.12 Syntax Coverage** (Severity: High)
  - No specification exists defining the required syntax subset for Py 3.12 compatibility.
- **CPython Test Integration** (Severity: High)
  - No specification exists for the integration of CPython test snippets into the Mamba fixture system.
