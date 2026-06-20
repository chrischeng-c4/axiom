---
id: projects-rig-rig-cli-src-bin-rig-rs
capability_refs:
  - id: scenario-engine
    role: primary
    claim: scenario-step-dsl-execution
    coverage: partial
    rationale: "This source unit implements rig scenario discovery, execution, verdict, or report behavior used by the scenario engine."
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/rig-cli/src/bin/rig.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/rig/rig-cli/src/bin/rig.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! `rig` standalone binary: parse → dispatch → one JSON document → exit code.

use clap::Parser;
use rig_cli::dispatch::{execute, print_report, RigCommand};

fn main() {
    let cmd = RigCommand::parse();
    let output = cmd.output.clone();
    let report = execute(cmd);
    let code = print_report(&report, &output);
    std::process::exit(code);
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/rig/rig-cli/src/bin/rig.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/rig/rig-cli/src/bin/rig.rs` captured during rig
      standardization onto the codegen ladder.
```
