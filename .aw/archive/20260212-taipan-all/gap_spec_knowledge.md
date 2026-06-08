---
change_id: taipan-all
type: gap_spec_knowledge
created_at: 2026-02-12T10:41:40.155529+00:00
updated_at: 2026-02-12T10:41:40.155529+00:00
---

# Gap Analysis: Spec vs Knowledge

## Spec Responsibilities vs Knowledge Architecture

- **FFI and GIL Management** (Severity: High)
  - The `taipan-backend-cranelift` spec mentions "External Function Support". If this involves Python integration, it must explicitly reference and adhere to the `GIL Release Strategy` and `Error Propagation` patterns documented in `orbit/bridge-internals.md` to avoid deadlocks.
- **Memory Management and Performance** (Severity: Medium)
  - While `taipan-ir` targets high performance via SSA form, it does not currently reflect the `Slab Allocator` pattern mentioned in `changelogs/orbit-architecture.md`. This pattern is critical for efficient management of short-lived compiler objects like IR instructions or interned types.

## Knowledge Patterns not reflected in Spec

- **Safety-First Testing** (Severity: Medium)
  - The `taipan-*` specs lack requirements or acceptance criteria for Fuzz testing and Miri validation, which are mandatory for core high-performance crates according to `changelogs/orbit-testing-safety.md`.
- **Modular Feature Flags** (Severity: Low)
  - The `taipan-cli-integration` and `taipan-backend-cranelift` specs do not incorporate modular feature flags for platform-specific optimizations (e.g., target architecture or OS-specific traits), violating the pattern established in `changelogs/orbit-architecture.md`.

## Responsibility Boundary Misalignments

- **Name Resolution Boundary** (Severity: Low)
  - The transition from `taipan-syntax` (AST) to `taipan-ir` (MIR) does not explicitly define where the `resolve` (name resolution) pass occurs. Documentation like `05-titan/architecture-guide.md` emphasizes decoupling, which should be reflected in the compiler's pass architecture.
