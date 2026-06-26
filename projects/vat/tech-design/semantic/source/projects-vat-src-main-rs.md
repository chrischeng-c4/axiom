---
id: vat-source-projects-vat-src-main-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/src/main.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/src/main.rs

## Overview
<!-- type: overview lang: markdown -->

Rust source-unit TD for `projects/vat/src/main.rs`, captured during #39 vat migration onto td_ast lossless source generation.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
use std::process::ExitCode;

fn main() -> ExitCode {
    match vat::cli::run() {
        Ok(code) => code,
        Err(err) => {
            // Print the full anyhow chain so an agent reading stderr gets the
            // root cause, not just the top-level message.
            eprintln!("vat: error: {err:#}");
            ExitCode::FAILURE
        }
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/main.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/src/main.rs` captured during #39 vat standardization.
```
