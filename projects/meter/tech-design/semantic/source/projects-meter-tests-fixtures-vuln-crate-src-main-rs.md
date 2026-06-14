---
id: projects-meter-tests-fixtures-vuln-crate-src-main-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: legacy-carried-internals
    role: primary
    gap: seeded-fuzz-and-injection-finding-generation
    claim: seeded-fuzz-and-injection-finding-generation
    coverage: full
    rationale: "Source template implements meter security, fuzzing, injection, or audit surfaces."
---

# Standardized projects/meter/tests/fixtures/vuln_crate/src/main.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/tests/fixtures/vuln_crate/src/main.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Intentionally vulnerable fixture crate for the meter audit trust-bug test.
//! It pins `time = "=0.1.45"` (RUSTSEC-2020-0071) so `cargo audit` reports
//! a vulnerability. It is never built as part of the real workspace.
fn main() {}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/tests/fixtures/vuln_crate/src/main.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/tests/fixtures/vuln_crate/src/main.rs` captured during meter full-codegen standardization.
```
