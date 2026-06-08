---
change: mamba-binding-tests
group: mamba-tests
date: 2026-03-22
status: answered
---

# Pre-Clarifications

### Q1: scope-split
- **Answer**: This change is ONLY for binding crates (cclab-*-mamba) + registry. NOT runtime/lower/resolve/stdlib — those are separate per #1035. Focus: 8 binding crates + registry = 9 crates total.

### Q2: binding-crates-mb-functions
- **Answer**: Each binding crate has methods.rs with pub fn mb_* functions taking MbValue args and returning MbValue. They're registered via MambaModule trait impl in lib.rs. Inspect each crate's methods.rs to enumerate all mb_* functions. No macro-based discovery — manual inspection.

### Q3: registry-ignored-tests
- **Answer**: Check the actual #[ignore] tests in cclab-mamba-registry. They may need MbValue runtime (NaN-boxing) which requires cclab-mamba dependency to be properly linked. Un-ignore if possible, or document why blocked.

### Q4: runtime-test-placement
- **Answer**: Out of scope for this change — runtime tests are #1035 separate work. This change only covers binding crate tests. For binding crates: tests go in crates/cclab-{name}-mamba/tests/ directory (integration test style).

### Q5: stdlib-scope
- **Answer**: Out of scope. This change is binding crates only. Stdlib tests are in #1035.

### Q6: coverage-tooling
- **Answer**: Out of scope. Coverage tooling setup is separate. This change just adds test files. No CI changes.

