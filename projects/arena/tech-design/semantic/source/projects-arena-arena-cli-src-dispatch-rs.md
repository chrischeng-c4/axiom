---
id: projects-arena-arena-cli-src-dispatch-rs
capability_refs:
  - id: n-target-comparison-runner
    role: primary
    claim: sequential-target-fanout-and-measurement
    coverage: partial
    rationale: "This source unit implements arena CLI, spec parsing, measurement, or runner orchestration for N-target comparisons."
fill_sections: [overview, source, changes]
---

# Standardized projects/arena/arena-cli/src/dispatch.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/arena/arena-cli/src/dispatch.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ArenaCommand` | projects/arena/arena-cli/src/dispatch.rs | struct | pub | 24 |  |
| `OutputOpts` | projects/arena/arena-cli/src/dispatch.rs | struct | pub | 33 |  |
| `RunArgs` | projects/arena/arena-cli/src/dispatch.rs | struct | pub | 54 |  |
| `Verb` | projects/arena/arena-cli/src/dispatch.rs | enum | pub | 41 |  |
| `execute` | projects/arena/arena-cli/src/dispatch.rs | function | pub | 71 | execute(cmd: ArenaCommand) -> ArenaReport |
| `print_report` | projects/arena/arena-cli/src/dispatch.rs | function | pub | 128 | print_report(report: &ArenaReport, out: &OutputOpts) -> i32 |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Verb parse + dispatch for the `arena` agent-first CLI. Every verb produces
//! one `arena.report/1` JSON document on stdout; the exit code is rig's
//! worst-wins ladder (0 clean / 1 findings / 2 regression / 3 usage /
//! 4 missing-tool / 5 io).

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use arena::engine::{run, RunOpts};
use arena::report::{ArenaReport, SCHEMA_VERSION};
use arena::spec::Spec;

#[derive(Parser, Debug)]
#[command(
    name = "arena",
    version,
    about = "arena — N-target competitive comparison (JSON on stdout by default)",
    disable_help_subcommand = true
)]
pub struct ArenaCommand {
    #[command(subcommand)]
    pub verb: Verb,
    #[command(flatten)]
    pub output: OutputOpts,
}

#[derive(Args, Debug, Clone, Default)]
pub struct OutputOpts {
    /// Emit the report as a single dense line (byte-stable golden form).
    #[arg(long, global = true)]
    pub compact: bool,
}

#[derive(Subcommand, Debug)]
pub enum Verb {
    /// Run a comparison spec: measure each target, ratio, ratchet-gate, report.
    Run(RunArgs),
    /// Re-project the persisted `.arena/last-report.json` (no measurement).
    Report,
    /// Offline self-describer: print the report schema id.
    Spec,
    /// Offline agent playbook stub.
    Llm,
}

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Path to the comparison spec (`arena.toml`).
    #[arg(long)]
    pub spec: PathBuf,
    /// Record each measured ratio as the new baseline (no gating this run).
    #[arg(long)]
    pub update_baselines: bool,
    /// Treat a missing baseline as a failure (default: Info).
    #[arg(long)]
    pub strict: bool,
    /// Reserved: wrap the run in a vat environment (not implemented in v1).
    #[arg(long)]
    pub vat: bool,
}

/// Dispatch a parsed command to one report.
pub fn execute(cmd: ArenaCommand) -> ArenaReport {
    match cmd.verb {
        Verb::Run(args) => run_run(args),
        Verb::Report => run_report(),
        Verb::Spec => ArenaReport::stub("spec", &format!("arena report schema: {SCHEMA_VERSION}")),
        Verb::Llm => ArenaReport::stub(
            "llm",
            "arena compares N targets on one workload: ratio = peer/base, ratchet-gated. Service ratios are end-to-end HTTP latency (floor-bound); mark floor-dominated cheap cells `gate = \"exempt\"`.",
        ),
    }
}

fn run_run(args: RunArgs) -> ArenaReport {
    if args.vat {
        // v1: vat wrapping is a documented runner pattern, not a CLI mode.
        return ArenaReport::tool_error(
            3,
            "--vat is not implemented in v1; declare arena as a vat [[runners]] entry instead",
        );
    }
    let text = match std::fs::read_to_string(&args.spec) {
        Ok(t) => t,
        Err(e) => {
            return ArenaReport::tool_error(
                5,
                format!("could not read spec `{}`: {e}", args.spec.display()),
            )
        }
    };
    let spec = match Spec::parse(&text) {
        Ok(s) => s,
        Err(e) => return ArenaReport::tool_error(3, e),
    };
    let opts = RunOpts {
        update_baselines: args.update_baselines,
        baseline_path: None,
        strict: args.strict,
    };
    let report = run(&spec, &opts);
    report.persist(std::path::Path::new("."));
    report
}

fn run_report() -> ArenaReport {
    match std::fs::read_to_string(".arena/last-report.json") {
        Ok(t) => serde_json::from_str(&t).unwrap_or_else(|e| {
            ArenaReport::tool_error(5, format!("corrupt .arena/last-report.json: {e}"))
        }),
        Err(_) => ArenaReport::tool_error(
            5,
            "no persisted report at .arena/last-report.json; run `arena run` first",
        ),
    }
}

/// Print the report (pretty unless `--compact`) and return its exit code.
pub fn print_report(report: &ArenaReport, out: &OutputOpts) -> i32 {
    let json = if out.compact {
        serde_json::to_string(report)
    } else {
        serde_json::to_string_pretty(report)
    }
    .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"));
    println!("{json}");
    report.exit_code()
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/arena/arena-cli/src/dispatch.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/arena/arena-cli/src/dispatch.rs` captured during arena
      standardization onto the codegen ladder.
```
