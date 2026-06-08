---
change_id: taipan-283-294
type: gap_codebase_spec
created_at: 2026-02-13T04:14:26.302150+00:00
updated_at: 2026-02-13T04:14:26.302150+00:00
---

# Gap Analysis: Codebase vs Spec

## Code without Spec

- **crates/cclab-taipan/src/runtime/value.rs**
  - Gap: The entire NaN-boxing implementation (TpValue) lacks a formal specification in cclab/specs. This is a critical architectural decision that affects all future codegen and runtime work.
  - Severity: High
- **crates/cclab-taipan/src/runtime/rc.rs**
  - Gap: The reference-counted heap object system (TpObject, ObjData, tp_retain/tp_release) is implemented but not specified. This governs memory management for strings, lists, and instances.
  - Severity: High
- **crates/cclab-taipan/src/mir/mod.rs** (MirExtern, CallExtern)
  - Gap: FFI support via extern declarations and external calls exists in the code but is not detailed in the `taipan-ir` spec.
  - Severity: Medium
- **crates/cclab-taipan/src/lower/ast_to_hir.rs** and **hir_to_mir.rs**
  - Gap: The actual desugaring and lowering logic is implemented but lacks a detailed algorithm specification beyond high-level flowcharts.
  - Severity: Medium

## Specs without Implementation

- **taipan-backend-cranelift** (Requirement R4: Object File Emission)
  - Gap: The spec requires producing standard object files (ELF/Mach-O), but the current implementation primarily focus on JIT/in-memory execution or is incomplete.
  - Severity: Medium
- **taipan-backend-cranelift** (Requirement R5: External Function Support)
  - Gap: Calling external native functions is specified but needs robust implementation to support the Taipan runtime library (e.g., calling tp_list_append from compiled code).
  - Severity: High

## Feature Gaps (Issues #283-294)

- **Requirement Gaps**: None of the 12 P1+P2 features requested in this change have existing requirements or specifications.
- **Implementation Gaps**: None of the 12 P1+P2 features have been implemented in the codebase.
