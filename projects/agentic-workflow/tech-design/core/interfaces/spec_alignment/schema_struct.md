---
id: projects-sdd-src-spec-alignment-schema-struct-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: traceability-closure-gate
    claim: traceability-closure-gate
    coverage: full
    rationale: "Spec alignment interfaces implement TD/source annotation and coverage checks used by the traceability closure gate."
---

# Standardized projects/agentic-workflow/src/spec_alignment/schema_struct.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/spec_alignment/schema_struct.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `check` | projects/agentic-workflow/src/spec_alignment/schema_struct.rs | function | pub | 24 | check(     _spec_dir: &std::path::Path,     _daemon_ready: bool, ) -> (Vec<Violation>, Vec<SchemaStructMismatchEntry>) |
| `json_schema_type_to_rust` | projects/agentic-workflow/src/spec_alignment/schema_struct.rs | function | pub | 47 | json_schema_type_to_rust(schema_type: &str, format: Option<&str>) -> &'static str |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:claim-code -->

<!-- source-snapshot: path=projects/agentic-workflow/src/spec_alignment/schema_struct.rs -->
```rust
//! Schema↔Struct validation.
//!
//! Compares JSON Schema `properties` from spec files with Rust struct fields
//! via Lens symbol index. Emits `schema_struct_mismatch` violations.
//!
//! Only active when daemon index is ready.

use super::models::{SchemaStructMismatchEntry, Violation};

/// Check JSON Schema definitions against Rust struct fields.
///
/// Currently a stub — full implementation requires daemon symbol index integration.
/// When the daemon is ready, this will:
/// 1. Extract JSON Schema `definitions` from spec files
/// 2. Query daemon for Rust struct definitions matching schema names
/// 3. Compare properties ↔ struct fields
/// 4. Emit violations for mismatches
///
/// Returns `(violations, mismatches)` where mismatches provide structured data
/// for the coverage report.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/schema_struct.md#source
pub fn check(
    _spec_dir: &std::path::Path,
    _daemon_ready: bool,
) -> (Vec<Violation>, Vec<SchemaStructMismatchEntry>) {
    if !_daemon_ready {
        return (Vec::new(), Vec::new());
    }

    // Phase 2 stub: daemon integration for struct field queries.
    // When daemon is available, this will:
    // 1. Parse JSON Schema definitions from spec files
    // 2. For each schema with `properties`, find matching Rust struct by name
    // 3. Query daemon.symbols() for struct field definitions
    // 4. Compare: missing_in_struct, missing_in_schema, type_mismatch
    // 5. Build violations and SchemaStructMismatchEntry records
    (Vec::new(), Vec::new())
}

/// Map JSON Schema type to approximate Rust type for comparison.
///
/// This is a best-effort mapping used for type_mismatch detection.
#[allow(dead_code)]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/schema_struct.md#source
pub fn json_schema_type_to_rust(schema_type: &str, format: Option<&str>) -> &'static str {
    match (schema_type, format) {
        ("string", Some("date-time")) => "DateTime",
        ("string", Some("uuid")) => "Uuid",
        ("string", _) => "String",
        ("integer", _) => "i64",
        ("number", _) => "f64",
        ("boolean", _) => "bool",
        ("array", _) => "Vec",
        ("object", _) => "HashMap",
        _ => "unknown",
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/spec_alignment/schema_struct.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:claim-code>"
    description: |
      Source template owns the complete spec-alignment schema/struct module.
```
