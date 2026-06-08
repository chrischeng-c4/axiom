//! CLI Module Registry
//!
//! Provides auto-registration infrastructure for CLI subcommands using linkme.
//! Each crate can self-register by implementing `CliModule` and using
//! `#[distributed_slice(CLI_MODULES)]`.

use anyhow::Result;
use clap::{ArgMatches, Command};
use linkme::distributed_slice;

/// Trait for CLI modules that can be auto-registered.
///
/// Implement this trait and register with `#[distributed_slice(CLI_MODULES)]`
/// to have your command automatically included in the main CLI.
///
/// # Example
///
/// ```ignore
/// use cclab_cli_registry::{CliModule, CLI_MODULES};
/// use linkme::distributed_slice;
///
/// pub struct MyCli;
///
/// impl CliModule for MyCli {
///     fn name(&self) -> &'static str { "my-cmd" }
///     fn command(&self) -> Command { Command::new("my-cmd").about("My command") }
///     fn execute(&self, matches: &ArgMatches) -> Result<()> { Ok(()) }
/// }
///
/// #[distributed_slice(CLI_MODULES)]
/// static MY_CLI: &dyn CliModule = &MyCli;
/// ```
pub trait CliModule: Send + Sync {
    /// Returns the command name (used for subcommand matching).
    fn name(&self) -> &'static str;

    /// Returns the clap Command definition.
    fn command(&self) -> Command;

    /// Executes the command with the given matches.
    fn execute(&self, matches: &ArgMatches) -> Result<()>;
}

/// Distributed slice collecting all registered CLI modules.
///
/// Modules register themselves at link time using:
/// ```ignore
/// #[distributed_slice(CLI_MODULES)]
/// static MY_CLI: &dyn CliModule = &MyCli;
/// ```
#[distributed_slice]
pub static CLI_MODULES: [&'static dyn CliModule];

/// Find a registered module by name.
pub fn find_module(name: &str) -> Option<&'static dyn CliModule> {
    CLI_MODULES.iter().find(|m| m.name() == name).copied()
}

/// Get all registered module names.
pub fn registered_names() -> Vec<&'static str> {
    CLI_MODULES.iter().map(|m| m.name()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_modules_accessible() {
        let count = CLI_MODULES.iter().count();
        assert_eq!(registered_names().len(), count);
    }
}
