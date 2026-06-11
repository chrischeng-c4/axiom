---
id: projects-meter-meter-cli-src-bin-meter-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: delegated-runner-exit-code-contract
    claim: delegated-runner-exit-code-contract
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/meter-cli/src/bin/meter.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/meter-cli/src/bin/meter.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/meter-cli/src/bin/meter.rs -->
````rust
//! Standalone `meter` binary.
//!
//! No cclab host binary exists in-tree yet, so this thin entrypoint makes
//! `meter <verb>` work today. It returns [`std::process::ExitCode`] so the `test`
//! verb FORWARDS the child runner's exit code cleanly (the registry path cannot
//! return an exit code, so it `std::process::exit`s instead).

use std::process::ExitCode;

use clap::Parser;

use meter_cli::dispatch::{dispatch, print_report, MeterCommand, OutputOpts};

fn main() -> ExitCode {
    // Parse the verb tree; the global `--human`/`--compact` flags are flattened
    // into `MeterCommand::output`, so there is exactly one source for them.
    let cmd = MeterCommand::parse();
    let out: OutputOpts = cmd.output.clone();

    let dispatched = dispatch(cmd, &out);
    // Offline self-describers already wrote their single stdout document; only
    // emit the wrapped report when they did not.
    if !dispatched.stdout_written {
        print_report(&dispatched.report, &out);
    } else if out.human {
        // Still surface the human stderr summary for offline verbs.
        meter::report::emit::diag(format!(
            "meter {} -> exit {}",
            dispatched.report.verb, dispatched.report.exit_code
        ));
    }

    // Forward the (clamped) exit code as the process exit. ExitCode takes a u8.
    ExitCode::from(dispatched.report.exit_code.clamp(0, 255) as u8)
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/meter-cli/src/bin/meter.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/meter-cli/src/bin/meter.rs` captured during meter full-codegen standardization.
```
