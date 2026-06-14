// SPEC-MANAGED: projects/guard/tech-design/semantic/source/projects-guard-guard-cli-src-lib-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Agent-first CLI surface for `guard`, registered as a [`CliModule`].
//!
//! The standalone `guard` binary and any future aggregating cclab host use the
//! same dispatch layer. JSON-on-stdout is the default; human output is stderr.

pub mod dispatch;

pub use dispatch::{dispatch, print_report, GuardCommand, OutputOpts, Verb};

use cclab_cli_registry::{CliModule, CLI_MODULES};
use clap::{ArgMatches, CommandFactory, FromArgMatches};
use linkme::distributed_slice;

/// @spec projects/guard/tech-design/semantic/source/projects-guard-guard-cli-src-lib-rs.md#source
pub struct GuardCli;

/// @spec projects/guard/tech-design/semantic/source/projects-guard-guard-cli-src-lib-rs.md#source
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
// CODEGEN-END
