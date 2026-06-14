---
id: projects-rig-rig-cli-src-bin-rig-rs
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/rig-cli/src/bin/rig.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/rig/rig-cli/src/bin/rig.rs`, captured as a rust-source-unit (td_ast) item-tree
during rig standardization onto the codegen ladder.

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
