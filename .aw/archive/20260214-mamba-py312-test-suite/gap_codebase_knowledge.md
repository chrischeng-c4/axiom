---
change_id: mamba-py312-test-suite
type: gap_codebase_knowledge
created_at: 2026-02-13T10:33:47.331130+00:00
updated_at: 2026-02-13T10:33:47.331130+00:00
---

# Gap Analysis: Codebase vs Knowledge

## Pattern Mismatches

- **crates/mamba/tests/fixture_tests.rs** (Severity: Medium)
  - Does not utilize the **Requirement Plus Traceability** pattern. Tests are implemented as file-based fixtures without explicit N:M mapping to requirements as defined in `knowledge:spec-to-code/spec-model.md`.
- **crates/mamba/src/error.rs** (Severity: Low)
  - Potential mismatch with **Structured Error Handling** patterns if `MambaError` does not fully integrate with the `cclab-core` error traits (need verification during implementation).

## Missing Knowledge

- **Mamba Parser Conventions** (Severity: High)
  - No knowledge document exists describing the conventions for Mamba-specific syntax (e.g., how it differs from CPython for 'force-typed' constructs).
- **Test Fixture Organization** (Severity: Medium)
  - No knowledge document defines the subdirectory structure for `tests/fixtures/` (e.g., when to use `parse/` vs `jit/`).
