---
change_id: mamba-py312-test-suite
type: gap_spec_knowledge
created_at: 2026-02-13T10:34:06.677826+00:00
updated_at: 2026-02-13T10:34:06.677826+00:00
---

# Gap Analysis: Spec vs Knowledge

## Missing Patterns in Specs

- **Requirement Plus Traceability** (Severity: High)
  - The `spec_context` specs (`mamba-jit-backend`, `state-machine`) do not include **Requirement Plus** diagrams, which are required for automated test generation according to `knowledge:spec-to-code/spec-model.md`.
- **GIL Management and Thread Safety** (Severity: Medium)
  - The `mamba-jit-backend` spec defines symbol wiring for runtime functions but does not reference the GIL release/acquire patterns documented in `knowledge:orbit/bridge-internals.md`. This is a gap in defining how JIT-executed code safely interacts with the Python runtime.

## Responsibility Misalignments

- **Test Infrastructure Ownership** (Severity: Medium)
  - The boundary between `cclab-probe` (generic test framework) and the Mamba-specific fixture harness is not clearly defined in the current specs, which may lead to duplication of test reporting logic.
