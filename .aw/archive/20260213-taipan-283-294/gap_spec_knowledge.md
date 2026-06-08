---
change_id: taipan-283-294
type: gap_spec_knowledge
created_at: 2026-02-13T04:15:26.932529+00:00
updated_at: 2026-02-13T04:15:26.932529+00:00
---

# Gap Analysis: Spec vs Knowledge

## Pattern Mismatches

- **taipan-backend-cranelift** vs **orbit/bridge-internals.md** (GIL Release)
  - Gap: The Cranelift backend specification lacks any requirement for GIL management during external function calls or async transitions. This is a critical pattern for Python-native interoperability.
  - Severity: High
- **taipan-ir** vs **changelogs/orbit-architecture.md** (Slab Allocator)
  - Gap: The IR specification defines a hierarchical structure (Module/Function/Block/Inst) but does not incorporate the Slab/Arena allocation pattern documented for high-performance Orbit components.
  - Severity: Medium
- **taipan-syntax** / **taipan-ir** vs **changelogs/orbit-testing-safety.md** (Test Hardening)
  - Gap: None of the existing Taipan specifications include requirements for fuzz testing or Miri validation, which are ecosystem-wide standards for core native crates.
  - Severity: Medium

## Boundary Misalignments

- **taipan-syntax** vs **cclab-core/structured-error-handling**
  - Gap: The syntax for exception handling (#283) is not yet defined, and there is no specified mapping between Taipan's internal error types and the structured error hierarchy established in Cclab Core.
  - Severity: Medium
- **taipan-runtime** (Missing Spec) vs **orbit/bridge-internals.md**
  - Gap: There is no specification for the Taipan runtime library, which is the natural place to implement the "Bridge Safety" patterns (error propagation, refcount safety) required by Orbit knowledge.
  - Severity: High
