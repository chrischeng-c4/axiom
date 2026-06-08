---
change_id: taipan-295-297
type: gap_codebase_spec
created_at: 2026-02-13T07:24:06.452962+00:00
updated_at: 2026-02-13T07:24:06.452962+00:00
---

# Gap Analysis: Codebase vs Spec

## Code without Spec

- **TpValue NaN-boxing Implementation**: `crates/cclab-taipan/src/runtime/value.rs` contains the complete NaN-boxing logic which is not formally specified in any current spec. (Severity: Medium)
- **Runtime tp_* Functions**: The specific implementations of list, dict, and tuple operations in `crates/cclab-taipan/src/runtime/*.rs` go beyond the high-level descriptions in `taipan-core-types.md`. (Severity: Medium)
- **MirInst Variants**: The IR instructions `GetAttr`, `SetAttr`, `GetItem`, `SetItem`, `MakeList`, `MakeDict`, `MakeTuple`, and `Raise` exist in `mir/mod.rs` and are partially handled in `codegen/cranelift/mod.rs` (placeholders), but their mapping to runtime functions is not defined in `taipan-backend-cranelift.md`. (Severity: High)

## Spec without Implementation

- **taipan-cli-integration (Scenario: Execute Run Command)**: The 'run' command is specified to compile and execute, but currently only prints a 'not yet implemented' message in `crates/cclab-cli/src/taipan.rs`. (Severity: High)
- **taipan-backend-cranelift (R5 - External Function Support)**: While specified, it is currently limited to basic FFI and does not cover the extensive runtime symbol wiring needed for object operations. (Severity: High)

## Other Gaps

- **JIT Support**: Neither the codebase nor the specs currently contain the `JITModule` implementation or its design. (Severity: High)
