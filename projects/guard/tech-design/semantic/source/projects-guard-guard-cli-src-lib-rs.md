---
id: projects-guard-guard-cli-src-lib-rs
summary: Lossless rust-source-unit coverage for `projects/guard/guard-cli/src/lib.rs`.
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

# Standardized projects/guard/guard-cli/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/guard/guard-cli/src/lib.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `GuardCli` | projects/guard/guard-cli/src/lib.rs | struct | pub | 17 |  |
| `dispatch` | projects/guard/guard-cli/src/lib.rs | module | pub | 8 |  |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Agent-first CLI surface for `guard`, registered as a [`CliModule`].
//!
//! The standalone `guard` binary and any future aggregating cclab host use the
//! same dispatch layer. JSON-on-stdout is the default; human output is stderr.

pub mod dispatch;

pub use dispatch::{dispatch, print_report, GuardCommand, OutputOpts, Verb};

use cclab_cli_registry::{CliModule, CLI_MODULES};
use clap::{ArgMatches, CommandFactory, FromArgMatches};
use linkme::distributed_slice;

pub struct GuardCli;

impl CliModule for GuardCli {
    fn name(&self) -> &'static str {
        "guard"
    }

    fn command(&self) -> clap::Command {
        GuardCommand::command()
    }

    fn execute(&self, matches: &ArgMatches) -> anyhow::Result<()> {
        let cmd = GuardCommand::from_arg_matches(matches)?;
        let out = cmd.output.clone();
        let report = dispatch(cmd);
        print_report(&report, &out);
        std::process::exit(report.exit_code);
    }
}

#[distributed_slice(CLI_MODULES)]
static GUARD_CLI: &dyn CliModule = &GuardCli;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_name_is_guard() {
        assert_eq!(GuardCli.name(), "guard");
    }

    #[test]
    fn command_tree_builds() {
        let cmd = GuardCli.command();
        assert_eq!(cmd.get_name(), "guard");
        let subs: Vec<_> = cmd
            .get_subcommands()
            .map(|s| s.get_name().to_string())
            .collect();
        assert!(subs.contains(&"scan".to_string()));
        assert!(subs.contains(&"report".to_string()));
    }

    #[test]
    fn registered_in_slice() {
        assert!(CLI_MODULES.iter().any(|m| m.name() == "guard"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/guard/guard-cli/src/lib.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/guard/guard-cli/src/lib.rs` captured during guard standardization onto the codegen ladder.
```
