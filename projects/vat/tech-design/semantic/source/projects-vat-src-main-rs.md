---
id: vat-source-projects-vat-src-main-rs
summary: Source replay payload for projects/vat/src/main.rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: copy-on-write-fork-and-snapshot-lifecycle
    claim: copy-on-write-fork-and-snapshot-lifecycle
    coverage: full
    rationale: "This source replay TD preserves vat's copy-on-write workspace, agent-legible state, resource isolation, and host GPU behavior."
---

# Source TD: projects/vat/src/main.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/main.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->

`````rust
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
`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/vat/src/main.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/vat/tech-design/semantic/vat-src.md#schema.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-vat-src-main-rs-source-replay-superseded>"
```
