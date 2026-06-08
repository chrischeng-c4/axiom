---
change_id: taipan-all
type: gap_codebase_spec
created_at: 2026-02-12T10:39:22.675951+00:00
updated_at: 2026-02-12T10:39:22.675951+00:00
---

# Gap Analysis: Codebase vs Spec

## Unspecced Code (Code exists, no matching spec)

- **crates/cclab-taipan/src/resolve** (Severity: Medium)
  - The name resolution and scoping logic is implemented but lacks a formal specification.
- **crates/cclab-taipan/src/diagnostic** (Severity: Low)
  - Error reporting and diagnostic rendering logic is present but unspecced.
- **crates/cclab-taipan/src/source** (Severity: Low)
  - Source management and `SourceMap` implementation are present but unspecced.
- **crates/cclab-taipan/src/types** (Severity: Medium)
  - Core type system and `TypeContext` (interner) lack a dedicated specification, though type mapping is briefly mentioned in backend specs.

## Unimplemented Specs (Spec exists, no matching implementation)

- **taipan-ir** (Severity: High)
  - SSA form requirements and Instruction Set Architecture Core are specified, but the `hir` and `mir` modules in the codebase are currently mostly skeletons.
- **taipan-backend-cranelift** (Severity: High)
  - Instruction translation and advanced type mapping are specified, but the implementation in `crates/cclab-taipan/src/codegen/cranelift/mod.rs` is rudimentary.
- **taipan-cli-integration** (Severity: Medium)
  - The integration of Taipan into the unified CLI tool is specified, but the implementation within `cclab-cli` and the full orchestration in `driver` are incomplete.
