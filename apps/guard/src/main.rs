// SPEC-MANAGED: apps/guard/tech-design/src/distribution.py
// HANDWRITE-BEGIN gap="existing-project-patch" tracker="#2823" reason="Guard standalone distribution entrypoint remains native Rust"
// @spec WI #2931: executable Python TD parity baseline.
//! Standalone Guard CLI. This package owns both the command surface and its
//! security policy implementation; it has no registry adapter crate.

use std::process::ExitCode;

use clap::Parser;

mod cli;
mod evidence;
mod policy;
mod report;
mod scan;

use cli::{dispatch, print_report, GuardCommand};

fn main() -> ExitCode {
    let cmd = GuardCommand::parse();
    let out = cmd.output.clone();
    let report = dispatch(cmd);
    print_report(&report, &out);
    ExitCode::from(report.exit_code.clamp(0, 255) as u8)
}
// HANDWRITE-END
