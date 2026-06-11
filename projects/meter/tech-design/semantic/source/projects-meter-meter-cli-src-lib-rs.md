---
id: projects-meter-meter-cli-src-lib-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: delegated-runner-exit-code-contract
    claim: delegated-runner-exit-code-contract
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/meter-cli/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/meter-cli/src/lib.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `MeterCli` | projects/meter/meter-cli/src/lib.rs | struct | pub | 34 |  |
| `dispatch` | projects/meter/meter-cli/src/lib.rs | module | pub | 24 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/meter-cli/src/lib.rs -->
````rust
//! Agent-first CLI surface for `meter`, registered as a [`CliModule`].
//!
//! Registers the `meter` subcommand via the `cclab-cli-registry` distributed slice
//! so it is automatically available when an aggregating host binary is built
//! with this crate linked. Because no cclab host binary exists in-tree yet, the
//! same dispatch logic also ships as a standalone `[[bin]] meter`
//! (`src/bin/meter.rs`) so `meter <verb>` works today.
//!
//! JSON-on-stdout is the UNFLAGGED default for every verb; `--human` and
//! `--compact` are the only opt-ins; diagnostics/progress go to stderr.
//!
//! # Exposed subcommand
//!
//! ```text
//! meter report | state               re-project .meter/last-report.json
//! meter spec [--json-schema|--catalog]
//! meter llm guide | recipes [--format json]
//! meter profile|bench|run            measure runtime resources and regressions
//! meter test [-- <runner args>]      delegate tests only as a carried signal
//! ```

pub mod dispatch;

pub use dispatch::{dispatch, print_report, Dispatched, MeterCommand, OutputOpts, Verb};

use cclab_cli_registry::{CliModule, CLI_MODULES};
use clap::{ArgMatches, CommandFactory, FromArgMatches};
use linkme::distributed_slice;

/// The `meter` CLI module: name, clap command tree, and execute hook.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-meter-cli-src-lib-rs.md#source
pub struct MeterCli;

/// @spec projects/meter/tech-design/semantic/source/projects-meter-meter-cli-src-lib-rs.md#source
impl CliModule for MeterCli {
    fn name(&self) -> &'static str {
        "meter"
    }

    fn command(&self) -> clap::Command {
        // The clap-derived tree, surfaced as a top-level `meter` subcommand.
        MeterCommand::command()
    }

    fn execute(&self, matches: &ArgMatches) -> anyhow::Result<()> {
        let cmd = MeterCommand::from_arg_matches(matches)?;
        // The global `--human`/`--compact` flags are flattened into the parsed
        // command, so there is a single source for them.
        let out: OutputOpts = cmd.output.clone();
        let dispatched = dispatch(cmd, &out);
        if !dispatched.stdout_written {
            print_report(&dispatched.report, &out);
        }
        // The registry trait returns Result<()>, so the exit-code/child-forward
        // contract is honored by exiting here (only the standalone bin can
        // return ExitCode). This is the documented honest wart.
        std::process::exit(dispatched.report.exit_code);
    }
}

/// Register `meter` into the shared CLI module slice.
#[distributed_slice(CLI_MODULES)]
static METER_CLI: &dyn CliModule = &MeterCli;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_name_is_meter() {
        assert_eq!(MeterCli.name(), "meter");
    }

    #[test]
    fn command_tree_builds() {
        let cmd = MeterCli.command();
        assert_eq!(cmd.get_name(), "meter");
        let subs: Vec<_> = cmd
            .get_subcommands()
            .map(|s| s.get_name().to_string())
            .collect();
        assert!(subs.contains(&"test".to_string()));
        assert!(subs.contains(&"spec".to_string()));
    }

    #[test]
    fn registered_in_slice() {
        assert!(CLI_MODULES.iter().any(|m| m.name() == "meter"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/meter-cli/src/lib.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/meter-cli/src/lib.rs` captured during meter full-codegen standardization.
```
