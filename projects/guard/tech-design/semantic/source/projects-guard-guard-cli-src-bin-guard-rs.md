---
id: projects-guard-guard-cli-src-bin-guard-rs
summary: Lossless rust-source-unit coverage for `projects/guard/guard-cli/src/bin/guard.rs`.
capability_refs:
  - id: static-security-scan
    role: primary
    gap: json-report-envelope
    claim: json-report-envelope
    coverage: full
    rationale: "The CLI source unit exposes the guard scan/report envelope as the agent-facing command surface."
  - id: security-policy-profile
    role: contributes
    gap: cli-module-registration
    claim: cli-module-registration
    coverage: full
    rationale: "The CLI source unit registers and dispatches the guard policy profile commands."
fill_sections: [overview, source, changes]
---

# Standardized projects/guard/guard-cli/src/bin/guard.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/guard/guard-cli/src/bin/guard.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
use std::process::ExitCode;

use clap::Parser;
use guard_cli::{dispatch, print_report, GuardCommand};

fn main() -> ExitCode {
    let cmd = GuardCommand::parse();
    let out = cmd.output.clone();
    let report = dispatch(cmd);
    print_report(&report, &out);
    ExitCode::from(report.exit_code.clamp(0, 255) as u8)
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/guard/guard-cli/src/bin/guard.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/guard/guard-cli/src/bin/guard.rs` captured during guard standardization onto the codegen ladder.
```
