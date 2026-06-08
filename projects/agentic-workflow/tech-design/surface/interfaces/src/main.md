---
id: projects-score-src-main-rs
fill_sections: [overview, changes]
capability_refs:
  - id: workflow-root-runner
    role: primary
    gap: cli-workflow-chain
    claim: cli-workflow-chain
    coverage: full
    rationale: "CLI entrypoint and dispatch surfaces support root command parsing and workflow command routing."
---

# Standardized projects/agentic-workflow/src/bin/aw.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/bin/aw.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/bin/aw.rs -->
```rust
//! Score — local Spec-Driven Development orchestrator via Claude Code.
//!
//! Standalone binary entry point. Delegates to the `score` library for
//! the `Commands` enum and `run_command` dispatch.

use anyhow::Context;
use clap::Parser;
use agentic_workflow::cli::{run_command, Commands};

#[derive(Parser)]
#[command(
    name = "agentic-workflow",
    version = env!("SCORE_BUILD_VERSION"),
    about = "Score — local Spec-Driven Development orchestrator via Claude Code"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let rt = tokio::runtime::Runtime::new().context("Failed to create tokio runtime")?;
    rt.block_on(run_command(cli.command))?;
    Ok(())
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/bin/aw.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Existing source claimed by `aw standardize managed run`. The code is
      wrapped in a tracked HANDWRITE block until deterministic generator
      coverage can replace it with CODEGEN.
```
