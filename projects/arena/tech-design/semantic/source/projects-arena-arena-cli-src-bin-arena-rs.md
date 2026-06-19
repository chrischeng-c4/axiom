---
id: projects-arena-arena-cli-src-bin-arena-rs
capability_refs:
  - id: n-target-comparison-runner
    role: primary
    claim: sequential-target-fanout-and-measurement
    coverage: partial
    rationale: "This source unit implements arena CLI, spec parsing, measurement, or runner orchestration for N-target comparisons."
fill_sections: [overview, source, changes]
---

# Standardized projects/arena/arena-cli/src/bin/arena.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/arena/arena-cli/src/bin/arena.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! `arena` standalone binary: parse → dispatch → one JSON document → exit code.

use arena_cli::dispatch::{execute, print_report, ArenaCommand};
use clap::Parser;

fn main() {
    let cmd = ArenaCommand::parse();
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
  - path: projects/arena/arena-cli/src/bin/arena.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/arena/arena-cli/src/bin/arena.rs` captured during arena
      standardization onto the codegen ladder.
```
