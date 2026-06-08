---
id: projects-score-tests-phase-migration-test-rs
fill_sections: [overview, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Standardization TDs support brownfield takeover, semantic coverage, traceability, and production readiness gates."
---

# Standardized projects/agentic-workflow/tests/cli/tests/phase_migration_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/tests/cli/tests/phase_migration_test.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/tests/cli/tests/phase_migration_test.rs -->
```rust
//! Tests for the phase-enum reader compatibility (Phase 1 migration).
//!
//! - Reader accepts both `cb_genned` (canonical) and `td_gen_coded`
//!   (legacy alias).
//! - Writer always emits `cb_genned`.
//! - Trailer reader accepts both `Cb-Gen` and `Td-GenCode`; writer
//!   emits `Cb-Gen`.
//!
//! @spec projects/agentic-workflow/tech-design/surface/specs/score-namespaces.md#test-plan

use agentic_workflow::issues::types::{lifecycle_trailer, td_phase};

#[test]
fn test_phase_reader_accepts_legacy() {
    // Legacy phase string normalises to canonical.
    assert_eq!(td_phase::normalize("td_gen_coded"), "cb_genned");
    // Canonical passes through unchanged.
    assert_eq!(td_phase::normalize("cb_genned"), "cb_genned");
    // Unrelated phases pass through unchanged.
    assert_eq!(td_phase::normalize("td_reviewed"), "td_reviewed");
}

#[test]
fn test_phase_writer_emits_canonical() {
    // Source-text proof: `td.rs::run_gen_code` writes the canonical
    // phase string. We verify by source inspection because mutating an
    // issue file requires a worktree fixture that is heavier than this
    // pure-string test calls for.
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/td.rs");
    let body = std::fs::read_to_string(&path).expect("read td.rs");
    assert!(
        body.contains(r#"phase: Some("cb_genned".to_string())"#),
        "td::run_gen_code must write the canonical 'cb_genned' phase"
    );
    assert!(
        !body.contains(r#"phase: Some("td_gen_coded".to_string())"#),
        "td::run_gen_code must not write the legacy 'td_gen_coded' phase"
    );
}

#[test]
fn test_trailer_reader_accepts_legacy() {
    assert_eq!(lifecycle_trailer::normalize("Td-GenCode"), "Cb-Gen");
    assert_eq!(lifecycle_trailer::normalize("Cb-Gen"), "Cb-Gen");
    assert_eq!(lifecycle_trailer::normalize("Td-Merge"), "Td-Merge");
}

#[test]
fn test_trailer_writer_emits_canonical() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/td.rs");
    let body = std::fs::read_to_string(&path).expect("read td.rs");
    // Writer emits Cb-Gen (post Phase 1).
    assert!(
        body.contains(r#""Cb-Gen""#),
        "td::run_gen_code must commit canonical 'Cb-Gen' trailer"
    );
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/cli/tests/phase_migration_test.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Existing source claimed by `aw standardize managed run`. The code is
      wrapped in a tracked HANDWRITE block until deterministic generator
      coverage can replace it with CODEGEN.
```
